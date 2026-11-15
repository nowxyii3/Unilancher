use crate::app::Launcher;
use crate::deals::{
    current_unix_timestamp, DealCategory, DealPlatform, DealType, DealsSortMode, DealsTab, FreeDeal,
};
use crate::process::open_uri;
use crate::theme::{
    ThemeColors, CARD_BORDER, ERROR, GOLD, GREEN, INFO, RED, SUCCESS, TEXT_PRIMARY,
    TEXT_SECONDARY, TEXT_TERTIARY, WARNING, WHITE,
};
use crate::ui::components::{icon_square_button, pill, stat_chip, Icon};
use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, CornerRadius, FontId, Id, Key, Layout, Margin,
    Rect, RichText, Sense, Stroke, StrokeKind, Ui,
};

// ═══════════════════════════════════════════════════════════════════════════════
//  UniLauncher — Plateforme Premium de Bons Plans & Jeux Gratuits
// ═══════════════════════════════════════════════════════════════════════════════

impl Launcher {
    pub fn deals_ui(&mut self, ui: &mut Ui) {
        let accent = self.accent();
        let th = self.theme();

        egui::ScrollArea::vertical()
            .id_salt("deals_page_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(4.0);

                // ── 1. SECTION HERO IMMERSIVE ──
                self.deals_hero_ui(ui, accent, th);

                ui.add_space(20.0);

                // ── 2. BARRE D'ONGLETS / CATÉGORIES ──
                self.deals_tabs_ui(ui, accent, th);

                ui.add_space(14.0);

                // ── 3. BARRE D'OUTILS & FILTRES ──
                self.deals_toolbar_ui(ui, accent, th);

                ui.add_space(16.0);

                // ── 4. BANNIÈRE D'ALERTE EXPIRATION FAVORIS ──
                self.deals_favorite_alert_ui(ui, accent, th);

                // ── 5. GRILLE DES BONS PLANS OU ÉTAT VIDE ──
                let filtered = self.filtered_deals();

                if self.loading_deals && self.deals.is_empty() {
                    self.deals_skeleton_loader_ui(ui, th);
                } else if filtered.is_empty() {
                    self.deals_empty_state_ui(ui, accent, th);
                } else {
                    self.deals_grid_ui(ui, &filtered, accent, th);
                }

                ui.add_space(32.0);
            });
    }

    // ───────────────────────────── Section Hero ──────────────────────────────
    fn deals_hero_ui(&mut self, ui: &mut Ui, accent: Color32, th: ThemeColors) {
        let available_w = ui.available_width();

        // ── En-tête avec Titre et Actions ──
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    ui.label(
                        RichText::new("Bons Plans & Jeux Gratuits")
                            .size(24.0)
                            .strong()
                            .color(TEXT_PRIMARY),
                    );
                    pill(
                        ui.painter(),
                        ui.cursor().min + vec2(0.0, 4.0),
                        "OFFICIEL & VÉRIFIÉ",
                        GREEN.gamma_multiply(0.20),
                        GREEN,
                    );
                });
                ui.add_space(3.0);
                ui.label(
                    RichText::new(
                        "Suivez et réclamez en direct les jeux offerts à 100%, réductions temporaires, DLCs et logiciels indispensables",
                    )
                    .size(12.5)
                    .color(TEXT_SECONDARY),
                );
            });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 8.0;

                // Bouton d'actualisation manuelle
                let refresh_tip = if self.loading_deals {
                    "Actualisation en cours..."
                } else {
                    "Actualiser le catalogue des bons plans"
                };
                if icon_square_button(ui, Icon::Refresh, refresh_tip, self.loading_deals, th) {
                    self.load_deals_async();
                }

                // Horodatage de la dernière synchronisation
                if let Some(ts) = self.last_deals_sync {
                    let now = current_unix_timestamp() as u64;
                    let diff_mins = (now.saturating_sub(ts)) / 60;
                    let sync_text = if diff_mins == 0 {
                        "Actualisé à l'instant".to_string()
                    } else if diff_mins < 60 {
                        format!("Actualisé il y a {diff_mins} min")
                    } else {
                        let hours = diff_mins / 60;
                        format!("Actualisé il y a {hours}h")
                    };
                    ui.label(
                        RichText::new(sync_text)
                            .size(11.0)
                            .color(TEXT_TERTIARY),
                    );
                }
            });
        });

        ui.add_space(14.0);

        // ── Bandeau de Métriques Clés (KPI Bar) ──
        let total_count = self.deals.len();
        let limited_count = self
            .deals
            .iter()
            .filter(|d| d.is_active_now && d.end_timestamp.is_some())
            .count();
        let fav_count = self
            .deals
            .iter()
            .filter(|d| self.settings.favorite_deals.contains(&d.id))
            .count();
        let software_count = self
            .deals
            .iter()
            .filter(|d| d.category == DealCategory::Software || d.category == DealCategory::Creative || d.category == DealCategory::DevTool || d.category == DealCategory::Productivity)
            .count();

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            self.kpi_chip(ui, "🎁", format!("{total_count} Bons plans"), "Actuellement disponibles", th.surface_2, accent);
            self.kpi_chip(ui, "⏳", format!("{limited_count} Offres limitées"), "À réclamer avant fin", th.surface_2, WARNING);
            self.kpi_chip(ui, "💻", format!("{software_count} Outils & Logiciels"), "100% gratuits & libres", th.surface_2, INFO);
            self.kpi_chip(ui, "⭐", format!("{fav_count} Favoris"), "Bons plans suivis", th.surface_2, GOLD);
        });

        ui.add_space(16.0);

        // ── Grand Format : Showcase Deal Vedette de la Semaine ──
        let featured_deals: Vec<FreeDeal> = self.deals.iter().filter(|d| d.is_featured).cloned().collect();
        if !featured_deals.is_empty() {
            let active_idx = self.hero_featured_index % featured_deals.len();
            let featured = featured_deals[active_idx].clone();
            self.showcase_banner_ui(ui, &featured, featured_deals.len(), active_idx, available_w, accent, th);
        }
    }

    fn kpi_chip(
        &self,
        ui: &mut Ui,
        icon_str: &str,
        primary: String,
        secondary: &'static str,
        bg: Color32,
        _accent_col: Color32,
    ) {
        egui::Frame::new()
            .fill(bg)
            .stroke(Stroke::new(1.0, CARD_BORDER))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::symmetric(12, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(icon_str).size(16.0));
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(primary)
                                .size(12.5)
                                .strong()
                                .color(TEXT_PRIMARY),
                        );
                        ui.label(
                            RichText::new(secondary)
                                .size(10.5)
                                .color(TEXT_TERTIARY),
                        );
                    });
                });
            });
    }

    // ───────────────────────────── Bannière Vedette ──────────────────────────
    #[allow(clippy::too_many_arguments)]
    fn showcase_banner_ui(
        &mut self,
        ui: &mut Ui,
        deal: &FreeDeal,
        total_featured: usize,
        current_idx: usize,
        width: f32,
        accent: Color32,
        th: ThemeColors,
    ) {
        let banner_h = 175.0;
        let (rect, resp) = ui.allocate_exact_size(vec2(width, banner_h), Sense::hover());
        let p = ui.painter().with_clip_rect(rect);

        let hov = resp.hovered();
        let border_col = if hov { accent } else { th.card_border };

        // Fond avec dégradé subtil et coins arrondis
        p.rect_filled(rect, CornerRadius::same(12), th.surface_2);

        // Halo lumineux discret lors du survol
        if hov {
            p.rect_filled(
                rect.expand(1.5),
                CornerRadius::same(13),
                accent.gamma_multiply(0.08),
            );
        }
        p.rect_stroke(rect, CornerRadius::same(12), Stroke::new(1.0, border_col), StrokeKind::Inside);

        // 1. Miniature / Jaquette à gauche
        let thumb_w = 240.0f32.min(width * 0.35);
        let thumb_rect = Rect::from_min_size(rect.min + vec2(12.0, 12.0), vec2(thumb_w, banner_h - 24.0));

        let tex = self.get_deal_texture(ui.ctx(), deal);
        if let Some(ref t) = tex {
            p.image(
                t.id(),
                thumb_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                WHITE,
            );
        } else {
            // Placeholder élégant pendant le chargement
            p.rect_filled(thumb_rect, CornerRadius::same(8), th.surface_3);
            let center = thumb_rect.center();
            p.text(
                center,
                Align2::CENTER_CENTER,
                deal.category.icon(),
                FontId::proportional(28.0),
                accent,
            );
        }
        p.rect_stroke(
            thumb_rect,
            CornerRadius::same(8),
            Stroke::new(1.0, th.card_border),
            StrokeKind::Inside,
        );

        // 2. Zone d'informations centrale
        let info_x = thumb_rect.right() + 18.0;
        let right_actions_w = 170.0;
        let info_w = (rect.right() - right_actions_w - info_x).max(100.0);
        let info_rect = Rect::from_min_size(
            pos2(info_x, rect.top() + 14.0),
            vec2(info_w, banner_h - 28.0),
        );

        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(info_rect)
                .layout(Layout::top_down(Align::Min)),
        );

        child_ui.horizontal(|ui| {
            stat_chip(ui, "🔥 EN VEDETTE CETTE SEMAINE", accent);
            stat_chip(ui, deal.platform.name(), deal.platform.color());
            stat_chip(ui, deal.category.label(), TEXT_TERTIARY);
        });

        child_ui.add_space(6.0);
        child_ui.label(
            RichText::new(&deal.title)
                .size(19.0)
                .strong()
                .color(TEXT_PRIMARY),
        );

        child_ui.add_space(4.0);
        let clean_desc = if deal.description.len() > 160 {
            format!("{}...", &deal.description[..157])
        } else {
            deal.description.clone()
        };
        child_ui.label(
            RichText::new(clean_desc)
                .size(11.5)
                .color(TEXT_SECONDARY),
        );

        child_ui.add_space(8.0);
        child_ui.horizontal(|ui| {
            if let Some(ref orig) = deal.original_price {
                ui.label(
                    RichText::new(format!("Habituellement : {orig}"))
                        .size(12.0)
                        .strikethrough()
                        .color(RED),
                );
            }
            ui.label(
                RichText::new("0,00 € (GRATUIT)")
                    .size(13.0)
                    .strong()
                    .color(SUCCESS),
            );

            if let Some(ref time_str) = deal.remaining_time_str() {
                ui.add_space(8.0);
                ui.label(
                    RichText::new(time_str)
                        .size(11.5)
                        .strong()
                        .color(WARNING),
                );
            }
        });

        // 3. Actions à droite
        let actions_rect = Rect::from_min_max(
            pos2(rect.right() - right_actions_w, rect.top() + 14.0),
            pos2(rect.right() - 14.0, rect.bottom() - 14.0),
        );

        let mut act_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(actions_rect)
                .layout(Layout::top_down(Align::Max)),
        );

        // Bouton Récupérer
        let claim_btn = egui::Button::new(
            RichText::new("⚡ Récupérer ↗")
                .size(13.0)
                .strong()
                .color(WHITE),
        )
        .fill(accent.gamma_multiply(0.35))
        .stroke(Stroke::new(1.0, accent.gamma_multiply(0.85)))
        .corner_radius(CornerRadius::same(7))
        .min_size(vec2(150.0, 36.0));

        if act_ui.add(claim_btn).clicked() {
            let _ = open_uri(&deal.store_url);
        }

        act_ui.add_space(6.0);

        // Bouton Détails
        let details_btn = egui::Button::new(
            RichText::new("Fiche détaillée")
                .size(11.5)
                .color(TEXT_PRIMARY),
        )
        .fill(th.surface_3)
        .stroke(Stroke::new(1.0, th.card_border))
        .corner_radius(CornerRadius::same(6))
        .min_size(vec2(150.0, 28.0));

        if act_ui.add(details_btn).clicked() {
            self.selected_deal_modal = Some(deal.clone());
        }

        act_ui.add_space(8.0);

        // Pagination du showcase si plusieurs offres vedettes
        if total_featured > 1 {
            act_ui.horizontal(|ui| {
                if ui.small_button("◀").clicked() {
                    self.hero_featured_index = if current_idx == 0 {
                        total_featured - 1
                    } else {
                        current_idx - 1
                    };
                }
                ui.label(
                    RichText::new(format!("{}/{}", current_idx + 1, total_featured))
                        .size(11.0)
                        .color(TEXT_TERTIARY),
                );
                if ui.small_button("▶").clicked() {
                    self.hero_featured_index = (current_idx + 1) % total_featured;
                }
            });
        }
    }

    // ───────────────────────────── Onglets Principaux ────────────────────────
    fn deals_tabs_ui(&mut self, ui: &mut Ui, accent: Color32, th: ThemeColors) {
        let tabs = [
            (DealsTab::All, "🔥 Tout"),
            (DealsTab::Games, "🎮 Jeux Gratuits"),
            (DealsTab::EndingSoon, "⚡ Offres Limitées"),
            (DealsTab::ToKeep, "🎁 -100% À Garder"),
            (DealsTab::Software, "💻 Logiciels & Outils"),
            (DealsTab::Discoveries, "💎 Pépites & Découvertes"),
            (DealsTab::Favorites, "⭐ Mes Favoris"),
        ];

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;

            for (tab, label) in tabs {
                let is_active = self.deals_active_tab == tab;
                let count = self.tab_deal_count(tab);

                let fill = if is_active {
                    accent.gamma_multiply(0.24)
                } else {
                    th.surface_2
                };
                let stroke = if is_active {
                    Stroke::new(1.2, accent)
                } else {
                    Stroke::new(1.0, th.card_border)
                };
                let text_col = if is_active { WHITE } else { TEXT_SECONDARY };

                let text = format!("{label} ({count})");
                let btn = egui::Button::new(
                    RichText::new(text)
                        .size(12.0)
                        .strong()
                        .color(text_col),
                )
                .fill(fill)
                .stroke(stroke)
                .corner_radius(CornerRadius::same(7))
                .min_size(vec2(0.0, 32.0));

                if ui.add(btn).clicked() {
                    self.deals_active_tab = tab;
                }
            }
        });
    }

    fn tab_deal_count(&self, tab: DealsTab) -> usize {
        match tab {
            DealsTab::All => self.deals.len(),
            DealsTab::Games => self
                .deals
                .iter()
                .filter(|d| d.category == DealCategory::Game || d.category == DealCategory::Dlc)
                .count(),
            DealsTab::EndingSoon => self
                .deals
                .iter()
                .filter(|d| d.is_active_now && d.end_timestamp.is_some() && !d.is_expired())
                .count(),
            DealsTab::ToKeep => self
                .deals
                .iter()
                .filter(|d| d.deal_type == DealType::FreeToKeep)
                .count(),
            DealsTab::Software => self
                .deals
                .iter()
                .filter(|d| {
                    d.category == DealCategory::Software
                        || d.category == DealCategory::Creative
                        || d.category == DealCategory::DevTool
                        || d.category == DealCategory::Productivity
                })
                .count(),
            DealsTab::Discoveries => self
                .deals
                .iter()
                .filter(|d| d.category == DealCategory::Indie)
                .count(),
            DealsTab::Favorites => self
                .deals
                .iter()
                .filter(|d| self.settings.favorite_deals.contains(&d.id))
                .count(),
        }
    }

    // ───────────────────────────── Barre d'Outils & Filtres ──────────────────
    fn deals_toolbar_ui(&mut self, ui: &mut Ui, accent: Color32, th: ThemeColors) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;

            // Barre de recherche instantanée
            let search_w = 260.0f32.min(ui.available_width() * 0.45);
            let search_rect = Rect::from_min_size(ui.cursor().min, vec2(search_w, 32.0));
            let mut search_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(search_rect)
                    .layout(Layout::left_to_right(Align::Center)),
            );

            egui::Frame::new()
                .fill(th.surface_2)
                .stroke(Stroke::new(1.0, th.card_border))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(8, 5))
                .show(&mut search_ui, |ui| {
                    ui.label(RichText::new("🔍").size(12.0));
                    let edit = egui::TextEdit::singleline(&mut self.deals_search)
                        .hint_text("Rechercher un jeu ou logiciel...")
                        .frame(egui::Frame::NONE)
                        .desired_width(search_w - 55.0);
                    ui.add(edit);

                    if !self.deals_search.is_empty() && ui.small_button("✕").clicked() {
                        self.deals_search.clear();
                    }
                });

            ui.add_space(search_w);

            // Filtre par plateforme
            ui.label(RichText::new("Plateforme :").size(11.5).color(TEXT_TERTIARY));

            let platforms = [
                (None, "Toutes"),
                (Some(DealPlatform::Epic), "Epic"),
                (Some(DealPlatform::Steam), "Steam"),
                (Some(DealPlatform::Gog), "GOG"),
                (Some(DealPlatform::Itch), "itch.io"),
                (Some(DealPlatform::OpenSource), "Open Source"),
            ];

            for (p_opt, p_name) in platforms {
                let is_sel = self.deals_platform_filter == p_opt;
                let bg = if is_sel {
                    accent.gamma_multiply(0.25)
                } else {
                    th.surface_2
                };
                let fg = if is_sel { WHITE } else { TEXT_SECONDARY };
                let border = if is_sel { accent } else { th.card_border };

                let btn = egui::Button::new(RichText::new(p_name).size(11.0).color(fg))
                    .fill(bg)
                    .stroke(Stroke::new(1.0, border))
                    .corner_radius(CornerRadius::same(5))
                    .min_size(vec2(0.0, 26.0));

                if ui.add(btn).clicked() {
                    self.deals_platform_filter = p_opt;
                }
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Tri
                egui::ComboBox::from_id_salt("deals_sort_combo")
                    .selected_text(
                        RichText::new(format!("Tri : {}", self.deals_sort_mode.label()))
                            .size(11.5)
                            .color(TEXT_PRIMARY),
                    )
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.deals_sort_mode,
                            DealsSortMode::Featured,
                            DealsSortMode::Featured.label(),
                        );
                        ui.selectable_value(
                            &mut self.deals_sort_mode,
                            DealsSortMode::EndingSoon,
                            DealsSortMode::EndingSoon.label(),
                        );
                        ui.selectable_value(
                            &mut self.deals_sort_mode,
                            DealsSortMode::HighestValue,
                            DealsSortMode::HighestValue.label(),
                        );
                        ui.selectable_value(
                            &mut self.deals_sort_mode,
                            DealsSortMode::Newest,
                            DealsSortMode::Newest.label(),
                        );
                        ui.selectable_value(
                            &mut self.deals_sort_mode,
                            DealsSortMode::Alphabetical,
                            DealsSortMode::Alphabetical.label(),
                        );
                    });

                // Réinitialiser les filtres si actifs
                if (!self.deals_search.is_empty() || self.deals_platform_filter.is_some())
                    && ui
                        .button(
                            RichText::new("✕ Réinitialiser")
                                .size(11.0)
                                .color(WARNING),
                        )
                        .clicked()
                {
                    self.deals_search.clear();
                    self.deals_platform_filter = None;
                }
            });
        });
    }

    // ───────────────────────────── Alerte Expiration Favoris ─────────────────
    fn deals_favorite_alert_ui(&mut self, ui: &mut Ui, _accent: Color32, _th: ThemeColors) {
        let expiring_favs: Vec<&FreeDeal> = self
            .deals
            .iter()
            .filter(|d| self.settings.favorite_deals.contains(&d.id) && d.expires_soon(48))
            .collect();

        if !expiring_favs.is_empty() && self.deals_active_tab != DealsTab::Favorites {
            egui::Frame::new()
                .fill(WARNING.gamma_multiply(0.12))
                .stroke(Stroke::new(1.0, WARNING.gamma_multiply(0.50)))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::symmetric(14, 10))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⚠️").size(16.0));
                        ui.label(
                            RichText::new(format!(
                                "Attention : {} offre(s) dans vos favoris expirent dans moins de 48 heures !",
                                expiring_favs.len()
                            ))
                            .size(12.5)
                            .strong()
                            .color(WARNING),
                        );

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .button(
                                    RichText::new("Consulter mes favoris")
                                        .size(11.5)
                                        .color(WHITE),
                                )
                                .clicked()
                            {
                                self.deals_active_tab = DealsTab::Favorites;
                            }
                        });
                    });
                });

            ui.add_space(14.0);
        }
    }

    // ───────────────────────────── Grille des Cartes ─────────────────────────
    fn deals_grid_ui(
        &mut self,
        ui: &mut Ui,
        deals: &[FreeDeal],
        accent: Color32,
        th: ThemeColors,
    ) {
        let avail_w = ui.available_width();
        let cols = if avail_w >= 1350.0 {
            4
        } else if avail_w >= 980.0 {
            3
        } else if avail_w >= 620.0 {
            2
        } else {
            1
        };

        ui.columns(cols, |col_uis| {
            for (idx, deal) in deals.iter().enumerate() {
                let col_idx = idx % cols;
                let c_ui = &mut col_uis[col_idx];
                self.deal_card_ui(c_ui, deal, accent, th);
                c_ui.add_space(16.0);
            }
        });
    }

    // ───────────────────────────── Carte d'un Bon Plan ───────────────────────
    fn deal_card_ui(
        &mut self,
        ui: &mut Ui,
        deal: &FreeDeal,
        accent: Color32,
        th: ThemeColors,
    ) {
        let card_w = ui.available_width();
        let card_h = 330.0;
        let (rect, resp) = ui.allocate_exact_size(vec2(card_w, card_h), Sense::hover());
        let p = ui.painter().with_clip_rect(rect);

        let hov = resp.hovered();
        let border_col = if hov { accent } else { th.card_border };

        // Fond avec bords arrondis
        p.rect_filled(rect, CornerRadius::same(10), th.surface_2);

        if hov {
            p.rect_filled(
                rect.expand(1.0),
                CornerRadius::same(11),
                accent.gamma_multiply(0.06),
            );
        }
        p.rect_stroke(rect, CornerRadius::same(10), Stroke::new(1.0, border_col), StrokeKind::Inside);

        // 1. Zone d'Image en haut de la carte
        let img_h = 145.0;
        let img_rect = Rect::from_min_size(rect.min, vec2(card_w, img_h));

        let tex = self.get_deal_texture(ui.ctx(), deal);
        if let Some(ref t) = tex {
            p.image(
                t.id(),
                img_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                WHITE,
            );
        } else {
            // Placeholder / Skeleton animé
            let pulse = ((ui.ctx().input(|i| i.time) * 2.0).sin() * 0.5 + 0.5) as f32;
            let bg_shade = th.surface_3.gamma_multiply(0.85 + pulse * 0.15);
            p.rect_filled(img_rect, CornerRadius::same(10), bg_shade);
            p.text(
                img_rect.center(),
                Align2::CENTER_CENTER,
                deal.category.icon(),
                FontId::proportional(32.0),
                accent.gamma_multiply(0.50),
            );
            ui.ctx().request_repaint();
        }

        // Dégradé au bas de l'image
        let fade_h = 40.0;
        let fade_rect = Rect::from_min_max(pos2(img_rect.left(), img_rect.bottom() - fade_h), img_rect.max);
        p.rect_filled(fade_rect, CornerRadius::ZERO, Color32::from_black_alpha(110));

        // Badges superposés sur l'image
        // Badge Type (Haut Gauche)
        pill(
            &p,
            img_rect.min + vec2(8.0, 8.0),
            deal.deal_type.label(),
            deal.deal_type.color().gamma_multiply(0.28),
            WHITE,
        );

        // Badge Plateforme (Haut Droite)
        let plat_col = deal.platform.color();
        let plat_txt = deal.platform.name();
        let plat_w = (plat_txt.len() as f32 * 6.5) + 14.0;
        pill(
            &p,
            pos2(img_rect.right() - plat_w - 8.0, img_rect.top() + 8.0),
            plat_txt,
            th.surface_1.gamma_multiply(0.85),
            plat_col,
        );

        // Compte à rebours ou statut d'expiration (Bas Gauche)
        if deal.is_expired() {
            pill(
                &p,
                pos2(img_rect.left() + 8.0, img_rect.bottom() - 24.0),
                "EXPIRÉ",
                ERROR.gamma_multiply(0.35),
                WHITE,
            );
        } else if let Some(ref time_str) = deal.remaining_time_str() {
            pill(
                &p,
                pos2(img_rect.left() + 8.0, img_rect.bottom() - 24.0),
                time_str,
                WARNING.gamma_multiply(0.32),
                WARNING,
            );
        }

        // 2. Corps de la carte
        let body_rect = Rect::from_min_max(
            pos2(rect.left() + 12.0, img_rect.bottom() + 10.0),
            pos2(rect.right() - 12.0, rect.bottom() - 48.0),
        );

        let mut body_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(body_rect)
                .layout(Layout::top_down(Align::Min)),
        );

        // Catégorie
        body_ui.label(
            RichText::new(deal.category.label().to_uppercase())
                .size(10.0)
                .strong()
                .color(TEXT_TERTIARY),
        );

        body_ui.add_space(2.0);

        // Titre
        let title_clean = if deal.title.len() > 36 {
            format!("{}...", &deal.title[..33])
        } else {
            deal.title.clone()
        };
        body_ui.label(
            RichText::new(title_clean)
                .size(14.5)
                .strong()
                .color(TEXT_PRIMARY),
        );

        body_ui.add_space(3.0);

        // Description courte
        let desc_clean = if deal.description.len() > 70 {
            format!("{}...", &deal.description[..67])
        } else {
            deal.description.clone()
        };
        body_ui.label(
            RichText::new(desc_clean)
                .size(11.0)
                .color(TEXT_SECONDARY),
        );

        body_ui.add_space(6.0);

        // Prix & Réduction
        body_ui.horizontal(|ui| {
            if let Some(ref orig) = deal.original_price {
                ui.label(
                    RichText::new(orig)
                        .size(11.5)
                        .strikethrough()
                        .color(RED),
                );
            }
            ui.label(
                RichText::new("GRATUIT")
                    .size(12.5)
                    .strong()
                    .color(SUCCESS),
            );
            stat_chip(ui, deal.discount_badge(), SUCCESS);
        });

        // 3. Pied de carte & Boutons d'action
        let foot_rect = Rect::from_min_max(
            pos2(rect.left() + 10.0, rect.bottom() - 44.0),
            pos2(rect.right() - 10.0, rect.bottom() - 8.0),
        );

        let mut foot_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(foot_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );

        // Bouton Étoile Favori
        let is_fav = self.settings.favorite_deals.contains(&deal.id);
        let star_icon = if is_fav { "⭐" } else { "☆" };
        let star_btn = egui::Button::new(RichText::new(star_icon).size(13.0))
            .fill(if is_fav { GOLD.gamma_multiply(0.20) } else { th.surface_3 })
            .stroke(Stroke::new(1.0, if is_fav { GOLD } else { th.card_border }))
            .corner_radius(CornerRadius::same(6))
            .min_size(vec2(28.0, 28.0));

        if foot_ui.add(star_btn).clicked() {
            if is_fav {
                self.settings.favorite_deals.remove(&deal.id);
                self.toast("Offre retirée des favoris");
            } else {
                self.settings.favorite_deals.insert(deal.id.clone());
                self.toast(format!("⭐ '{}' ajouté aux favoris", deal.title));
            }
            self.settings.save();
        }

        foot_ui.add_space(4.0);

        // Bouton Fiche Détails
        let details_btn = egui::Button::new(
            RichText::new("Détails")
                .size(11.0)
                .color(TEXT_PRIMARY),
        )
        .fill(th.surface_3)
        .stroke(Stroke::new(1.0, th.card_border))
        .corner_radius(CornerRadius::same(6))
        .min_size(vec2(60.0, 28.0));

        if foot_ui.add(details_btn).clicked() {
            self.selected_deal_modal = Some(deal.clone());
        }

        foot_ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            // Bouton Principal Obtenir
            let get_btn = egui::Button::new(
                RichText::new("Obtenir ↗")
                    .size(11.5)
                    .strong()
                    .color(WHITE),
            )
            .fill(if deal.is_expired() {
                th.surface_3
            } else {
                accent.gamma_multiply(0.30)
            })
            .stroke(Stroke::new(
                1.0,
                if deal.is_expired() {
                    th.card_border
                } else {
                    accent.gamma_multiply(0.80)
                },
            ))
            .corner_radius(CornerRadius::same(6))
            .min_size(vec2(80.0, 28.0));

            if ui.add(get_btn).clicked() {
                let _ = open_uri(&deal.store_url);
            }
        });
    }

    // ───────────────────────────── Modale Fiche Détaillée ────────────────────
    pub fn deal_details_modal_ui(&mut self, ctx: &egui::Context) {
        let deal = match self.selected_deal_modal.clone() {
            Some(d) => d,
            None => return,
        };

        let screen_rect = ctx.content_rect();
        let accent = self.accent();
        let th = self.theme();

        let mut close_modal = false;

        // Fermeture avec la touche Échap
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            close_modal = true;
        }

        // 1. Arrière-plan assombri (Backdrop)
        egui::Area::new(Id::unique("deals_modal_backdrop"))
            .order(egui::Order::Middle)
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                let (rect, resp) = ui.allocate_exact_size(screen_rect.size(), Sense::click());
                ui.painter()
                    .rect_filled(rect, CornerRadius::ZERO, Color32::from_black_alpha(205));
                if resp.clicked() {
                    close_modal = true;
                }
            });

        // 2. Fenêtre Modale Centrée
        let modal_w = 840.0f32.min(screen_rect.width() - 36.0);
        let modal_h = 660.0f32.min(screen_rect.height() - 36.0);
        let modal_pos = pos2(
            (screen_rect.width() - modal_w) / 2.0,
            (screen_rect.height() - modal_h) / 2.0,
        );

        egui::Area::new(Id::unique("deals_modal_window"))
            .order(egui::Order::Foreground)
            .fixed_pos(modal_pos)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(th.surface_1)
                    .stroke(Stroke::new(1.0, th.border_hover))
                    .corner_radius(CornerRadius::same(14))
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 20],
                        blur: 40,
                        spread: 0,
                        color: Color32::from_black_alpha(230),
                    })
                    .inner_margin(Margin::same(24))
                    .show(ui, |ui| {
                        ui.set_width(modal_w - 48.0);
                        ui.set_height(modal_h - 48.0);

                        // ── EN-TÊTE : Catégorie, Plateforme, Favori et Fermer ──
                        ui.horizontal(|ui| {
                            stat_chip(ui, deal.platform.name(), deal.platform.color());
                            stat_chip(ui, deal.category.label(), TEXT_SECONDARY);
                            stat_chip(ui, deal.deal_type.label(), deal.deal_type.color());

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui
                                    .button(
                                        RichText::new("✕")
                                            .size(15.0)
                                            .color(TEXT_SECONDARY),
                                    )
                                    .clicked()
                                {
                                    close_modal = true;
                                }

                                let is_fav = self.settings.favorite_deals.contains(&deal.id);
                                let fav_text = if is_fav { "⭐ En favoris" } else { "☆ Ajouter aux favoris" };
                                if ui
                                    .button(RichText::new(fav_text).size(12.0).color(if is_fav { GOLD } else { TEXT_SECONDARY }))
                                    .clicked()
                                {
                                    if is_fav {
                                        self.settings.favorite_deals.remove(&deal.id);
                                        self.toast("Offre retirée des favoris");
                                    } else {
                                        self.settings.favorite_deals.insert(deal.id.clone());
                                        self.toast(format!("⭐ '{}' ajouté aux favoris", deal.title));
                                    }
                                    self.settings.save();
                                }
                            });
                        });

                        ui.add_space(14.0);

                        // ── BANNIÈRE VISUELLE ──
                        let banner_h = 160.0;
                        let (banner_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), banner_h), Sense::hover());
                        let p = ui.painter().with_clip_rect(banner_rect);

                        let tex = self.get_deal_texture(ui.ctx(), &deal);
                        if let Some(ref t) = tex {
                            p.image(
                                t.id(),
                                banner_rect,
                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                WHITE,
                            );
                        } else {
                            p.rect_filled(banner_rect, CornerRadius::same(10), th.surface_2);
                            p.text(
                                banner_rect.center(),
                                Align2::CENTER_CENTER,
                                deal.category.icon(),
                                FontId::proportional(40.0),
                                accent,
                            );
                        }
                        p.rect_stroke(banner_rect, CornerRadius::same(10), Stroke::new(1.0, th.card_border), StrokeKind::Inside);

                        // Dégradé assombrissant sur le bas de la bannière
                        let banner_fade = Rect::from_min_max(pos2(banner_rect.left(), banner_rect.bottom() - 60.0), banner_rect.max);
                        p.rect_filled(banner_fade, CornerRadius::ZERO, Color32::from_black_alpha(160));

                        // Titre en bas de la bannière
                        p.text(
                            pos2(banner_rect.left() + 16.0, banner_rect.bottom() - 20.0),
                            Align2::LEFT_CENTER,
                            &deal.title,
                            FontId::proportional(22.0),
                            WHITE,
                        );

                        ui.add_space(16.0);

                        // ── BLOC RÉDUCTION & COMPTE À REBOURS ──
                        egui::Frame::new()
                            .fill(th.surface_2)
                            .stroke(Stroke::new(1.0, th.card_border))
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(Margin::symmetric(14, 10))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            if let Some(ref orig) = deal.original_price {
                                                ui.label(
                                                    RichText::new(format!("Prix habituel : {orig}"))
                                                        .size(12.0)
                                                        .strikethrough()
                                                        .color(RED),
                                                );
                                                ui.label(RichText::new("➔").size(12.0).color(TEXT_TERTIARY));
                                            }
                                            ui.label(
                                                RichText::new("0,00 € (100% de réduction)")
                                                    .size(14.0)
                                                    .strong()
                                                    .color(SUCCESS),
                                            );
                                        });

                                        if let Some(ref dev) = deal.publisher_or_dev {
                                            ui.label(
                                                RichText::new(format!("Éditeur / Développeur : {dev}"))
                                                    .size(11.0)
                                                    .color(TEXT_TERTIARY),
                                            );
                                        }
                                    });

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if let Some(ref time_str) = deal.remaining_time_str() {
                                            stat_chip(ui, time_str, WARNING);
                                        } else if deal.deal_type == DealType::FreeToKeep || deal.deal_type == DealType::OpenSource {
                                            stat_chip(ui, "✨ CONSERVEZ À VIE DANS VOTRE BIBLIOTHÈQUE", GREEN);
                                        }
                                    });
                                });
                            });

                        ui.add_space(14.0);

                        // ── CONTENU SCROLLABLE : DESCRIPTION & GUIDE ──
                        egui::ScrollArea::vertical()
                            .id_salt("deal_modal_desc_scroll")
                            .max_height(170.0)
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new("À propos de cette offre")
                                        .size(13.5)
                                        .strong()
                                        .color(TEXT_PRIMARY),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(&deal.description)
                                        .size(12.0)
                                        .color(TEXT_SECONDARY),
                                );

                                if let Some(ref instr) = deal.claim_instructions {
                                    ui.add_space(12.0);
                                    ui.label(
                                        RichText::new("Comment réclamer cette offre :")
                                            .size(13.0)
                                            .strong()
                                            .color(accent),
                                    );
                                    ui.add_space(3.0);
                                    ui.label(
                                        RichText::new(instr)
                                            .size(11.5)
                                            .color(TEXT_SECONDARY),
                                    );
                                }

                                if !deal.tags.is_empty() {
                                    ui.add_space(12.0);
                                    ui.horizontal_wrapped(|ui| {
                                        for tag in &deal.tags {
                                            stat_chip(ui, tag, th.surface_3);
                                        }
                                    });
                                }
                            });

                        ui.add_space(16.0);

                        // ── ACTIONS INFÉRIEURES ──
                        ui.horizontal(|ui| {
                            let claim_btn = egui::Button::new(
                                RichText::new(format!("⚡ Récupérer sur {} ↗", deal.platform.name()))
                                    .size(13.5)
                                    .strong()
                                    .color(WHITE),
                            )
                            .fill(accent.gamma_multiply(0.35))
                            .stroke(Stroke::new(1.0, accent.gamma_multiply(0.90)))
                            .corner_radius(CornerRadius::same(7))
                            .min_size(vec2(220.0, 36.0));

                            if ui.add(claim_btn).clicked() {
                                let _ = open_uri(&deal.store_url);
                            }

                            let copy_btn = egui::Button::new(
                                RichText::new("📋 Copier le lien officiel")
                                    .size(12.0)
                                    .color(TEXT_PRIMARY),
                            )
                            .fill(th.surface_2)
                            .stroke(Stroke::new(1.0, th.card_border))
                            .corner_radius(CornerRadius::same(6))
                            .min_size(vec2(160.0, 36.0));

                            if ui.add(copy_btn).clicked() {
                                ctx.copy_text(deal.store_url.clone());
                                self.toast("Lien copié dans le presse-papiers !");
                            }
                        });
                    });
            });

        if close_modal {
            self.selected_deal_modal = None;
        }
    }

    // ───────────────────────────── États Vides & Squelette ───────────────────
    fn deals_empty_state_ui(&mut self, ui: &mut Ui, accent: Color32, th: ThemeColors) {
        egui::Frame::new()
            .fill(th.surface_2)
            .stroke(Stroke::new(1.0, th.card_border))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::same(48))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    let icon_str = match self.deals_active_tab {
                        DealsTab::Favorites => "⭐",
                        _ => "🔍",
                    };
                    ui.label(RichText::new(icon_str).size(36.0));
                    ui.add_space(8.0);

                    let title = match self.deals_active_tab {
                        DealsTab::Favorites => "Aucun bon plan dans vos favoris",
                        _ => "Aucun bon plan ne correspond à votre recherche",
                    };
                    ui.label(
                        RichText::new(title)
                            .size(16.0)
                            .strong()
                            .color(TEXT_PRIMARY),
                    );

                    ui.add_space(4.0);
                    let sub = match self.deals_active_tab {
                        DealsTab::Favorites => "Cliquez sur l'étoile d'une offre pour la suivre et recevoir des alertes avant qu'elle n'expire !",
                        _ => "Essayez d'ajuster vos critères de recherche ou réinitialisez les filtres.",
                    };
                    ui.label(
                        RichText::new(sub)
                            .size(12.0)
                            .color(TEXT_SECONDARY),
                    );

                    ui.add_space(16.0);

                    if self.deals_active_tab == DealsTab::Favorites {
                        let btn = egui::Button::new(
                            RichText::new("Explorer tous les bons plans")
                                .size(12.5)
                                .color(WHITE),
                        )
                        .fill(accent.gamma_multiply(0.25))
                        .stroke(Stroke::new(1.0, accent))
                        .corner_radius(CornerRadius::same(6));

                        if ui.add(btn).clicked() {
                            self.deals_active_tab = DealsTab::All;
                        }
                    } else if !self.deals_search.is_empty() || self.deals_platform_filter.is_some() {
                        let btn = egui::Button::new(
                            RichText::new("Réinitialiser les filtres")
                                .size(12.5)
                                .color(WHITE),
                        )
                        .fill(th.surface_3)
                        .stroke(Stroke::new(1.0, th.card_border))
                        .corner_radius(CornerRadius::same(6));

                        if ui.add(btn).clicked() {
                            self.deals_search.clear();
                            self.deals_platform_filter = None;
                        }
                    }
                });
            });
    }

    fn deals_skeleton_loader_ui(&self, ui: &mut Ui, th: ThemeColors) {
        egui::Frame::new()
            .fill(th.surface_2)
            .stroke(Stroke::new(1.0, th.card_border))
            .corner_radius(CornerRadius::same(12))
            .inner_margin(Margin::same(40))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    ui.spinner();
                    ui.add_space(14.0);
                    ui.label(
                        RichText::new("Interrogation des boutiques et flux officiels en cours...")
                            .size(13.5)
                            .color(TEXT_PRIMARY),
                    );
                    ui.add_space(3.0);
                    ui.label(
                        RichText::new("Epic Games Store • Steam • GOG • GamerPower • Logiciels Libres")
                            .size(11.5)
                            .color(TEXT_TERTIARY),
                    );
                });
            });
    }

    // ───────────────────────────── Logique de Filtrage & Tri ─────────────────
    fn filtered_deals(&self) -> Vec<FreeDeal> {
        let q = self.deals_search.to_lowercase();
        let tab = self.deals_active_tab;
        let plat_filter = &self.deals_platform_filter;
        let favs = &self.settings.favorite_deals;

        let mut list: Vec<FreeDeal> = self
            .deals
            .iter()
            .filter(|d| {
                // 1. Filtre par onglet
                let in_tab = match tab {
                    DealsTab::All => true,
                    DealsTab::Games => d.category == DealCategory::Game || d.category == DealCategory::Dlc,
                    DealsTab::EndingSoon => d.is_active_now && d.end_timestamp.is_some() && !d.is_expired(),
                    DealsTab::ToKeep => d.deal_type == DealType::FreeToKeep,
                    DealsTab::Software => {
                        d.category == DealCategory::Software
                            || d.category == DealCategory::Creative
                            || d.category == DealCategory::DevTool
                            || d.category == DealCategory::Productivity
                    }
                    DealsTab::Discoveries => d.category == DealCategory::Indie,
                    DealsTab::Favorites => favs.contains(&d.id),
                };
                if !in_tab {
                    return false;
                }

                // 2. Filtre par plateforme
                if let Some(ref target_plat) = plat_filter {
                    if &d.platform != target_plat {
                        return false;
                    }
                }

                // 3. Filtre par recherche textuelle
                if !q.is_empty() {
                    let title_match = d.title.to_lowercase().contains(&q);
                    let desc_match = d.description.to_lowercase().contains(&q);
                    let plat_match = d.platform.name().to_lowercase().contains(&q);
                    let tag_match = d.tags.iter().any(|t| t.to_lowercase().contains(&q));
                    if !title_match && !desc_match && !plat_match && !tag_match {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // 4. Tri
        match self.deals_sort_mode {
            DealsSortMode::Featured => {
                list.sort_by(|a, b| {
                    b.is_featured
                        .cmp(&a.is_featured)
                        .then_with(|| b.is_active_now.cmp(&a.is_active_now))
                });
            }
            DealsSortMode::EndingSoon => {
                list.sort_by(|a, b| {
                    let ts_a = a.end_timestamp.unwrap_or(i64::MAX);
                    let ts_b = b.end_timestamp.unwrap_or(i64::MAX);
                    ts_a.cmp(&ts_b)
                });
            }
            DealsSortMode::HighestValue => {
                list.sort_by(|a, b| {
                    let val_a = a.original_price.as_deref().unwrap_or("");
                    let val_b = b.original_price.as_deref().unwrap_or("");
                    val_b.cmp(val_a)
                });
            }
            DealsSortMode::Newest => {
                // Récents d'abord
                list.reverse();
            }
            DealsSortMode::Alphabetical => {
                list.sort_by_key(|a| a.title.to_lowercase());
            }
        }

        list
    }
}

// ───────────────────────────── Tests UI Bons Plans ───────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::deals::DealsTracker;

    #[test]
    fn test_deals_ui_tabs_and_filtering() {
        let mut launcher = Launcher::test_instance();
        launcher.deals = DealsTracker::fallback_curated_deals();
        assert!(!launcher.deals.is_empty());

        let ctx = egui::Context::default();

        // 1. Tester le rendu de tous les onglets
        let tabs = [
            DealsTab::All,
            DealsTab::Games,
            DealsTab::EndingSoon,
            DealsTab::ToKeep,
            DealsTab::Software,
            DealsTab::Discoveries,
            DealsTab::Favorites,
        ];

        for tab in tabs {
            launcher.deals_active_tab = tab;
            let _ = launcher.filtered_deals();
            let mut output = ctx.run_ui(Default::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    launcher.deals_ui(ui);
                });
            });
            output.textures_delta.clear();
        }

        // 2. Tester la recherche
        launcher.deals_active_tab = DealsTab::All;
        launcher.deals_search = "Blender".to_string();
        let search_results = launcher.filtered_deals();
        assert!(!search_results.is_empty());
        assert!(search_results.iter().any(|d| d.title.contains("Blender")));

        // 3. Tester le filtre plateforme
        launcher.deals_search.clear();
        launcher.deals_platform_filter = Some(DealPlatform::Steam);
        let steam_deals = launcher.filtered_deals();
        assert!(steam_deals.iter().all(|d| d.platform == DealPlatform::Steam));

        // 4. Tester les favoris
        launcher.deals_platform_filter = None;
        let first_id = launcher.deals[0].id.clone();
        launcher.settings.favorite_deals.insert(first_id.clone());
        launcher.deals_active_tab = DealsTab::Favorites;
        let fav_deals = launcher.filtered_deals();
        assert_eq!(fav_deals.len(), 1);
        assert_eq!(fav_deals[0].id, first_id);

        // 5. Tester les modes de tri
        for sort_mode in [
            DealsSortMode::Featured,
            DealsSortMode::EndingSoon,
            DealsSortMode::HighestValue,
            DealsSortMode::Newest,
            DealsSortMode::Alphabetical,
        ] {
            launcher.deals_sort_mode = sort_mode;
            let sorted = launcher.filtered_deals();
            assert!(!sorted.is_empty());
        }

        // 6. Tester la modale de fiche détaillée
        launcher.selected_deal_modal = Some(launcher.deals[0].clone());
        let mut output_modal = ctx.run_ui(Default::default(), |ctx| {
            launcher.deal_details_modal_ui(ctx);
        });
        output_modal.textures_delta.clear();
    }
}
