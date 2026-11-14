use crate::app::Launcher;
use crate::emulators::detect_installed_emulators;
use crate::theme::{STEAM_C, SUCCESS, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_TERTIARY, WHITE};
use crate::types::Action;
use crate::ui::components::{icon_square_button, stat_chip, Icon};
use eframe::egui::{self, vec2, Align, CornerRadius, Layout, Margin, RichText, Stroke, Ui};

impl Launcher {
    pub fn emulators_ui(&mut self, ui: &mut Ui) -> Vec<Action> {
        let mut actions = vec![];
        let accent = self.accent();
        let th = self.theme();

        // ─── En-tête de la page ───
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    ui.label(
                        RichText::new("Émulateurs & Rétrogaming")
                            .size(22.0)
                            .strong()
                            .color(TEXT_PRIMARY),
                    );
                    stat_chip(ui, "Scanner Multi-Systèmes", STEAM_C);
                });
                ui.add_space(2.0);
                ui.label(
                    RichText::new(
                        "Détection automatique de Dolphin, RPCS3, PCSX2, DuckStation, RetroArch, Cemu et de vos ROMs",
                    )
                    .size(12.0)
                    .color(TEXT_SECONDARY),
                );
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 6.0;

                if icon_square_button(ui, Icon::Plus, "Ajouter un dossier de ROMs", false, th) {
                    if let Some(folder) = rfd::FileDialog::new()
                        .set_title("Sélectionner un dossier contenant des ROMs")
                        .pick_folder()
                    {
                        self.settings.custom_scan_folders.push(folder);
                        self.settings.save();
                        self.rescan_roms();
                    }
                }

                if icon_square_button(ui, Icon::Refresh, "Scanner les dossiers de ROMs", false, th) {
                    self.rescan_roms();
                }
            });
        });

        ui.add_space(16.0);

        // ── 1. Liste des Émulateurs supportés et leur statut d'installation ──
        egui::Frame::new()
            .fill(th.surface_2)
            .stroke(Stroke::new(1.0, th.card_border))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(18))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new("Émulateurs détectés sur votre système :")
                        .size(14.0)
                        .strong()
                        .color(TEXT_PRIMARY),
                );
                ui.add_space(8.0);

                let emus = detect_installed_emulators();
                if emus.is_empty() {
                    ui.label(
                        RichText::new(
                            "Aucun émulateur standard détecté automatiquement dans C:\\Emulators ou Program Files.",
                        )
                        .size(12.0)
                        .color(TEXT_TERTIARY),
                    );
                } else {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
                        for emu in &emus {
                            egui::Frame::new()
                                .fill(th.surface_1)
                                .stroke(Stroke::new(1.0, SUCCESS.gamma_multiply(0.35)))
                                .corner_radius(CornerRadius::same(8))
                                .inner_margin(Margin::symmetric(12, 7))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 6.0;
                                        let (dot_r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), egui::Sense::hover());
                                        ui.painter().circle_filled(dot_r.center(), 3.5, SUCCESS);

                                        ui.label(RichText::new(emu.name).size(12.5).strong().color(TEXT_PRIMARY));
                                        stat_chip(ui, emu.system, SUCCESS);
                                    });
                                });
                        }
                    });
                }
            });

        ui.add_space(16.0);

        // ── 2. Liste des ROMs indexées dans la bibliothèque ──
        let rom_games: Vec<(usize, &crate::config::Game)> = self
            .games
            .iter()
            .enumerate()
            .filter(|(_, g)| g.id.starts_with("rom:"))
            .collect();

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("ROMs et jeux rétros indexés ({})", rom_games.len()))
                    .size(14.5)
                    .strong()
                    .color(TEXT_PRIMARY),
            );
        });
        ui.add_space(8.0);

        if rom_games.is_empty() {
            egui::Frame::new()
                .fill(th.surface_2)
                .stroke(Stroke::new(1.0, th.card_border))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::same(30))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Aucune ROM détectée pour le moment.")
                                .size(14.0)
                                .strong()
                                .color(TEXT_PRIMARY),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(
                                "Ajoutez un dossier contenant vos fichiers .iso, .rvz, .chd, .pkg, .sfc via le bouton '+' ci-dessus.",
                            )
                            .size(12.0)
                            .color(TEXT_SECONDARY),
                        );
                    });
                });
        } else {
            egui::ScrollArea::vertical()
                .id_salt("roms_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for (orig_idx, g) in rom_games {
                        egui::Frame::new()
                            .fill(th.surface_2)
                            .stroke(Stroke::new(1.0, th.card_border))
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(Margin::symmetric(14, 10))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 8.0;

                                    ui.label(RichText::new("🕹").size(14.0));
                                    ui.label(
                                        RichText::new(&g.name)
                                            .size(13.5)
                                            .strong()
                                            .color(TEXT_PRIMARY),
                                    );
                                    stat_chip(ui, "ROM Rétro", STEAM_C);

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        let btn = egui::Button::new(
                                            RichText::new("▶  Lancer")
                                                .size(12.0)
                                                .strong()
                                                .color(WHITE),
                                        )
                                        .fill(accent.gamma_multiply(0.25))
                                        .stroke(Stroke::new(1.0, accent.gamma_multiply(0.70)))
                                        .corner_radius(CornerRadius::same(6))
                                        .min_size(vec2(0.0, 26.0));

                                        if ui.add(btn).clicked() {
                                            actions.push(Action::Launch(orig_idx));
                                        }
                                    });
                                });
                            });
                        ui.add_space(6.0);
                    }
                });
        }

        actions
    }
}
