use crate::app::Launcher;
use crate::config::Settings;
use crate::discord::{RpcEvent, RpcStatus};
use crate::process::open_uri;
use crate::theme::{ACCENTS, BNET_C, DIM, EA_C, EPIC_C, GOG_C, GOLD, GREEN, RED, STEAM_C, UBI_C, WHITE, XBOX_C};
use crate::ui::components::{modern_action_button, row, section, toggle, Icon};
use eframe::egui::{self, vec2, Align, Id, Layout, Margin, RichText, Sense, Stroke, Ui};

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
                section(ui, "Thème & Ambiance Visuelle (8 Thèmes)", th, |ui| {
                    ui.label(RichText::new("Palette d'ambiance de l'interface").size(13.5).strong().color(WHITE));
                    ui.add_space(2.0);
                    ui.label(RichText::new("Choisissez parmi 8 thèmes ultra-modernes avec contrastes optimisés").size(11.5).color(DIM));
                    ui.add_space(10.0);

                    // Grille des 8 thèmes
                    ui.columns(2, |cols| {
                        for idx in 0..8 {
                            let col = &mut cols[idx % 2];
                            let theme_meta = crate::theme::get_theme(idx);
                            let name = crate::theme::THEME_NAMES[idx];
                            let is_sel = self.settings.theme == idx;

                            let bg = if is_sel { accent.gamma_multiply(0.2) } else { theme_meta.card };
                            let border_col = if is_sel { accent } else { theme_meta.card_border };

                            egui::Frame::new()
                                .fill(bg)
                                .stroke(Stroke::new(if is_sel { 1.5 } else { 1.0 }, border_col))
                                .corner_radius(10.0)
                                .inner_margin(Margin::symmetric(14, 10))
                                .show(col, |ui| {
                                    let resp = ui.interact(ui.max_rect(), Id::unique(("th_pick", idx)), Sense::click());
                                    if resp.clicked() {
                                        self.settings.theme = idx;
                                        changed = true;
                                    }

                                    ui.horizontal(|ui| {
                                        // Pastilles de preview de la palette du thème
                                        let (r, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                                        ui.painter().circle_filled(r.center(), 6.0, theme_meta.bg);
                                        ui.painter().circle_stroke(r.center(), 6.5, Stroke::new(1.0, theme_meta.card_border));

                                        let (r2, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                                        ui.painter().circle_filled(r2.center(), 6.0, theme_meta.card);
                                        ui.painter().circle_stroke(r2.center(), 6.5, Stroke::new(1.0, theme_meta.card_border));

                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new(name)
                                                .size(13.0)
                                                .strong()
                                                .color(if is_sel { WHITE } else { DIM }),
                                        );

                                        if is_sel {
                                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                ui.label(RichText::new("✓ Actif").size(11.5).strong().color(accent));
                                            });
                                        }
                                    });
                                });

                            col.add_space(6.0);
                        }
                    });

                    ui.add_space(14.0);
                    ui.label(RichText::new("Couleur d'accentuation dynamique").size(13.5).strong().color(WHITE));
                    ui.add_space(2.0);
                    ui.label(RichText::new("Personnalise la couleur des halos, boutons et indicateurs d'action").size(11.5).color(DIM));
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        for (idx, &acc_col) in ACCENTS.iter().enumerate() {
                            let is_sel = self.settings.accent == idx;
                            let (rect, resp) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::click());
                            if resp.clicked() {
                                self.settings.accent = idx;
                                changed = true;
                            }
                            if resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            let p = ui.painter();
                            p.circle_filled(rect.center(), if is_sel { 13.0 } else { 11.0 }, acc_col);
                            if is_sel {
                                p.circle_stroke(rect.center(), 15.0, Stroke::new(2.0, WHITE));
                            } else if resp.hovered() {
                                p.circle_stroke(rect.center(), 14.0, Stroke::new(1.5, WHITE.gamma_multiply(0.5)));
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

                section(ui, "Plateformes Détectées sur Windows & Scan Multi-Disques", th, |ui| {
                    ui.label(RichText::new("État de détection des lanceurs et magasins").size(13.5).strong().color(WHITE));
                    ui.add_space(2.0);
                    ui.label(RichText::new("UniLauncher vérifie le registre et tous les disques (C:, D:, E:...) pour récupérer automatiquement vos jeux").size(11.5).color(DIM));
                    ui.add_space(10.0);

                    let platforms = crate::scan::detect_installed_platforms(&self.settings, &self.games);
                    ui.columns(2, |cols| {
                        for (idx, p) in platforms.iter().enumerate() {
                            let col = &mut cols[idx % 2];
                            let p_color = match p.source {
                                crate::config::Source::Steam => STEAM_C,
                                crate::config::Source::Epic => EPIC_C,
                                crate::config::Source::Gog => GOG_C,
                                crate::config::Source::Ubisoft => UBI_C,
                                crate::config::Source::Ea => EA_C,
                                crate::config::Source::Battlenet => BNET_C,
                                crate::config::Source::Xbox => XBOX_C,
                                crate::config::Source::Manual => accent,
                            };

                            egui::Frame::new()
                                .fill(th.card)
                                .stroke(Stroke::new(1.0, th.card_border))
                                .corner_radius(8.0)
                                .inner_margin(Margin::symmetric(12, 10))
                                .show(col, |ui| {
                                    ui.horizontal(|ui| {
                                        let (r, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                                        ui.painter().circle_filled(r.center(), 4.5, if p.is_installed { GREEN } else { DIM });

                                        ui.label(RichText::new(p.name).size(13.0).strong().color(WHITE));

                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            if p.is_installed {
                                                let tag = format!("{} titres", p.game_count);
                                                ui.label(RichText::new(tag).size(11.5).strong().color(p_color));
                                            } else {
                                                ui.label(RichText::new("Non détecté").size(11.0).color(DIM));
                                            }
                                        });
                                    });
                                });
                            col.add_space(6.0);
                        }
                    });

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(10.0);

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

                section(ui, "Personnalisation de la Bibliothèque & Interface", th, |ui| {
                    ui.label(RichText::new("Disposition et Apparence des Cartes").size(13.5).strong().color(WHITE));
                    ui.add_space(2.0);
                    ui.label(RichText::new("Ajustez la taille, l'espacement et les détails visibles dans votre catalogue").size(11.5).color(DIM));
                    ui.add_space(10.0);

                    // 1. Taille des cartes
                    row(
                        ui,
                        "Échelle des cartes de jeu",
                        "Ajustez le format d'affichage des jaquettes dans la grille.",
                        |ui| {
                            let mut scale = self.settings.card_scale;
                            if ui.add(egui::Slider::new(&mut scale, 0.70..=1.40).text("x").step_by(0.05)).changed() {
                                self.settings.card_scale = scale;
                                changed = true;
                            }
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);

                    // 2. Espacement
                    row(
                        ui,
                        "Espacement entre les cartes (Grid Spacing)",
                        "Définit la distance en pixels entre chaque jaquette de la grille.",
                        |ui| {
                            let mut spacing = self.settings.grid_spacing;
                            if ui.add(egui::Slider::new(&mut spacing, 8.0..=32.0).text("px").step_by(2.0)).changed() {
                                self.settings.grid_spacing = spacing;
                                changed = true;
                            }
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);

                    // 3. Mode d'affichage par défaut
                    row(
                        ui,
                        "Mode d'affichage de la bibliothèque",
                        "Choisissez entre une grille moderne de jaquettes ou une liste compacte.",
                        |ui| {
                            ui.horizontal(|ui| {
                                if ui.selectable_label(self.view_mode == crate::types::ViewMode::Grid, "🔲 Grille").clicked() {
                                    self.view_mode = crate::types::ViewMode::Grid;
                                }
                                if ui.selectable_label(self.view_mode == crate::types::ViewMode::List, "📄 Liste").clicked() {
                                    self.view_mode = crate::types::ViewMode::List;
                                }
                            });
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);

                    // 4. Tri par défaut
                    row(
                        ui,
                        "Ordre de tri par défaut",
                        "Critère de tri appliqué à l'ouverture de votre catalogue.",
                        |ui| {
                            egui::ComboBox::from_id_salt("settings_sort_combo")
                                .selected_text(self.sort.label())
                                .show_ui(ui, |ui| {
                                    for &s in &[
                                        crate::types::SortMode::Playtime,
                                        crate::types::SortMode::Name,
                                        crate::types::SortMode::Favorites,
                                        crate::types::SortMode::SizeOnDisk,
                                    ] {
                                        if ui.selectable_value(&mut self.sort, s, s.label()).clicked() {
                                            changed = true;
                                        }
                                    }
                                });
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);

                    // 5. Informations visibles
                    row(
                        ui,
                        "Afficher le temps de jeu sur les cartes",
                        "Indique le temps total passé sur le jeu directement sur la jaquette.",
                        |ui| {
                            changed |= toggle(
                                ui,
                                "tog_card_playtime",
                                &mut self.settings.show_card_playtime,
                                accent,
                            )
                            .changed();
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);

                    row(
                        ui,
                        "Afficher les badges de plateforme sur les cartes",
                        "Affiche la pastille du lanceur d'origine (Steam, Epic, Battle.net, etc.).",
                        |ui| {
                            changed |= toggle(
                                ui,
                                "tog_card_badges",
                                &mut self.settings.show_card_badges,
                                accent,
                            )
                            .changed();
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);

                    row(
                        ui,
                        "Animations et transitions fluides",
                        "Active les effets de survol, transitions douces et respirations visuelles.",
                        |ui| {
                            changed |= toggle(
                                ui,
                                "tog_enable_animations",
                                &mut self.settings.enable_animations,
                                accent,
                            )
                            .changed();
                        },
                    );
                });

                section(ui, "Game Performance & Overlay Flottant (F9)", th, |ui| {
                    row(
                        ui,
                        "Activer l'Overlay en jeu (Raccourci : F9)",
                        "Affiche un widget discret au-dessus de vos jeux sans toucher aux DLL ni déclencher d'anti-triche.",
                        |ui| {
                            changed |= toggle(
                                ui,
                                "tog_overlay_enabled",
                                &mut self.settings.overlay.enabled,
                                accent,
                            )
                            .changed();
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);

                    row(
                        ui,
                        "Position de l'overlay à l'écran",
                        "Choisissez le coin de l'écran où afficher le moniteur de performances.",
                        |ui| {
                            ui.horizontal(|ui| {
                                for (pos, lbl) in [
                                    (crate::performance::OverlayPosition::TopRight, "Haut Droit"),
                                    (crate::performance::OverlayPosition::TopLeft, "Haut Gauche"),
                                    (crate::performance::OverlayPosition::BottomRight, "Bas Droit"),
                                    (crate::performance::OverlayPosition::BottomLeft, "Bas Gauche"),
                                ] {
                                    if ui.selectable_label(self.settings.overlay.position == pos, lbl).clicked() {
                                        self.settings.overlay.position = pos;
                                        changed = true;
                                    }
                                }
                            });
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);

                    row(
                        ui,
                        "Opacité / Transparence de l'overlay",
                        "Règle le niveau de transparence du fond pour ne pas gêner en jeu.",
                        |ui| {
                            let mut op = self.settings.overlay.opacity;
                            if ui.add(egui::Slider::new(&mut op, 0.20..=1.00).text("%").custom_formatter(|n, _| format!("{:.0}%", n * 100.0))).changed() {
                                self.settings.overlay.opacity = op;
                                changed = true;
                            }
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);

                    row(
                        ui,
                        "Échelle / Taille de l'overlay",
                        "Ajustez la taille d'affichage générale du widget HUD.",
                        |ui| {
                            let mut sc = self.settings.overlay.scale;
                            if ui.add(egui::Slider::new(&mut sc, 0.75..=1.50).text("x").step_by(0.05)).changed() {
                                self.settings.overlay.scale = sc;
                                changed = true;
                            }
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);

                    ui.label(RichText::new("Métriques affichées sur l'overlay :").size(12.5).strong().color(WHITE));
                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        changed |= ui.checkbox(&mut self.settings.overlay.show_fps, "FPS & 1% Low").changed();
                        changed |= ui.checkbox(&mut self.settings.overlay.show_frametime, "Frametime (ms)").changed();
                        changed |= ui.checkbox(&mut self.settings.overlay.show_cpu, "CPU (Usage, GHz, Temp)").changed();
                        changed |= ui.checkbox(&mut self.settings.overlay.show_gpu, "GPU (Usage, VRAM)").changed();
                        changed |= ui.checkbox(&mut self.settings.overlay.show_ram, "RAM (Go utilisée)").changed();
                    });
                });

                section(ui, "Succès Steam & API Web Officielle", th, |ui| {
                    ui.label(RichText::new("Configuration du suivi des succès Steam").size(13.5).strong().color(WHITE));
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("UniLauncher récupère déjà le catalogue officiel de succès. Pour synchroniser vos trophées personnels privés, indiquez votre clé API Steam et votre SteamID64.")
                            .size(11.5)
                            .color(DIM),
                    );
                    ui.add_space(10.0);

                    ui.label(RichText::new("Clé Web API Steam").size(12.5).color(WHITE));
                    let w = (ui.available_width() - 140.0).max(120.0);
                    ui.horizontal(|ui| {
                        changed |= ui
                            .add(
                                egui::TextEdit::singleline(&mut self.settings.steam_api_key)
                                    .hint_text("Collez ici votre clé API Steam (facultatif)...")
                                    .password(true)
                                    .margin(Margin::symmetric(10, 8))
                                    .desired_width(w),
                            )
                            .changed();
                        if ui.button("Obtenir ma clé").clicked() {
                            let _ = open_uri("https://steamcommunity.com/dev/apikey");
                        }
                    });

                    ui.add_space(8.0);
                    ui.label(RichText::new("SteamID64 de votre compte").size(12.5).color(WHITE));
                    ui.horizontal(|ui| {
                        changed |= ui
                            .add(
                                egui::TextEdit::singleline(&mut self.settings.steam_user_id)
                                    .hint_text("76561198... (facultatif si profil public)")
                                    .margin(Margin::symmetric(10, 8))
                                    .desired_width(w),
                            )
                            .changed();
                        if ui.button("Trouver mon ID").clicked() {
                            let _ = open_uri("https://steamid.io/");
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
