use crate::app::Launcher;
use crate::config::Source;
use crate::scan;
use crate::theme::{
    BNET_C, EA_C, EPIC_C, ERROR, GOG_C, GOLD, STEAM_C, SUCCESS, TEXT_PRIMARY,
    TEXT_SECONDARY, TEXT_TERTIARY, UBI_C, WHITE, XBOX_C,
};
use crate::types::{format_playtime, Action};
use crate::ui::components::{cover_colors, draw_gradient_rect, trunc};
use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, CornerRadius, FontId, Id, Key, Layout, Margin, Rect,
    RichText, Sense, Stroke, StrokeKind,
};
use std::time::{SystemTime, UNIX_EPOCH};

fn format_last_played(ts: Option<u64>) -> String {
    let Some(t) = ts else {
        return "Jamais joué".to_string();
    };
    if t == 0 {
        return "Jamais joué".to_string();
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if now < t {
        return "À l'instant".to_string();
    }
    let diff = now - t;
    if diff < 60 {
        "À l'instant".to_string()
    } else if diff < 3600 {
        format!("Il y a {} min", diff / 60)
    } else if diff < 86400 {
        format!("Il y a {} h", diff / 3600)
    } else if diff < 86400 * 30 {
        format!("Il y a {} j", diff / 86400)
    } else {
        format!("Il y a {} mois", diff / (86400 * 30))
    }
}

impl Launcher {
    // ───────────────────────────── Fiche Détaillée & Propriétés (Style Riot / Steam) ─────────────────────────────
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

        // Fermeture automatique sur Échap
        if ctx.input(|i| i.key_pressed(Key::Escape)) && !self.edit_info_open {
            close_modal = true;
        }

        // ── 1. Arrière-plan assombri (Backdrop) avec clic pour fermer ──
        egui::Area::new(Id::unique("properties_modal_backdrop"))
            .order(egui::Order::Middle)
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                let (rect, resp) = ui.allocate_exact_size(screen_rect.size(), Sense::click());
                ui.painter()
                    .rect_filled(rect, CornerRadius::ZERO, Color32::from_black_alpha(200));
                if resp.clicked() {
                    close_modal = true;
                }
            });

        // ── 2. Fenêtre Modale Centrée ──
        let modal_w = 900.0f32.min(screen_rect.width() - 32.0);
        let modal_h = 680.0f32.min(screen_rect.height() - 32.0);
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
            Source::Battlenet => BNET_C,
            Source::Xbox => XBOX_C,
            Source::Manual => accent,
        };

        egui::Area::new(Id::unique("properties_modal_window"))
            .order(egui::Order::Foreground)
            .fixed_pos(modal_pos)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(th.surface_2)
                    .stroke(Stroke::new(1.0, th.border_hover))
                    .corner_radius(CornerRadius::same(14))
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 18],
                        blur: 36,
                        spread: 0,
                        color: Color32::from_black_alpha(225),
                    })
                    .inner_margin(Margin::symmetric(24, 20))
                    .show(ui, |ui| {
                        ui.set_width(modal_w - 48.0);
                        ui.set_height(modal_h - 40.0);

                        // ── EN-TÊTE : Titre, Miniature & Contrôle Fermer ──
                        ui.horizontal(|ui| {
                            let (thumb_rect, _) =
                                ui.allocate_exact_size(vec2(48.0, 64.0), Sense::hover());
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
                                Stroke::new(1.0, th.border_subtle),
                                StrokeKind::Inside,
                            );

                            ui.add_space(14.0);

                            // Titre + Badges d'état
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(&game.name)
                                        .size(20.0)
                                        .strong()
                                        .color(TEXT_PRIMARY),
                                );
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 6.0;

                                    // Badge Plateforme
                                    egui::Frame::new()
                                        .fill(src_col.gamma_multiply(0.18))
                                        .stroke(Stroke::new(1.0, src_col.gamma_multiply(0.50)))
                                        .corner_radius(CornerRadius::same(5))
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
                                        .fill(th.surface_1)
                                        .stroke(Stroke::new(1.0, th.border_subtle))
                                        .corner_radius(CornerRadius::same(5))
                                        .inner_margin(Margin::symmetric(8, 2))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(format_playtime(play_mins))
                                                    .size(11.0)
                                                    .color(TEXT_SECONDARY),
                                            );
                                        });

                                    // Badge Taille Disque
                                    if game.size_bytes > 0 {
                                        egui::Frame::new()
                                            .fill(th.surface_1)
                                            .stroke(Stroke::new(1.0, th.border_subtle))
                                            .corner_radius(CornerRadius::same(5))
                                            .inner_margin(Margin::symmetric(8, 2))
                                            .show(ui, |ui| {
                                                ui.label(
                                                    RichText::new(scan::format_size(
                                                        game.size_bytes,
                                                    ))
                                                    .size(11.0)
                                                    .color(GOLD),
                                                );
                                            });
                                    }

                                    // Badge Favori
                                    if self.settings.favorites.contains(&game.id) {
                                        egui::Frame::new()
                                            .fill(GOLD.gamma_multiply(0.18))
                                            .stroke(Stroke::new(1.0, GOLD.gamma_multiply(0.50)))
                                            .corner_radius(CornerRadius::same(5))
                                            .inner_margin(Margin::symmetric(8, 2))
                                            .show(ui, |ui| {
                                                ui.label(
                                                    RichText::new("★ Favori")
                                                        .size(11.0)
                                                        .strong()
                                                        .color(GOLD),
                                                );
                                            });
                                    }
                                });
                            });

                            // Bouton Fermer (✕)
                            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                let (close_rect, close_resp) =
                                    ui.allocate_exact_size(vec2(30.0, 30.0), Sense::click());
                                let hov = close_resp.hovered();

                                let bg = if hov {
                                    ERROR.gamma_multiply(0.18)
                                } else {
                                    th.surface_1
                                };
                                let stroke_c = if hov {
                                    ERROR
                                } else {
                                    th.border_subtle
                                };

                                ui.painter().rect_filled(close_rect, CornerRadius::same(6), bg);
                                ui.painter().rect_stroke(
                                    close_rect,
                                    CornerRadius::same(6),
                                    Stroke::new(1.0, stroke_c),
                                    StrokeKind::Inside,
                                );

                                let close_col = if hov { ERROR } else { TEXT_SECONDARY };
                                let c = close_rect.center();
                                let st = Stroke::new(1.4, close_col);
                                ui.painter()
                                    .line_segment([c + vec2(-4.0, -4.0), c + vec2(4.0, 4.0)], st);
                                ui.painter()
                                    .line_segment([c + vec2(4.0, -4.0), c + vec2(-4.0, 4.0)], st);

                                if close_resp.clicked() {
                                    close_modal = true;
                                }
                            });
                        });

                        ui.add_space(14.0);

                        // ── 5 ONGLETS : Vue d'ensemble, Statistiques, Captures d'écran, Succès, Paramètres ──
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
                                        (3, "🏆  Succès"),
                                        (4, "⚙  Paramètres"),
                                    ];
                                    for (idx_tab, lbl) in tabs {
                                        let sel = self.properties_tab == idx_tab;
                                        let bg = if sel {
                                            accent.gamma_multiply(0.24)
                                        } else {
                                            Color32::TRANSPARENT
                                        };
                                        let fg = if sel { WHITE } else { TEXT_SECONDARY };
                                        let btn = egui::Button::new(
                                            RichText::new(lbl).size(12.0).strong().color(fg),
                                        )
                                        .fill(bg)
                                        .stroke(Stroke::new(
                                            1.0,
                                            if sel {
                                                accent.gamma_multiply(0.70)
                                            } else {
                                                Color32::TRANSPARENT
                                            },
                                        ))
                                        .corner_radius(CornerRadius::same(6))
                                        .min_size(vec2(0.0, 26.0));

                                        if ui
                                            .add(btn)
                                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                                            .clicked()
                                        {
                                            self.properties_tab = idx_tab;
                                        }
                                    }
                                });
                        });

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // ── CONTENU DE L'ONGLET SÉLECTIONNÉ ──
                        let content_h = modal_h - 225.0;
                        egui::ScrollArea::vertical()
                            .id_salt("properties_modal_scroll")
                            .max_height(content_h)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                match self.properties_tab {
                                    // ──────────────── 0. VUE D'ENSEMBLE ────────────────
                                    0 => {
                                        ui.add_space(4.0);

                                        // Formulaire d'édition inline si actif
                                        if self.edit_info_open {
                                            egui::Frame::new()
                                                .fill(th.surface_1)
                                                .stroke(Stroke::new(1.0, accent.gamma_multiply(0.8)))
                                                .corner_radius(CornerRadius::same(8))
                                                .inner_margin(Margin::same(14))
                                                .show(ui, |ui| {
                                                    ui.label(
                                                        RichText::new(
                                                            "✏️  Modifier les informations du jeu",
                                                        )
                                                        .size(13.5)
                                                        .strong()
                                                        .color(TEXT_PRIMARY),
                                                    );
                                                    ui.add_space(6.0);

                                                    ui.label(
                                                        RichText::new("Nom du titre")
                                                            .size(11.5)
                                                            .color(TEXT_SECONDARY),
                                                    );
                                                    ui.add(
                                                        egui::TextEdit::singleline(
                                                            &mut self.edit_name_input,
                                                        )
                                                        .desired_width(f32::INFINITY),
                                                    );

                                                    ui.add_space(4.0);
                                                    ui.label(
                                                        RichText::new("Date de sortie")
                                                            .size(11.5)
                                                            .color(TEXT_SECONDARY),
                                                    );
                                                    ui.add(
                                                        egui::TextEdit::singleline(
                                                            &mut self.edit_date_input,
                                                        )
                                                        .hint_text("ex: 19 mai 2015")
                                                        .desired_width(f32::INFINITY),
                                                    );

                                                    ui.add_space(4.0);
                                                    ui.label(
                                                        RichText::new("Description / Synopsis")
                                                            .size(11.5)
                                                            .color(TEXT_SECONDARY),
                                                    );
                                                    ui.add(
                                                        egui::TextEdit::multiline(
                                                            &mut self.edit_desc_input,
                                                        )
                                                        .desired_rows(3)
                                                        .desired_width(f32::INFINITY),
                                                    );

                                                    ui.add_space(8.0);
                                                    ui.horizontal(|ui| {
                                                        let save_btn = egui::Button::new(
                                                            RichText::new("✓  Enregistrer")
                                                                .size(12.0)
                                                                .strong()
                                                                .color(WHITE),
                                                        )
                                                        .fill(accent.gamma_multiply(0.28))
                                                        .stroke(Stroke::new(1.0, accent.gamma_multiply(0.70)))
                                                        .corner_radius(CornerRadius::same(6))
                                                        .min_size(vec2(0.0, 26.0));

                                                        if ui.add(save_btn).clicked() {
                                                            if !self.edit_name_input.trim().is_empty() {
                                                                if let Some(target) = self.games.iter_mut().find(|g| g.id == game.id) {
                                                                    target.name = self.edit_name_input.trim().to_string();
                                                                }
                                                                for mg in &mut self.settings.manual_games {
                                                                    if mg.id == game.id {
                                                                        mg.name = self.edit_name_input.trim().to_string();
                                                                    }
                                                                }
                                                            }
                                                            if !self.edit_date_input.trim().is_empty() {
                                                                self.settings.game_release_dates.insert(game.id.clone(), self.edit_date_input.trim().to_string());
                                                            }
                                                            if !self.edit_desc_input.trim().is_empty() {
                                                                self.settings.game_descriptions.insert(game.id.clone(), self.edit_desc_input.trim().to_string());
                                                            }
                                                            save_needed = true;
                                                            self.edit_info_open = false;
                                                            self.toast("Informations enregistrées");
                                                        }

                                                        let cancel_btn = egui::Button::new(
                                                            RichText::new("Annuler")
                                                                .size(12.0)
                                                                .color(TEXT_SECONDARY),
                                                        )
                                                        .fill(th.surface_2)
                                                        .stroke(Stroke::new(1.0, th.border_default))
                                                        .corner_radius(CornerRadius::same(6))
                                                        .min_size(vec2(0.0, 26.0));

                                                        if ui.add(cancel_btn).clicked() {
                                                            self.edit_info_open = false;
                                                        }
                                                    });
                                                });
                                            ui.add_space(10.0);
                                        }

                                        // Hero Banner : Jaquette, Description & Actions
                                        ui.horizontal(|ui| {
                                            // Jaquette HD
                                            let (card_rect, _) =
                                                ui.allocate_exact_size(vec2(120.0, 168.0), Sense::hover());
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
                                                CornerRadius::same(8),
                                                Stroke::new(1.0, th.border_subtle),
                                                StrokeKind::Inside,
                                            );

                                            ui.add_space(16.0);

                                            // Description & Actions rapides
                                            ui.vertical(|ui| {
                                                let desc = self
                                                    .settings
                                                    .game_descriptions
                                                    .get(&game.id)
                                                    .cloned()
                                                    .unwrap_or_else(|| {
                                                        if self.loading_details.contains(&game.id) {
                                                            "Récupération de la description officielle depuis Steam...".to_string()
                                                        } else {
                                                            "Aucune description enregistrée. Cliquez sur 'Modifier les infos' ci-dessous pour ajouter un synopsis personnel.".to_string()
                                                        }
                                                    });

                                                ui.label(
                                                    RichText::new("Description")
                                                        .size(13.0)
                                                        .strong()
                                                        .color(TEXT_PRIMARY),
                                                );
                                                ui.add_space(2.0);
                                                ui.label(
                                                    RichText::new(desc)
                                                        .size(11.5)
                                                        .color(TEXT_SECONDARY)
                                                        .line_height(Some(16.0)),
                                                );

                                                ui.add_space(14.0);

                                                // Barre d'actions complète
                                                ui.horizontal_wrapped(|ui| {
                                                    ui.spacing_mut().item_spacing = vec2(6.0, 6.0);

                                                    // 1. Lancer le jeu
                                                    let play_btn = egui::Button::new(
                                                        RichText::new("▶  Lancer")
                                                            .size(12.5)
                                                            .strong()
                                                            .color(WHITE),
                                                    )
                                                    .fill(accent)
                                                    .corner_radius(CornerRadius::same(6))
                                                    .min_size(vec2(0.0, 26.0));
                                                    if ui.add(play_btn).clicked() {
                                                        actions.push(Action::Launch(idx));
                                                        close_modal = true;
                                                    }

                                                    // 2. Favoris
                                                    let is_fav =
                                                        self.settings.favorites.contains(&game.id);
                                                    let fav_lbl = if is_fav {
                                                        "★  Retirer des favoris"
                                                    } else {
                                                        "☆  Ajouter aux favoris"
                                                    };
                                                    let fav_btn = egui::Button::new(
                                                        RichText::new(fav_lbl)
                                                            .size(12.0)
                                                            .color(TEXT_PRIMARY),
                                                    )
                                                    .fill(th.surface_1)
                                                    .stroke(Stroke::new(1.0, th.border_default))
                                                    .corner_radius(CornerRadius::same(6))
                                                    .min_size(vec2(0.0, 26.0));
                                                    if ui.add(fav_btn).clicked() {
                                                        actions.push(Action::Fav(game.id.clone()));
                                                    }

                                                    // 3. Ouvrir le dossier
                                                    if let Some(dir) = &game.install_dir {
                                                        if dir.exists() {
                                                            let dir_btn = egui::Button::new(
                                                                RichText::new("📁  Dossier local")
                                                                    .size(12.0)
                                                                    .color(TEXT_PRIMARY),
                                                            )
                                                            .fill(th.surface_1)
                                                            .stroke(Stroke::new(1.0, th.border_default))
                                                            .corner_radius(CornerRadius::same(6))
                                                            .min_size(vec2(0.0, 26.0));
                                                            if ui.add(dir_btn).clicked() {
                                                                actions.push(Action::OpenDir(dir.clone()));
                                                            }
                                                        }
                                                    }

                                                    // 4. Modifier les informations
                                                    let edit_btn = egui::Button::new(
                                                        RichText::new("✏️  Modifier les infos")
                                                            .size(12.0)
                                                            .color(TEXT_PRIMARY),
                                                    )
                                                    .fill(th.surface_1)
                                                    .stroke(Stroke::new(1.0, th.border_default))
                                                    .corner_radius(CornerRadius::same(6))
                                                    .min_size(vec2(0.0, 26.0));
                                                    if ui.add(edit_btn).clicked() {
                                                        self.edit_name_input = game.name.clone();
                                                        self.edit_date_input = self
                                                            .settings
                                                            .game_release_dates
                                                            .get(&game.id)
                                                            .cloned()
                                                            .unwrap_or_default();
                                                        self.edit_desc_input = self
                                                            .settings
                                                            .game_descriptions
                                                            .get(&game.id)
                                                            .cloned()
                                                            .unwrap_or_default();
                                                        self.edit_info_open = !self.edit_info_open;
                                                    }

                                                    // 5. Masquer le jeu
                                                    let is_hidden =
                                                        self.settings.hidden_games.contains(&game.id);
                                                    let hide_lbl = if is_hidden {
                                                        "👁️  Restaurer"
                                                    } else {
                                                        "👁️  Masquer"
                                                    };
                                                    let hide_btn = egui::Button::new(
                                                        RichText::new(hide_lbl)
                                                            .size(12.0)
                                                            .color(TEXT_SECONDARY),
                                                    )
                                                    .fill(th.surface_1)
                                                    .stroke(Stroke::new(1.0, th.border_default))
                                                    .corner_radius(CornerRadius::same(6))
                                                    .min_size(vec2(0.0, 26.0));
                                                    if ui.add(hide_btn).clicked() {
                                                        actions.push(Action::ToggleHide(game.id.clone()));
                                                        close_modal = true;
                                                    }

                                                    // 6. Désinstaller via launcher officiel
                                                    let uninst_btn = egui::Button::new(
                                                        RichText::new("🗑️  Désinstaller")
                                                            .size(12.0)
                                                            .color(ERROR),
                                                    )
                                                    .fill(ERROR.gamma_multiply(0.12))
                                                    .stroke(Stroke::new(1.0, ERROR.gamma_multiply(0.35)))
                                                    .corner_radius(CornerRadius::same(6))
                                                    .min_size(vec2(0.0, 26.0));
                                                    if ui.add(uninst_btn).clicked() {
                                                        if let Err(e) = scan::uninstall_game(&game) {
                                                            self.toast(format!(
                                                                "Erreur lors de la désinstallation : {e}"
                                                            ));
                                                        } else {
                                                            self.toast(
                                                                "Gestionnaire de désinstallation officiel ouvert",
                                                            );
                                                        }
                                                    }
                                                });
                                            });
                                        });

                                        ui.add_space(18.0);
                                        ui.separator();
                                        ui.add_space(12.0);

                                        // Grille des fiches d'informations
                                        ui.label(
                                            RichText::new("Caractéristiques & Emplacement")
                                                .size(13.5)
                                                .strong()
                                                .color(TEXT_PRIMARY),
                                        );
                                        ui.add_space(8.0);

                                        let release_date = self
                                            .settings
                                            .game_release_dates
                                            .get(&game.id)
                                            .cloned()
                                            .unwrap_or_else(|| "Non spécifiée".into());
                                        let play_mins = self.total_playtime_mins(&game);
                                        let last_played = format_last_played(
                                            self.settings.last_played_dates.get(&game.id).copied(),
                                        );
                                        let session_cnt = self
                                            .settings
                                            .session_counts
                                            .get(&game.id)
                                            .copied()
                                            .unwrap_or(0);
                                        let size_str = if game.size_bytes > 0 {
                                            scan::format_size(game.size_bytes)
                                        } else {
                                            "Non calculée".into()
                                        };
                                        let install_path = game
                                            .install_dir
                                            .as_ref()
                                            .map(|p| p.to_string_lossy().to_string())
                                            .unwrap_or_else(|| "Géré par le launcher".into());

                                        let info_grid = [
                                            (
                                                "🏷️  Plateforme",
                                                game.source.label().to_string(),
                                                src_col,
                                            ),
                                            ("📅  Date de sortie", release_date, TEXT_PRIMARY),
                                            (
                                                "⏱️  Temps total",
                                                format_playtime(play_mins),
                                                accent,
                                            ),
                                            ("🕒  Dernière session", last_played, TEXT_PRIMARY),
                                            (
                                                "🎮  Nombre de sessions",
                                                format!("{session_cnt} parties"),
                                                TEXT_PRIMARY,
                                            ),
                                            ("💾  Taille du jeu", size_str, GOLD),
                                        ];

                                        ui.columns(3, |cols| {
                                            for (i, (lbl, val, col)) in info_grid.iter().enumerate() {
                                                let col_ui = &mut cols[i % 3];
                                                egui::Frame::new()
                                                    .fill(th.surface_1)
                                                    .stroke(Stroke::new(1.0, th.border_subtle))
                                                    .corner_radius(CornerRadius::same(7))
                                                    .inner_margin(Margin::symmetric(12, 10))
                                                    .show(col_ui, |ui| {
                                                        ui.label(
                                                            RichText::new(*lbl)
                                                                .size(11.0)
                                                                .color(TEXT_TERTIARY),
                                                        );
                                                        ui.add_space(2.0);
                                                        ui.label(
                                                            RichText::new(val)
                                                                .size(12.5)
                                                                .strong()
                                                                .color(*col),
                                                        );
                                                    });
                                                col_ui.add_space(6.0);
                                            }
                                        });

                                        ui.add_space(8.0);
                                        egui::Frame::new()
                                            .fill(th.surface_1)
                                            .stroke(Stroke::new(1.0, th.border_subtle))
                                            .corner_radius(CornerRadius::same(7))
                                            .inner_margin(Margin::symmetric(14, 10))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        RichText::new("📂  Chemin local :")
                                                            .size(11.5)
                                                            .color(TEXT_TERTIARY),
                                                    );
                                                    ui.label(
                                                        RichText::new(trunc(&install_path, 60))
                                                            .size(11.5)
                                                            .monospace()
                                                            .color(TEXT_PRIMARY),
                                                    );
                                                    ui.with_layout(
                                                        Layout::right_to_left(Align::Center),
                                                        |ui| {
                                                            if let Some(dir) = &game.install_dir {
                                                                if dir.exists() {
                                                                    let open_btn = egui::Button::new(
                                                                        RichText::new("Ouvrir")
                                                                            .size(11.0)
                                                                            .color(TEXT_PRIMARY),
                                                                    )
                                                                    .fill(th.surface_2)
                                                                    .stroke(Stroke::new(1.0, th.border_default))
                                                                    .corner_radius(CornerRadius::same(5))
                                                                    .min_size(vec2(0.0, 22.0));

                                                                    if ui.add(open_btn).clicked() {
                                                                        actions.push(Action::OpenDir(
                                                                            dir.clone(),
                                                                        ));
                                                                    }
                                                                }
                                                            }
                                                        },
                                                    );
                                                });
                                            });
                                    }

                                    // ──────────────── 1. STATISTIQUES ────────────────
                                    1 => {
                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new("Statistiques Avancées de Session")
                                                .size(13.5)
                                                .strong()
                                                .color(TEXT_PRIMARY),
                                        );
                                        ui.add_space(8.0);

                                        let play_mins = self.total_playtime_mins(&game);
                                        let session_cnt = self
                                            .settings
                                            .session_counts
                                            .get(&game.id)
                                            .copied()
                                            .unwrap_or(0);
                                        let avg_session = play_mins.checked_div(session_cnt).unwrap_or(0);

                                        ui.columns(3, |cols| {
                                            let metrics = [
                                                (
                                                    format_playtime(play_mins),
                                                    "Temps de jeu total",
                                                    accent,
                                                ),
                                                (
                                                    format!("{session_cnt} parties"),
                                                    "Sessions enregistrées",
                                                    Color32::from_rgb(56, 189, 248),
                                                ),
                                                (
                                                    format_playtime(avg_session),
                                                    "Durée moyenne / session",
                                                    SUCCESS,
                                                ),
                                            ];
                                            for (i, (v, l, c)) in metrics.iter().enumerate() {
                                                let col_ui = &mut cols[i];
                                                egui::Frame::new()
                                                    .fill(th.surface_1)
                                                    .stroke(Stroke::new(1.0, th.border_subtle))
                                                    .corner_radius(CornerRadius::same(8))
                                                    .inner_margin(Margin::same(12))
                                                    .show(col_ui, |ui| {
                                                        ui.label(
                                                            RichText::new(*l)
                                                                .size(11.0)
                                                                .color(TEXT_TERTIARY),
                                                        );
                                                        ui.add_space(2.0);
                                                        ui.label(
                                                            RichText::new(v)
                                                                .size(18.0)
                                                                .strong()
                                                                .color(*c),
                                                        );
                                                    });
                                            }
                                        });

                                        ui.add_space(18.0);
                                        ui.label(
                                            RichText::new("Intégrité et Fichiers Locaux")
                                                .size(13.5)
                                                .strong()
                                                .color(TEXT_PRIMARY),
                                        );
                                        ui.add_space(8.0);

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
                                        let dir_exists = game
                                            .install_dir
                                            .as_ref()
                                            .map(|d| d.exists())
                                            .unwrap_or(false);

                                        egui::Frame::new()
                                            .fill(th.surface_1)
                                            .stroke(Stroke::new(1.0, th.border_subtle))
                                            .corner_radius(CornerRadius::same(8))
                                            .inner_margin(Margin::same(14))
                                            .show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        RichText::new("Statut de l'exécutable :")
                                                            .size(12.0)
                                                            .color(TEXT_PRIMARY),
                                                    );
                                                    if exe_exists {
                                                        ui.label(
                                                            RichText::new(
                                                                "✓ Exécutable valide et opérationnel",
                                                            )
                                                            .size(12.0)
                                                            .strong()
                                                            .color(SUCCESS),
                                                        );
                                                    } else {
                                                        ui.label(
                                                            RichText::new(
                                                                "⚠ Exécutable introuvable ou non assigné",
                                                            )
                                                            .size(12.0)
                                                            .strong()
                                                            .color(ERROR),
                                                        );
                                                    }
                                                });
                                                ui.add_space(6.0);
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        RichText::new("Statut du répertoire :")
                                                            .size(12.0)
                                                            .color(TEXT_PRIMARY),
                                                    );
                                                    if dir_exists {
                                                        ui.label(
                                                            RichText::new(
                                                                "✓ Dossier présent sur le disque",
                                                            )
                                                            .size(12.0)
                                                            .color(SUCCESS),
                                                        );
                                                    } else {
                                                        ui.label(
                                                            RichText::new("⚠ Répertoire manquant")
                                                                .size(12.0)
                                                                .color(ERROR),
                                                        );
                                                    }
                                                });
                                                if let Some(exe_p) = &current_exe {
                                                    ui.add_space(8.0);
                                                    ui.label(
                                                        RichText::new(format!(
                                                            "Chemin binaire : {}",
                                                            exe_p.display()
                                                        ))
                                                        .size(11.0)
                                                        .monospace()
                                                        .color(TEXT_TERTIARY),
                                                    );
                                                }
                                            });

                                        ui.add_space(18.0);
                                        ui.label(
                                            RichText::new("Télémétrie Matérielle en Direct")
                                                .size(13.5)
                                                .strong()
                                                .color(TEXT_PRIMARY),
                                        );
                                        ui.add_space(8.0);
                                        let perf = self.perf_monitor.snapshot();
                                        ui.horizontal(|ui| {
                                            let card_w = (ui.available_width() - 16.0) / 3.0;
                                            egui::Frame::new()
                                                .fill(th.surface_1)
                                                .stroke(Stroke::new(1.0, th.border_subtle))
                                                .corner_radius(CornerRadius::same(8))
                                                .inner_margin(Margin::same(12))
                                                .show(ui, |ui| {
                                                    ui.set_width(card_w);
                                                    ui.label(
                                                        RichText::new("CPU")
                                                            .size(11.5)
                                                            .strong()
                                                            .color(Color32::from_rgb(56, 189, 248)),
                                                    );
                                                    ui.label(
                                                        RichText::new(format!("{:.1}%", perf.cpu_usage))
                                                            .size(16.0)
                                                            .strong()
                                                            .color(TEXT_PRIMARY),
                                                    );
                                                    ui.label(
                                                        RichText::new(format!(
                                                            "{:.2} GHz",
                                                            perf.cpu_freq_mhz as f32 / 1000.0
                                                        ))
                                                        .size(10.5)
                                                        .color(TEXT_TERTIARY),
                                                    );
                                                });
                                            egui::Frame::new()
                                                .fill(th.surface_1)
                                                .stroke(Stroke::new(1.0, th.border_subtle))
                                                .corner_radius(CornerRadius::same(8))
                                                .inner_margin(Margin::same(12))
                                                .show(ui, |ui| {
                                                    ui.set_width(card_w);
                                                    ui.label(
                                                        RichText::new("GPU").size(11.5).strong().color(SUCCESS),
                                                    );
                                                    ui.label(
                                                        RichText::new(format!("{:.1}%", perf.gpu_usage))
                                                            .size(16.0)
                                                            .strong()
                                                            .color(TEXT_PRIMARY),
                                                    );
                                                    ui.label(
                                                        RichText::new(format!(
                                                            "{:.1} Go VRAM",
                                                            perf.gpu_vram_used_mb as f32 / 1024.0
                                                        ))
                                                        .size(10.5)
                                                        .color(TEXT_TERTIARY),
                                                    );
                                                });
                                            egui::Frame::new()
                                                .fill(th.surface_1)
                                                .stroke(Stroke::new(1.0, th.border_subtle))
                                                .corner_radius(CornerRadius::same(8))
                                                .inner_margin(Margin::same(12))
                                                .show(ui, |ui| {
                                                    ui.set_width(card_w);
                                                    ui.label(RichText::new("RAM").size(11.5).strong().color(GOLD));
                                                    ui.label(
                                                        RichText::new(format!(
                                                            "{:.1} Go",
                                                            perf.ram_used_mb as f32 / 1024.0
                                                        ))
                                                        .size(16.0)
                                                        .strong()
                                                        .color(TEXT_PRIMARY),
                                                    );
                                                    ui.label(
                                                        RichText::new(format!(
                                                            "Total: {:.0} Go",
                                                            perf.ram_total_mb as f32 / 1024.0
                                                        ))
                                                        .size(10.5)
                                                        .color(TEXT_TERTIARY),
                                                    );
                                                });
                                        });
                                    }

                                    // ──────────────── 2. CAPTURES D'ÉCRAN ────────────────
                                    2 => {
                                        ui.add_space(4.0);
                                        let screenshots = self
                                            .cached_screenshots
                                            .entry(game.id.clone())
                                            .or_insert_with(|| {
                                                scan::find_game_screenshots(&game, &self.settings)
                                            })
                                            .clone();

                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(format!(
                                                    "Galerie de Captures d'Écran ({} trouvée(s))",
                                                    screenshots.len()
                                                ))
                                                .size(13.5)
                                                .strong()
                                                .color(TEXT_PRIMARY),
                                            );
                                            ui.with_layout(
                                                Layout::right_to_left(Align::Center),
                                                |ui| {
                                                    let refresh_btn = egui::Button::new(
                                                        RichText::new("🔄  Actualiser")
                                                            .size(11.5)
                                                            .color(TEXT_PRIMARY),
                                                    )
                                                    .fill(th.surface_1)
                                                    .stroke(Stroke::new(1.0, th.border_default))
                                                    .corner_radius(CornerRadius::same(5))
                                                    .min_size(vec2(0.0, 24.0));

                                                    if ui.add(refresh_btn).clicked() {
                                                        let fresh = scan::find_game_screenshots(
                                                            &game,
                                                            &self.settings,
                                                        );
                                                        self.cached_screenshots
                                                            .insert(game.id.clone(), fresh);
                                                    }

                                                    if let Some(first) = screenshots.first() {
                                                        if let Some(parent) = first.parent() {
                                                            let dir_btn = egui::Button::new(
                                                                RichText::new("📂  Dossier des captures")
                                                                    .size(11.5)
                                                                    .color(TEXT_PRIMARY),
                                                            )
                                                            .fill(th.surface_1)
                                                            .stroke(Stroke::new(1.0, th.border_default))
                                                            .corner_radius(CornerRadius::same(5))
                                                            .min_size(vec2(0.0, 24.0));

                                                            if ui.add(dir_btn).clicked() {
                                                                actions.push(Action::OpenDir(
                                                                    parent.to_path_buf(),
                                                                ));
                                                            }
                                                        }
                                                    }
                                                },
                                            );
                                        });

                                        ui.add_space(10.0);

                                        if screenshots.is_empty() {
                                            egui::Frame::new()
                                                .fill(th.surface_1)
                                                .stroke(Stroke::new(1.0, th.border_subtle))
                                                .corner_radius(CornerRadius::same(10))
                                                .inner_margin(Margin::same(24))
                                                .show(ui, |ui| {
                                                    ui.vertical_centered(|ui| {
                                                        ui.label(RichText::new("📸").size(32.0));
                                                        ui.add_space(6.0);
                                                        ui.label(
                                                            RichText::new(
                                                                "Aucune capture d'écran détectée pour ce titre",
                                                            )
                                                            .size(13.5)
                                                            .strong()
                                                            .color(TEXT_PRIMARY),
                                                        );
                                                        ui.add_space(4.0);
                                                        ui.label(
                                                            RichText::new(
                                                                "UniLauncher inspecte automatiquement le répertoire Steam userdata ainsi que les dossiers 'Screenshots' du jeu.",
                                                            )
                                                            .size(11.5)
                                                            .color(TEXT_SECONDARY),
                                                        );
                                                        ui.add_space(10.0);
                                                        if let Some(dir) = &game.install_dir {
                                                            let open_btn = egui::Button::new(
                                                                RichText::new("📁  Ouvrir le dossier du jeu")
                                                                    .size(12.0)
                                                                    .color(TEXT_PRIMARY),
                                                            )
                                                            .fill(th.surface_2)
                                                            .stroke(Stroke::new(1.0, th.border_default))
                                                            .corner_radius(CornerRadius::same(6))
                                                            .min_size(vec2(0.0, 26.0));

                                                            if ui.add(open_btn).clicked() {
                                                                actions.push(Action::OpenDir(dir.clone()));
                                                            }
                                                        }
                                                    });
                                                });
                                        } else {
                                            ui.columns(3, |cols| {
                                                for (idx_sc, pic_path) in
                                                    screenshots.iter().enumerate()
                                                {
                                                    let col_ui = &mut cols[idx_sc % 3];
                                                    let pic_path_clone = pic_path.clone();
                                                    let img_uri = format!(
                                                        "file://{}",
                                                        pic_path.to_string_lossy().replace('\\', "/")
                                                    );

                                                    egui::Frame::new()
                                                        .fill(th.surface_1)
                                                        .stroke(Stroke::new(1.0, th.border_subtle))
                                                        .corner_radius(CornerRadius::same(7))
                                                        .inner_margin(Margin::same(5))
                                                        .show(col_ui, |ui| {
                                                            let (r, resp) = ui.allocate_exact_size(
                                                                vec2(ui.available_width(), 130.0),
                                                                Sense::click(),
                                                            );
                                                            let img = egui::Image::new(&img_uri)
                                                                .corner_radius(CornerRadius::same(5))
                                                                .fit_to_exact_size(r.size());
                                                            img.paint_at(ui, r);

                                                            if resp.hovered() {
                                                                ui.painter().rect_stroke(
                                                                    r,
                                                                    CornerRadius::same(5),
                                                                    Stroke::new(1.5, accent),
                                                                    StrokeKind::Inside,
                                                                );
                                                                ui.ctx().set_cursor_icon(
                                                                    egui::CursorIcon::PointingHand,
                                                                );
                                                            }
                                                            if resp.clicked() {
                                                                self.selected_screenshot =
                                                                    Some(pic_path_clone);
                                                            }
                                                        });
                                                    col_ui.add_space(8.0);
                                                }
                                            });
                                        }
                                    }

                                    // ──────────────── 3. SUCCÈS & TROPHÉES ────────────────
                                    3 => {
                                        ui.add_space(4.0);
                                        let is_steam = game.id.starts_with("steam:");

                                        if !is_steam {
                                            egui::Frame::new()
                                                .fill(th.surface_1)
                                                .stroke(Stroke::new(1.0, th.border_subtle))
                                                .corner_radius(CornerRadius::same(10))
                                                .inner_margin(Margin::same(24))
                                                .show(ui, |ui| {
                                                    ui.vertical_centered(|ui| {
                                                        ui.label(RichText::new("🏆").size(32.0));
                                                        ui.add_space(6.0);
                                                        ui.label(
                                                            RichText::new(
                                                                "Suivi des succès non disponible pour cette plateforme",
                                                            )
                                                            .size(13.5)
                                                            .strong()
                                                            .color(TEXT_PRIMARY),
                                                        );
                                                        ui.add_space(4.0);
                                                        ui.label(
                                                            RichText::new(
                                                                "Le suivi officiel des trophées en direct est actuellement implémenté pour l'écosystème Steam.",
                                                            )
                                                            .size(11.5)
                                                            .color(TEXT_SECONDARY),
                                                        );
                                                    });
                                                });
                                        } else {
                                            let maybe_achs =
                                                self.cached_achievements.get(&game.id).cloned();
                                            let is_loading =
                                                self.loading_achievements.contains(&game.id);

                                            if is_loading {
                                                ui.vertical_centered(|ui| {
                                                    ui.add_space(20.0);
                                                    ui.spinner();
                                                    ui.add_space(8.0);
                                                    ui.label(
                                                        RichText::new(
                                                            "Interrogation des serveurs Steam pour synchroniser les succès officiels...",
                                                        )
                                                        .size(12.0)
                                                        .color(TEXT_SECONDARY),
                                                    );
                                                });
                                            } else if let Some(Some(achs)) = maybe_achs {
                                                if achs.total == 0 {
                                                    ui.label(
                                                        RichText::new(
                                                            "Aucun succès officiel configuré pour ce titre sur Steam.",
                                                        )
                                                        .size(12.5)
                                                        .color(TEXT_SECONDARY),
                                                    );
                                                } else {
                                                    // Carte de Progression Globale
                                                    egui::Frame::new()
                                                        .fill(th.surface_1)
                                                        .stroke(Stroke::new(1.0, th.border_subtle))
                                                        .corner_radius(CornerRadius::same(9))
                                                        .inner_margin(Margin::same(16))
                                                        .show(ui, |ui| {
                                                            ui.horizontal(|ui| {
                                                                ui.vertical(|ui| {
                                                                    ui.label(
                                                                        RichText::new(
                                                                            "Progression des Succès",
                                                                        )
                                                                        .size(13.5)
                                                                        .strong()
                                                                        .color(TEXT_PRIMARY),
                                                                    );
                                                                    ui.add_space(2.0);
                                                                    ui.label(
                                                                        RichText::new(format!(
                                                                            "{} sur {} débloqués ({:.1}%)",
                                                                            achs.unlocked_count,
                                                                            achs.total,
                                                                            achs.progress_percent
                                                                        ))
                                                                        .size(11.5)
                                                                        .color(TEXT_SECONDARY),
                                                                    );
                                                                });

                                                                ui.with_layout(
                                                                    Layout::right_to_left(
                                                                        Align::Center,
                                                                    ),
                                                                    |ui| {
                                                                        ui.label(
                                                                            RichText::new(format!(
                                                                                "{:.0}%",
                                                                                achs.progress_percent
                                                                            ))
                                                                            .size(20.0)
                                                                            .strong()
                                                                            .color(
                                                                                if achs.progress_percent
                                                                                    >= 100.0
                                                                                {
                                                                                    GOLD
                                                                                } else {
                                                                                    accent
                                                                                },
                                                                            ),
                                                                        );
                                                                    },
                                                                );
                                                            });

                                                            ui.add_space(8.0);
                                                            let pb = egui::ProgressBar::new(
                                                                achs.progress_percent / 100.0,
                                                            )
                                                            .desired_height(6.0)
                                                            .fill(
                                                                if achs.progress_percent >= 100.0 {
                                                                    GOLD
                                                                } else {
                                                                    accent
                                                                },
                                                            );
                                                            ui.add(pb);
                                                        });

                                                    ui.add_space(12.0);

                                                    // Filtres : Tous / Débloqués / Verrouillés
                                                    ui.horizontal(|ui| {
                                                        ui.spacing_mut().item_spacing.x = 4.0;
                                                        let f_tabs = [
                                                            (0, format!("Tous ({})", achs.total)),
                                                            (
                                                                1,
                                                                format!(
                                                                    "🏆  Débloqués ({})",
                                                                    achs.unlocked_count
                                                                ),
                                                            ),
                                                            (
                                                                2,
                                                                format!(
                                                                    "🔒  Verrouillés ({})",
                                                                    achs.total.saturating_sub(
                                                                        achs.unlocked_count,
                                                                    )
                                                                ),
                                                            ),
                                                        ];
                                                        for (f_idx, f_lbl) in f_tabs {
                                                            let sel =
                                                                self.achievements_filter == f_idx;
                                                            let bg = if sel {
                                                                accent.gamma_multiply(0.24)
                                                            } else {
                                                                th.surface_1
                                                            };
                                                            let fg =
                                                                if sel { WHITE } else { TEXT_SECONDARY };
                                                            let btn = egui::Button::new(
                                                                RichText::new(f_lbl)
                                                                    .size(11.5)
                                                                    .strong()
                                                                    .color(fg),
                                                            )
                                                            .fill(bg)
                                                            .stroke(Stroke::new(
                                                                1.0,
                                                                if sel {
                                                                    accent
                                                                } else {
                                                                    th.border_subtle
                                                                },
                                                            ))
                                                            .corner_radius(CornerRadius::same(6))
                                                            .min_size(vec2(0.0, 24.0));

                                                            if ui.add(btn).clicked() {
                                                                self.achievements_filter = f_idx;
                                                            }
                                                        }
                                                    });

                                                    ui.add_space(10.0);

                                                    // Liste des cartes de succès
                                                    let filtered_list: Vec<_> = achs
                                                        .list
                                                        .iter()
                                                        .filter(|a| match self.achievements_filter {
                                                            1 => a.unlocked,
                                                            2 => !a.unlocked,
                                                            _ => true,
                                                        })
                                                        .collect();

                                                    for ach in filtered_list {
                                                        let card_bg = if ach.unlocked {
                                                            th.surface_1
                                                        } else {
                                                            th.surface_1.gamma_multiply(0.65)
                                                        };
                                                        let border_c = if ach.unlocked {
                                                            accent.gamma_multiply(0.40)
                                                        } else {
                                                            th.border_subtle
                                                        };

                                                        egui::Frame::new()
                                                            .fill(card_bg)
                                                            .stroke(Stroke::new(1.0, border_c))
                                                            .corner_radius(CornerRadius::same(7))
                                                            .inner_margin(Margin::symmetric(12, 10))
                                                            .show(ui, |ui| {
                                                                ui.horizontal(|ui| {
                                                                    // Icône trophée ou verrou
                                                                    let (icon_r, _) = ui.allocate_exact_size(
                                                                        vec2(34.0, 34.0),
                                                                        Sense::hover(),
                                                                    );
                                                                    let p = ui.painter();
                                                                    p.rect_filled(
                                                                        icon_r,
                                                                        CornerRadius::same(6),
                                                                        if ach.unlocked {
                                                                            GOLD.gamma_multiply(0.20)
                                                                        } else {
                                                                            th.surface_2
                                                                        },
                                                                    );
                                                                    p.rect_stroke(
                                                                        icon_r,
                                                                        CornerRadius::same(6),
                                                                        Stroke::new(
                                                                            1.0,
                                                                            if ach.unlocked {
                                                                                GOLD
                                                                            } else {
                                                                                th.border_subtle
                                                                            },
                                                                        ),
                                                                        StrokeKind::Inside,
                                                                    );
                                                                    p.text(
                                                                        icon_r.center(),
                                                                        Align2::CENTER_CENTER,
                                                                        if ach.unlocked { "🏆" } else { "🔒" },
                                                                        FontId::proportional(15.0),
                                                                        if ach.unlocked { GOLD } else { TEXT_TERTIARY },
                                                                    );

                                                                    ui.add_space(10.0);

                                                                    ui.vertical(|ui| {
                                                                        ui.label(
                                                                            RichText::new(&ach.name)
                                                                                .size(12.5)
                                                                                .strong()
                                                                                .color(
                                                                                    if ach.unlocked {
                                                                                        TEXT_PRIMARY
                                                                                    } else {
                                                                                        TEXT_SECONDARY
                                                                                    },
                                                                                ),
                                                                        );
                                                                        if !ach.description.is_empty() {
                                                                            ui.add_space(1.0);
                                                                            ui.label(
                                                                                RichText::new(
                                                                                    &ach.description,
                                                                                )
                                                                                .size(11.0)
                                                                                .color(TEXT_TERTIARY),
                                                                            );
                                                                        }
                                                                    });

                                                                    ui.with_layout(
                                                                        Layout::right_to_left(
                                                                            Align::Center,
                                                                        ),
                                                                        |ui| {
                                                                            if ach.unlocked {
                                                                                let date_str = ach
                                                                                    .unlock_time
                                                                                    .map(|t| {
                                                                                        format_last_played(
                                                                                            Some(t),
                                                                                        )
                                                                                    })
                                                                                    .unwrap_or_else(|| {
                                                                                        "Débloqué".into()
                                                                                    });
                                                                                ui.label(
                                                                                    RichText::new(format!(
                                                                                        "✓ {}",
                                                                                        date_str
                                                                                    ))
                                                                                    .size(11.0)
                                                                                    .color(SUCCESS),
                                                                                );
                                                                            } else {
                                                                                ui.label(
                                                                                    RichText::new(
                                                                                        "Verrouillé",
                                                                                    )
                                                                                    .size(11.0)
                                                                                    .color(TEXT_TERTIARY),
                                                                                );
                                                                            }
                                                                        },
                                                                    );
                                                                });
                                                            });
                                                        ui.add_space(4.0);
                                                    }
                                                }
                                            } else {
                                                ui.vertical_centered(|ui| {
                                                    ui.add_space(16.0);
                                                    ui.label(
                                                        RichText::new("Succès Steam non synchronisés")
                                                            .size(13.0)
                                                            .strong()
                                                            .color(TEXT_PRIMARY),
                                                    );
                                                    ui.add_space(4.0);
                                                    ui.label(
                                                        RichText::new(
                                                            "Assurez-vous d'être connecté à Internet ou renseignez votre clé Web API Steam dans les Paramètres.",
                                                        )
                                                        .size(11.5)
                                                        .color(TEXT_SECONDARY),
                                                    );
                                                    ui.add_space(10.0);
                                                    let retry_btn = egui::Button::new(
                                                        RichText::new("🔄  Retenter la synchronisation")
                                                            .size(12.0)
                                                            .color(TEXT_PRIMARY),
                                                    )
                                                    .fill(th.surface_1)
                                                    .stroke(Stroke::new(1.0, th.border_default))
                                                    .corner_radius(CornerRadius::same(6))
                                                    .min_size(vec2(0.0, 26.0));

                                                    if ui.add(retry_btn).clicked() {
                                                        self.cached_achievements.remove(&game.id);
                                                    }
                                                });
                                            }
                                        }
                                    }

                                    // ──────────────── 4. PARAMÈTRES DU JEU ────────────────
                                    4 => {
                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new("Arguments de Ligne de Commande")
                                                .size(13.5)
                                                .strong()
                                                .color(TEXT_PRIMARY),
                                        );
                                        ui.add_space(2.0);
                                        ui.label(
                                            RichText::new(
                                                "Options supplémentaires transmises à l'exécutable lors de son exécution",
                                            )
                                            .size(11.5)
                                            .color(TEXT_SECONDARY),
                                        );
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
                                                .margin(Margin::symmetric(10, 8)),
                                        );
                                        if resp.changed() {
                                            if args.trim().is_empty() {
                                                self.settings.launch_args.remove(&game.id);
                                            } else {
                                                self.settings
                                                    .launch_args
                                                    .insert(game.id.clone(), args.clone());
                                            }
                                            save_needed = true;
                                        }

                                        ui.add_space(10.0);
                                        ui.label(
                                            RichText::new(
                                                "Raccourcis rapides (Cliquer pour ajouter / retirer) :",
                                            )
                                            .size(11.5)
                                            .color(TEXT_TERTIARY),
                                        );
                                        ui.add_space(4.0);
                                        ui.horizontal_wrapped(|ui| {
                                            ui.spacing_mut().item_spacing = vec2(5.0, 5.0);
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
                                                let col = if contains { accent } else { th.surface_1 };
                                                let fg = if contains { WHITE } else { TEXT_SECONDARY };
                                                let btn = egui::Button::new(
                                                    RichText::new(flag).size(11.5).color(fg),
                                                )
                                                .fill(col.gamma_multiply(if contains { 0.30 } else { 1.0 }))
                                                .stroke(Stroke::new(
                                                    1.0,
                                                    if contains {
                                                        accent
                                                    } else {
                                                        th.border_subtle
                                                    },
                                                ))
                                                .corner_radius(CornerRadius::same(5))
                                                .min_size(vec2(0.0, 24.0));

                                                if ui.add(btn).on_hover_text(desc).clicked() {
                                                    if contains {
                                                        args = args.replace(flag, "").trim().to_string();
                                                    } else {
                                                        if !args.is_empty() && !args.ends_with(' ') {
                                                            args.push(' ');
                                                        }
                                                        args.push_str(flag);
                                                    }
                                                    self.settings
                                                        .launch_args
                                                        .insert(game.id.clone(), args.clone());
                                                    save_needed = true;
                                                }
                                            }
                                        });

                                        ui.add_space(18.0);
                                        ui.separator();
                                        ui.add_space(12.0);

                                        // Exécutable personnalisé
                                        ui.label(
                                            RichText::new("Exécutable Principal (.exe)")
                                                .size(13.5)
                                                .strong()
                                                .color(TEXT_PRIMARY),
                                        );
                                        ui.add_space(4.0);
                                        let current_exe = self
                                            .settings
                                            .exe_overrides
                                            .get(&game.id)
                                            .cloned()
                                            .or_else(|| game.exe.clone());

                                        ui.horizontal(|ui| {
                                            let p_str = current_exe
                                                .as_ref()
                                                .map(|p| p.to_string_lossy().to_string())
                                                .unwrap_or_else(|| "Automatique".into());
                                            ui.label(
                                                RichText::new(trunc(&p_str, 50))
                                                    .size(11.5)
                                                    .monospace()
                                                    .color(TEXT_SECONDARY),
                                            );

                                            let browse_btn = egui::Button::new(
                                                RichText::new("📁  Parcourir…")
                                                    .size(11.5)
                                                    .color(TEXT_PRIMARY),
                                            )
                                            .fill(th.surface_1)
                                            .stroke(Stroke::new(1.0, th.border_default))
                                            .corner_radius(CornerRadius::same(5))
                                            .min_size(vec2(0.0, 24.0));

                                            if ui.add(browse_btn).clicked() {
                                                if let Some(p) = rfd::FileDialog::new()
                                                    .add_filter("Exécutables", &["exe", "bat"])
                                                    .pick_file()
                                                {
                                                    self.settings
                                                        .exe_overrides
                                                        .insert(game.id.clone(), p);
                                                    save_needed = true;
                                                    self.toast("Exécutable mis à jour");
                                                }
                                            }

                                            if self.settings.exe_overrides.contains_key(&game.id) {
                                                let reset_btn = egui::Button::new(
                                                    RichText::new("🔄  Réinitialiser")
                                                        .size(11.5)
                                                        .color(TEXT_SECONDARY),
                                                )
                                                .fill(th.surface_1)
                                                .stroke(Stroke::new(1.0, th.border_default))
                                                .corner_radius(CornerRadius::same(5))
                                                .min_size(vec2(0.0, 24.0));

                                                if ui.add(reset_btn).clicked() {
                                                    self.settings.exe_overrides.remove(&game.id);
                                                    save_needed = true;
                                                    self.toast("Exécutable réinitialisé");
                                                }
                                            }
                                        });

                                        ui.add_space(18.0);
                                        ui.separator();
                                        ui.add_space(12.0);

                                        // Jaquette personnalisée
                                        ui.label(
                                            RichText::new("Illustration HD & Jaquette")
                                                .size(13.5)
                                                .strong()
                                                .color(TEXT_PRIMARY),
                                        );
                                        ui.add_space(6.0);
                                        ui.horizontal(|ui| {
                                            let dl_btn = egui::Button::new(
                                                RichText::new("🌐  Télécharger en HD (Steam)")
                                                    .size(11.5)
                                                    .color(TEXT_PRIMARY),
                                            )
                                            .fill(th.surface_1)
                                            .stroke(Stroke::new(1.0, th.border_default))
                                            .corner_radius(CornerRadius::same(5))
                                            .min_size(vec2(0.0, 24.0));
                                            if ui.add(dl_btn).clicked() {
                                                actions.push(Action::DownloadCover(game.id.clone()));
                                            }

                                            let pick_btn = egui::Button::new(
                                                RichText::new("📁  Choisir une image sur le PC…")
                                                    .size(11.5)
                                                    .color(TEXT_PRIMARY),
                                            )
                                            .fill(th.surface_1)
                                            .stroke(Stroke::new(1.0, th.border_default))
                                            .corner_radius(CornerRadius::same(5))
                                            .min_size(vec2(0.0, 24.0));
                                            if ui.add(pick_btn).clicked() {
                                                actions.push(Action::PickCover(game.id.clone()));
                                            }

                                            if self.settings.cover_overrides.contains_key(&game.id) {
                                                let clr_btn = egui::Button::new(
                                                    RichText::new("🗑️  Réinitialiser")
                                                        .size(11.5)
                                                        .color(ERROR),
                                                )
                                                .fill(ERROR.gamma_multiply(0.12))
                                                .stroke(Stroke::new(1.0, ERROR.gamma_multiply(0.35)))
                                                .corner_radius(CornerRadius::same(5))
                                                .min_size(vec2(0.0, 24.0));
                                                if ui.add(clr_btn).clicked() {
                                                    actions.push(Action::ClearCover(game.id.clone()));
                                                }
                                            }
                                        });

                                        ui.add_space(18.0);
                                        ui.separator();
                                        ui.add_space(12.0);

                                        // Sauvegardes
                                        ui.label(
                                            RichText::new("Gestionnaire de Sauvegardes Locales")
                                                .size(13.5)
                                                .strong()
                                                .color(TEXT_PRIMARY),
                                        );
                                        ui.add_space(6.0);
                                        ui.horizontal(|ui| {
                                            let backup_btn = egui::Button::new(
                                                RichText::new("💾  Créer un Backup instantané")
                                                    .size(11.5)
                                                    .color(TEXT_PRIMARY),
                                            )
                                            .fill(th.surface_1)
                                            .stroke(Stroke::new(1.0, th.border_default))
                                            .corner_radius(CornerRadius::same(5))
                                            .min_size(vec2(0.0, 24.0));
                                            if ui.add(backup_btn).clicked() {
                                                actions.push(Action::BackupSave(game.id.clone()));
                                            }

                                            let backup_root = dirs::document_dir()
                                                .map(|d| d.join("UniLauncher_Backups"))
                                                .unwrap_or_else(|| {
                                                    std::path::PathBuf::from("UniLauncher_Backups")
                                                });
                                            if backup_root.exists() {
                                                let open_backup_btn = egui::Button::new(
                                                    RichText::new("📂  Dossier des sauvegardes")
                                                        .size(11.5)
                                                        .color(TEXT_PRIMARY),
                                                )
                                                .fill(th.surface_1)
                                                .stroke(Stroke::new(1.0, th.border_default))
                                                .corner_radius(CornerRadius::same(5))
                                                .min_size(vec2(0.0, 24.0));
                                                if ui.add(open_backup_btn).clicked() {
                                                    actions.push(Action::OpenDir(backup_root));
                                                }
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
                                "👁️  Restaurer dans la bibliothèque"
                            } else {
                                "👁️  Masquer ce jeu"
                            };
                            let hide_btn = egui::Button::new(
                                RichText::new(hide_lbl).size(12.0).color(TEXT_SECONDARY),
                            )
                            .fill(th.surface_1)
                            .stroke(Stroke::new(1.0, th.border_default))
                            .corner_radius(CornerRadius::same(6))
                            .min_size(vec2(0.0, 26.0));

                            if ui.add(hide_btn).clicked() {
                                actions.push(Action::ToggleHide(game.id.clone()));
                                close_modal = true;
                            }

                            if game.source == Source::Manual {
                                let del_btn = egui::Button::new(
                                    RichText::new("🗑️  Supprimer de la liste").size(12.0).color(ERROR),
                                )
                                .fill(ERROR.gamma_multiply(0.12))
                                .stroke(Stroke::new(1.0, ERROR.gamma_multiply(0.35)))
                                .corner_radius(CornerRadius::same(6))
                                .min_size(vec2(0.0, 26.0));

                                if ui.add(del_btn).clicked() {
                                    actions.push(Action::Remove(game.id.clone()));
                                    close_modal = true;
                                }
                            }

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let play_btn = egui::Button::new(
                                    RichText::new("▶  Jouer maintenant")
                                        .size(13.0)
                                        .strong()
                                        .color(WHITE),
                                )
                                .fill(accent)
                                .corner_radius(CornerRadius::same(7))
                                .min_size(vec2(160.0, 34.0));

                                if ui
                                    .add(play_btn)
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    actions.push(Action::Launch(idx));
                                    close_modal = true;
                                }

                                ui.add_space(8.0);
                                let close_btn = egui::Button::new(
                                    RichText::new("Fermer").size(12.0).color(TEXT_SECONDARY),
                                )
                                .fill(th.surface_1)
                                .stroke(Stroke::new(1.0, th.border_default))
                                .corner_radius(CornerRadius::same(6))
                                .min_size(vec2(0.0, 26.0));

                                if ui.add(close_btn).clicked() {
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
            self.edit_info_open = false;
        }

        actions
    }
}
