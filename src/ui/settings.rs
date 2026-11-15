use crate::app::Launcher;
use crate::config::Settings;
use crate::discord::{RpcEvent, RpcStatus};
use crate::process::open_uri;
use crate::theme::{
    get_theme, ThemeColors, ACCENTS, BNET_C, EA_C, EPIC_C, ERROR, GOG_C, GOLD, INFO,
    STEAM_C, SUCCESS, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_TERTIARY, THEME_NAMES, UBI_C,
    WARNING, WHITE, XBOX_C,
};
use crate::ui::components::{modern_action_button, row, section, toggle, Icon};
use eframe::egui::{self, vec2, Align, CornerRadius, FontId, Id, Layout, Margin, Rect, RichText, Sense, Stroke, Ui};

// ─────────────────────────────────────────────────────────────────────────────
//  Boutons & Puces d'Action Dédiés aux Paramètres
// ─────────────────────────────────────────────────────────────────────────────

fn action_chip(ui: &mut Ui, label: &str, th: ThemeColors) -> bool {
    let btn = egui::Button::new(RichText::new(label).size(12.0).color(TEXT_PRIMARY))
        .fill(th.surface_2)
        .stroke(Stroke::new(1.0, th.border_default))
        .corner_radius(CornerRadius::same(6))
        .min_size(vec2(0.0, 26.0));
    ui.add(btn).clicked()
}

fn primary_chip(ui: &mut Ui, label: &str, accent: egui::Color32) -> bool {
    let btn = egui::Button::new(RichText::new(label).size(12.0).strong().color(WHITE))
        .fill(accent.gamma_multiply(0.25))
        .stroke(Stroke::new(1.0, accent.gamma_multiply(0.70)))
        .corner_radius(CornerRadius::same(6))
        .min_size(vec2(0.0, 26.0));
    ui.add(btn).clicked()
}

impl Launcher {
    // ───────────────────── Vue Paramètres ─────────────────────
    pub fn settings_ui(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let accent = self.accent();
        let th = self.theme();
        let mut changed = false;
        let mut rescan = false;

        // ─── En-tête de la page ───
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Paramètres").size(22.0).strong().color(TEXT_PRIMARY));
                ui.add_space(2.0);
                ui.label(
                    RichText::new("Personnalisez l'apparence, l'affichage de vos jeux, les intégrations et les performances.")
                        .size(12.0)
                        .color(TEXT_SECONDARY),
                );
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                egui::Frame::new()
                    .fill(th.surface_2)
                    .stroke(Stroke::new(1.0, th.border_subtle))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::symmetric(9, 4))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 5.0;
                            let (dot, _) = ui.allocate_exact_size(vec2(6.0, 6.0), Sense::hover());
                            ui.painter().circle_filled(dot.center(), 2.5, SUCCESS);
                            ui.label(RichText::new("Sauvegarde auto").size(11.0).color(TEXT_TERTIARY));
                        });
                    });
            });
        });

        ui.add_space(16.0);

        egui::ScrollArea::vertical()
            .id_salt("settings_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // ═════════════════════════════════════════════════════════════
                // 1. Thème & Ambiance Visuelle
                // ═════════════════════════════════════════════════════════════
                section(ui, "🎨  Thème & Ambiance Visuelle", th, |ui| {
                    ui.label(RichText::new("Palette d'ambiance de l'interface").size(13.0).strong().color(TEXT_PRIMARY));
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("Choisissez parmi 8 thèmes calibrés avec contrastes optimisés")
                            .size(11.5)
                            .color(TEXT_SECONDARY),
                    );
                    ui.add_space(10.0);

                    // Grille des 8 thèmes
                    ui.columns(2, |cols| {
                        for idx in 0..8 {
                            let col = &mut cols[idx % 2];
                            let theme_meta = get_theme(idx);
                            let name = THEME_NAMES[idx];
                            let is_sel = self.settings.theme == idx;

                            let base_bg = if is_sel {
                                accent.gamma_multiply(0.14)
                            } else {
                                th.surface_1
                            };
                            let border_col = if is_sel {
                                accent
                            } else {
                                th.border_subtle
                            };

                            egui::Frame::new()
                                .fill(base_bg)
                                .stroke(Stroke::new(if is_sel { 1.5 } else { 1.0 }, border_col))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::symmetric(14, 10))
                                .show(col, |ui| {
                                    let resp = ui.interact(
                                        ui.max_rect(),
                                        Id::unique(("th_pick", idx)),
                                        Sense::click(),
                                    );
                                    if resp.clicked() {
                                        self.settings.theme = idx;
                                        changed = true;
                                    }
                                    if resp.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    }

                                    ui.horizontal(|ui| {
                                        // 3 pastilles d'échantillons de couleurs
                                        let (r1, _) = ui.allocate_exact_size(vec2(13.0, 13.0), Sense::hover());
                                        let (r2, _) = ui.allocate_exact_size(vec2(13.0, 13.0), Sense::hover());
                                        let (r3, _) = ui.allocate_exact_size(vec2(13.0, 13.0), Sense::hover());

                                        let p = ui.painter();
                                        p.circle_filled(r1.center(), 5.5, theme_meta.bg);
                                        p.circle_stroke(r1.center(), 5.5, Stroke::new(1.0, theme_meta.card_border));

                                        p.circle_filled(r2.center(), 5.5, theme_meta.panel);
                                        p.circle_stroke(r2.center(), 5.5, Stroke::new(1.0, theme_meta.card_border));

                                        p.circle_filled(r3.center(), 5.5, theme_meta.card);
                                        p.circle_stroke(r3.center(), 5.5, Stroke::new(1.0, theme_meta.card_border));

                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new(name)
                                                .size(13.0)
                                                .strong()
                                                .color(if is_sel { TEXT_PRIMARY } else { TEXT_SECONDARY }),
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
                    ui.label(RichText::new("Couleur d'accentuation dynamique").size(13.0).strong().color(TEXT_PRIMARY));
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("Personnalise la couleur des halos, boutons d'action et indicateurs d'état")
                            .size(11.5)
                            .color(TEXT_SECONDARY),
                    );
                    ui.add_space(10.0);

                    // Pastilles d'accents interactives
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 10.0;
                        for (idx, &acc_col) in ACCENTS.iter().enumerate() {
                            let is_sel = self.settings.accent == idx;
                            let (rect, resp) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::click());
                            let t = ui.ctx().animate_bool(Id::unique(("accent_pick", idx)), is_sel);
                            let hov_t = ui.ctx().animate_bool(Id::unique(("accent_hov", idx)), resp.hovered());

                            if resp.clicked() {
                                self.settings.accent = idx;
                                changed = true;
                            }
                            if resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }

                            let p = ui.painter();
                            let base_r = 11.0 + t * 2.0;
                            p.circle_filled(rect.center(), base_r, acc_col);

                            if t > 0.01 {
                                p.circle_stroke(rect.center(), 15.5, Stroke::new(1.8 * t, WHITE));
                            } else if hov_t > 0.01 {
                                p.circle_stroke(
                                    rect.center(),
                                    14.5,
                                    Stroke::new(1.2 * hov_t, WHITE.gamma_multiply(0.45)),
                                );
                            }
                        }
                    });
                });

                // ═════════════════════════════════════════════════════════════
                // 2. Affichage & Personnalisation de la Bibliothèque
                // ═════════════════════════════════════════════════════════════
                section(ui, "🎮  Affichage & Bibliothèque", th, |ui| {
                    ui.label(RichText::new("Disposition et format des jeux").size(13.0).strong().color(TEXT_PRIMARY));
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("Ajustez l'échelle des jaquettes, l'espacement et les détails visibles dans le catalogue")
                            .size(11.5)
                            .color(TEXT_SECONDARY),
                    );
                    ui.add_space(10.0);

                    // Échelle des cartes
                    row(
                        ui,
                        "Échelle des cartes de jeu",
                        "Ajustez la taille d'affichage des jaquettes dans la grille.",
                        |ui| {
                            let mut scale = self.settings.card_scale;
                            if ui
                                .add(
                                    egui::Slider::new(&mut scale, 0.70..=1.40)
                                        .step_by(0.05)
                                        .suffix("x"),
                                )
                                .changed()
                            {
                                self.settings.card_scale = scale;
                                changed = true;
                            }
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);

                    // Espacement de la grille
                    row(
                        ui,
                        "Espacement entre les cartes (Grid Spacing)",
                        "Définit la distance en pixels entre chaque jaquette de la grille.",
                        |ui| {
                            let mut spacing = self.settings.grid_spacing;
                            if ui
                                .add(
                                    egui::Slider::new(&mut spacing, 8.0..=32.0)
                                        .step_by(2.0)
                                        .suffix(" px"),
                                )
                                .changed()
                            {
                                self.settings.grid_spacing = spacing;
                                changed = true;
                            }
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);

                    // Mode d'affichage par défaut (Grille vs Liste)
                    row(
                        ui,
                        "Mode d'affichage de la bibliothèque",
                        "Choisissez entre une grille moderne de jaquettes ou une liste compacte.",
                        |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;
                                let is_grid = self.view_mode == crate::types::ViewMode::Grid;
                                let is_list = self.view_mode == crate::types::ViewMode::List;

                                let grid_btn = egui::Button::new(
                                    RichText::new("🔲  Grille").size(12.0).color(if is_grid { WHITE } else { TEXT_SECONDARY }),
                                )
                                .fill(if is_grid { accent.gamma_multiply(0.25) } else { th.surface_2 })
                                .stroke(Stroke::new(1.0, if is_grid { accent } else { th.border_default }))
                                .corner_radius(CornerRadius::same(6))
                                .min_size(vec2(0.0, 26.0));

                                if ui.add(grid_btn).clicked() {
                                    self.view_mode = crate::types::ViewMode::Grid;
                                }

                                let list_btn = egui::Button::new(
                                    RichText::new("📄  Liste").size(12.0).color(if is_list { WHITE } else { TEXT_SECONDARY }),
                                )
                                .fill(if is_list { accent.gamma_multiply(0.25) } else { th.surface_2 })
                                .stroke(Stroke::new(1.0, if is_list { accent } else { th.border_default }))
                                .corner_radius(CornerRadius::same(6))
                                .min_size(vec2(0.0, 26.0));

                                if ui.add(list_btn).clicked() {
                                    self.view_mode = crate::types::ViewMode::List;
                                }
                            });
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);

                    // Ordre de tri par défaut
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

                    // Informations visibles sur les cartes
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
                    ui.separator();
                    ui.add_space(6.0);

                    row(
                        ui,
                        "Plein écran au démarrage (Raccourci : F11)",
                        "Lance automatiquement UniLauncher en mode plein écran sans bordure.",
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
                        "Affiche uniquement les icônes dans la navigation gauche pour maximiser l'espace.",
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
                });

                // ═════════════════════════════════════════════════════════════
                // 3. Lancement & Optimisations Système
                // ═════════════════════════════════════════════════════════════
                section(ui, "⚡  Lancement & Optimisations Système", th, |ui| {
                    row(
                        ui,
                        "Mode Performance Boost au lancement",
                        "Active le plan d'alimentation 'Performances Élevées' de Windows au démarrage d'un jeu et le restaure ensuite.",
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
                    ui.separator();
                    ui.add_space(6.0);

                    row(
                        ui,
                        "Réduire UniLauncher au lancement d'un jeu",
                        "Minimise l'application dans la barre des tâches dès qu'un jeu démarre pour libérer l'écran.",
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
                        "Lancement direct des exécutables Epic Games",
                        "Permet de mesurer précisément le temps de jeu et de passer des arguments sans lancer le client complet.",
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
                        "Téléchargement automatique des jaquettes HD",
                        "Télécharge automatiquement les illustrations haute définition manquantes depuis Steam CDN et Steam Store.",
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
                });

                // ═════════════════════════════════════════════════════════════
                // 4. Discord & Intégrations Rich Presence
                // ═════════════════════════════════════════════════════════════
                section(ui, "💬  Discord & Intégrations Rich Presence", th, |ui| {
                    row(
                        ui,
                        "Activer Discord Rich Presence",
                        "Affiche en temps réel sur votre profil Discord votre statut (menu, jeu en cours, temps écoulé).",
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

                    // Pastille d'état de connexion Discord
                    let (dot_col, txt) = match &self.rpc_status {
                        RpcStatus::Connected => (SUCCESS, "Connecté à Discord (Statut actif sur votre profil)"),
                        RpcStatus::WaitingDiscord => (WARNING, "En attente de Discord (Ouvrez Discord sur votre PC)"),
                        RpcStatus::Connecting => (INFO, "Connexion au socket Discord IPC..."),
                        RpcStatus::Error(e) => (ERROR, e.as_str()),
                        RpcStatus::Disabled => (TEXT_TERTIARY, "Désactivé"),
                    };

                    egui::Frame::new()
                        .fill(dot_col.gamma_multiply(0.12))
                        .stroke(Stroke::new(1.0, dot_col.gamma_multiply(0.35)))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(Margin::symmetric(10, 6))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 8.0;
                                let (dot_r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                                ui.painter().circle_filled(dot_r.center(), 3.5, dot_col);
                                ui.label(RichText::new(txt).size(12.0).strong().color(dot_col));
                            });
                        });

                    ui.add_space(10.0);
                    ui.label(RichText::new("ID d'application Discord (Client ID)").size(13.0).color(TEXT_PRIMARY));
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("L'ID de l'application créée sur le portail développeur de Discord pour personnaliser le nom affiché.")
                            .size(11.5)
                            .color(TEXT_SECONDARY),
                    );
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        let w = (ui.available_width() - 240.0).max(120.0);
                        let id_resp = ui.add(
                            egui::TextEdit::singleline(&mut self.settings.discord_client_id)
                                .hint_text("Collez ici l'Application ID Discord...")
                                .margin(Margin::symmetric(10, 7))
                                .desired_width(w),
                        );
                        if id_resp.changed() {
                            changed = true;
                        }

                        if primary_chip(ui, "Appliquer & Tester", accent) {
                            let _ = self
                                .rpc_tx
                                .send(RpcEvent::SetClientId(self.settings.discord_client_id.clone()));
                            let _ = self.rpc_tx.send(RpcEvent::Reconnect);
                            self.update_rpc_menu();
                            self.toast("Tentative de reconnexion à Discord...");
                        }

                        let guide_lbl = if self.show_discord_guide {
                            "Masquer le guide"
                        } else {
                            "📖 Guide"
                        };
                        if action_chip(ui, guide_lbl, th) {
                            self.show_discord_guide = !self.show_discord_guide;
                        }
                    });

                    // Accordéon guide d'explication Discord
                    if self.show_discord_guide {
                        ui.add_space(10.0);
                        egui::Frame::new()
                            .fill(th.surface_1)
                            .stroke(Stroke::new(1.0, th.border_subtle))
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(Margin::same(14))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new("📖 Comment configurer Discord Rich Presence :")
                                        .size(13.0)
                                        .strong()
                                        .color(TEXT_PRIMARY),
                                );
                                ui.add_space(6.0);

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("1. Rendez-vous sur le Portail Développeur Discord :").size(12.0).color(TEXT_SECONDARY));
                                    if action_chip(ui, "🌐 Ouvrir discord.com/developers", th) {
                                        let _ = open_uri("https://discord.com/developers/applications");
                                    }
                                });

                                ui.add_space(3.0);
                                ui.label(
                                    RichText::new("2. Cliquez sur 'New Application' en haut à droite et nommez-la 'UniLauncher' (ou le nom de votre choix).")
                                        .size(12.0)
                                        .color(TEXT_SECONDARY),
                                );
                                ui.add_space(2.0);
                                ui.label(
                                    RichText::new("3. Dans l'onglet 'General Information', copiez l'Application ID et collez-le ci-dessus, puis validez.")
                                        .size(12.0)
                                        .color(TEXT_SECONDARY),
                                );
                                ui.add_space(6.0);

                                egui::Frame::new()
                                    .fill(GOLD.gamma_multiply(0.12))
                                    .stroke(Stroke::new(1.0, GOLD.gamma_multiply(0.35)))
                                    .corner_radius(CornerRadius::same(6))
                                    .inner_margin(Margin::symmetric(10, 6))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new("⚠️ Important : Dans Discord > Paramètres utilisateur > Confidentialité de l'activité, assurez-vous que l'option 'Afficher l'activité actuelle sous forme de message de statut' est bien ACTIVÉE.")
                                                .size(11.5)
                                                .color(GOLD),
                                        );
                                    });
                            });
                    }
                });

                // ═════════════════════════════════════════════════════════════
                // 5. Plateformes Détectées & Chemins d'Accès
                // ═════════════════════════════════════════════════════════════
                section(ui, "📂  Plateformes Détectées & Chemins d'Accès", th, |ui| {
                    ui.label(RichText::new("État de détection des magasins et lanceurs").size(13.0).strong().color(TEXT_PRIMARY));
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("UniLauncher vérifie le registre et tous les disques (C:, D:, E:...) pour récupérer automatiquement vos jeux")
                            .size(11.5)
                            .color(TEXT_SECONDARY),
                    );
                    ui.add_space(10.0);

                    // Mini-cartes 2 colonnes des plateformes installées
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
                                .fill(th.surface_1)
                                .stroke(Stroke::new(1.0, th.border_subtle))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::symmetric(12, 10))
                                .show(col, |ui| {
                                    ui.horizontal(|ui| {
                                        let (r, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                                        ui.painter().circle_filled(
                                            r.center(),
                                            4.0,
                                            if p.is_installed { SUCCESS } else { TEXT_TERTIARY },
                                        );

                                        ui.label(RichText::new(p.name).size(12.5).strong().color(TEXT_PRIMARY));

                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            if p.is_installed {
                                                let tag = format!("{} titres", p.game_count);
                                                egui::Frame::new()
                                                    .fill(p_color.gamma_multiply(0.15))
                                                    .stroke(Stroke::new(1.0, p_color.gamma_multiply(0.35)))
                                                    .corner_radius(CornerRadius::same(5))
                                                    .inner_margin(Margin::symmetric(7, 2))
                                                    .show(ui, |ui| {
                                                        ui.label(RichText::new(tag).size(11.0).strong().color(p_color));
                                                    });
                                            } else {
                                                ui.label(RichText::new("Non détecté").size(11.0).color(TEXT_TERTIARY));
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

                    // Chemin d'installation Steam
                    ui.label(RichText::new("Chemin d'installation Steam").size(12.5).color(TEXT_PRIMARY));
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let w = (ui.available_width() - 110.0).max(120.0);
                        changed |= ui
                            .add(
                                egui::TextEdit::singleline(&mut self.settings.steam_path)
                                    .hint_text("Automatique (C:\\Program Files (x86)\\Steam)")
                                    .margin(Margin::symmetric(10, 7))
                                    .desired_width(w),
                            )
                            .changed();
                        if action_chip(ui, "Parcourir…", th) {
                            if let Some(p) = rfd::FileDialog::new().pick_folder() {
                                self.settings.steam_path = p.to_string_lossy().to_string();
                                changed = true;
                                rescan = true;
                            }
                        }
                    });

                    ui.add_space(10.0);

                    // Chemin manifestes Epic Games
                    ui.label(RichText::new("Dossier des manifestes Epic Games").size(12.5).color(TEXT_PRIMARY));
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        let w = (ui.available_width() - 110.0).max(120.0);
                        changed |= ui
                            .add(
                                egui::TextEdit::singleline(&mut self.settings.epic_manifests_path)
                                    .hint_text("Automatique (C:\\ProgramData\\Epic\\...)")
                                    .margin(Margin::symmetric(10, 7))
                                    .desired_width(w),
                            )
                            .changed();
                        if action_chip(ui, "Parcourir…", th) {
                            if let Some(p) = rfd::FileDialog::new().pick_folder() {
                                self.settings.epic_manifests_path = p.to_string_lossy().to_string();
                                changed = true;
                                rescan = true;
                            }
                        }
                    });

                    ui.add_space(14.0);

                    // Dossiers personnalisés scannés
                    ui.label(
                        RichText::new("Dossiers personnalisés scannés (Jeux externes, GOG, ROMs...)")
                            .size(12.5)
                            .color(TEXT_PRIMARY),
                    );
                    ui.add_space(4.0);

                    let mut remove_folder = None;
                    if self.settings.custom_scan_folders.is_empty() {
                        ui.label(RichText::new("Aucun dossier personnalisé ajouté pour le moment.").size(11.5).color(TEXT_TERTIARY));
                    } else {
                        for (idx, folder) in self.settings.custom_scan_folders.iter().enumerate() {
                            egui::Frame::new()
                                .fill(th.surface_1)
                                .stroke(Stroke::new(1.0, th.border_subtle))
                                .corner_radius(CornerRadius::same(6))
                                .inner_margin(Margin::symmetric(10, 6))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("📁").size(12.0));
                                        ui.label(
                                            RichText::new(folder.to_string_lossy().to_string())
                                                .size(11.5)
                                                .color(TEXT_PRIMARY),
                                        );
                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            let del_btn = egui::Button::new(
                                                RichText::new("✕").size(11.0).color(ERROR),
                                            )
                                            .fill(ERROR.gamma_multiply(0.12))
                                            .stroke(Stroke::new(1.0, ERROR.gamma_multiply(0.30)))
                                            .corner_radius(CornerRadius::same(4));
                                            if ui.add(del_btn).clicked() {
                                                remove_folder = Some(idx);
                                            }
                                        });
                                    });
                                });
                            ui.add_space(3.0);
                        }
                    }

                    if let Some(idx) = remove_folder {
                        self.settings.custom_scan_folders.remove(idx);
                        changed = true;
                        rescan = true;
                    }

                    ui.add_space(6.0);
                    if action_chip(ui, "➕  Ajouter un dossier de jeux à scanner…", th) {
                        if let Some(p) = rfd::FileDialog::new()
                            .set_title("Sélectionner un dossier de jeux")
                            .pick_folder()
                        {
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

                // ═════════════════════════════════════════════════════════════
                // 6. Game Performance & Overlay Flottant (F9)
                // ═════════════════════════════════════════════════════════════
                section(ui, "📊  Game Performance & Overlay Flottant (F9)", th, |ui| {
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

                    // Position de l'overlay
                    row(
                        ui,
                        "Position de l'overlay à l'écran",
                        "Choisissez le coin de l'écran où afficher le moniteur de performances.",
                        |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 4.0;
                                for (pos, lbl) in [
                                    (crate::performance::OverlayPosition::TopRight, "Haut Droit"),
                                    (crate::performance::OverlayPosition::TopLeft, "Haut Gauche"),
                                    (crate::performance::OverlayPosition::BottomRight, "Bas Droit"),
                                    (crate::performance::OverlayPosition::BottomLeft, "Bas Gauche"),
                                ] {
                                    let is_sel = self.settings.overlay.position == pos;
                                    let btn = egui::Button::new(
                                        RichText::new(lbl)
                                            .size(11.5)
                                            .color(if is_sel { WHITE } else { TEXT_SECONDARY }),
                                    )
                                    .fill(if is_sel { accent.gamma_multiply(0.30) } else { th.surface_2 })
                                    .stroke(Stroke::new(1.0, if is_sel { accent } else { th.border_default }))
                                    .corner_radius(CornerRadius::same(6))
                                    .min_size(vec2(0.0, 24.0));

                                    if ui.add(btn).clicked() {
                                        self.settings.overlay.position = pos;
                                        changed = true;
                                    }
                                }
                            });
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);

                    // Opacité de l'overlay
                    row(
                        ui,
                        "Opacité / Transparence de l'overlay",
                        "Règle le niveau de transparence du fond pour ne pas gêner en jeu.",
                        |ui| {
                            let mut op = self.settings.overlay.opacity;
                            if ui
                                .add(
                                    egui::Slider::new(&mut op, 0.20..=1.00)
                                        .step_by(0.05)
                                        .custom_formatter(|n, _| format!("{:.0}%", n * 100.0))
                                        .custom_parser(|s| {
                                            s.trim_end_matches('%')
                                                .trim()
                                                .parse::<f64>()
                                                .ok()
                                                .map(|v| (v / 100.0).clamp(0.20, 1.00))
                                        }),
                                )
                                .changed()
                            {
                                self.settings.overlay.opacity = op;
                                changed = true;
                            }
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);

                    // Échelle de l'overlay
                    row(
                        ui,
                        "Échelle / Taille de l'overlay",
                        "Ajustez la taille d'affichage générale du widget HUD.",
                        |ui| {
                            let mut sc = self.settings.overlay.scale;
                            if ui
                                .add(
                                    egui::Slider::new(&mut sc, 0.75..=1.50)
                                        .step_by(0.05)
                                        .suffix("x"),
                                )
                                .changed()
                            {
                                self.settings.overlay.scale = sc;
                                changed = true;
                            }
                        },
                    );
                    ui.separator();
                    ui.add_space(6.0);

                    // Métriques affichées
                    ui.label(RichText::new("Métriques affichées sur l'overlay :").size(12.5).strong().color(TEXT_PRIMARY));
                    ui.add_space(6.0);
                    ui.horizontal_wrapped(|ui| {
                        changed |= ui.checkbox(&mut self.settings.overlay.show_fps, "FPS & 1% Low").changed();
                        changed |= ui.checkbox(&mut self.settings.overlay.show_frametime, "Frametime (ms)").changed();
                        changed |= ui.checkbox(&mut self.settings.overlay.show_cpu, "CPU (Usage, GHz, Temp)").changed();
                        changed |= ui.checkbox(&mut self.settings.overlay.show_gpu, "GPU (Usage, VRAM)").changed();
                        changed |= ui.checkbox(&mut self.settings.overlay.show_ram, "RAM (Go utilisée)").changed();
                    });
                });

                // ═════════════════════════════════════════════════════════════
                // 7. Succès Steam & API Web Officielle
                // ═════════════════════════════════════════════════════════════
                section(ui, "🏆  Succès Steam & API Web Officielle", th, |ui| {
                    ui.label(RichText::new("Configuration du suivi des succès Steam").size(13.0).strong().color(TEXT_PRIMARY));
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new("UniLauncher récupère déjà le catalogue officiel de succès. Pour synchroniser vos trophées personnels privés, indiquez votre clé API Steam et votre SteamID64.")
                            .size(11.5)
                            .color(TEXT_SECONDARY),
                    );
                    ui.add_space(10.0);

                    ui.label(RichText::new("Clé Web API Steam").size(12.5).color(TEXT_PRIMARY));
                    ui.add_space(3.0);
                    let w = (ui.available_width() - 140.0).max(120.0);
                    ui.horizontal(|ui| {
                        changed |= ui
                            .add(
                                egui::TextEdit::singleline(&mut self.settings.steam_api_key)
                                    .hint_text("Collez ici votre clé API Steam (facultatif)...")
                                    .password(true)
                                    .margin(Margin::symmetric(10, 7))
                                    .desired_width(w),
                            )
                            .changed();
                        if action_chip(ui, "Obtenir ma clé", th) {
                            let _ = open_uri("https://steamcommunity.com/dev/apikey");
                        }
                    });

                    ui.add_space(8.0);
                    ui.label(RichText::new("SteamID64 de votre compte").size(12.5).color(TEXT_PRIMARY));
                    ui.add_space(3.0);
                    ui.horizontal(|ui| {
                        changed |= ui
                            .add(
                                egui::TextEdit::singleline(&mut self.settings.steam_user_id)
                                    .hint_text("76561198... (facultatif si profil public)")
                                    .margin(Margin::symmetric(10, 7))
                                    .desired_width(w),
                            )
                            .changed();
                        if action_chip(ui, "Trouver mon ID", th) {
                            let _ = open_uri("https://steamid.io/");
                        }
                    });
                });

                // ═════════════════════════════════════════════════════════════
                // 8. Maintenance & Données Locales
                // ═════════════════════════════════════════════════════════════
                section(ui, "💾  Maintenance & Données Locales", th, |ui| {
                    egui::Frame::new()
                        .fill(th.surface_1)
                        .stroke(Stroke::new(1.0, th.border_subtle))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(Margin::symmetric(10, 6))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("📂").size(12.0));
                                ui.label(
                                    RichText::new(format!(
                                        "Dossier de configuration : {}",
                                        Settings::dir().display()
                                    ))
                                    .size(11.5)
                                    .color(TEXT_SECONDARY),
                                );
                            });
                        });

                    ui.add_space(10.0);

                    ui.horizontal_wrapped(|ui| {
                        if action_chip(ui, "📂 Ouvrir le dossier de données", th) {
                            let _ = open_uri(&Settings::dir().to_string_lossy());
                        }
                        if action_chip(ui, "📤 Exporter les paramètres (JSON)…", th) {
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
                        if action_chip(ui, "📥 Importer une configuration…", th) {
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
                        if action_chip(ui, "⭐ Réinitialiser les favoris", th) {
                            self.settings.favorites.clear();
                            changed = true;
                            self.toast("Liste des favoris vidée");
                        }
                        if action_chip(ui, "👁 Afficher tous les jeux masqués", th) {
                            self.settings.hidden_games.clear();
                            changed = true;
                            self.toast("Tous les jeux masqués ont été restaurés");
                        }
                    });
                });

                // ═════════════════════════════════════════════════════════════
                // 9. Mises à jour du Logiciel & Auto-Update
                // ═════════════════════════════════════════════════════════════
                section(ui, "🔄  Mises à jour du Logiciel & Auto-Update", th, |ui| {
                    let cur_status = {
                        let st = self.update_mgr.status.lock().unwrap();
                        st.clone()
                    };

                    egui::Frame::new()
                        .fill(th.surface_1)
                        .stroke(Stroke::new(1.0, th.border_subtle))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(Margin::symmetric(14, 10))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Version installée :").size(13.0).strong().color(TEXT_PRIMARY));
                                        ui.label(RichText::new(format!("v{}", crate::update::CURRENT_VERSION)).size(13.0).strong().color(accent));
                                        ui.add_space(8.0);
                                        let (tag_r, _) = ui.allocate_exact_size(vec2(72.0, 18.0), Sense::hover());
                                        ui.painter().rect_filled(tag_r, CornerRadius::same(4), th.surface_2);
                                        ui.painter().text(
                                            tag_r.center(),
                                            egui::Align2::CENTER_CENTER,
                                            "CANAL STABLE",
                                            FontId::proportional(9.0),
                                            TEXT_TERTIARY,
                                        );
                                    });
                                    ui.add_space(3.0);
                                    ui.label(
                                        RichText::new("Distribué via GitHub Releases officiel avec vérification d'intégrité.")
                                            .size(11.5)
                                            .color(TEXT_SECONDARY),
                                    );
                                });

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if primary_chip(ui, "🔍 Rechercher des mises à jour", accent) {
                                        self.update_mgr.check_for_updates();
                                    }
                                });
                            });
                        });

                    ui.add_space(12.0);

                    match cur_status {
                        crate::update::UpdateStatus::Idle => {
                            ui.label(RichText::new("Cliquez sur « Rechercher des mises à jour » pour vérifier la dernière release disponible.").size(12.0).color(TEXT_TERTIARY));
                        }
                        crate::update::UpdateStatus::Checking => {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.add_space(6.0);
                                ui.label(RichText::new("Interrogation de l'API GitHub Releases...").size(12.5).color(TEXT_SECONDARY));
                            });
                        }
                        crate::update::UpdateStatus::UpToDate { version } => {
                            egui::Frame::new()
                                .fill(SUCCESS.gamma_multiply(0.12))
                                .stroke(Stroke::new(1.0, SUCCESS.gamma_multiply(0.40)))
                                .corner_radius(CornerRadius::same(6))
                                .inner_margin(Margin::symmetric(12, 8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("✓").size(14.0).strong().color(SUCCESS));
                                        ui.label(RichText::new(format!("UniLauncher est parfaitement à jour (v{}).", version)).size(12.5).color(TEXT_PRIMARY));
                                    });
                                });
                        }
                        crate::update::UpdateStatus::UpdateAvailable(info) => {
                            egui::Frame::new()
                                .fill(accent.gamma_multiply(0.12))
                                .stroke(Stroke::new(1.0, accent.gamma_multiply(0.50)))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::same(12))
                                .show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new("★ Nouvelle version disponible !").size(14.0).strong().color(accent));
                                            ui.label(RichText::new(format!("v{}", info.version)).size(13.0).strong().color(WHITE));
                                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                let btn_text = format!("⬇ Télécharger et installer ({} Mo)", (info.asset_size as f64 / 1_048_576.0).round());
                                                if primary_chip(ui, &btn_text, accent) {
                                                    self.update_mgr.start_download(info.clone());
                                                }
                                            });
                                        });

                                        ui.add_space(6.0);
                                        ui.label(RichText::new(&info.title).size(12.5).strong().color(TEXT_PRIMARY));

                                        if !info.changelog.is_empty() {
                                            ui.add_space(6.0);
                                            egui::Frame::new()
                                                .fill(th.surface_2)
                                                .corner_radius(CornerRadius::same(6))
                                                .inner_margin(Margin::same(8))
                                                .show(ui, |ui| {
                                                    egui::ScrollArea::vertical()
                                                        .id_salt("changelog_scroll")
                                                        .max_height(100.0)
                                                        .show(ui, |ui| {
                                                            ui.label(RichText::new(&info.changelog).size(11.5).color(TEXT_SECONDARY));
                                                        });
                                                });
                                        }
                                    });
                                });
                        }
                        crate::update::UpdateStatus::Downloading { version, percent, downloaded_bytes, total_bytes } => {
                            egui::Frame::new()
                                .fill(th.surface_1)
                                .stroke(Stroke::new(1.0, th.border_subtle))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::same(12))
                                .show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(format!("Téléchargement de la v{}...", version)).size(13.0).strong().color(TEXT_PRIMARY));
                                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                ui.label(RichText::new(format!("{:.1} / {:.1} Mo ({:.0}%)", downloaded_bytes as f64 / 1_048_576.0, total_bytes as f64 / 1_048_576.0, percent * 100.0)).size(12.0).color(TEXT_SECONDARY));
                                            });
                                        });

                                        ui.add_space(8.0);
                                        let (bar_r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 10.0), Sense::hover());
                                        ui.painter().rect_filled(bar_r, CornerRadius::same(5), th.surface_2);
                                        let fill_w = bar_r.width() * percent.clamp(0.0, 1.0);
                                        if fill_w > 0.0 {
                                            let fill_r = Rect::from_min_size(bar_r.min, vec2(fill_w, bar_r.height()));
                                            ui.painter().rect_filled(fill_r, CornerRadius::same(5), accent);
                                        }
                                    });
                                });
                        }
                        crate::update::UpdateStatus::ReadyToRestart { version, installer_path } => {
                            egui::Frame::new()
                                .fill(SUCCESS.gamma_multiply(0.15))
                                .stroke(Stroke::new(1.0, SUCCESS.gamma_multiply(0.50)))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::same(12))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.vertical(|ui| {
                                            ui.label(RichText::new("✓ Mise à jour prête à être appliquée !").size(13.5).strong().color(SUCCESS));
                                            ui.label(RichText::new(format!("La version v{} a été vérifiée et téléchargée.", version)).size(11.5).color(TEXT_SECONDARY));
                                        });

                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            if primary_chip(ui, "🔄 Redémarrer et installer", SUCCESS) {
                                                let _ = self.update_mgr.apply_update_and_restart(&installer_path);
                                            }
                                        });
                                    });
                                });
                        }
                        crate::update::UpdateStatus::Error(err) => {
                            egui::Frame::new()
                                .fill(ERROR.gamma_multiply(0.12))
                                .stroke(Stroke::new(1.0, ERROR.gamma_multiply(0.40)))
                                .corner_radius(CornerRadius::same(6))
                                .inner_margin(Margin::symmetric(12, 8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("✕").size(14.0).strong().color(ERROR));
                                        ui.label(RichText::new(&err).size(12.0).color(TEXT_PRIMARY));
                                    });
                                });
                        }
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SortMode, Tab, ViewMode};
    use std::collections::{HashMap, HashSet};
    use std::sync::mpsc;

    impl Launcher {
        pub fn test_instance() -> Self {
            let (playtime_tx, playtime_rx) = mpsc::channel();
            let (tex_tx, tex_rx) = mpsc::channel();
            let (rpc_tx, _rpc_rx) = mpsc::channel();
            let (_rpc_status_tx, rpc_status_rx) = mpsc::channel();
            let (deals_tx, deals_rx) = mpsc::channel();
            let (deal_tex_tx, deal_tex_rx) = mpsc::channel();
            let (scan_tx, scan_rx) = mpsc::channel();
            let (ach_tx, ach_rx) = mpsc::channel();
            let (details_tx, details_rx) = mpsc::channel();
            let (banner_tx, banner_rx) = mpsc::channel();
            let perf_monitor = crate::performance::PerformanceMonitor::new();
            let settings = Settings::default();
            Self {
                settings,
                games: vec![],
                tab: Tab::Settings,
                sort: SortMode::Playtime,
                view_mode: ViewMode::Grid,
                search: String::new(),
                focus_search: false,
                is_fullscreen: false,
                status: None,
                applied_accent: 0,
                applied_theme: 0,
                is_big_picture: false,
                big_picture_index: 0,
                covers: HashMap::new(),
                loading_covers: HashSet::new(),
                banners: HashMap::new(),
                loading_banners: HashSet::new(),
                banner_tx,
                banner_rx,
                tex_tx,
                tex_rx,
                playtime_rx,
                playtime_tx,
                rpc_tx,
                rpc_status_rx,
                rpc_status: RpcStatus::Disabled,
                active_game: None,
                selected_game_id: None,
                properties_tab: 0,
                show_discord_guide: false,
                new_tag_input: String::new(),
                spotlight_open: false,
                spotlight_query: String::new(),
                spotlight_selected_idx: 0,
                spotlight_focus_needed: false,
                deals: vec![],
                loading_deals: false,
                deals_tx,
                deals_rx,
                deals_search: String::new(),
                deals_active_tab: crate::deals::DealsTab::All,
                deals_platform_filter: None,
                deals_sort_mode: crate::deals::DealsSortMode::Featured,
                selected_deal_modal: None,
                hero_featured_index: 0,
                last_deals_sync: None,
                deal_textures: HashMap::new(),
                loading_deal_textures: HashSet::new(),
                deal_tex_tx,
                deal_tex_rx,
                centered_once: false,
                scan_rx,
                scan_tx,
                scan_progress: None,
                is_scanning: false,
                cached_achievements: HashMap::new(),
                loading_achievements: HashSet::new(),
                ach_tx,
                ach_rx,
                achievements_filter: 0,
                details_tx,
                details_rx,
                loading_details: HashSet::new(),
                cached_screenshots: HashMap::new(),
                selected_screenshot: None,
                perf_monitor,
                edit_info_open: false,
                edit_name_input: String::new(),
                edit_desc_input: String::new(),
                edit_date_input: String::new(),
                update_mgr: crate::update::UpdateManager::new(),
            }
        }
    }

    #[test]
    fn test_settings_ui_runs_without_panic() {
        let mut launcher = Launcher::test_instance();
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                launcher.settings_ui(ui);
            });
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_all_theme_indices_valid_and_no_recursion() {
        for idx in 0..=16 {
            let th = get_theme(idx);
            assert!(th.bg != egui::Color32::TRANSPARENT);
        }
    }

    #[test]
    fn test_stats_ui_runs_without_panic() {
        let mut launcher = Launcher::test_instance();
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                launcher.stats_ui(ui);
            });
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_deals_ui_runs_without_panic() {
        let mut launcher = Launcher::test_instance();
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                launcher.deals_ui(ui);
            });
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_library_ui_runs_without_panic() {
        let mut launcher = Launcher::test_instance();
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let _ = launcher.library_ui(ui);
            });
        });
        output.textures_delta.clear();

        // Mode Liste
        launcher.view_mode = ViewMode::List;
        let mut output2 = ctx.run_ui(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let _ = launcher.library_ui(ui);
            });
        });
        output2.textures_delta.clear();
    }

    #[test]
    fn test_emulators_and_big_picture_ui_runs_without_panic() {
        let mut launcher = Launcher::test_instance();
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let _ = launcher.emulators_ui(ui);
                let _ = launcher.big_picture_ui(ui);
            });
        });
        output.textures_delta.clear();
    }

    #[test]
    fn test_spotlight_and_properties_modal() {
        let mut launcher = Launcher::test_instance();
        let dummy_game = crate::config::Game {
            id: "steam:12345".to_string(),
            name: "Test Game".to_string(),
            source: crate::config::Source::Steam,
            install_dir: None,
            exe: None,
            launch_uri: None,
            cover_path: None,
            playtime_mins: 120,
            size_bytes: 1024 * 1024 * 500,
        };
        launcher.games.push(dummy_game);
        launcher.selected_game_id = Some("steam:12345".to_string());
        launcher.spotlight_open = true;

        let ctx = egui::Context::default();
        for tab_idx in 0..5 {
            launcher.properties_tab = tab_idx;
            let mut output = ctx.run_ui(Default::default(), |ctx| {
                let _ = launcher.properties_modal_ui(ctx);
                let _ = launcher.spotlight_ui(ctx);
            });
            output.textures_delta.clear();
        }
    }

    #[test]
    fn test_library_filtering_and_sorting() {
        let mut launcher = Launcher::test_instance();
        launcher.tab = Tab::All;
        launcher.games = vec![
            crate::config::Game {
                id: "g1".to_string(),
                name: "Zelda".to_string(),
                source: crate::config::Source::Manual,
                install_dir: None,
                exe: None,
                launch_uri: None,
                cover_path: None,
                playtime_mins: 10,
                size_bytes: 1000,
            },
            crate::config::Game {
                id: "g2".to_string(),
                name: "Apex".to_string(),
                source: crate::config::Source::Steam,
                install_dir: None,
                exe: None,
                launch_uri: None,
                cover_path: None,
                playtime_mins: 0,
                size_bytes: 5000,
            },
            crate::config::Game {
                id: "g3".to_string(),
                name: "Minecraft".to_string(),
                source: crate::config::Source::Manual,
                install_dir: None,
                exe: None,
                launch_uri: None,
                cover_path: None,
                playtime_mins: 500,
                size_bytes: 200,
            },
        ];

        launcher.settings.favorites.insert("g2".to_string());

        // Test favorites filter
        launcher.settings.status_filter = crate::types::LibraryStatusFilter::Favorites;
        let indices = launcher.filtered_indices();
        assert_eq!(indices.len(), 1);
        assert_eq!(indices[0], 1); // "Apex"

        // Test never played filter
        launcher.settings.status_filter = crate::types::LibraryStatusFilter::NeverPlayed;
        let indices = launcher.filtered_indices();
        assert_eq!(indices.len(), 1);
        assert_eq!(indices[0], 1); // "Apex" has 0 mins

        // Test all + sort by size
        launcher.settings.status_filter = crate::types::LibraryStatusFilter::All;
        launcher.sort = SortMode::SizeOnDisk;
        let indices = launcher.filtered_indices();
        assert_eq!(indices.len(), 3);
        assert_eq!(indices[0], 1); // Apex: 5000
        assert_eq!(indices[1], 0); // Zelda: 1000
        assert_eq!(indices[2], 2); // Minecraft: 200

        // Test sort NameDesc (Z-A)
        launcher.sort = SortMode::NameDesc;
        let indices = launcher.filtered_indices();
        assert_eq!(indices.len(), 3);
        assert_eq!(indices[0], 0); // Zelda
        assert_eq!(indices[1], 2); // Minecraft
        assert_eq!(indices[2], 1); // Apex
    }
}

