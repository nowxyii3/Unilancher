use crate::app::Launcher;
use crate::config::Source;
use crate::process::open_uri;
use crate::scan;
use crate::theme::{DIM, EA_C, EPIC_C, GOG_C, GOLD, GREEN, RED, STEAM_C, UBI_C, WHITE, XBOX_C};
use crate::types::{format_playtime, Action};
use crate::ui::components::{cover_colors, draw_gradient_rect, trunc};
use eframe::egui::{
    self, pos2, vec2, Align, Color32, CornerRadius, Id, Key, Layout, Margin,
    Rect, RichText, Sense, Stroke, StrokeKind,
};

impl Launcher {
    // ───────────────────────────── Modale Propriétés Réactive (Style Riot / Steam) ─────────────────────────────
    pub fn properties_modal_ui(&mut self, ctx: &egui::Context) -> Vec<Action> {
        let mut actions = vec![];
        let Some(game_id) = self.selected_game_id.clone() else {
            return actions;
        };
        let Some((idx, game)) = self
            .games
            .iter()
            .enumerate()
            .find(|(_, g)| g.id == game_id)
            .map(|(i, g)| (i, g.clone()))
        else {
            self.selected_game_id = None;
            return actions;
        };

        let th = self.theme();
        let accent = self.accent();
        let screen_rect = ctx.content_rect();
        let mut close_modal = false;
        let mut save_needed = false;

        // Fermeture automatique sur Échap
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            close_modal = true;
        }

        // ── 1. Arrière-plan assombri (Backdrop) avec clic pour fermer ──
        egui::Area::new(Id::unique("properties_modal_backdrop"))
            .order(egui::Order::Middle)
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                let (rect, resp) = ui.allocate_exact_size(screen_rect.size(), Sense::click());
                ui.painter()
                    .rect_filled(rect, CornerRadius::ZERO, Color32::from_black_alpha(195));
                if resp.clicked() {
                    close_modal = true;
                }
            });

        // ── 2. Fenêtre Modale Centrée ──
        let modal_w = 780.0f32.min(screen_rect.width() - 40.0);
        let modal_h = 630.0f32.min(screen_rect.height() - 40.0);
        let modal_pos = pos2(
            (screen_rect.width() - modal_w) / 2.0,
            (screen_rect.height() - modal_h) / 2.0,
        );

        let src_col = match game.source {
            Source::Steam => STEAM_C,
            Source::Epic => EPIC_C,
            Source::Gog => GOG_C,
            Source::Ubisoft => UBI_C,
            Source::Ea => EA_C,
            Source::Xbox => XBOX_C,
            Source::Manual => accent,
        };

        egui::Area::new(Id::unique("properties_modal_window"))
            .order(egui::Order::Foreground)
            .fixed_pos(modal_pos)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(th.panel)
                    .stroke(Stroke::new(1.2, th.panel_border))
                    .corner_radius(16.0)
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 16],
                        blur: 36,
                        spread: 0,
                        color: Color32::from_black_alpha(210),
                    })
                    .inner_margin(Margin::symmetric(24, 20))
                    .show(ui, |ui| {
                        ui.set_width(modal_w - 48.0);
                        ui.set_height(modal_h - 40.0);

                        // ── EN-TÊTE : Titre, Miniature & Contrôle Fermer ──
                        ui.horizontal(|ui| {
                            // Miniature Jaquette 3D
                            let (thumb_rect, _) = ui.allocate_exact_size(vec2(44.0, 58.0), Sense::hover());
                            let thumb_p = ui.painter().with_clip_rect(thumb_rect);
                            let cover_tex = self.get_cover_texture(ctx, &game);
                            if let Some(t) = &cover_tex {
                                thumb_p.image(
                                    t.id(),
                                    thumb_rect,
                                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                    WHITE,
                                );
                            } else {
                                let (c1, c2) = cover_colors(&game.name);
                                draw_gradient_rect(&thumb_p, thumb_rect, c1, c2);
                            }
                            ui.painter().rect_stroke(
                                thumb_rect,
                                CornerRadius::same(6),
                                Stroke::new(1.0, th.card_border),
                                StrokeKind::Inside,
                            );

                            ui.add_space(14.0);

                            // Titre + Badges d'état
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(&game.name)
                                        .size(20.0)
                                        .strong()
                                        .color(WHITE),
                                );
                                ui.add_space(3.0);
                                ui.horizontal(|ui| {
                                    // Badge Source
                                    egui::Frame::new()
                                        .fill(src_col.gamma_multiply(0.20))
                                        .stroke(Stroke::new(1.0, src_col.gamma_multiply(0.60)))
                                        .corner_radius(6.0)
                                        .inner_margin(Margin::symmetric(8, 2))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(game.source.label())
                                                    .size(11.0)
                                                    .strong()
                                                    .color(WHITE),
                                            );
                                        });

                                    // Badge Temps de jeu
                                    let play_mins = self.total_playtime_mins(&game);
                                    egui::Frame::new()
                                        .fill(th.card)
                                        .stroke(Stroke::new(1.0, th.card_border))
                                        .corner_radius(6.0)
                                        .inner_margin(Margin::symmetric(8, 2))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(format_playtime(play_mins))
                                                    .size(11.0)
                                                    .color(DIM),
                                            );
                                        });

                                    // Badge Taille Disque
                                    if game.size_bytes > 0 {
                                        egui::Frame::new()
                                            .fill(th.card)
                                            .stroke(Stroke::new(1.0, th.card_border))
                                            .corner_radius(6.0)
                                            .inner_margin(Margin::symmetric(8, 2))
                                            .show(ui, |ui| {
                                                ui.label(
                                                    RichText::new(scan::format_size(game.size_bytes))
                                                        .size(11.0)
                                                        .color(GOLD),
                                                );
                                            });
                                    }
                                });
                            });

                            // Bouton Fermer Riot Games Style (✕)
                            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                let (close_rect, close_resp) =
                                    ui.allocate_exact_size(vec2(32.0, 32.0), Sense::click());
                                let hov = close_resp.hovered();
                                if hov {
                                    ui.painter().rect_filled(
                                        close_rect,
                                        CornerRadius::same(8),
                                        Color32::from_rgb(232, 17, 35),
                                    );
                                }
                                let close_col = if hov { WHITE } else { DIM };
                                let c = close_rect.center();
                                let st = Stroke::new(1.4, close_col);
                                ui.painter().line_segment([c + vec2(-4.0, -4.0), c + vec2(4.0, 4.0)], st);
                                ui.painter().line_segment([c + vec2(4.0, -4.0), c + vec2(-4.0, 4.0)], st);
                                if close_resp.clicked() {
                                    close_modal = true;
                                }
                            });
                        });

                        ui.add_space(14.0);

                        // ── BARRE D'ONGLETS REACT / TAILWIND TABS ──
                        ui.horizontal(|ui| {
                            egui::Frame::new()
                                .fill(th.card)
                                .stroke(Stroke::new(1.0, th.card_border))
                                .corner_radius(10.0)
                                .inner_margin(Margin::same(3))
                                .show(ui, |ui| {
                                    ui.spacing_mut().item_spacing.x = 2.0;
                                    let tabs = [
                                        (0, "⚙ Général"),
                                        (1, "🚀 Lancement & Boost"),
                                        (2, "🎨 Illustration HD"),
                                        (3, "💾 Sauvegardes"),
                                        (4, "📝 Notes & Communauté"),
                                    ];
                                    for (idx, lbl) in tabs {
                                        let sel = self.properties_tab == idx;
                                        let bg = if sel {
                                            accent.gamma_multiply(0.28)
                                        } else {
                                            Color32::TRANSPARENT
                                        };
                                        let fg = if sel { WHITE } else { DIM };
                                        let btn = egui::Button::new(
                                            RichText::new(lbl).size(12.0).strong().color(fg),
                                        )
                                        .fill(bg)
                                        .stroke(Stroke::new(
                                            1.0,
                                            if sel {
                                                accent.gamma_multiply(0.8)
                                            } else {
                                                Color32::TRANSPARENT
                                            },
                                        ))
                                        .corner_radius(8.0);
                                        if ui.add(btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                            self.properties_tab = idx;
                                        }
                                    }
                                });
                        });

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // ── CONTENU DÉROULANT DE L'ONGLET ──
                        let content_h = modal_h - 220.0;
                        egui::ScrollArea::vertical()
                            .id_salt("properties_modal_scroll")
                            .max_height(content_h)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                match self.properties_tab {
                                    // ──────────────── 0. GÉNÉRAL ────────────────
                                    0 => {
                                        ui.add_space(6.0);
                                        ui.label(RichText::new("Cible Exécutable (.exe)").size(13.5).strong().color(WHITE));
                                        ui.add_space(4.0);

                                        let current_exe = self
                                            .settings
                                            .exe_overrides
                                            .get(&game.id)
                                            .cloned()
                                            .or_else(|| game.exe.clone());

                                        let exe_exists = current_exe
                                            .as_ref()
                                            .map(|p| p.exists())
                                            .unwrap_or(false);

                                        egui::Frame::new()
                                            .fill(th.card)
                                            .stroke(Stroke::new(1.0, th.card_border))
                                            .corner_radius(8.0)
                                            .inner_margin(Margin::symmetric(12, 10))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    let path_str = current_exe
                                                        .as_ref()
                                                        .map(|p| p.to_string_lossy().to_string())
                                                        .unwrap_or_else(|| "Aucun exécutable assigné (lancement par URI ou magasin)".into());
                                                    ui.label(RichText::new(trunc(&path_str, 58)).size(12.0).monospace().color(if exe_exists { WHITE } else { DIM }));

                                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                        if exe_exists {
                                                            ui.label(RichText::new("✓ Valide").size(11.0).color(GREEN));
                                                        } else if current_exe.is_some() {
                                                            ui.label(RichText::new("⚠ Introuvable").size(11.0).color(RED));
                                                        }
                                                    });
                                                });
                                            });

                                        ui.add_space(6.0);
                                        ui.horizontal(|ui| {
                                            if ui.button("📁 Parcourir un exécutable…").clicked() {
                                                if let Some(p) = rfd::FileDialog::new()
                                                    .set_title("Sélectionner l'exécutable du jeu (.exe)")
                                                    .add_filter("Exécutables", &["exe", "bat", "cmd"])
                                                    .pick_file()
                                                {
                                                    self.settings.exe_overrides.insert(game.id.clone(), p);
                                                    save_needed = true;
                                                    self.toast("Exécutable mis à jour");
                                                }
                                            }
                                            if self.settings.exe_overrides.contains_key(&game.id)
                                                && ui.button("🔄 Réinitialiser l'exécutable").clicked()
                                            {
                                                self.settings.exe_overrides.remove(&game.id);
                                                save_needed = true;
                                                self.toast("Exécutable réinitialisé");
                                            }
                                            if ui.button("🧪 Tester le lancement").clicked() {
                                                actions.push(Action::Launch(idx));
                                            }
                                        });

                                        ui.add_space(18.0);
                                        ui.label(RichText::new("Emplacement d'Installation").size(13.5).strong().color(WHITE));
                                        ui.add_space(4.0);
                                        if let Some(dir) = &game.install_dir {
                                            egui::Frame::new()
                                                .fill(th.card)
                                                .stroke(Stroke::new(1.0, th.card_border))
                                                .corner_radius(8.0)
                                                .inner_margin(Margin::symmetric(12, 10))
                                                .show(ui, |ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.label(RichText::new(trunc(&dir.to_string_lossy(), 60)).size(12.0).color(DIM));
                                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                            if dir.exists() && ui.button("📂 Ouvrir le dossier").clicked() {
                                                                actions.push(Action::OpenDir(dir.clone()));
                                                            }
                                                        });
                                                    });
                                                });
                                        } else {
                                            ui.label(RichText::new("Dossier d'installation géré automatiquement par le client officiel").size(12.0).color(DIM));
                                        }

                                        ui.add_space(18.0);
                                        ui.label(RichText::new("Informations Internes").size(13.5).strong().color(WHITE));
                                        ui.add_space(6.0);
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(format!("Identifiant : {}", game.id)).size(11.5).color(DIM));
                                            if let Some(uri) = &game.launch_uri {
                                                ui.add_space(12.0);
                                                ui.label(RichText::new(format!("URI : {uri}")).size(11.5).color(DIM));
                                            }
                                        });
                                    }

                                    // ──────────────── 1. LANCEMENT & BOOST ────────────────
                                    1 => {
                                        ui.add_space(6.0);
                                        ui.label(RichText::new("Arguments de Ligne de Commande").size(13.5).strong().color(WHITE));
                                        ui.label(RichText::new("Options supplémentaires transmises à l'exécutable lors de son exécution").size(11.5).color(DIM));
                                        ui.add_space(8.0);

                                        let mut args = self
                                            .settings
                                            .launch_args
                                            .get(&game.id)
                                            .cloned()
                                            .unwrap_or_default();

                                        let resp = ui.add(
                                            egui::TextEdit::singleline(&mut args)
                                                .hint_text("-novid -high -dx11 -windowed")
                                                .desired_width(f32::INFINITY)
                                                .margin(Margin::symmetric(12, 10)),
                                        );
                                        if resp.changed() {
                                            if args.trim().is_empty() {
                                                self.settings.launch_args.remove(&game.id);
                                            } else {
                                                self.settings.launch_args.insert(game.id.clone(), args.clone());
                                            }
                                            save_needed = true;
                                        }

                                        ui.add_space(10.0);
                                        ui.label(RichText::new("Raccourcis rapides (Cliquer pour ajouter / retirer) :").size(11.5).color(DIM));
                                        ui.add_space(4.0);
                                        ui.horizontal_wrapped(|ui| {
                                            let quick_args = [
                                                ("-novid", "Passer les cinématiques d'introduction"),
                                                ("-high", "Priorité CPU haute"),
                                                ("-dx11", "Forcer DirectX 11"),
                                                ("-dx12", "Forcer DirectX 12"),
                                                ("-windowed", "Mode fenêtré"),
                                                ("-fullscreen", "Plein écran exclusif"),
                                                ("-threads 4", "Multi-threading 4 cœurs"),
                                            ];
                                            for (flag, desc) in quick_args {
                                                let contains = args.contains(flag);
                                                let col = if contains { accent } else { th.card };
                                                let fg = if contains { WHITE } else { DIM };
                                                let btn = egui::Button::new(RichText::new(flag).size(11.5).color(fg))
                                                    .fill(col.gamma_multiply(if contains { 0.35 } else { 0.7 }))
                                                    .stroke(Stroke::new(1.0, if contains { accent } else { th.card_border }))
                                                    .corner_radius(6.0);
                                                if ui.add(btn).on_hover_text(desc).clicked() {
                                                    if contains {
                                                        args = args.replace(flag, "").trim().to_string();
                                                    } else {
                                                        if !args.is_empty() && !args.ends_with(' ') {
                                                            args.push(' ');
                                                        }
                                                        args.push_str(flag);
                                                    }
                                                    self.settings.launch_args.insert(game.id.clone(), args.clone());
                                                    save_needed = true;
                                                }
                                            }
                                        });

                                        ui.add_space(20.0);
                                        ui.separator();
                                        ui.add_space(12.0);

                                        ui.label(RichText::new("Optimisation & Mode Game Booster").size(13.5).strong().color(WHITE));
                                        ui.add_space(6.0);
                                        egui::Frame::new()
                                            .fill(th.card)
                                            .stroke(Stroke::new(1.0, th.card_border))
                                            .corner_radius(8.0)
                                            .inner_margin(Margin::same(14))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.vertical(|ui| {
                                                        ui.label(RichText::new("Activer le Plan Haute Performance Windows").size(13.0).strong().color(WHITE));
                                                        ui.add_space(2.0);
                                                        ui.label(RichText::new("Bascule automatiquement le système en performances ultimes au démarrage du jeu").size(11.5).color(DIM));
                                                    });
                                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                        let mut active = self.settings.performance_mode;
                                                        if ui.checkbox(&mut active, "").changed() {
                                                            self.settings.performance_mode = active;
                                                            save_needed = true;
                                                        }
                                                    });
                                                });
                                            });
                                    }

                                    // ──────────────── 2. ILLUSTRATION HD ────────────────
                                    2 => {
                                        ui.add_space(6.0);
                                        ui.horizontal(|ui| {
                                            // Grande prévisualisation Jaquette
                                            let (card_rect, _) = ui.allocate_exact_size(vec2(130.0, 185.0), Sense::hover());
                                            let card_p = ui.painter().with_clip_rect(card_rect);
                                            let cover_tex = self.get_cover_texture(ctx, &game);
                                            if let Some(t) = &cover_tex {
                                                card_p.image(
                                                    t.id(),
                                                    card_rect,
                                                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                                    WHITE,
                                                );
                                            } else {
                                                let (c1, c2) = cover_colors(&game.name);
                                                draw_gradient_rect(&card_p, card_rect, c1, c2);
                                            }
                                            ui.painter().rect_stroke(
                                                card_rect,
                                                CornerRadius::same(10),
                                                Stroke::new(1.2, th.card_border),
                                                StrokeKind::Inside,
                                            );

                                            ui.add_space(18.0);
                                            ui.vertical(|ui| {
                                                ui.label(RichText::new("Personnalisation de la Jaquette").size(14.0).strong().color(WHITE));
                                                ui.add_space(4.0);
                                                ui.label(RichText::new("UniLauncher prend en charge les illustrations personnalisées haute résolution (PNG, JPG, WEBP).").size(12.0).color(DIM));
                                                ui.add_space(12.0);

                                                if ui.button("🌐 Télécharger automatiquement en HD (SteamGridDB)").clicked() {
                                                    actions.push(Action::DownloadCover(game.id.clone()));
                                                }
                                                ui.add_space(6.0);
                                                if ui.button("📁 Choisir une image sur le PC…").clicked() {
                                                    actions.push(Action::PickCover(game.id.clone()));
                                                }
                                                if self.settings.cover_overrides.contains_key(&game.id) {
                                                    ui.add_space(6.0);
                                                    if ui.button("🗑️ Réinitialiser l'illustration officielle").clicked() {
                                                        actions.push(Action::ClearCover(game.id.clone()));
                                                    }
                                                }
                                            });
                                        });
                                    }

                                    // ──────────────── 3. SAUVEGARDES ────────────────
                                    3 => {
                                        ui.add_space(6.0);
                                        ui.label(RichText::new("Gestionnaire de Sauvegardes Locales (Cloud & Backup)").size(13.5).strong().color(WHITE));
                                        ui.label(RichText::new("Créez des copies de secours de vos progressions avant un modding ou une mise à jour.").size(11.5).color(DIM));
                                        ui.add_space(12.0);

                                        let detected_save = scan::find_save_folder(&game);
                                        egui::Frame::new()
                                            .fill(th.card)
                                            .stroke(Stroke::new(1.0, th.card_border))
                                            .corner_radius(8.0)
                                            .inner_margin(Margin::same(14))
                                            .show(ui, |ui| {
                                                if let Some(save_p) = &detected_save {
                                                    ui.horizontal(|ui| {
                                                        ui.vertical(|ui| {
                                                            ui.label(RichText::new("✓ Dossier de sauvegarde localisé").size(12.5).strong().color(GREEN));
                                                            ui.add_space(2.0);
                                                            ui.label(RichText::new(trunc(&save_p.to_string_lossy(), 64)).size(11.5).color(DIM));
                                                        });
                                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                            if ui.button("📂 Ouvrir").clicked() {
                                                                actions.push(Action::OpenDir(save_p.clone()));
                                                            }
                                                        });
                                                    });
                                                } else {
                                                    ui.label(RichText::new("Dossier de sauvegarde standard non identifié automatiquement.").size(12.0).color(DIM));
                                                }
                                            });

                                        ui.add_space(10.0);
                                        if ui.button("💾 Créer une sauvegarde instantanée (Backup)").clicked() {
                                            actions.push(Action::BackupSave(game.id.clone()));
                                        }

                                        let backup_root = dirs::document_dir()
                                            .map(|d| d.join("UniLauncher_Backups"))
                                            .unwrap_or_else(|| std::path::PathBuf::from("UniLauncher_Backups"));

                                        ui.add_space(16.0);
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(format!("Répertoire de stockage des sauvegardes : {}", backup_root.display())).size(11.0).color(DIM));
                                            if backup_root.exists() && ui.small_button("📂 Ouvrir").clicked() {
                                                actions.push(Action::OpenDir(backup_root));
                                            }
                                        });
                                    }

                                    // ──────────────── 4. NOTES & COMMUNAUTÉ ────────────────
                                    4 => {
                                        ui.add_space(6.0);
                                        ui.label(RichText::new("Notes Personnelles").size(13.5).strong().color(WHITE));
                                        ui.label(RichText::new("Vos configurations, astuces, codes et objectifs sur ce titre").size(11.5).color(DIM));
                                        ui.add_space(6.0);

                                        let mut notes = self
                                            .settings
                                            .game_notes
                                            .get(&game.id)
                                            .cloned()
                                            .unwrap_or_default();

                                        let notes_resp = ui.add(
                                            egui::TextEdit::multiline(&mut notes)
                                                .hint_text("Astuces, mods installés, réglages d'affichage, objectifs...")
                                                .desired_rows(4)
                                                .desired_width(f32::INFINITY)
                                                .margin(Margin::symmetric(10, 8)),
                                        );
                                        if notes_resp.changed() {
                                            if notes.trim().is_empty() {
                                                self.settings.game_notes.remove(&game.id);
                                            } else {
                                                self.settings.game_notes.insert(game.id.clone(), notes);
                                            }
                                            save_needed = true;
                                        }

                                        ui.add_space(16.0);
                                        ui.label(RichText::new("Tags & Catégories Personnalisées").size(13.5).strong().color(WHITE));
                                        ui.add_space(6.0);

                                        let mut current_tags = self
                                            .settings
                                            .game_tags
                                            .get(&game.id)
                                            .cloned()
                                            .unwrap_or_default();

                                        if !current_tags.is_empty() {
                                            let mut to_remove = None;
                                            ui.horizontal_wrapped(|ui| {
                                                for (idx, tag) in current_tags.iter().enumerate() {
                                                    let label = format!("{tag} ✕");
                                                    if ui.button(RichText::new(label).size(11.0).color(WHITE)).clicked() {
                                                        to_remove = Some(idx);
                                                    }
                                                }
                                            });
                                            if let Some(i) = to_remove {
                                                current_tags.remove(i);
                                                if current_tags.is_empty() {
                                                    self.settings.game_tags.remove(&game.id);
                                                } else {
                                                    self.settings.game_tags.insert(game.id.clone(), current_tags.clone());
                                                }
                                                save_needed = true;
                                            }
                                        }

                                        ui.add_space(6.0);
                                        ui.horizontal(|ui| {
                                            let w = (ui.available_width() - 80.0).max(80.0);
                                            let resp = ui.add(
                                                egui::TextEdit::singleline(&mut self.new_tag_input)
                                                    .hint_text("Ajouter un tag personnalisé...")
                                                    .desired_width(w)
                                                    .margin(Margin::symmetric(8, 6)),
                                            );
                                            let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
                                            if (ui.button("Ajouter").clicked() || enter) && !self.new_tag_input.trim().is_empty() {
                                                let trimmed = self.new_tag_input.trim().to_string();
                                                if !current_tags.iter().any(|t| t.eq_ignore_ascii_case(&trimmed)) {
                                                    current_tags.push(trimmed);
                                                    self.settings.game_tags.insert(game.id.clone(), current_tags.clone());
                                                    self.new_tag_input.clear();
                                                    save_needed = true;
                                                }
                                            }
                                        });

                                        ui.add_space(4.0);
                                        ui.horizontal_wrapped(|ui| {
                                            for sugg in ["RPG", "Action", "Solo", "Coop", "Terminé", "En cours", "Favori"] {
                                                if !current_tags.iter().any(|t| t.eq_ignore_ascii_case(sugg))
                                                    && ui.small_button(format!("+ {sugg}")).clicked()
                                                {
                                                    current_tags.push(sugg.to_string());
                                                    self.settings.game_tags.insert(game.id.clone(), current_tags.clone());
                                                    save_needed = true;
                                                }
                                            }
                                        });

                                        ui.add_space(18.0);
                                        ui.separator();
                                        ui.add_space(10.0);

                                        ui.label(RichText::new("Hub Communautaire & Outils Web").size(13.5).strong().color(WHITE));
                                        ui.add_space(6.0);
                                        ui.horizontal_wrapped(|ui| {
                                            let enc = game.name.replace(' ', "%20");
                                            if ui.button("🌐 PCGamingWiki").on_hover_text("Fixes, ultrawide, FOV & patchs").clicked() {
                                                let _ = open_uri(&format!("https://www.pcgamingwiki.com/w/index.php?search={enc}"));
                                            }
                                            if ui.button("🧩 NexusMods").on_hover_text("Mods & utilitaires").clicked() {
                                                let _ = open_uri(&format!("https://www.nexusmods.com/search/?gsearch={enc}"));
                                            }
                                            if ui.button("⏱️ HowLongToBeat").on_hover_text("Durée de vie moyenne").clicked() {
                                                let _ = open_uri(&format!("https://howlongtobeat.com/?q={enc}"));
                                            }
                                            if ui.button("🐧 ProtonDB").on_hover_text("Optimisations et compatibilité").clicked() {
                                                let _ = open_uri(&format!("https://www.protondb.com/search?q={enc}"));
                                            }
                                        });
                                    }

                                    _ => {}
                                }
                            });

                        // ── PIED DE PAGE : Actions Rapides & Fermeture ──
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            let is_hidden = self.settings.hidden_games.contains(&game.id);
                            let hide_lbl = if is_hidden {
                                "👁️ Restaurer dans la bibliothèque"
                            } else {
                                "👁️ Masquer ce jeu"
                            };
                            if ui.button(hide_lbl).clicked() {
                                actions.push(Action::ToggleHide(game.id.clone()));
                                close_modal = true;
                            }

                            if game.source == Source::Manual
                                && ui.button(RichText::new("🗑️ Supprimer de la liste").color(RED)).clicked()
                            {
                                actions.push(Action::Remove(game.id.clone()));
                                close_modal = true;
                            }

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                // Bouton Jouer Principal
                                let play_btn = egui::Button::new(
                                    RichText::new("▶ Jouer maintenant").size(13.5).strong().color(WHITE),
                                )
                                .fill(accent)
                                .corner_radius(8.0)
                                .min_size(vec2(160.0, 36.0));
                                if ui.add(play_btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                    actions.push(Action::Launch(idx));
                                    close_modal = true;
                                }

                                ui.add_space(8.0);
                                if ui.button("Fermer").clicked() {
                                    close_modal = true;
                                }
                            });
                        });
                    });
            });

        if save_needed {
            self.settings.save();
        }
        if close_modal {
            self.selected_game_id = None;
        }

        actions
    }
}
