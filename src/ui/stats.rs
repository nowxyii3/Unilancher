use crate::app::Launcher;
use crate::config::{Game, Source};
use crate::scan;
use crate::theme::{BNET_C, DIM, EA_C, EPIC_C, GOG_C, GOLD, GREEN, RED, STEAM_C, UBI_C, WHITE, XBOX_C};
use crate::types::format_playtime;
use crate::ui::components::trunc;
use eframe::egui::{
    self, vec2, Align, Color32, CornerRadius, Layout, Margin, Rect, RichText, Sense, Stroke, Ui,
};
use std::collections::HashSet;

impl Launcher {
    // ───────────────────── Vue Statistiques & Espace Disque ─────────────────────
    pub fn stats_ui(&mut self, ui: &mut Ui) {
        let th = self.theme();
        let accent = self.accent();

        ui.label(RichText::new("Statistiques & Analyseur de Stockage").size(26.0).strong().color(WHITE));
        ui.add_space(2.0);
        ui.label(
            RichText::new("Vue globale sur l'espace disque, tes temps de jeu et la répartition de ta bibliothèque")
                .size(12.5)
                .color(DIM),
        );
        ui.add_space(18.0);

        let total_games = self.games.len();
        let total_mins: u32 = self.games.iter().map(|g| self.total_playtime_mins(g)).sum();
        let total_bytes: u64 = self.games.iter().map(|g| g.size_bytes).sum();
        let platform_count = self.games.iter().map(|g| g.source).collect::<HashSet<_>>().len();

        egui::ScrollArea::vertical()
            .id_salt("stats_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // 4 Cartes métriques clés
                ui.horizontal(|ui| {
                    let card_w = ((ui.available_width() - 36.0) / 4.0).max(140.0);
                    for (val, lbl, col) in [
                        (scan::format_size(total_bytes), "Espace disque occupé", GOLD),
                        (format!("{:.1} h", total_mins as f32 / 60.0), "Temps de jeu total", Color32::from_rgb(56, 189, 248)),
                        (format!("{total_games} titres"), "Jeux installés", accent),
                        (format!("{platform_count} services"), "Plateformes liées", GREEN),
                    ] {
                        egui::Frame::new()
                            .fill(th.card)
                            .stroke(Stroke::new(1.0, th.card_border))
                            .corner_radius(10.0)
                            .inner_margin(Margin::same(14))
                            .show(ui, |ui| {
                                ui.set_width(card_w);
                                ui.label(RichText::new(val).size(20.0).strong().color(col));
                                ui.add_space(3.0);
                                ui.label(RichText::new(lbl).size(12.0).color(DIM));
                            });
                    }
                });

                ui.add_space(20.0);

                // Répartition par plateforme
                egui::Frame::new()
                    .fill(th.card)
                    .stroke(Stroke::new(1.0, th.card_border))
                    .corner_radius(12.0)
                    .inner_margin(Margin::same(18))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(RichText::new("Répartition par Plateforme").size(16.0).strong().color(WHITE));
                        ui.add_space(10.0);

                        let platforms = [
                            (Source::Steam, STEAM_C),
                            (Source::Epic, EPIC_C),
                            (Source::Gog, GOG_C),
                            (Source::Ubisoft, UBI_C),
                            (Source::Ea, EA_C),
                            (Source::Battlenet, BNET_C),
                            (Source::Xbox, XBOX_C),
                            (Source::Manual, accent),
                        ];

                        for (src, col) in platforms {
                            let matching: Vec<&Game> = self.games.iter().filter(|g| g.source == src).collect();
                            if matching.is_empty() {
                                continue;
                            }
                            let count = matching.len();
                            let p_mins: u32 = matching.iter().map(|g| self.total_playtime_mins(g)).sum();
                            let p_bytes: u64 = matching.iter().map(|g| g.size_bytes).sum();
                            let pct = if total_games > 0 { count as f32 / total_games as f32 } else { 0.0 };

                            ui.horizontal(|ui| {
                                let (r, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                                ui.painter().circle_filled(r.center(), 4.0, col);
                                ui.label(RichText::new(src.label()).size(13.0).strong().color(WHITE));
                                ui.label(RichText::new(format!("•  {count} jeux  •  {:.1} h  •  {}", p_mins as f32 / 60.0, scan::format_size(p_bytes))).size(12.0).color(DIM));

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(RichText::new(format!("{:.0}%", pct * 100.0)).size(12.5).strong().color(WHITE));
                                });
                            });
                            ui.add_space(3.0);
                            let (bar_r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 6.0), Sense::hover());
                            ui.painter().rect_filled(bar_r, CornerRadius::same(3), th.card_border);
                            let fill_w = (bar_r.width() * pct).max(4.0);
                            let fill_r = Rect::from_min_size(bar_r.min, vec2(fill_w, bar_r.height()));
                            ui.painter().rect_filled(fill_r, CornerRadius::same(3), col);
                            ui.add_space(8.0);
                        }
                    });

                ui.add_space(16.0);

                // Podium des jeux les plus joués & Plus gros jeux
                ui.horizontal(|ui| {
                    let half_w = ((ui.available_width() - 14.0) / 2.0).max(200.0);

                    // Top 5 Playtime
                    egui::Frame::new()
                        .fill(th.card)
                        .stroke(Stroke::new(1.0, th.card_border))
                        .corner_radius(12.0)
                        .inner_margin(Margin::same(18))
                        .show(ui, |ui| {
                            ui.set_width(half_w);
                            ui.label(RichText::new("🏆 Jeux les plus joués").size(15.5).strong().color(WHITE));
                            ui.add_space(10.0);

                            let mut sorted_play = self.games.clone();
                            sorted_play.sort_by_key(|a| std::cmp::Reverse(self.total_playtime_mins(a)));
                            let max_play = sorted_play.first().map(|g| self.total_playtime_mins(g)).unwrap_or(1).max(1);

                            for (idx, g) in sorted_play.iter().take(5).enumerate() {
                                let mins = self.total_playtime_mins(g);
                                let ratio = mins as f32 / max_play as f32;
                                ui.horizontal(|ui| {
                                    let rank_col = match idx {
                                        0 => GOLD,
                                        1 => Color32::from_rgb(200, 200, 210),
                                        2 => Color32::from_rgb(205, 127, 50),
                                        _ => DIM,
                                    };
                                    ui.label(RichText::new(format!("#{}", idx + 1)).size(13.0).strong().color(rank_col));
                                    ui.label(RichText::new(trunc(&g.name, 22)).size(13.0).color(WHITE));
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(RichText::new(format_playtime(mins)).size(12.0).strong().color(WHITE));
                                    });
                                });
                                let (bar_r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 4.0), Sense::hover());
                                ui.painter().rect_filled(bar_r, CornerRadius::same(2), th.card_border);
                                let fill_r = Rect::from_min_size(bar_r.min, vec2(bar_r.width() * ratio, bar_r.height()));
                                ui.painter().rect_filled(fill_r, CornerRadius::same(2), accent);
                                ui.add_space(6.0);
                            }
                        });

                    // Top 5 Espace Disque
                    egui::Frame::new()
                        .fill(th.card)
                        .stroke(Stroke::new(1.0, th.card_border))
                        .corner_radius(12.0)
                        .inner_margin(Margin::same(18))
                        .show(ui, |ui| {
                            ui.set_width(half_w);
                            ui.label(RichText::new("💾 Plus volumineux sur le disque").size(15.5).strong().color(WHITE));
                            ui.add_space(10.0);

                            let mut sorted_size = self.games.clone();
                            sorted_size.sort_by_key(|a| std::cmp::Reverse(a.size_bytes));
                            let max_size = sorted_size.first().map(|g| g.size_bytes).unwrap_or(1).max(1);

                            for (idx, g) in sorted_size.iter().take(5).enumerate() {
                                let ratio = g.size_bytes as f32 / max_size as f32;
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("#{}", idx + 1)).size(13.0).color(DIM));
                                    ui.label(RichText::new(trunc(&g.name, 22)).size(13.0).color(WHITE));
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(RichText::new(scan::format_size(g.size_bytes)).size(12.0).strong().color(GOLD));
                                    });
                                });
                                let (bar_r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 4.0), Sense::hover());
                                ui.painter().rect_filled(bar_r, CornerRadius::same(2), th.card_border);
                                let fill_r = Rect::from_min_size(bar_r.min, vec2(bar_r.width() * ratio, bar_r.height()));
                                ui.painter().rect_filled(fill_r, CornerRadius::same(2), GOLD);
                                ui.add_space(6.0);
                            }
                        });
                });

                ui.add_space(16.0);

                // Module Game Performance & Télémétrie en Direct
                let perf = self.perf_monitor.snapshot();
                egui::Frame::new()
                    .fill(th.card)
                    .stroke(Stroke::new(1.0, th.card_border))
                    .corner_radius(12.0)
                    .inner_margin(Margin::same(18))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("🚀 Game Performance & Télémétrie PC en Direct").size(16.0).strong().color(WHITE));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let mut overlay_active = self.settings.overlay.enabled;
                                if ui.checkbox(&mut overlay_active, "Activer l'Overlay In-Game (F9)").changed() {
                                    self.settings.overlay.enabled = overlay_active;
                                    self.settings.save();
                                }
                            });
                        });
                        ui.add_space(2.0);
                        ui.label(RichText::new("Surveillance des ressources CPU, GPU, RAM, FPS et Frametime temps réel sans hook DirectX").size(11.5).color(DIM));
                        ui.add_space(12.0);

                        ui.columns(4, |cols| {
                            // 1. CPU
                            egui::Frame::new()
                                .fill(th.panel)
                                .stroke(Stroke::new(1.0, th.card_border))
                                .corner_radius(8.0)
                                .inner_margin(Margin::same(12))
                                .show(&mut cols[0], |ui| {
                                    ui.label(RichText::new("CPU").size(12.0).strong().color(Color32::from_rgb(56, 189, 248)));
                                    ui.add_space(2.0);
                                    ui.label(RichText::new(format!("{:.1}%", perf.cpu_usage)).size(20.0).strong().color(WHITE));
                                    let temp_s = perf.cpu_temp_c.map(|t| format!("{:.0}°C", t)).unwrap_or_else(|| "N/A".into());
                                    ui.label(RichText::new(format!("Fréq : {:.2} GHz  •  Temp : {}", perf.cpu_freq_mhz as f32 / 1000.0, temp_s)).size(10.5).color(DIM));
                                });

                            // 2. GPU
                            egui::Frame::new()
                                .fill(th.panel)
                                .stroke(Stroke::new(1.0, th.card_border))
                                .corner_radius(8.0)
                                .inner_margin(Margin::same(12))
                                .show(&mut cols[1], |ui| {
                                    ui.label(RichText::new("GPU").size(12.0).strong().color(GREEN));
                                    ui.add_space(2.0);
                                    ui.label(RichText::new(format!("{:.1}%", perf.gpu_usage)).size(20.0).strong().color(WHITE));
                                    ui.label(RichText::new(format!("VRAM : {:.1} / {:.0} Go", perf.gpu_vram_used_mb as f32 / 1024.0, perf.gpu_vram_total_mb as f32 / 1024.0)).size(10.5).color(DIM));
                                });

                            // 3. RAM
                            egui::Frame::new()
                                .fill(th.panel)
                                .stroke(Stroke::new(1.0, th.card_border))
                                .corner_radius(8.0)
                                .inner_margin(Margin::same(12))
                                .show(&mut cols[2], |ui| {
                                    ui.label(RichText::new("RAM").size(12.0).strong().color(GOLD));
                                    ui.add_space(2.0);
                                    ui.label(RichText::new(format!("{:.1} Go", perf.ram_used_mb as f32 / 1024.0)).size(20.0).strong().color(WHITE));
                                    ui.label(RichText::new(format!("Dispo : {:.1} Go  •  Total : {:.0} Go", perf.ram_avail_mb as f32 / 1024.0, perf.ram_total_mb as f32 / 1024.0)).size(10.5).color(DIM));
                                });

                            // 4. FPS & Frametime
                            egui::Frame::new()
                                .fill(th.panel)
                                .stroke(Stroke::new(1.0, th.card_border))
                                .corner_radius(8.0)
                                .inner_margin(Margin::same(12))
                                .show(&mut cols[3], |ui| {
                                    ui.label(RichText::new("EN JEU / FPS").size(12.0).strong().color(accent));
                                    ui.add_space(2.0);
                                    let fps_c = if perf.fps >= 60.0 { GREEN } else if perf.fps >= 30.0 { GOLD } else { RED };
                                    ui.label(RichText::new(format!("{:.0} FPS", perf.fps)).size(20.0).strong().color(fps_c));
                                    ui.label(RichText::new(format!("Avg : {:.0}  •  1% Low : {:.0}  •  {:.1} ms", perf.avg_fps, perf.one_percent_low_fps, perf.frametime_ms)).size(10.0).color(DIM));
                                });
                        });
                    });

                ui.add_space(20.0);
            });
    }
}
