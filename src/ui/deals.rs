use crate::app::Launcher;
use crate::process::open_uri;
use crate::theme::{DIM, GOLD, GREEN, STEAM_C, WHITE};
use crate::ui::components::{icon_square_button, stat_chip, Icon};
use eframe::egui::{self, Align, Layout, Margin, RichText, Stroke, Ui};

impl Launcher {
    pub fn deals_ui(&mut self, ui: &mut Ui) {
        let accent = self.accent();
        let th = self.theme();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Bons Plans & Jeux Gratuits").size(24.0).strong().color(WHITE));
                    stat_chip(ui, "Epic Games Store", GREEN);
                });
                ui.add_space(2.0);
                ui.label(
                    RichText::new("Suivi automatique des jeux gratuits hebdomadaires sans quitter votre launcher")
                        .size(12.5)
                        .color(DIM),
                );
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if icon_square_button(ui, Icon::Refresh, "Actualiser les offres gratuites", false, th) {
                    self.load_deals_async();
                }
            });
        });

        ui.add_space(16.0);

        if self.loading_deals {
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                ui.spinner();
                ui.add_space(10.0);
                ui.label(RichText::new("Recherche des jeux gratuits en cours...").size(13.5).color(DIM));
            });
            return;
        }

        if self.deals.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                ui.label(RichText::new("Aucune offre gratuite chargée pour l'instant.").size(14.0).color(DIM));
                ui.add_space(10.0);
                let btn = egui::Button::new(RichText::new("Charger les offres").size(13.0).strong().color(WHITE))
                    .fill(accent)
                    .corner_radius(8.0);
                if ui.add(btn).clicked() {
                    self.load_deals_async();
                }
            });
            return;
        }

        egui::ScrollArea::vertical()
            .id_salt("deals_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.columns(2, |cols| {
                    for (i, deal) in self.deals.iter().enumerate() {
                        let col = &mut cols[i % 2];

                        egui::Frame::new()
                            .fill(th.card)
                            .stroke(Stroke::new(1.0, th.card_border))
                            .corner_radius(12.0)
                            .inner_margin(Margin::same(16))
                            .show(col, |ui| {
                                ui.set_width(ui.available_width());

                                ui.horizontal(|ui| {
                                    if deal.is_active_now {
                                        stat_chip(ui, "GRATUIT MAINTENANT", GREEN);
                                    } else {
                                        stat_chip(ui, "PROCHAINEMENT", GOLD);
                                    }

                                    if let Some(ref orig) = deal.original_price {
                                        stat_chip(ui, &format!("Prix habituel: {orig}"), STEAM_C);
                                    }
                                });

                                ui.add_space(8.0);
                                ui.label(RichText::new(&deal.title).size(17.0).strong().color(WHITE));

                                if !deal.description.is_empty() {
                                    ui.add_space(4.0);
                                    ui.label(RichText::new(&deal.description).size(12.0).color(DIM));
                                }

                                ui.add_space(10.0);
                                if let Some(ref end) = deal.end_date {
                                    if deal.is_active_now {
                                        ui.label(RichText::new(format!("⏰ Se termine le {end}")).size(11.5).color(GOLD));
                                    } else if let Some(ref start) = deal.start_date {
                                        ui.label(RichText::new(format!("📅 Disponible du {start} au {end}")).size(11.5).color(STEAM_C));
                                    }
                                }

                                ui.add_space(10.0);
                                let store_btn = egui::Button::new(
                                    RichText::new("Obtenir sur l'Epic Games Store ↗").size(12.5).strong().color(WHITE),
                                )
                                .fill(if deal.is_active_now { GREEN } else { accent })
                                .corner_radius(8.0);

                                if ui.add(store_btn).clicked() {
                                    let _ = open_uri(&deal.store_url);
                                }
                            });

                        col.add_space(14.0);
                    }
                });
            });
    }
}
