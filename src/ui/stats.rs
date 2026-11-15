use crate::app::Launcher;
use crate::config::{Game, Source};
use crate::scan;
use crate::theme::{
    BNET_C, EA_C, EPIC_C, ERROR, GOG_C, GOLD, STEAM_C, SUCCESS, TEXT_PRIMARY, TEXT_SECONDARY,
    TEXT_TERTIARY, UBI_C, XBOX_C,
};
use crate::types::format_playtime;
use crate::ui::components::trunc;
use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, CornerRadius, FontId, Layout, Margin, Rect, RichText, Sense, Stroke,
    Ui,
};
use std::collections::HashSet;

impl Launcher {
    // ───────────────────── Vue Statistiques & Espace Disque ─────────────────────
    pub fn stats_ui(&mut self, ui: &mut Ui) {
        let th = self.theme();
        let accent = self.accent();

        // ─── En-tête de la page ───
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Statistiques & Analyseur de Stockage")
                        .size(22.0)
                        .strong()
                        .color(TEXT_PRIMARY),
                );
                ui.add_space(2.0);
                ui.label(
                    RichText::new(
                        "Vue globale sur l'espace disque, le temps de jeu et la répartition de votre bibliothèque",
                    )
                    .size(12.0)
                    .color(TEXT_SECONDARY),
                );
            });
        });

        ui.add_space(16.0);

        let total_games = self.games.len();
        let total_mins: u32 = self.games.iter().map(|g| self.total_playtime_mins(g)).sum();
        let total_bytes: u64 = self.games.iter().map(|g| g.size_bytes).sum();
        let platform_count = self
            .games
            .iter()
            .map(|g| g.source)
            .collect::<HashSet<_>>()
            .len();

        egui::ScrollArea::vertical()
            .id_salt("stats_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // ─── 4 Cartes métriques clés ───
                ui.horizontal(|ui| {
                    let card_w = ((ui.available_width() - 36.0) / 4.0).max(130.0);
                    for (val, lbl, col) in [
                        (scan::format_size(total_bytes), "Espace disque occupé", GOLD),
                        (
                            format!("{:.1} h", total_mins as f32 / 60.0),
                            "Temps de jeu total",
                            Color32::from_rgb(56, 189, 248),
                        ),
                        (format!("{total_games} titres"), "Jeux installés", accent),
                        (format!("{platform_count} services"), "Plateformes liées", SUCCESS),
                    ] {
                        egui::Frame::new()
                            .fill(th.surface_2)
                            .stroke(Stroke::new(1.0, th.card_border))
                            .corner_radius(CornerRadius::same(10))
                            .inner_margin(Margin::same(14))
                            .show(ui, |ui| {
                                ui.set_width(card_w);
                                ui.label(RichText::new(lbl).size(11.5).color(TEXT_SECONDARY));
                                ui.add_space(4.0);
                                ui.label(RichText::new(val).size(20.0).strong().color(col));
                            });
                    }
                });

                ui.add_space(16.0);

                // ─── Répartition par Plateforme ───
                egui::Frame::new()
                    .fill(th.surface_2)
                    .stroke(Stroke::new(1.0, th.card_border))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::same(18))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(
                            RichText::new("Répartition par Plateforme")
                                .size(14.5)
                                .strong()
                                .color(TEXT_PRIMARY),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(
                                "Proportion de jeux, volume disque et temps passé par magasin",
                            )
                            .size(11.5)
                            .color(TEXT_SECONDARY),
                        );
                        ui.add_space(12.0);

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

                        // Barre segmentée proportionnelle (style macOS / GitHub)
                        let (stacked_r, _) =
                            ui.allocate_exact_size(vec2(ui.available_width(), 8.0), Sense::hover());
                        ui.painter()
                            .rect_filled(stacked_r, CornerRadius::same(4), th.surface_1);

                        if total_games > 0 {
                            let mut cur_x = stacked_r.left();
                            let total_w = stacked_r.width();
                            for (src, col) in platforms {
                                let matching_count =
                                    self.games.iter().filter(|g| g.source == src).count();
                                if matching_count == 0 {
                                    continue;
                                }
                                let seg_w =
                                    (total_w * (matching_count as f32 / total_games as f32)).max(2.0);
                                let seg_r = Rect::from_min_max(
                                    pos2(cur_x, stacked_r.top()),
                                    pos2((cur_x + seg_w).min(stacked_r.right()), stacked_r.bottom()),
                                );
                                ui.painter().rect_filled(seg_r, CornerRadius::same(2), col);
                                cur_x += seg_w;
                            }
                        }

                        ui.add_space(14.0);

                        for (src, col) in platforms {
                            let matching: Vec<&Game> =
                                self.games.iter().filter(|g| g.source == src).collect();
                            if matching.is_empty() {
                                continue;
                            }
                            let count = matching.len();
                            let p_mins: u32 =
                                matching.iter().map(|g| self.total_playtime_mins(g)).sum();
                            let p_bytes: u64 = matching.iter().map(|g| g.size_bytes).sum();
                            let pct = if total_games > 0 {
                                count as f32 / total_games as f32
                            } else {
                                0.0
                            };

                            let avail_w = ui.available_width();
                            let pct_w = 48.0;
                            let info_w = (avail_w - pct_w - 12.0).max(100.0);

                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 6.0;
                                let (r, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                                ui.painter().circle_filled(r.center(), 3.5, col);

                                let (info_r, _) = ui.allocate_exact_size(vec2(info_w, 20.0), Sense::hover());
                                let summary_text = format!(
                                    "{}  •  {count} jeux  •  {:.1} h  •  {}",
                                    src.label(),
                                    p_mins as f32 / 60.0,
                                    scan::format_size(p_bytes)
                                );
                                ui.painter().text(
                                    pos2(info_r.left(), info_r.center().y),
                                    Align2::LEFT_CENTER,
                                    trunc(&summary_text, (info_w / 7.0).max(15.0) as usize),
                                    FontId::proportional(12.0),
                                    TEXT_PRIMARY,
                                );

                                let (pct_r, _) = ui.allocate_exact_size(vec2(pct_w, 20.0), Sense::hover());
                                ui.painter().text(
                                    pos2(pct_r.right(), pct_r.center().y),
                                    Align2::RIGHT_CENTER,
                                    format!("{:.0}%", pct * 100.0),
                                    FontId::proportional(12.0),
                                    TEXT_PRIMARY,
                                );
                            });
                            ui.add_space(3.0);
                            let (bar_r, _) =
                                ui.allocate_exact_size(vec2(ui.available_width(), 4.0), Sense::hover());
                            ui.painter()
                                .rect_filled(bar_r, CornerRadius::same(2), th.surface_1);
                            let fill_w = (bar_r.width() * pct).max(2.0);
                            let fill_r = Rect::from_min_size(bar_r.min, vec2(fill_w, bar_r.height()));
                            ui.painter().rect_filled(fill_r, CornerRadius::same(2), col);
                            ui.add_space(8.0);
                        }
                    });

                ui.add_space(16.0);

                // ─── Podium des jeux les plus joués & Plus gros jeux ───
                ui.horizontal(|ui| {
                    let half_w = ((ui.available_width() - 14.0) / 2.0).max(200.0);

                    // Top 5 Playtime
                    egui::Frame::new()
                        .fill(th.surface_2)
                        .stroke(Stroke::new(1.0, th.card_border))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::same(18))
                        .show(ui, |ui| {
                            ui.set_width(half_w);
                            ui.label(
                                RichText::new("🏆  Jeux les plus joués")
                                    .size(14.5)
                                    .strong()
                                    .color(TEXT_PRIMARY),
                            );
                            ui.add_space(10.0);

                            let mut sorted_play = self.games.clone();
                            sorted_play
                                .sort_by_key(|a| std::cmp::Reverse(self.total_playtime_mins(a)));
                            let max_play = sorted_play
                                .first()
                                .map(|g| self.total_playtime_mins(g))
                                .unwrap_or(1)
                                .max(1);

                            for (idx, g) in sorted_play.iter().take(5).enumerate() {
                                let mins = self.total_playtime_mins(g);
                                let ratio = mins as f32 / max_play as f32;
                                let rank_col = match idx {
                                    0 => GOLD,
                                    1 => Color32::from_rgb(226, 232, 240),
                                    2 => Color32::from_rgb(245, 158, 11),
                                    _ => TEXT_TERTIARY,
                                };

                                let avail_w = ui.available_width();
                                let rank_w = 28.0;
                                let val_w = 95.0;
                                let name_w = (avail_w - rank_w - val_w - 14.0).max(60.0);
                                let char_limit = (name_w / 7.2).max(10.0) as usize;

                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 6.0;
                                    let (rank_r, _) = ui.allocate_exact_size(vec2(rank_w, 20.0), Sense::hover());
                                    ui.painter().text(
                                        pos2(rank_r.left(), rank_r.center().y),
                                        Align2::LEFT_CENTER,
                                        format!("#{}", idx + 1),
                                        FontId::proportional(12.0),
                                        rank_col,
                                    );
                                    let (name_r, name_resp) = ui.allocate_exact_size(vec2(name_w, 20.0), Sense::hover());
                                    ui.painter().text(
                                        pos2(name_r.left(), name_r.center().y),
                                        Align2::LEFT_CENTER,
                                        trunc(&g.name, char_limit),
                                        FontId::proportional(12.5),
                                        TEXT_PRIMARY,
                                    );
                                    name_resp.on_hover_text(&g.name);

                                    let (val_r, _) = ui.allocate_exact_size(vec2(val_w, 20.0), Sense::hover());
                                    ui.painter().text(
                                        pos2(val_r.right(), val_r.center().y),
                                        Align2::RIGHT_CENTER,
                                        format_playtime(mins),
                                        FontId::proportional(11.5),
                                        TEXT_SECONDARY,
                                    );
                                });
                                let (bar_r, _) =
                                    ui.allocate_exact_size(vec2(ui.available_width(), 3.5), Sense::hover());
                                ui.painter()
                                    .rect_filled(bar_r, CornerRadius::same(2), th.surface_1);
                                let fill_r = Rect::from_min_size(
                                    bar_r.min,
                                    vec2(bar_r.width() * ratio, bar_r.height()),
                                );
                                ui.painter().rect_filled(fill_r, CornerRadius::same(2), accent);
                                ui.add_space(7.0);
                            }
                        });

                    // Top 5 Espace Disque
                    egui::Frame::new()
                        .fill(th.surface_2)
                        .stroke(Stroke::new(1.0, th.card_border))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::same(18))
                        .show(ui, |ui| {
                            ui.set_width(half_w);
                            ui.label(
                                RichText::new("💾  Plus volumineux sur le disque")
                                    .size(14.5)
                                    .strong()
                                    .color(TEXT_PRIMARY),
                            );
                            ui.add_space(10.0);

                            let mut sorted_size = self.games.clone();
                            sorted_size.sort_by_key(|a| std::cmp::Reverse(a.size_bytes));
                            let max_size = sorted_size
                                .first()
                                .map(|g| g.size_bytes)
                                .unwrap_or(1)
                                .max(1);

                            for (idx, g) in sorted_size.iter().take(5).enumerate() {
                                let ratio = g.size_bytes as f32 / max_size as f32;
                                let avail_w = ui.available_width();
                                let rank_w = 28.0;
                                let val_w = 95.0;
                                let name_w = (avail_w - rank_w - val_w - 14.0).max(60.0);
                                let char_limit = (name_w / 7.2).max(10.0) as usize;

                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 6.0;
                                    let (rank_r, _) = ui.allocate_exact_size(vec2(rank_w, 20.0), Sense::hover());
                                    ui.painter().text(
                                        pos2(rank_r.left(), rank_r.center().y),
                                        Align2::LEFT_CENTER,
                                        format!("#{}", idx + 1),
                                        FontId::proportional(12.0),
                                        TEXT_TERTIARY,
                                    );
                                    let (name_r, name_resp) = ui.allocate_exact_size(vec2(name_w, 20.0), Sense::hover());
                                    ui.painter().text(
                                        pos2(name_r.left(), name_r.center().y),
                                        Align2::LEFT_CENTER,
                                        trunc(&g.name, char_limit),
                                        FontId::proportional(12.5),
                                        TEXT_PRIMARY,
                                    );
                                    name_resp.on_hover_text(&g.name);

                                    let (val_r, _) = ui.allocate_exact_size(vec2(val_w, 20.0), Sense::hover());
                                    ui.painter().text(
                                        pos2(val_r.right(), val_r.center().y),
                                        Align2::RIGHT_CENTER,
                                        scan::format_size(g.size_bytes),
                                        FontId::proportional(11.5),
                                        GOLD,
                                    );
                                });
                                let (bar_r, _) =
                                    ui.allocate_exact_size(vec2(ui.available_width(), 3.5), Sense::hover());
                                ui.painter()
                                    .rect_filled(bar_r, CornerRadius::same(2), th.surface_1);
                                let fill_r = Rect::from_min_size(
                                    bar_r.min,
                                    vec2(bar_r.width() * ratio, bar_r.height()),
                                );
                                ui.painter().rect_filled(fill_r, CornerRadius::same(2), GOLD);
                                ui.add_space(7.0);
                            }
                        });
                });

                ui.add_space(16.0);

                // ─── Module Game Performance & Télémétrie en Direct ───
                let perf = self.perf_monitor.snapshot();
                egui::Frame::new()
                    .fill(th.surface_2)
                    .stroke(Stroke::new(1.0, th.card_border))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::same(18))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("🚀  Game Performance & Télémétrie PC en Direct")
                                    .size(14.5)
                                    .strong()
                                    .color(TEXT_PRIMARY),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let mut overlay_active = self.settings.overlay.enabled;
                                if ui
                                    .checkbox(&mut overlay_active, "Activer l'Overlay In-Game (F9)")
                                    .changed()
                                {
                                    self.settings.overlay.enabled = overlay_active;
                                    self.settings.save();
                                }
                            });
                        });
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new(
                                "Surveillance des ressources CPU, GPU, RAM, FPS et Frametime en temps réel sans hook DirectX",
                            )
                            .size(11.5)
                            .color(TEXT_SECONDARY),
                        );
                        ui.add_space(12.0);

                        ui.columns(4, |cols| {
                            // 1. CPU
                            egui::Frame::new()
                                .fill(th.surface_1)
                                .stroke(Stroke::new(1.0, th.border_subtle))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::same(12))
                                .show(&mut cols[0], |ui| {
                                    ui.label(
                                        RichText::new("CPU")
                                            .size(11.5)
                                            .strong()
                                            .color(Color32::from_rgb(56, 189, 248)),
                                    );
                                    ui.add_space(2.0);
                                    ui.label(
                                        RichText::new(format!("{:.1}%", perf.cpu_usage))
                                            .size(20.0)
                                            .strong()
                                            .color(TEXT_PRIMARY),
                                    );
                                    ui.add_space(3.0);
                                    // Mini barre CPU
                                    let (bar_r, _) = ui
                                        .allocate_exact_size(vec2(ui.available_width(), 3.0), Sense::hover());
                                    ui.painter()
                                        .rect_filled(bar_r, CornerRadius::same(1), th.card_border);
                                    let fill_w = (bar_r.width()
                                        * (perf.cpu_usage / 100.0).clamp(0.0, 1.0))
                                    .max(2.0);
                                    ui.painter().rect_filled(
                                        Rect::from_min_size(bar_r.min, vec2(fill_w, bar_r.height())),
                                        CornerRadius::same(1),
                                        Color32::from_rgb(56, 189, 248),
                                    );

                                    ui.add_space(4.0);
                                    let temp_s = perf
                                        .cpu_temp_c
                                        .map(|t| format!("{:.0}°C", t))
                                        .unwrap_or_else(|| "N/A".into());
                                    ui.label(
                                        RichText::new(format!(
                                            "Fréq : {:.2} GHz  •  Temp : {}",
                                            perf.cpu_freq_mhz as f32 / 1000.0,
                                            temp_s
                                        ))
                                        .size(10.5)
                                        .color(TEXT_TERTIARY),
                                    );
                                });

                            // 2. GPU
                            egui::Frame::new()
                                .fill(th.surface_1)
                                .stroke(Stroke::new(1.0, th.border_subtle))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::same(12))
                                .show(&mut cols[1], |ui| {
                                    ui.label(RichText::new("GPU").size(11.5).strong().color(SUCCESS));
                                    ui.add_space(2.0);
                                    ui.label(
                                        RichText::new(format!("{:.1}%", perf.gpu_usage))
                                            .size(20.0)
                                            .strong()
                                            .color(TEXT_PRIMARY),
                                    );
                                    ui.add_space(3.0);
                                    // Mini barre GPU
                                    let (bar_r, _) = ui
                                        .allocate_exact_size(vec2(ui.available_width(), 3.0), Sense::hover());
                                    ui.painter()
                                        .rect_filled(bar_r, CornerRadius::same(1), th.card_border);
                                    let fill_w = (bar_r.width()
                                        * (perf.gpu_usage / 100.0).clamp(0.0, 1.0))
                                    .max(2.0);
                                    ui.painter().rect_filled(
                                        Rect::from_min_size(bar_r.min, vec2(fill_w, bar_r.height())),
                                        CornerRadius::same(1),
                                        SUCCESS,
                                    );

                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(format!(
                                            "VRAM : {:.1} / {:.0} Go",
                                            perf.gpu_vram_used_mb as f32 / 1024.0,
                                            perf.gpu_vram_total_mb as f32 / 1024.0
                                        ))
                                        .size(10.5)
                                        .color(TEXT_TERTIARY),
                                    );
                                });

                            // 3. RAM
                            egui::Frame::new()
                                .fill(th.surface_1)
                                .stroke(Stroke::new(1.0, th.border_subtle))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::same(12))
                                .show(&mut cols[2], |ui| {
                                    ui.label(RichText::new("RAM").size(11.5).strong().color(GOLD));
                                    ui.add_space(2.0);
                                    ui.label(
                                        RichText::new(format!(
                                            "{:.1} Go",
                                            perf.ram_used_mb as f32 / 1024.0
                                        ))
                                        .size(20.0)
                                        .strong()
                                        .color(TEXT_PRIMARY),
                                    );
                                    ui.add_space(3.0);
                                    // Mini barre RAM
                                    let ram_ratio = if perf.ram_total_mb > 0 {
                                        perf.ram_used_mb as f32 / perf.ram_total_mb as f32
                                    } else {
                                        0.0
                                    };
                                    let (bar_r, _) = ui
                                        .allocate_exact_size(vec2(ui.available_width(), 3.0), Sense::hover());
                                    ui.painter()
                                        .rect_filled(bar_r, CornerRadius::same(1), th.card_border);
                                    let fill_w = (bar_r.width() * ram_ratio.clamp(0.0, 1.0)).max(2.0);
                                    ui.painter().rect_filled(
                                        Rect::from_min_size(bar_r.min, vec2(fill_w, bar_r.height())),
                                        CornerRadius::same(1),
                                        GOLD,
                                    );

                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(format!(
                                            "Dispo : {:.1} Go  •  Total : {:.0} Go",
                                            perf.ram_avail_mb as f32 / 1024.0,
                                            perf.ram_total_mb as f32 / 1024.0
                                        ))
                                        .size(10.5)
                                        .color(TEXT_TERTIARY),
                                    );
                                });

                            // 4. FPS & Frametime
                            egui::Frame::new()
                                .fill(th.surface_1)
                                .stroke(Stroke::new(1.0, th.border_subtle))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::same(12))
                                .show(&mut cols[3], |ui| {
                                    ui.label(
                                        RichText::new("EN JEU / FPS")
                                            .size(11.5)
                                            .strong()
                                            .color(accent),
                                    );
                                    ui.add_space(2.0);
                                    let fps_c = if perf.fps >= 60.0 {
                                        SUCCESS
                                    } else if perf.fps >= 30.0 {
                                        GOLD
                                    } else {
                                        ERROR
                                    };
                                    ui.label(
                                        RichText::new(format!("{:.0} FPS", perf.fps))
                                            .size(20.0)
                                            .strong()
                                            .color(fps_c),
                                    );
                                    ui.add_space(7.0);
                                    ui.label(
                                        RichText::new(format!(
                                            "Avg : {:.0}  •  1% Low : {:.0}  •  {:.1} ms",
                                            perf.avg_fps,
                                            perf.one_percent_low_fps,
                                            perf.frametime_ms
                                        ))
                                        .size(10.0)
                                        .color(TEXT_TERTIARY),
                                    );
                                });
                        });
                    });

                ui.add_space(20.0);
            });
    }
}
