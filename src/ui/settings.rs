use crate::app::Launcher;
use crate::config::Settings;
use crate::discord::{RpcEvent, RpcStatus};
use crate::process::open_uri;
use crate::theme::{ACCENTS, DIM, GOLD, GREEN, RED, STEAM_C, WHITE};
use crate::ui::components::{modern_action_button, row, section, toggle, Icon};
use eframe::egui::{self, vec2, Id, Margin, RichText, Sense, Stroke, Ui};

impl Launcher {
    // ───────────────────── Vue Paramètres ─────────────────────
    pub fn settings_ui(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let accent = self.accent();
        let th = self.theme();
        let mut changed = false;
        let mut rescan = false;

        ui.label(RichText::new("Paramètres").size(26.0).strong().color(WHITE));
        ui.add_space(2.0);
        ui.label(
            RichText::new("Configuration générale, Discord Rich Presence, affichage et chemins")
                .size(12.5)
                .color(DIM),
        );
        ui.add_space(18.0);

        egui::ScrollArea::vertical()
            .id_salt("settings_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                section(ui, "Thème & Ambiance Visuelle", th, |ui| {
                    ui.label(RichText::new("Palette d'ambiance de l'interface").size(13.0).color(WHITE));
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        for (idx, name) in [
                            (0, "Obsidian Zinc"),
                            (1, "OLED Noir Pur"),
                            (2, "Cyberpunk Néon"),
                            (3, "Midnight Blue"),
                        ] {
                            let sel = self.settings.theme == idx;
                            let bg = if sel { accent.gamma_multiply(0.25) } else { th.card };
                            let fg = if sel { WHITE } else { DIM };
                            let btn = egui::Button::new(RichText::new(name).size(12.0).color(fg))
                                .fill(bg)
                                .stroke(Stroke::new(1.0, if sel { accent } else { th.card_border }))
                                .corner_radius(7.0);
                            if ui.add(btn).clicked() {
                                self.settings.theme = idx;
                                changed = true;
                            }
                        }
                    });
                });

                section(ui, "Discord & Intégrations", th, |ui| {
                    row(
                        ui,
                        "Activer Discord Rich Presence",
                        "Affiche en temps réel sur ton profil Discord si tu es dans le menu ou en train de jouer.",
                        |ui| {
                            if toggle(
                                ui,
                                "tog_discord_rpc",
                                &mut self.settings.discord_rpc,
                                accent,
                            )
                            .changed()
                            {
                                changed = true;
                                let _ = self
                                    .rpc_tx
                                    .send(RpcEvent::SetEnabled(self.settings.discord_rpc));
                                if self.settings.discord_rpc {
                                    self.update_rpc_menu();
                                }
                            }
                        },
                    );

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Statut de la connexion :").size(12.5).color(WHITE));
                        let (dot_col, txt) = match &self.rpc_status {
                            RpcStatus::Connected => (GREEN, "🟢 Connecté à Discord (Statut actif sur votre profil)"),
                            RpcStatus::WaitingDiscord => (GOLD, "🟡 En attente de Discord (Ouvrez Discord sur votre PC)"),
                            RpcStatus::Connecting => (STEAM_C, "🔵 Connexion au socket Discord IPC..."),
                            RpcStatus::Error(e) => (RED, e.as_str()),
                            RpcStatus::Disabled => (DIM, "⚪ Désactivé"),
                        };
                        ui.label(RichText::new(txt).size(12.5).color(dot_col).strong());
                    });

                    ui.add_space(8.0);
                    ui.label(RichText::new("ID d'application Discord (Client ID)").size(13.0).color(WHITE));
                    ui.label(
                        RichText::new("L'ID de l'application créée sur le portail développeur de Discord.")
                            .size(11.5)
                            .color(DIM),
                    );
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let w = (ui.available_width() - 210.0).max(120.0);
                        let id_resp = ui.add(
                            egui::TextEdit::singleline(&mut self.settings.discord_client_id)
                                .hint_text("Collez ici l'Application ID Discord...")
                                .margin(Margin::symmetric(10, 8))
                                .desired_width(w),
                        );
                        if id_resp.changed() {
                            changed = true;
                        }

                        if ui.button("Appliquer & Tester").clicked() {
                            let _ = self.rpc_tx.send(RpcEvent::SetClientId(self.settings.discord_client_id.clone()));
                            let _ = self.rpc_tx.send(RpcEvent::Reconnect);
                            self.update_rpc_menu();
                            self.toast("Tentative de connexion à Discord...");
                        }

                        if ui.button("Guide").clicked() {
                            self.show_discord_guide = !self.show_discord_guide;
                        }
                    });

                    if self.show_discord_guide {
                        ui.add_space(10.0);
                        egui::Frame::new()
                            .fill(th.card)
                            .stroke(Stroke::new(1.0, th.card_border))
                            .corner_radius(8.0)
                            .inner_margin(Margin::same(12))
                            .show(ui, |ui| {
                                ui.label(RichText::new("📖 Comment faire fonctionner Discord Rich Presence :").size(13.0).strong().color(WHITE));
                                ui.add_space(6.0);
                                ui.label(RichText::new("1. Ouvrez le Portail Développeur Discord :").size(12.0).color(WHITE));
                                if ui.button("🌐 Ouvrir discord.com/developers/applications").clicked() {
                                    let _ = open_uri("https://discord.com/developers/applications");
                                }
                                ui.add_space(4.0);
                                ui.label(RichText::new("2. Cliquez sur 'New Application' en haut à droite et nommez-la 'UniLauncher' (ou le nom de ton choix).").size(12.0).color(DIM));
                                ui.label(RichText::new("3. Dans l'onglet 'General Information', copiez l'Application ID et collez-le dans le champ ci-dessus, puis cliquez sur 'Appliquer & Tester'.").size(12.0).color(DIM));
                                ui.add_space(4.0);
                                ui.label(RichText::new("⚠️ Important : Dans Discord > Paramètres utilisateur > Confidentialité de l'activité, assurez-vous que l'option 'Afficher l'activité actuelle sous forme de message de statut' est bien ACTIVÉE.").size(12.0).color(GOLD));
                            });
                    }
                });

                section(ui, "Affichage & Comportement", th, |ui| {
                    row(
                        ui,
                        "Plein écran au démarrage (Raccourci : F11)",
                        "Lance automatiquement UniLauncher en mode plein écran.",
                        |ui| {
                            if toggle(
                                ui,
                                "tog_fullscreen",
                                &mut self.settings.start_fullscreen,
                                accent,
                            )
                            .changed()
                            {
                                changed = true;
                                self.is_fullscreen = self.settings.start_fullscreen;
                                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                                    self.is_fullscreen,
                                ));
                            }
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);
                    row(
                        ui,
                        "Barre latérale compacte",
                        "Affiche uniquement les icônes dans le menu de navigation à gauche.",
                        |ui| {
                            changed |= toggle(
                                ui,
                                "tog_compact_side",
                                &mut self.settings.compact_sidebar,
                                accent,
                            )
                            .changed();
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);
                    row(
                        ui,
                        "Lancement direct des exécutables Epic Games",
                        "Permet de mesurer précisément le temps de jeu et de passer des arguments.",
                        |ui| {
                            changed |= toggle(
                                ui,
                                "tog_epic_direct",
                                &mut self.settings.epic_direct,
                                accent,
                            )
                            .changed();
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);
                    row(
                        ui,
                        "Réduire UniLauncher au lancement d'un jeu",
                        "Minimise l'application dans la barre des tâches dès qu'un jeu démarre.",
                        |ui| {
                            changed |= toggle(
                                ui,
                                "tog_min_launch",
                                &mut self.settings.minimize_on_launch,
                                accent,
                            )
                            .changed();
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);
                    row(
                        ui,
                        "Téléchargement automatique des jaquettes HD",
                        "Télécharge automatiquement les illustrations manquantes depuis Steam CDN et Steam Store.",
                        |ui| {
                            changed |= toggle(
                                ui,
                                "tog_auto_covers",
                                &mut self.settings.auto_download_covers,
                                accent,
                            )
                            .changed();
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);
                    row(
                        ui,
                        "Mode Performance Boost au lancement",
                        "Active le profil Performances Élevées de Windows au démarrage d'un jeu et le restaure ensuite.",
                        |ui| {
                            changed |= toggle(
                                ui,
                                "tog_perf_boost",
                                &mut self.settings.performance_mode,
                                accent,
                            )
                            .changed();
                        },
                    );
                });

                section(ui, "Dossiers de détection & Bibliothèques", th, |ui| {
                    ui.label(RichText::new("Chemin d'installation Steam").size(13.0).color(WHITE));
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let w = (ui.available_width() - 120.0).max(120.0);
                        changed |= ui
                            .add(
                                egui::TextEdit::singleline(&mut self.settings.steam_path)
                                    .hint_text("Automatique (C:\\Program Files (x86)\\Steam)")
                                    .margin(Margin::symmetric(12, 8))
                                    .desired_width(w),
                            )
                            .changed();
                        if ui.button("Parcourir…").clicked() {
                            if let Some(p) = rfd::FileDialog::new().pick_folder() {
                                self.settings.steam_path = p.to_string_lossy().to_string();
                                changed = true;
                                rescan = true;
                            }
                        }
                    });
                    ui.add_space(10.0);
                    ui.label(RichText::new("Dossier des manifestes Epic Games").size(13.0).color(WHITE));
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let w = (ui.available_width() - 120.0).max(120.0);
                        changed |= ui
                            .add(
                                egui::TextEdit::singleline(&mut self.settings.epic_manifests_path)
                                    .hint_text("Automatique (C:\\ProgramData\\Epic\\...)")
                                    .margin(Margin::symmetric(12, 8))
                                    .desired_width(w),
                            )
                            .changed();
                        if ui.button("Parcourir…").clicked() {
                            if let Some(p) = rfd::FileDialog::new().pick_folder() {
                                self.settings.epic_manifests_path = p.to_string_lossy().to_string();
                                changed = true;
                                rescan = true;
                            }
                        }
                    });
                    ui.add_space(12.0);

                    ui.label(RichText::new("Dossiers personnalisés scannés automatiquement (Jeux, GOG, ROMs...)").size(13.0).color(WHITE));
                    ui.add_space(4.0);
                    let mut remove_folder = None;
                    for (idx, folder) in self.settings.custom_scan_folders.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("📁 {}", folder.display())).size(12.0).color(WHITE));
                            if ui.small_button("✕").clicked() {
                                remove_folder = Some(idx);
                            }
                        });
                    }
                    if let Some(idx) = remove_folder {
                        self.settings.custom_scan_folders.remove(idx);
                        changed = true;
                        rescan = true;
                    }

                    if ui.button("➕ Ajouter un dossier de jeux à scanner…").clicked() {
                        if let Some(p) = rfd::FileDialog::new().set_title("Sélectionner un dossier de jeux").pick_folder() {
                            if !self.settings.custom_scan_folders.contains(&p) {
                                self.settings.custom_scan_folders.push(p);
                                changed = true;
                                rescan = true;
                            }
                        }
                    }

                    ui.add_space(14.0);
                    if modern_action_button(
                        ui,
                        "Rescanner toutes les bibliothèques",
                        Icon::Refresh,
                        accent,
                        true,
                        th,
                    ) {
                        rescan = true;
                    }
                });

                section(ui, "Couleur d'accentuation", th, |ui| {
                    ui.horizontal(|ui| {
                        for (i, c) in ACCENTS.iter().enumerate() {
                            let (rect, resp) =
                                ui.allocate_exact_size(vec2(34.0, 34.0), Sense::click());
                            let t = ui.ctx().animate_bool(
                                Id::unique(("accent_pick", i)),
                                self.settings.accent == i,
                            );
                            let p = ui.painter();
                            p.circle_filled(rect.center(), 12.0 + t * 1.5, *c);
                            if t > 0.01 {
                                p.circle_stroke(rect.center(), 16.0, Stroke::new(1.8 * t, WHITE));
                            }
                            if resp
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .clicked()
                            {
                                self.settings.accent = i;
                                changed = true;
                            }
                        }
                    });
                });

                section(ui, "Maintenance & Données", th, |ui| {
                    ui.label(
                        RichText::new(format!(
                            "Dossier de configuration : {}",
                            Settings::dir().display()
                        ))
                        .size(12.0)
                        .color(DIM),
                    );
                    ui.add_space(10.0);
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Ouvrir le dossier de données").clicked() {
                            let _ = open_uri(&Settings::dir().to_string_lossy());
                        }
                        if ui.button("Exporter la configuration (JSON)…").clicked() {
                            if let Some(p) = rfd::FileDialog::new()
                                .set_title("Exporter la configuration")
                                .set_file_name("unilauncher_config.json")
                                .save_file()
                            {
                                if let Ok(json) = serde_json::to_string_pretty(&self.settings) {
                                    let _ = std::fs::write(p, json);
                                    self.toast("Configuration exportée !");
                                }
                            }
                        }
                        if ui.button("Importer une configuration…").clicked() {
                            if let Some(p) = rfd::FileDialog::new()
                                .set_title("Importer une configuration")
                                .add_filter("JSON", &["json"])
                                .pick_file()
                            {
                                if let Ok(txt) = std::fs::read_to_string(&p) {
                                    if let Ok(s) = serde_json::from_str::<Settings>(&txt) {
                                        self.settings = s;
                                        self.settings.save();
                                        self.refresh();
                                        self.toast("Configuration importée !");
                                    } else {
                                        self.toast("Fichier JSON invalide.");
                                    }
                                }
                            }
                        }
                        if ui.button("Réinitialiser les favoris").clicked() {
                            self.settings.favorites.clear();
                            changed = true;
                            self.toast("Liste des favoris vidée");
                        }
                        if ui.button("Afficher tous les jeux masqués").clicked() {
                            self.settings.hidden_games.clear();
                            changed = true;
                            self.toast("Tous les jeux masqués ont été restaurés");
                        }
                    });
                });
            });

        if changed {
            self.settings.save();
        }
        if rescan {
            self.refresh();
            self.toast(format!("{} titres synchronisés", self.games.len()));
        }
    }
}
