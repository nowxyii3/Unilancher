use crate::app::Launcher;
use crate::config::Source;
use crate::theme::{
    BNET_C, EA_C, EPIC_C, GOG_C, STEAM_C, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_TERTIARY,
    UBI_C, WHITE, XBOX_C,
};
use crate::types::{format_playtime, Action};
use crate::ui::components::{draw_icon, stat_chip, Icon};
use eframe::egui::{
    self, vec2, Align, Align2, Color32, CornerRadius, FontId, Id, Key, Layout, Margin,
    RichText, Sense, Stroke, StrokeKind,
};

impl Launcher {
    pub fn spotlight_ui(&mut self, ctx: &egui::Context) -> Option<Action> {
        if !self.spotlight_open {
            return None;
        }

        let mut action = None;
        let th = self.theme();
        let accent = self.accent();

        // Gestion de la touche Échap pour fermer
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.spotlight_open = false;
            return None;
        }

        // Overlay sombre sur tout l'écran avec fermeture au clic extérieur
        let screen_rect = ctx.content_rect();
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            Id::unique("spotlight_backdrop"),
        ));
        painter.rect_filled(screen_rect, CornerRadius::ZERO, Color32::from_black_alpha(190));

        // Fenêtre flottante centrale Spotlight (Raycast style)
        egui::Area::new(Id::unique("spotlight_window"))
            .anchor(Align2::CENTER_CENTER, vec2(0.0, -80.0))
            .order(egui::Order::Tooltip)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(th.surface_2)
                    .stroke(Stroke::new(1.2, th.border_hover.lerp_to_gamma(accent, 0.45)))
                    .corner_radius(CornerRadius::same(14))
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 18],
                        blur: 36,
                        spread: 0,
                        color: Color32::from_black_alpha(220),
                    })
                    .inner_margin(Margin::ZERO)
                    .show(ui, |ui| {
                        ui.set_width(620.0);

                        // ── 1. Barre de recherche rapide ──
                        egui::Frame::new()
                            .inner_margin(Margin::symmetric(18, 14))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 10.0;
                                    let (icon_r, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
                                    draw_icon(ui.painter(), Icon::Search, icon_r.center(), accent);

                                    let edit = egui::TextEdit::singleline(&mut self.spotlight_query)
                                        .hint_text("Rechercher un jeu ou une plateforme (Entrée pour lancer)...")
                                        .font(FontId::proportional(15.0))
                                        .text_color(TEXT_PRIMARY)
                                        .desired_width(ui.available_width() - 60.0);

                                    let resp = ui.add(edit);
                                    if self.spotlight_focus_needed {
                                        resp.request_focus();
                                        self.spotlight_focus_needed = false;
                                    }

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        egui::Frame::new()
                                            .fill(th.surface_1)
                                            .stroke(Stroke::new(1.0, th.border_subtle))
                                            .corner_radius(CornerRadius::same(5))
                                            .inner_margin(Margin::symmetric(6, 2))
                                            .show(ui, |ui| {
                                                ui.label(RichText::new("Échap").size(10.5).color(TEXT_TERTIARY));
                                            });
                                    });
                                });
                            });

                        // Séparateur fin
                        let (sep_r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
                        ui.painter().rect_filled(sep_r, CornerRadius::ZERO, th.border_subtle);

                        // ── 2. Résultats de recherche ──
                        let query = self.spotlight_query.trim().to_lowercase();
                        let matches: Vec<(usize, crate::config::Game, u32)> = self
                            .games
                            .iter()
                            .enumerate()
                            .filter(|(_, g)| {
                                if self.settings.hidden_games.contains(&g.id) {
                                    return false;
                                }
                                if query.is_empty() {
                                    true
                                } else {
                                    g.name.to_lowercase().contains(&query)
                                }
                            })
                            .take(6)
                            .map(|(i, g)| (i, g.clone(), self.total_playtime_mins(g)))
                            .collect();

                        if matches.is_empty() {
                            egui::Frame::new()
                                .inner_margin(Margin::same(24))
                                .show(ui, |ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.label(
                                            RichText::new("Aucun jeu correspondant dans votre bibliothèque.")
                                                .size(13.0)
                                                .color(TEXT_SECONDARY),
                                        );
                                    });
                                });
                        } else {
                            // Ajuster l'index sélectionné au clavier
                            if self.spotlight_selected_idx >= matches.len() {
                                self.spotlight_selected_idx = 0;
                            }

                            if ctx.input(|i| i.key_pressed(Key::ArrowDown)) {
                                if self.spotlight_selected_idx + 1 < matches.len() {
                                    self.spotlight_selected_idx += 1;
                                } else {
                                    self.spotlight_selected_idx = 0;
                                }
                            }
                            if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
                                if self.spotlight_selected_idx > 0 {
                                    self.spotlight_selected_idx -= 1;
                                } else {
                                    self.spotlight_selected_idx = matches.len().saturating_sub(1);
                                }
                            }

                            // Touche Entrée pour lancer directement le jeu sélectionné
                            if ctx.input(|i| i.key_pressed(Key::Enter)) {
                                if let Some(&(orig_idx, _, _)) = matches.get(self.spotlight_selected_idx) {
                                    action = Some(Action::Launch(orig_idx));
                                    self.spotlight_open = false;
                                }
                            }

                            egui::Frame::new()
                                .inner_margin(Margin::symmetric(10, 8))
                                .show(ui, |ui| {
                                    for (i, (orig_idx, g, playtime)) in matches.iter().enumerate() {
                                        let is_sel = i == self.spotlight_selected_idx;
                                        let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 46.0), Sense::click());
                                        let hov_t = ui.ctx().animate_bool(Id::unique(("sp_item_hov", *orig_idx)), resp.hovered() || is_sel);

                                        if resp.clicked() {
                                            action = Some(Action::Launch(*orig_idx));
                                            self.spotlight_open = false;
                                        }
                                        if resp.hovered() {
                                            self.spotlight_selected_idx = i;
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }

                                        let p = ui.painter();
                                        let bg = if is_sel {
                                            accent.gamma_multiply(0.18)
                                        } else {
                                            th.surface_2.lerp_to_gamma(th.surface_3, hov_t)
                                        };
                                        let border_col = if is_sel {
                                            accent.gamma_multiply(0.70)
                                        } else {
                                            th.border_subtle.lerp_to_gamma(th.border_default, hov_t)
                                        };

                                        p.rect(rect, CornerRadius::same(8), bg, Stroke::new(1.0, border_col), StrokeKind::Inside);

                                        let mut item_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink2(vec2(12.0, 6.0))));
                                        item_ui.horizontal(|ui| {
                                            ui.spacing_mut().item_spacing.x = 8.0;

                                            let src_col = match g.source {
                                                Source::Steam => STEAM_C,
                                                Source::Epic => EPIC_C,
                                                Source::Gog => GOG_C,
                                                Source::Ubisoft => UBI_C,
                                                Source::Ea => EA_C,
                                                Source::Battlenet => BNET_C,
                                                Source::Xbox => XBOX_C,
                                                Source::Manual => accent,
                                            };

                                            let (dot_r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                                            ui.painter().circle_filled(dot_r.center(), 3.5, src_col);

                                            ui.label(
                                                RichText::new(&g.name)
                                                    .size(13.5)
                                                    .strong()
                                                    .color(if is_sel { TEXT_PRIMARY } else { TEXT_SECONDARY }),
                                            );

                                            stat_chip(ui, g.source.label(), src_col);

                                            if *playtime > 0 {
                                                ui.label(RichText::new(format_playtime(*playtime)).size(11.5).color(TEXT_TERTIARY));
                                            }

                                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                if is_sel {
                                                    egui::Frame::new()
                                                        .fill(accent.gamma_multiply(0.25))
                                                        .stroke(Stroke::new(1.0, accent.gamma_multiply(0.75)))
                                                        .corner_radius(CornerRadius::same(5))
                                                        .inner_margin(Margin::symmetric(8, 3))
                                                        .show(ui, |ui| {
                                                            ui.label(RichText::new("⏎  Lancer").size(11.0).strong().color(WHITE));
                                                        });
                                                }
                                            });
                                        });

                                        ui.add_space(2.0);
                                    }
                                });
                        }

                        // ── 3. Barre inférieure de raccourcis ──
                        egui::Frame::new()
                            .fill(th.surface_1)
                            .corner_radius(CornerRadius {
                                nw: 0,
                                ne: 0,
                                sw: 14,
                                se: 14,
                            })
                            .inner_margin(Margin::symmetric(16, 9))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 12.0;

                                    ui.label(RichText::new("↑ ↓  Naviguer").size(11.0).color(TEXT_TERTIARY));
                                    ui.label(RichText::new("•").size(11.0).color(th.border_subtle));
                                    ui.label(RichText::new("⏎  Lancer").size(11.0).color(TEXT_TERTIARY));
                                    ui.label(RichText::new("•").size(11.0).color(th.border_subtle));
                                    ui.label(RichText::new("Échap  Fermer").size(11.0).color(TEXT_TERTIARY));

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(
                                            RichText::new(format!("{} jeux indexés", self.games.len()))
                                                .size(10.5)
                                                .color(TEXT_TERTIARY),
                                        );
                                    });
                                });
                            });
                    });
            });

        action
    }
}
