use crate::app::Launcher;
use crate::scan;
use crate::theme::{GOLD, TEXT_PRIMARY, TEXT_SECONDARY, WHITE};
use crate::types::{format_playtime, Action};
use crate::ui::components::{cover_colors, draw_gradient_rect, stat_chip};
use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, CornerRadius, FontId, Key, Layout, Rect,
    RichText, Sense, Stroke, StrokeKind, Ui,
};
use std::time::{SystemTime, UNIX_EPOCH};

impl Launcher {
    // ───────────────────── Mode Big Picture / Salon ─────────────────────
    pub fn big_picture_ui(&mut self, ui: &mut Ui) -> Vec<Action> {
        let mut actions = vec![];
        let th = self.theme();
        let accent = self.accent();
        let ctx = ui.ctx().clone();

        let filtered = self.filtered_indices();
        if filtered.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label(RichText::new("Aucun jeu dans cette vue").size(22.0).strong().color(TEXT_PRIMARY));
                ui.add_space(12.0);
                let btn = egui::Button::new(RichText::new("Quitter Big Picture (Échap)").size(13.0).color(WHITE))
                    .fill(th.surface_2)
                    .stroke(Stroke::new(1.0, th.border_default))
                    .corner_radius(CornerRadius::same(6));
                if ui.add(btn).clicked() {
                    self.is_big_picture = false;
                }
            });
            return actions;
        }

        if self.big_picture_index >= filtered.len() {
            self.big_picture_index = 0;
        }

        // Navigation clavier / manette
        ctx.input(|i| {
            if i.key_pressed(Key::ArrowLeft) {
                if self.big_picture_index > 0 {
                    self.big_picture_index -= 1;
                } else {
                    self.big_picture_index = filtered.len() - 1;
                }
            }
            if i.key_pressed(Key::ArrowRight) {
                if self.big_picture_index + 1 < filtered.len() {
                    self.big_picture_index += 1;
                } else {
                    self.big_picture_index = 0;
                }
            }
            if i.key_pressed(Key::Enter) || i.key_pressed(Key::Space) {
                actions.push(Action::Launch(filtered[self.big_picture_index]));
            }
            if i.key_pressed(Key::F) {
                actions.push(Action::Fav(self.games[filtered[self.big_picture_index]].id.clone()));
            }
            if i.key_pressed(Key::P) {
                actions.push(Action::OpenDetails(self.games[filtered[self.big_picture_index]].id.clone()));
            }
            if i.key_pressed(Key::Escape) || i.key_pressed(Key::F10) {
                self.is_big_picture = false;
            }
        });

        let current_game_idx = filtered[self.big_picture_index];
        let g = self.games[current_game_idx].clone();
        let is_fav = self.settings.favorites.contains(&g.id);
        let play_mins = self.total_playtime_mins(&g);
        let tex = self.get_cover_texture(&ctx, &g);

        // ── 1. En-tête Big Picture ──
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            ui.label(RichText::new("🎮  UniLauncher Big Picture").size(17.0).strong().color(TEXT_PRIMARY));

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let quit_btn = egui::Button::new(
                    RichText::new("✕  Quitter (Échap / F10)").size(12.0).color(TEXT_SECONDARY),
                )
                .fill(th.surface_2)
                .stroke(Stroke::new(1.0, th.border_default))
                .corner_radius(CornerRadius::same(6))
                .min_size(vec2(0.0, 26.0));

                if ui.add(quit_btn).clicked() {
                    self.is_big_picture = false;
                }

                ui.add_space(14.0);
                let time_str = {
                    let dur = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
                    let hours = (dur / 3600 % 24 + 1) % 24;
                    let mins = dur / 60 % 60;
                    format!("{:02}:{:02}", hours, mins)
                };
                ui.label(RichText::new(time_str).size(15.0).strong().color(TEXT_PRIMARY));
            });
        });

        ui.add_space(20.0);

        // ── 2. Section centrale : Focus sur le jeu sélectionné ──
        ui.horizontal(|ui| {
            ui.add_space(28.0);
            let cover_size = vec2(250.0, 355.0);
            let (cover_rect, _) = ui.allocate_exact_size(cover_size, Sense::hover());
            let p = ui.painter();

            // Halo d'ambiance doux
            p.rect_filled(cover_rect.expand(4.0), CornerRadius::same(16), accent.gamma_multiply(0.20));
            p.rect_filled(cover_rect, CornerRadius::same(14), th.surface_2);

            let inner = cover_rect.shrink(3.0);
            let clip = p.with_clip_rect(inner);
            if let Some(t) = tex.as_ref() {
                clip.image(t.id(), inner, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), WHITE);
            } else {
                let (c1, c2) = cover_colors(&g.name);
                draw_gradient_rect(&clip, inner, c1, c2);
                let initials: String = g
                    .name
                    .split_whitespace()
                    .filter_map(|w| w.chars().next())
                    .take(2)
                    .collect::<String>()
                    .to_uppercase();
                clip.text(
                    inner.center() - vec2(0.0, 16.0),
                    Align2::CENTER_CENTER,
                    if initials.is_empty() { "?" } else { &initials },
                    FontId::proportional(42.0),
                    Color32::from_white_alpha(40),
                );
            }
            p.rect_stroke(cover_rect, CornerRadius::same(14), Stroke::new(1.8, accent), StrokeKind::Inside);

            ui.add_space(32.0);

            ui.vertical(|ui| {
                ui.add_space(10.0);
                ui.label(RichText::new(&g.name).size(28.0).strong().color(TEXT_PRIMARY));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    stat_chip(ui, g.source.label(), accent);
                    stat_chip(ui, &format_playtime(play_mins), Color32::from_rgb(56, 189, 248));
                    if g.size_bytes > 0 {
                        stat_chip(ui, &scan::format_size(g.size_bytes), GOLD);
                    }
                });

                ui.add_space(24.0);

                // Bouton principal : Lancer le jeu
                let btn_play = egui::Button::new(
                    RichText::new("▶  Lancer le jeu (Entrée / A)").size(15.0).strong().color(WHITE),
                )
                .fill(accent)
                .corner_radius(CornerRadius::same(8))
                .min_size(vec2(280.0, 46.0));

                if ui.add(btn_play).clicked() {
                    actions.push(Action::Launch(current_game_idx));
                }

                ui.add_space(10.0);

                // Favori
                let fav_txt = if is_fav { "★  Retirer des favoris (F)" } else { "☆  Ajouter aux favoris (F)" };
                let btn_fav = egui::Button::new(RichText::new(fav_txt).size(13.5).color(TEXT_PRIMARY))
                    .fill(th.surface_2)
                    .stroke(Stroke::new(1.0, th.border_default))
                    .corner_radius(CornerRadius::same(8))
                    .min_size(vec2(280.0, 36.0));

                if ui.add(btn_fav).clicked() {
                    actions.push(Action::Fav(g.id.clone()));
                }

                ui.add_space(8.0);

                // Propriétés & Détails
                let btn_det = egui::Button::new(RichText::new("⚙  Fiche détaillée & Propriétés (P)").size(13.0).color(TEXT_SECONDARY))
                    .fill(th.surface_2)
                    .stroke(Stroke::new(1.0, th.border_default))
                    .corner_radius(CornerRadius::same(8))
                    .min_size(vec2(280.0, 34.0));

                if ui.add(btn_det).clicked() {
                    actions.push(Action::OpenDetails(g.id.clone()));
                }
            });
        });

        ui.add_space(26.0);

        // ── 3. Carrousel horizontal en bas ──
        ui.horizontal(|ui| {
            ui.add_space(28.0);
            let start = self.big_picture_index.saturating_sub(3);
            let end = (self.big_picture_index + 4).min(filtered.len());

            #[allow(clippy::needless_range_loop)]
            for idx in start..end {
                let real_idx = filtered[idx];
                let item_game = self.games[real_idx].clone();
                let is_cur = idx == self.big_picture_index;
                let item_w = if is_cur { 108.0 } else { 86.0 };
                let item_h = if is_cur { 152.0 } else { 122.0 };
                let (r, resp) = ui.allocate_exact_size(vec2(item_w, item_h), Sense::click());
                let p = ui.painter();
                let round = CornerRadius::same(8);

                p.rect_filled(r, round, th.surface_2);
                let item_tex = self.get_cover_texture(&ctx, &item_game);
                let inner = r.shrink(2.0);
                let clip = p.with_clip_rect(inner);

                if let Some(t) = item_tex {
                    clip.image(t.id(), inner, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), WHITE);
                } else {
                    let (c1, c2) = cover_colors(&item_game.name);
                    draw_gradient_rect(&clip, inner, c1, c2);
                }

                if is_cur {
                    p.rect_stroke(r, round, Stroke::new(2.0, accent), StrokeKind::Inside);
                } else {
                    p.rect_stroke(r, round, Stroke::new(1.0, th.border_subtle), StrokeKind::Inside);
                }

                if resp.clicked() {
                    self.big_picture_index = idx;
                }
            }
        });

        actions
    }
}
