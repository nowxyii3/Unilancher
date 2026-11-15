use crate::app::Launcher;
use crate::config::Source;
use crate::scan;
use crate::theme::{
    BNET_C, EA_C, EPIC_C, ERROR, GOG_C, GOLD, STEAM_C, SUCCESS, TEXT_PRIMARY,
    TEXT_SECONDARY, TEXT_TERTIARY, UBI_C, WHITE, XBOX_C,
};
use crate::types::{format_last_played_time, format_playtime, Action};
use crate::ui::components::{cover_colors, draw_gradient_rect, draw_vertical_fade, trunc};
use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, CornerRadius, FontId, Id, Key, Layout, Margin, Rect,
    RichText, Sense, Stroke, StrokeKind, Ui,
};

impl Launcher {
    // ───────────────────────────── Page Complète de Détails Immersive (v1.1.0) ─────────────────────────────
    pub fn game_details_page_ui(&mut self, ui: &mut Ui, game_id: &str) -> Vec<Action> {
        let mut actions = vec![];
        let ctx = ui.ctx().clone();
        let th = self.theme();
        let accent = self.accent();

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

        // Raccourci Échap ou Backspace pour revenir immédiatement à la bibliothèque
        if ctx.input(|i| i.key_pressed(Key::Escape) || (!self.edit_info_open && i.key_pressed(Key::Backspace))) {
            actions.push(Action::CloseDetails);
        }

        // Déclenchement automatique de la récupération des métadonnées (description & date) en arrière-plan
        if let Some(appid) = game.id.strip_prefix("steam:") {
            if !self.settings.game_descriptions.contains_key(&game.id)
                && !self.loading_details.contains(&game.id)
            {
                let tx = self.details_tx.clone();
                let gid = game.id.clone();
                let appid_s = appid.to_string();
                self.loading_details.insert(gid.clone());
                std::thread::spawn(move || {
                    let res = scan::fetch_steam_game_details(&appid_s);
                    let _ = tx.send((gid, res));
                });
            }

            // Déclenchement automatique du chargement des succès Steam en arrière-plan
            if !self.cached_achievements.contains_key(&game.id)
                && !self.loading_achievements.contains(&game.id)
            {
                let tx = self.ach_tx.clone();
                let gid = game.id.clone();
                let appid_s = appid.to_string();
                let api_key = if self.settings.steam_api_key.trim().is_empty() {
                    None
                } else {
                    Some(self.settings.steam_api_key.clone())
                };
                let sid = if self.settings.steam_user_id.trim().is_empty() {
                    None
                } else {
                    Some(self.settings.steam_user_id.clone())
                };
                self.loading_achievements.insert(gid.clone());
                std::thread::spawn(move || {
                    let res = crate::achievements::fetch_steam_achievements(
                        &appid_s,
                        api_key.as_deref(),
                        sid.as_deref(),
                    );
                    let _ = tx.send((gid, res));
                });
            }
        }

        let src_col = match game.source {
            Source::Steam => STEAM_C,
            Source::Epic => EPIC_C,
            Source::Gog => GOG_C,
            Source::Ubisoft => UBI_C,
            Source::Ea => EA_C,
            Source::Battlenet => BNET_C,
            Source::Xbox => XBOX_C,
            Source::Manual => accent,
        };

        let is_running = self
            .active_game
            .as_ref()
            .map(|(name, _)| name == &game.name)
            .unwrap_or(false);
        let ovr = self.settings.exe_overrides.get(&game.id).cloned();
        let fixed = ovr.as_ref().map(|p| p.exists()).unwrap_or(false);
        let broken = game.is_broken() && !fixed;
        let is_fav = self.settings.favorites.contains(&game.id);
        let play_mins = self.total_playtime_mins(&game);
        let last_session_ts = self.settings.last_played_dates.get(&game.id).copied();

        // ═══════════════════════════════════════════════════════════════════════════
        // 1. BANDEAU SUPÉRIEUR DE NAVIGATION (FIL D'ARIANE & BOUTON RETOUR)
        // ═══════════════════════════════════════════════════════════════════════════
        ui.horizontal(|ui| {
            let back_btn = egui::Button::new(
                RichText::new("←  Retour à la bibliothèque")
                    .size(12.5)
                    .strong()
                    .color(TEXT_PRIMARY),
            )
            .fill(th.surface_1)
            .stroke(Stroke::new(1.0, th.border_subtle))
            .corner_radius(CornerRadius::same(6));

            if ui.add(back_btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                actions.push(Action::CloseDetails);
            }

            ui.add_space(10.0);
            ui.label(RichText::new("Bibliothèque").size(12.0).color(TEXT_TERTIARY));
            ui.label(RichText::new("›").size(12.0).color(TEXT_TERTIARY));
            ui.label(RichText::new(game.source.label()).size(12.0).color(src_col));
            ui.label(RichText::new("›").size(12.0).color(TEXT_TERTIARY));
            ui.label(RichText::new(&game.name).size(12.0).strong().color(TEXT_PRIMARY));

            // Statut en direct aligné à droite
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if is_running {
                    let (dot_r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                    ui.painter().circle_filled(dot_r.center(), 3.5, Color32::from_rgb(16, 185, 129));
                    ui.label(RichText::new("SESSION EN COURS").size(11.0).strong().color(Color32::from_rgb(16, 185, 129)));
                } else if broken {
                    let (dot_r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                    ui.painter().circle_filled(dot_r.center(), 3.5, ERROR);
                    ui.label(RichText::new("FICHIERS MANQUANTS").size(11.0).strong().color(ERROR));
                } else {
                    let (dot_r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                    ui.painter().circle_filled(dot_r.center(), 3.5, SUCCESS);
                    ui.label(RichText::new("INSTALLÉ ET PRÊT").size(11.0).strong().color(TEXT_SECONDARY));
                }
            });
        });

        ui.add_space(12.0);

        // ═══════════════════════════════════════════════════════════════════════════
        // 2. HERO BANNER IMMERSIVE HAUTE DÉFINITION
        // ═══════════════════════════════════════════════════════════════════════════
        let banner_w = ui.available_width();
        let banner_h = 240.0;
        let (banner_rect, _) = ui.allocate_exact_size(vec2(banner_w, banner_h), Sense::hover());
        let p = ui.painter().clone();
        let clip = p.with_clip_rect(banner_rect);

        // A. Fond Hero : Texture bannière ou dégradé mesh de secours
        let banner_tex = self.get_banner_texture(&ctx, &game);
        if let Some(t) = &banner_tex {
            clip.image(
                t.id(),
                banner_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                WHITE,
            );
        } else {
            let (_c1, c2) = cover_colors(&game.name);
            draw_gradient_rect(&clip, banner_rect, c2, th.surface_1);
        }

        // B. Dégradés de contraste sombre (pour garantir une lisibilité 100% parfaite des textes)
        let fade_rect = Rect::from_min_max(
            pos2(banner_rect.left(), banner_rect.bottom() - 150.0),
            banner_rect.right_bottom(),
        );
        draw_vertical_fade(&clip, fade_rect, Color32::TRANSPARENT, Color32::from_black_alpha(240));

        let left_fade = Rect::from_min_max(
            banner_rect.min,
            pos2(banner_rect.left() + banner_w * 0.65, banner_rect.bottom()),
        );
        draw_gradient_rect(&clip, left_fade, Color32::from_black_alpha(210), Color32::TRANSPARENT);

        clip.rect_stroke(
            banner_rect,
            CornerRadius::same(12),
            Stroke::new(1.0, th.card_border),
            StrokeKind::Inside,
        );

        // C. Incrustation : Jaquette HD en relief
        let thumb_w = 114.0;
        let thumb_h = 160.0;
        let thumb_rect = Rect::from_min_size(
            pos2(banner_rect.left() + 24.0, banner_rect.bottom() - thumb_h - 22.0),
            vec2(thumb_w, thumb_h),
        );

        let cover_tex = self.get_cover_texture(&ctx, &game);
        if let Some(t) = &cover_tex {
            clip.image(
                t.id(),
                thumb_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                WHITE,
            );
        } else {
            let (c1, c2) = cover_colors(&game.name);
            draw_gradient_rect(&clip, thumb_rect, c1, c2);
        }

        clip.rect_stroke(
            thumb_rect,
            CornerRadius::same(8),
            Stroke::new(1.2, th.border_hover),
            StrokeKind::Inside,
        );

        // D. Textes et Actions dans la bannière Hero
        let info_x = thumb_rect.right() + 24.0;
        let text_top = thumb_rect.top() + 6.0;

        // Badge Plateforme
        let gal_src = clip.layout_no_wrap(
            game.source.label().to_string(),
            FontId::proportional(11.0),
            WHITE,
        );
        let src_pill_w = gal_src.size().x + 14.0;
        let src_pill_r = Rect::from_min_size(pos2(info_x, text_top), vec2(src_pill_w, 20.0));
        clip.rect_filled(src_pill_r, CornerRadius::same(4), src_col.gamma_multiply(0.85));
        clip.galley(src_pill_r.min + vec2(7.0, 3.0), gal_src, WHITE);

        // Grand Titre du jeu
        clip.text(
            pos2(info_x, text_top + 28.0),
            Align2::LEFT_TOP,
            &game.name,
            FontId::proportional(24.0),
            WHITE,
        );

        // Badges d'informations essentiels
        let chips_y = text_top + 68.0;
        let info_line = format!(
            "Temps de jeu : {}   •   Dernière session : {}   •   Taille : {}",
            format_playtime(play_mins),
            format_last_played_time(last_session_ts),
            if game.size_bytes > 0 { scan::format_size(game.size_bytes) } else { "Non mesuré".into() }
        );
        clip.text(
            pos2(info_x, chips_y),
            Align2::LEFT_TOP,
            info_line,
            FontId::proportional(12.0),
            TEXT_SECONDARY,
        );

        // E. Barre d'Actions de la Bannière Hero
        let actions_y = text_top + 104.0;

        // Bouton Principal : JOUER
        let btn_play_rect = Rect::from_min_size(pos2(info_x, actions_y), vec2(150.0, 34.0));
        let play_resp = ui.interact(btn_play_rect, Id::unique(("hero_play", &game.id)), Sense::click());
        let play_hov = play_resp.hovered();
        let play_bg = if play_hov { accent.lerp_to_gamma(WHITE, 0.15) } else { accent };

        clip.rect_filled(btn_play_rect, CornerRadius::same(7), play_bg);
        clip.rect_stroke(
            btn_play_rect,
            CornerRadius::same(7),
            Stroke::new(1.0, Color32::WHITE.gamma_multiply(0.35)),
            StrokeKind::Inside,
        );
        clip.text(
            btn_play_rect.center(),
            Align2::CENTER_CENTER,
            "▶  Jouer maintenant",
            FontId::proportional(13.0),
            WHITE,
        );

        if play_resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            actions.push(Action::Launch(idx));
        }

        // Bouton : Favori
        let btn_fav_rect = Rect::from_min_size(pos2(btn_play_rect.right() + 10.0, actions_y), vec2(100.0, 34.0));
        let fav_resp = ui.interact(btn_fav_rect, Id::unique(("hero_fav", &game.id)), Sense::click());
        let fav_hov = fav_resp.hovered();

        clip.rect_filled(btn_fav_rect, CornerRadius::same(7), if fav_hov { th.surface_3 } else { th.surface_2 });
        clip.rect_stroke(
            btn_fav_rect,
            CornerRadius::same(7),
            Stroke::new(1.0, if is_fav { GOLD.gamma_multiply(0.7) } else { th.border_default }),
            StrokeKind::Inside,
        );
        clip.text(
            btn_fav_rect.center(),
            Align2::CENTER_CENTER,
            if is_fav { "★ Favori" } else { "☆ Ajouter" },
            FontId::proportional(12.5),
            if is_fav { GOLD } else { TEXT_PRIMARY },
        );

        if fav_resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            actions.push(Action::Fav(game.id.clone()));
        }

        // Bouton : Ouvrir le dossier (si existant)
        if let Some(ref d) = game.install_dir {
            if d.exists() {
                let btn_dir_rect = Rect::from_min_size(pos2(btn_fav_rect.right() + 10.0, actions_y), vec2(130.0, 34.0));
                let dir_resp = ui.interact(btn_dir_rect, Id::unique(("hero_dir", &game.id)), Sense::click());
                let dir_hov = dir_resp.hovered();

                clip.rect_filled(btn_dir_rect, CornerRadius::same(7), if dir_hov { th.surface_3 } else { th.surface_2 });
                clip.rect_stroke(
                    btn_dir_rect,
                    CornerRadius::same(7),
                    Stroke::new(1.0, th.border_default),
                    StrokeKind::Inside,
                );
                clip.text(
                    btn_dir_rect.center(),
                    Align2::CENTER_CENTER,
                    "📂 Ouvrir dossier",
                    FontId::proportional(12.0),
                    TEXT_PRIMARY,
                );

                if dir_resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    actions.push(Action::OpenDir(d.clone()));
                }
            }
        }

        // Bouton : Sauvegarder
        let btn_save_x = if game.install_dir.as_ref().map(|d| d.exists()).unwrap_or(false) {
            btn_fav_rect.right() + 150.0
        } else {
            btn_fav_rect.right() + 10.0
        };
        let btn_save_rect = Rect::from_min_size(pos2(btn_save_x, actions_y), vec2(110.0, 34.0));
        let save_resp = ui.interact(btn_save_rect, Id::unique(("hero_save", &game.id)), Sense::click());
        let save_hov = save_resp.hovered();

        clip.rect_filled(btn_save_rect, CornerRadius::same(7), if save_hov { th.surface_3 } else { th.surface_2 });
        clip.rect_stroke(btn_save_rect, CornerRadius::same(7), Stroke::new(1.0, th.border_default), StrokeKind::Inside);
        clip.text(btn_save_rect.center(), Align2::CENTER_CENTER, "💾 Backup", FontId::proportional(12.0), TEXT_PRIMARY);

        if save_resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            actions.push(Action::BackupSave(game.id.clone()));
        }

        ui.add_space(14.0);

        // ═══════════════════════════════════════════════════════════════════════════
        // 3. BARRE D'ONGLETS MODERNE (5 ONGLETS)
        // ═══════════════════════════════════════════════════════════════════════════
        ui.horizontal(|ui| {
            egui::Frame::new()
                .fill(th.surface_1)
                .stroke(Stroke::new(1.0, th.border_subtle))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::same(3))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    let tabs = [
                        (0, "📋  Vue d'ensemble"),
                        (1, "📊  Statistiques"),
                        (2, "📸  Captures d'écran"),
                        (3, "🏆  Succès & Trophées"),
                        (4, "⚙  Paramètres du jeu"),
                    ];
                    for (idx_tab, lbl) in tabs {
                        let sel = self.properties_tab == idx_tab;
                        let bg = if sel { accent.gamma_multiply(0.24) } else { Color32::TRANSPARENT };
                        let fg = if sel { WHITE } else { TEXT_SECONDARY };
                        let btn = egui::Button::new(RichText::new(lbl).size(12.0).strong().color(fg))
                            .fill(bg)
                            .stroke(Stroke::new(1.0, if sel { accent.gamma_multiply(0.70) } else { Color32::TRANSPARENT }))
                            .corner_radius(CornerRadius::same(6))
                            .min_size(vec2(0.0, 28.0));

                        if ui.add(btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            self.properties_tab = idx_tab;
                        }
                    }
                });
        });

        ui.add_space(10.0);

        // ═══════════════════════════════════════════════════════════════════════════
        // 4. CONTENU DÉTAILLÉ DE L'ONGLET SÉLECTIONNÉ
        // ═══════════════════════════════════════════════════════════════════════════
        egui::ScrollArea::vertical()
            .id_salt("game_details_page_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                match self.properties_tab {
                    // ──────────────── 0. VUE D'ENSEMBLE ────────────────
                    0 => {
                        ui.add_space(4.0);

                        // Formulaire d'édition inline si activé
                        if self.edit_info_open {
                            egui::Frame::new()
                                .fill(th.surface_2)
                                .stroke(Stroke::new(1.0, accent.gamma_multiply(0.8)))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::same(14))
                                .show(ui, |ui| {
                                    ui.label(RichText::new("✏️  Modifier les informations du jeu").size(13.5).strong().color(TEXT_PRIMARY));
                                    ui.add_space(6.0);

                                    ui.label(RichText::new("Nom du jeu").size(11.5).color(TEXT_SECONDARY));
                                    ui.add(egui::TextEdit::singleline(&mut self.edit_name_input).desired_width(ui.available_width()));

                                    ui.add_space(6.0);
                                    ui.label(RichText::new("Description / Synopsis").size(11.5).color(TEXT_SECONDARY));
                                    ui.add(egui::TextEdit::multiline(&mut self.edit_desc_input).desired_width(ui.available_width()).desired_rows(3));

                                    ui.add_space(6.0);
                                    ui.label(RichText::new("Date de sortie").size(11.5).color(TEXT_SECONDARY));
                                    ui.add(egui::TextEdit::singleline(&mut self.edit_date_input).desired_width(ui.available_width()));

                                    ui.add_space(8.0);
                                    ui.horizontal(|ui| {
                                        if ui.button("💾  Enregistrer").clicked() {
                                            if !self.edit_name_input.trim().is_empty() {
                                                if let Some(g_mut) = self.games.iter_mut().find(|g| g.id == game.id) {
                                                    g_mut.name = self.edit_name_input.trim().to_string();
                                                }
                                            }
                                            self.settings.game_descriptions.insert(game.id.clone(), self.edit_desc_input.trim().to_string());
                                            self.settings.game_release_dates.insert(game.id.clone(), self.edit_date_input.trim().to_string());
                                            self.settings.save();
                                            self.edit_info_open = false;
                                            self.toast("Informations mises à jour avec succès");
                                        }
                                        if ui.button("Annuler").clicked() {
                                            self.edit_info_open = false;
                                        }
                                    });
                                });
                            ui.add_space(10.0);
                        }

                        // Description / Synopsis du titre
                        let desc_text = self
                            .settings
                            .game_descriptions
                            .get(&game.id)
                            .cloned()
                            .unwrap_or_else(|| "Aucun synopsis disponible. Cliquez sur Modifier pour renseigner une description.".to_string());

                        egui::Frame::new()
                            .fill(th.surface_1)
                            .stroke(Stroke::new(1.0, th.border_subtle))
                            .corner_radius(CornerRadius::same(10))
                            .inner_margin(Margin::same(16))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("À propos du jeu").size(14.0).strong().color(TEXT_PRIMARY));
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if ui.button("✏️  Modifier").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                            self.edit_info_open = true;
                                            self.edit_name_input = game.name.clone();
                                            self.edit_desc_input = desc_text.clone();
                                            self.edit_date_input = self.settings.game_release_dates.get(&game.id).cloned().unwrap_or_default();
                                        }
                                    });
                                });
                                ui.add_space(8.0);
                                ui.label(RichText::new(&desc_text).size(12.5).color(TEXT_SECONDARY));
                            });

                        ui.add_space(12.0);

                        // Spécifications techniques
                        egui::Frame::new()
                            .fill(th.surface_1)
                            .stroke(Stroke::new(1.0, th.border_subtle))
                            .corner_radius(CornerRadius::same(10))
                            .inner_margin(Margin::same(16))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(RichText::new("Informations Techniques & Emplacements").size(14.0).strong().color(TEXT_PRIMARY));
                                ui.add_space(10.0);

                                let render_spec_row = |ui: &mut egui::Ui, label: &str, value: &str| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(label).size(12.0).color(TEXT_TERTIARY));
                                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                            ui.label(RichText::new(value).size(12.0).strong().color(TEXT_PRIMARY));
                                        });
                                    });
                                    ui.add_space(4.0);
                                };

                                render_spec_row(ui, "Plateforme de distribution", game.source.label());
                                render_spec_row(ui, "Identifiant interne", &game.id);

                                if let Some(ref dir) = game.install_dir {
                                    render_spec_row(ui, "Dossier d'installation", &dir.to_string_lossy());
                                }
                                if let Some(ref e) = game.exe {
                                    render_spec_row(ui, "Exécutable cible", &e.to_string_lossy());
                                }
                                if let Some(ref uri) = game.launch_uri {
                                    render_spec_row(ui, "Protocole de lancement URI", uri);
                                }
                                render_spec_row(ui, "Taille sur le disque", &scan::format_size(game.size_bytes));
                            });
                    }

                    // ──────────────── 1. STATISTIQUES ────────────────
                    1 => {
                        ui.add_space(4.0);
                        let sessions_count = self.settings.session_counts.get(&game.id).copied().unwrap_or(0);
                        let avg_session = if sessions_count > 0 {
                            play_mins / sessions_count
                        } else {
                            0
                        };

                        ui.horizontal(|ui| {
                            let card_w = ((ui.available_width() - 24.0) / 3.0).max(140.0);
                            for (title, val, col) in [
                                ("Temps de jeu total", format_playtime(play_mins), accent),
                                ("Nombre de sessions", format!("{sessions_count} lancements"), GOLD),
                                ("Durée moyenne par session", format_playtime(avg_session), Color32::from_rgb(56, 189, 248)),
                            ] {
                                egui::Frame::new()
                                    .fill(th.surface_1)
                                    .stroke(Stroke::new(1.0, th.border_subtle))
                                    .corner_radius(CornerRadius::same(10))
                                    .inner_margin(Margin::same(16))
                                    .show(ui, |ui| {
                                        ui.set_width(card_w);
                                        ui.label(RichText::new(title).size(11.5).color(TEXT_TERTIARY));
                                        ui.add_space(6.0);
                                        ui.label(RichText::new(val).size(18.0).strong().color(col));
                                    });
                            }
                        });

                        ui.add_space(14.0);

                        egui::Frame::new()
                            .fill(th.surface_1)
                            .stroke(Stroke::new(1.0, th.border_subtle))
                            .corner_radius(CornerRadius::same(10))
                            .inner_margin(Margin::same(16))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(RichText::new("Optimisation des Performances & Game Boost").size(14.0).strong().color(TEXT_PRIMARY));
                                ui.add_space(6.0);
                                ui.label(RichText::new("UniLauncher active automatiquement le plan d'alimentation Haute Performance sous Windows au lancement de vos jeux pour réduire la latence d'entrée et stabiliser le taux de rafraîchissement.").size(12.0).color(TEXT_SECONDARY));
                                ui.add_space(10.0);
                                ui.horizontal(|ui| {
                                    let is_perf = self.settings.performance_mode;
                                    ui.label(RichText::new(format!("Game Boost : {}", if is_perf { "Activé (recommandé)" } else { "Standard" })).size(12.0).color(if is_perf { SUCCESS } else { TEXT_SECONDARY }));
                                });
                            });
                    }

                    // ──────────────── 2. CAPTURES D'ÉCRAN ────────────────
                    2 => {
                        ui.add_space(4.0);
                        let settings_clone = self.settings.clone();
                        let screenshots = self
                            .cached_screenshots
                            .entry(game.id.clone())
                            .or_insert_with(|| scan::find_game_screenshots(&game, &settings_clone))
                            .clone();

                        if screenshots.is_empty() {
                            ui.add_space(30.0);
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new("📸  Aucune capture d'écran détectée").size(15.0).strong().color(TEXT_PRIMARY));
                                ui.add_space(4.0);
                                ui.label(RichText::new("Prenez des captures dans votre jeu (F12 sous Steam) ou déposez des images dans le dossier du jeu.").size(12.0).color(TEXT_SECONDARY));
                                ui.add_space(12.0);
                                if let Some(ref dir) = game.install_dir {
                                    if dir.exists() && ui.button("📂  Ouvrir le dossier du jeu").clicked() {
                                        actions.push(Action::OpenDir(dir.clone()));
                                    }
                                }
                            });
                        } else {
                            ui.label(RichText::new(format!("{} captures d'écran trouvées", screenshots.len())).size(13.0).strong().color(TEXT_PRIMARY));
                            ui.add_space(10.0);

                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
                                for pic in &screenshots {
                                    let (r, resp) = ui.allocate_exact_size(vec2(160.0, 95.0), Sense::click());
                                    let hov = resp.hovered();

                                    ui.painter().rect_filled(r, CornerRadius::same(6), th.surface_2);
                                    ui.painter().rect_stroke(
                                        r,
                                        CornerRadius::same(6),
                                        Stroke::new(1.0, if hov { accent } else { th.border_subtle }),
                                        StrokeKind::Inside,
                                    );

                                    let name = pic.file_name().unwrap_or_default().to_string_lossy();
                                    ui.painter().text(
                                        r.center(),
                                        Align2::CENTER_CENTER,
                                        trunc(&name, 18),
                                        FontId::proportional(11.0),
                                        if hov { WHITE } else { TEXT_SECONDARY },
                                    );

                                    if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                        self.selected_screenshot = Some(pic.clone());
                                    }
                                }
                            });
                        }
                    }

                    // ──────────────── 3. SUCCÈS & TROPHÉES ────────────────
                    3 => {
                        ui.add_space(4.0);
                        if let Some(ach_opt) = self.cached_achievements.get(&game.id) {
                            if let Some(ach) = ach_opt {
                                let total = ach.total;
                                let unlocked = ach.unlocked_count;
                                let pct = ach.progress_percent;

                                egui::Frame::new()
                                    .fill(th.surface_1)
                                    .stroke(Stroke::new(1.0, th.border_subtle))
                                    .corner_radius(CornerRadius::same(10))
                                    .inner_margin(Margin::same(16))
                                    .show(ui, |ui| {
                                        ui.set_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new("Progression des Succès").size(14.0).strong().color(TEXT_PRIMARY));
                                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                ui.label(RichText::new(format!("{unlocked} / {total} ({:.0}%)", pct)).size(12.5).strong().color(GOLD));
                                            });
                                        });
                                        ui.add_space(8.0);
                                        let (bar_r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 6.0), Sense::hover());
                                        ui.painter().rect_filled(bar_r, CornerRadius::same(3), th.surface_2);
                                        let fill_w = (bar_r.width() * (pct / 100.0)).clamp(2.0, bar_r.width());
                                        ui.painter().rect_filled(Rect::from_min_size(bar_r.min, vec2(fill_w, 6.0)), CornerRadius::same(3), GOLD);
                                    });

                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Filtrer :").size(12.0).color(TEXT_SECONDARY));
                                    ui.selectable_value(&mut self.achievements_filter, 0, "Tous");
                                    ui.selectable_value(&mut self.achievements_filter, 1, "Débloqués");
                                    ui.selectable_value(&mut self.achievements_filter, 2, "Verrouillés");
                                });
                                ui.add_space(8.0);

                                for item in &ach.list {
                                    if self.achievements_filter == 1 && !item.unlocked {
                                        continue;
                                    }
                                    if self.achievements_filter == 2 && item.unlocked {
                                        continue;
                                    }
                                    egui::Frame::new()
                                        .fill(th.surface_1)
                                        .stroke(Stroke::new(1.0, th.border_subtle))
                                        .corner_radius(CornerRadius::same(8))
                                        .inner_margin(Margin::symmetric(14, 10))
                                        .show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            ui.horizontal(|ui| {
                                                let icon_char = if item.unlocked { "🏆" } else { "🔒" };
                                                ui.label(RichText::new(icon_char).size(16.0));
                                                ui.add_space(8.0);
                                                ui.vertical(|ui| {
                                                    ui.label(RichText::new(&item.name).size(12.5).strong().color(if item.unlocked { TEXT_PRIMARY } else { TEXT_TERTIARY }));
                                                    if !item.description.is_empty() {
                                                        ui.label(RichText::new(&item.description).size(11.0).color(TEXT_SECONDARY));
                                                    }
                                                });
                                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                    ui.label(RichText::new(if item.unlocked { "DÉBLOQUÉ" } else { "VERROUILLÉ" }).size(10.5).strong().color(if item.unlocked { GOLD } else { TEXT_TERTIARY }));
                                                });
                                            });
                                        });
                                    ui.add_space(4.0);
                                }
                            } else {
                                ui.add_space(30.0);
                                ui.vertical_centered(|ui| {
                                    ui.label(RichText::new("🏆  Succès non disponibles").size(15.0).strong().color(TEXT_PRIMARY));
                                    ui.add_space(6.0);
                                    ui.label(RichText::new("Cette plateforme n'expose pas de données publiques de succès, ou le jeu ne prend pas en charge les trophées.").size(12.0).color(TEXT_SECONDARY));
                                    ui.add_space(4.0);
                                    ui.label(RichText::new("Si ce titre est un jeu Steam, configurez votre clé Web API dans les Paramètres d'UniLauncher.").size(11.5).color(TEXT_TERTIARY));
                                });
                            }
                        } else {
                            ui.add_space(24.0);
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new("⏳ Chargement des succès en cours…").size(13.0).color(TEXT_SECONDARY));
                            });
                        }
                    }

                    // ──────────────── 4. PARAMÈTRES DU JEU ────────────────
                    4 => {
                        ui.add_space(4.0);

                        egui::Frame::new()
                            .fill(th.surface_1)
                            .stroke(Stroke::new(1.0, th.border_subtle))
                            .corner_radius(CornerRadius::same(10))
                            .inner_margin(Margin::same(16))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(RichText::new("Personnalisation Visuelle").size(14.0).strong().color(TEXT_PRIMARY));
                                ui.add_space(10.0);

                                ui.horizontal(|ui| {
                                    if ui.button("🎨  Modifier la jaquette HD…").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                        actions.push(Action::PickCover(game.id.clone()));
                                    }
                                    if self.settings.cover_overrides.contains_key(&game.id) {
                                        if ui.button("🔄  Réinitialiser la jaquette").clicked() {
                                            actions.push(Action::ClearCover(game.id.clone()));
                                        }
                                    }
                                    if ui.button("🌐  Télécharger jaquette Steam auto").clicked() {
                                        actions.push(Action::DownloadCover(game.id.clone()));
                                    }
                                });

                                ui.add_space(10.0);

                                ui.horizontal(|ui| {
                                    if ui.button("🖼️  Modifier la bannière Hero HD…").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                        actions.push(Action::PickBanner(game.id.clone()));
                                    }
                                    if self.settings.banner_overrides.contains_key(&game.id) {
                                        if ui.button("🔄  Réinitialiser la bannière").clicked() {
                                            actions.push(Action::ClearBanner(game.id.clone()));
                                        }
                                    }
                                });
                            });

                        ui.add_space(12.0);

                        egui::Frame::new()
                            .fill(th.surface_1)
                            .stroke(Stroke::new(1.0, th.border_subtle))
                            .corner_radius(CornerRadius::same(10))
                            .inner_margin(Margin::same(16))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(RichText::new("Lancement & Arguments Système").size(14.0).strong().color(TEXT_PRIMARY));
                                ui.add_space(10.0);

                                ui.horizontal(|ui| {
                                    if ui.button("🎯  Choisir un exécutable alternatif…").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                        actions.push(Action::PickExe(game.id.clone()));
                                    }
                                    if self.settings.exe_overrides.contains_key(&game.id) {
                                        if ui.button("Réinitialiser l'exécutable").clicked() {
                                            actions.push(Action::ClearExe(game.id.clone()));
                                        }
                                    }
                                });

                                ui.add_space(10.0);
                                ui.label(RichText::new("Arguments de lancement personnalisés (ex: -novid -fullscreen -dx11)").size(11.5).color(TEXT_SECONDARY));

                                let mut cur_args = self.settings.launch_args.get(&game.id).cloned().unwrap_or_default();
                                let resp = ui.add(egui::TextEdit::singleline(&mut cur_args).desired_width(ui.available_width()));
                                if resp.changed() {
                                    if cur_args.trim().is_empty() {
                                        self.settings.launch_args.remove(&game.id);
                                    } else {
                                        self.settings.launch_args.insert(game.id.clone(), cur_args);
                                    }
                                    self.settings.save();
                                }
                            });

                        ui.add_space(12.0);

                        egui::Frame::new()
                            .fill(th.surface_1)
                            .stroke(Stroke::new(1.0, th.border_subtle))
                            .corner_radius(CornerRadius::same(10))
                            .inner_margin(Margin::same(16))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(RichText::new("Gestion de la Bibliothèque").size(14.0).strong().color(TEXT_PRIMARY));
                                ui.add_space(10.0);

                                let is_hidden = self.settings.hidden_games.contains(&game.id);
                                if ui.button(if is_hidden { "👁️  Restaurer dans la bibliothèque principale" } else { "👁️  Masquer ce titre de la bibliothèque" }).clicked() {
                                    actions.push(Action::ToggleHide(game.id.clone()));
                                }

                                ui.add_space(10.0);
                                ui.separator();
                                ui.add_space(10.0);

                                if game.source == Source::Manual {
                                    if ui.button(RichText::new("🗑️  Supprimer définitivement de la bibliothèque").color(ERROR)).clicked() {
                                        actions.push(Action::Remove(game.id.clone()));
                                        actions.push(Action::CloseDetails);
                                    }
                                } else {
                                    if ui.button(RichText::new("Désinstaller le jeu via son launcher officiel…").color(TEXT_SECONDARY)).clicked() {
                                        let _ = scan::uninstall_game(&game);
                                    }
                                }
                            });
                    }

                    _ => {}
                }
            });

        actions
    }

    // ───────────────────────────── Wrapper de compatibilité pour les tests ─────────────────────────────
    #[allow(dead_code)]
    pub fn properties_modal_ui(&mut self, ctx: &egui::Context) -> Vec<Action> {
        let mut actions = vec![];
        if let Some(id) = self.selected_game_id.clone() {
            egui::Area::new(Id::unique("test_props_modal")).show(ctx, |ui| {
                actions = self.game_details_page_ui(ui, &id);
            });
        }
        actions
    }
}
