use crate::app::Launcher;
use crate::emulators::detect_installed_emulators;
use crate::theme::{DIM, GREEN, STEAM_C, WHITE};
use crate::types::Action;
use crate::ui::components::{icon_square_button, stat_chip, Icon};
use eframe::egui::{self, Align, Layout, Margin, RichText, Stroke, Ui};

impl Launcher {
    pub fn emulators_ui(&mut self, ui: &mut Ui) -> Vec<Action> {
        let mut actions = vec![];
        let accent = self.accent();
        let th = self.theme();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Émulateurs & Rétrogaming").size(24.0).strong().color(WHITE));
                    stat_chip(ui, "Scanner Multi-Systèmes", STEAM_C);
                });
                ui.add_space(2.0);
                ui.label(
                    RichText::new("Détection automatique de Dolphin, RPCS3, PCSX2, DuckStation, RetroArch, Cemu et de vos ROMs")
                        .size(12.5)
                        .color(DIM),
                );
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if icon_square_button(ui, Icon::Refresh, "Scanner les dossiers de ROMs", false, th) {
                    self.rescan_roms();
                }

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
            });
        });

        ui.add_space(14.0);

        // ── 1. Liste des Émulateurs supportés et leur statut d'installation ──
        egui::Frame::new()
            .fill(th.card)
            .stroke(Stroke::new(1.0, th.card_border))
            .corner_radius(12.0)
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.label(RichText::new("Émulateurs détectés sur votre PC :").size(14.0).strong().color(WHITE));
                ui.add_space(8.0);

                let emus = detect_installed_emulators();
                if emus.is_empty() {
                    ui.label(
                        RichText::new(
                            "Aucun émulateur standard détecté automatiquement dans C:\\Emulators ou Program Files.",
                        )
                        .size(12.0)
                        .color(DIM),
                    );
                } else {
                    ui.horizontal_wrapped(|ui| {
                        for emu in &emus {
                            egui::Frame::new()
                                .fill(th.bg)
                                .stroke(Stroke::new(1.0, GREEN.gamma_multiply(0.4)))
                                .corner_radius(8.0)
                                .inner_margin(Margin::symmetric(10, 6))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(emu.name).size(13.0).strong().color(WHITE));
                                        stat_chip(ui, emu.system, GREEN);
                                    });
                                });
                            ui.add_space(6.0);
                        }
                    });
                }
            });

        ui.add_space(14.0);

        // ── 2. Liste des ROMs indexées dans la bibliothèque ──
        ui.label(RichText::new("ROMs et jeux rétros indexés :").size(15.0).strong().color(WHITE));
        ui.add_space(8.0);

        let rom_games: Vec<(usize, &crate::config::Game)> = self
            .games
            .iter()
            .enumerate()
            .filter(|(_, g)| g.id.starts_with("rom:"))
            .collect();

        if rom_games.is_empty() {
            egui::Frame::new()
                .fill(th.card)
                .stroke(Stroke::new(1.0, th.card_border))
                .corner_radius(10.0)
                .inner_margin(Margin::same(20))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("Aucune ROM détectée pour le moment.").size(14.0).color(DIM));
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("Ajoutez un dossier contenant vos fichiers .iso, .rvz, .chd, .pkg, .sfc via le bouton '+' ci-dessus.")
                                .size(12.0)
                                .color(DIM),
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
                            .fill(th.card)
                            .stroke(Stroke::new(1.0, th.card_border))
                            .corner_radius(8.0)
                            .inner_margin(Margin::symmetric(14, 10))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&g.name).size(13.5).strong().color(WHITE));
                                    stat_chip(ui, "ROM Rétro", STEAM_C);

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        let btn = egui::Button::new(RichText::new("▶ Lancer").size(12.0).strong().color(WHITE))
                                            .fill(accent)
                                            .corner_radius(6.0);
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
