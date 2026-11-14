use crate::app::Launcher;
use crate::config::Source;
use crate::theme::{BNET_C, DIM, EA_C, EPIC_C, GOG_C, STEAM_C, UBI_C, WHITE, XBOX_C};
use crate::types::{format_playtime, Action};
use crate::ui::components::stat_chip;
use eframe::egui::{
    self, vec2, Align, Align2, Color32, CornerRadius, Id, Key, Layout, Margin, RichText, Sense,
    Stroke,
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

        // Overlay sombre sur tout l'écran
        let screen_rect = ctx.content_rect();
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            Id::unique("spotlight_backdrop"),
        ));
        painter.rect_filled(screen_rect, CornerRadius::ZERO, Color32::from_black_alpha(175));

        // Fenêtre flottante centrale Spotlight
        egui::Area::new(Id::unique("spotlight_window"))
            .anchor(Align2::CENTER_CENTER, vec2(0.0, -100.0))
            .order(egui::Order::Tooltip)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(th.card)
                    .stroke(Stroke::new(1.5, accent))
                    .corner_radius(16.0)
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 16],
                        blur: 32,
                        spread: 0,
                        color: Color32::from_black_alpha(200),
                    })
                    .inner_margin(Margin::same(18))
                    .show(ui, |ui| {
                        ui.set_width(580.0);

                        // ── Barre de recherche rapide ──
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🔍").size(18.0));
                            let edit = egui::TextEdit::singleline(&mut self.spotlight_query)
                                .hint_text("Rechercher un jeu (Appuyez sur Entrée pour lancer)...")
                                .font(egui::FontId::proportional(16.0))
                                .text_color(WHITE)
                                .desired_width(ui.available_width() - 40.0);

                            let resp = ui.add(edit);
                            if self.spotlight_focus_needed {
                                resp.request_focus();
                                self.spotlight_focus_needed = false;
                            }
                        });

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // ── Résultats de recherche ──
                        let query = self.spotlight_query.trim().to_lowercase();
                        let matches: Vec<(usize, &crate::config::Game)> = self
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
                            .collect();

                        if matches.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.add_space(14.0);
                                ui.label(RichText::new("Aucun jeu correspondant trouvé.").size(13.0).color(DIM));
                                ui.add_space(14.0);
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
                                if let Some(&(orig_idx, _)) = matches.get(self.spotlight_selected_idx) {
                                    action = Some(Action::Launch(orig_idx));
                                    self.spotlight_open = false;
                                }
                            }

                            for (i, &(orig_idx, g)) in matches.iter().enumerate() {
                                let is_sel = i == self.spotlight_selected_idx;
                                let bg = if is_sel {
                                    accent.gamma_multiply(0.2)
                                } else {
                                    Color32::TRANSPARENT
                                };
                                let border_col = if is_sel {
                                    accent
                                } else {
                                    Color32::TRANSPARENT
                                };

                                let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 44.0), Sense::click());
                                if resp.clicked() {
                                    action = Some(Action::Launch(orig_idx));
                                    self.spotlight_open = false;
                                }
                                if resp.hovered() {
                                    self.spotlight_selected_idx = i;
                                }

                                ui.painter().rect(rect, CornerRadius::same(8), bg, Stroke::new(1.0, border_col), egui::StrokeKind::Inside);

                                let mut item_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink(8.0)));
                                item_ui.horizontal(|ui| {
                                    ui.label(RichText::new(&g.name).size(14.0).strong().color(if is_sel { WHITE } else { DIM }));
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
                                    stat_chip(ui, g.source.label(), src_col);

                                    let playtime = self.total_playtime_mins(g);
                                    if playtime > 0 {
                                        ui.label(RichText::new(format_playtime(playtime)).size(11.5).color(DIM));
                                    }

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if is_sel {
                                            ui.label(RichText::new("⏎ Entrée pour lancer").size(11.0).color(accent));
                                        }
                                    });
                                });
                            }
                        }

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Échap pour fermer • Flèches ↑ ↓ pour naviguer").size(11.0).color(DIM));
                        });
                    });
            });

        action
    }
}
