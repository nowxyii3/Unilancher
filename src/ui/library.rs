use crate::app::Launcher;
use crate::config::{Game, Source};
use crate::theme::{
    BNET_C, EA_C, EPIC_C, GOG_C, GOLD, RED, STEAM_C, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_TERTIARY,
    UBI_C, WHITE, XBOX_C,
};
use crate::types::{
    format_playtime, Action, GridCardSize, LibraryStatusFilter, SortMode, Tab, ViewMode,
};
use crate::ui::components::{
    draw_icon, game_card, game_list_row, hero_banner, icon_square_button, modern_action_button,
    search_bar, stat_chip, Icon,
};
use eframe::egui::{
    self, vec2, Align, Color32, CornerRadius, Layout, Margin, Rect, RichText, Sense, Stroke, Ui,
};
use std::collections::HashSet;

// ═══════════════════════════════════════════════════════════════════════════════
//  UniLauncher — Vue Principale de la Bibliothèque
//  Design inspiré de Linear, Raycast, Steam et Arc Browser
// ═══════════════════════════════════════════════════════════════════════════════

impl Launcher {
    pub fn library_ui(&mut self, ui: &mut Ui) -> Vec<Action> {
        let ctx = ui.ctx().clone();
        let accent = self.accent();
        let th = self.theme();
        let mut actions: Vec<Action> = vec![];

        let (title, subtitle) = match self.tab {
            Tab::All => ("Bibliothèque", "Catalogue unifié de vos jeux PC détectés et installés"),
            Tab::Favorites => ("Favoris", "Accès direct et prioritaire à vos titres favoris"),
            Tab::Steam => ("Steam", "Titres synchronisés depuis votre bibliothèque Steam"),
            Tab::Epic => ("Epic Games", "Titres détectés via les manifestes Epic Games Store"),
            Tab::Gog => ("GOG Galaxy", "Jeux sans DRM synchronisés depuis GOG Galaxy"),
            Tab::Ubisoft => ("Ubisoft Connect", "Titres détectés via le client Ubisoft Connect"),
            Tab::Ea => ("EA App", "Titres détectés via l'écosystème Electronic Arts"),
            Tab::Battlenet => ("Battle.net", "Titres Blizzard et Battle.net installés"),
            Tab::Xbox => ("Xbox & Game Pass", "Jeux Windows Store et PC Game Pass"),
            Tab::Manual => ("Jeux manuels", "Exécutables indépendants et jeux sans launcher"),
            Tab::Emulators => ("Émulateurs & ROMs", "Jeux rétros et consoles émulées"),
            Tab::Deals => ("Bons Plans Gratuits", "Jeux offerts sur l'Epic Games Store"),
            Tab::Hidden => ("Jeux masqués", "Titres retirés de l'affichage principal"),
            Tab::Stats => ("Statistiques", "Vue d'ensemble et répartition"),
            Tab::Settings => ("Paramètres", ""),
        };

        let filtered = self.filtered_indices();
        let favs = self.settings.favorites.clone();

        let total_mins: u32 = filtered
            .iter()
            .map(|&i| self.total_playtime_mins(&self.games[i]))
            .sum();

        let platform_count = self
            .games
            .iter()
            .map(|g| g.source)
            .collect::<HashSet<_>>()
            .len();

        // ── 1. EN-TÊTE PRINCIPAL & BARRE D'ACTIONS RAPIDES ──
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(title).size(24.0).strong().color(TEXT_PRIMARY));
                    ui.add_space(8.0);
                    stat_chip(ui, &format!("{} titres", filtered.len()), accent);
                    if total_mins > 0 {
                        stat_chip(
                            ui,
                            &format!("{:.1} h", total_mins as f32 / 60.0),
                            Color32::from_rgb(56, 189, 248),
                        );
                    }
                    if platform_count > 1 {
                        stat_chip(ui, &format!("{} plateformes", platform_count), GOLD);
                    }
                });
                ui.add_space(2.0);
                ui.label(RichText::new(subtitle).size(12.0).color(TEXT_SECONDARY));
            });

            // Barre d'outils alignée à droite
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if modern_action_button(ui, "Ajouter un jeu", Icon::Plus, accent, true, th) {
                    actions.push(Action::Add);
                }
                ui.add_space(4.0);
                if icon_square_button(ui, Icon::Folder, "Scanner un dossier de jeux", false, th) {
                    actions.push(Action::AddScanFolder);
                }
                ui.add_space(4.0);
                if icon_square_button(ui, Icon::Refresh, "Actualiser la bibliothèque", false, th) {
                    actions.push(Action::Refresh);
                }
                if icon_square_button(ui, Icon::Dice, "Lancer un jeu aléatoire", false, th) {
                    actions.push(Action::LaunchRandom);
                }
                ui.add_space(6.0);
                search_bar(ui, &mut self.search, &mut self.focus_search, accent, th);
            });
        });

        // ── 2. BANNIÈRE D'ÉTAT DU SCAN EN TEMPS RÉEL (STYLE STEAM/ARC) ──
        if let Some(prog) = &self.scan_progress {
            ui.add_space(8.0);
            egui::Frame::new()
                .fill(th.surface_2)
                .stroke(Stroke::new(1.0, accent.gamma_multiply(0.40)))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::symmetric(14, 8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;

                        // Pastille animée de synchronisation
                        let (icon_r, _) = ui.allocate_exact_size(vec2(14.0, 14.0), Sense::hover());
                        draw_icon(ui.painter(), Icon::Refresh, icon_r.center(), accent);

                        let desc = if prog.games_found > 0 {
                            format!("{} ({} trouvés)", prog.stage, prog.games_found)
                        } else {
                            prog.stage.clone()
                        };
                        ui.label(RichText::new(desc).size(12.0).strong().color(TEXT_PRIMARY));

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let pct_str = format!("{:.0}%", prog.percent * 100.0);
                            ui.label(RichText::new(pct_str).size(11.5).strong().color(accent));

                            // Barre de progression fine (4px)
                            let (bar_rect, _) = ui.allocate_exact_size(vec2(120.0, 4.0), Sense::hover());
                            ui.painter().rect_filled(bar_rect, CornerRadius::same(2), th.surface_3);
                            let fill_w = (bar_rect.width() * prog.percent).clamp(2.0, bar_rect.width());
                            let fill_r = Rect::from_min_size(bar_rect.min, vec2(fill_w, 4.0));
                            ui.painter().rect_filled(fill_r, CornerRadius::same(2), accent);
                        });
                    });
                });
        }

        ui.add_space(12.0);

        // ── 3. CONTRÔLES SEGMENTÉS (FILTRES AVANCÉS, TRIS & MODES D'AFFICHAGE) ──
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 6.0);

            // Segmented Control Filtres Avancés
            egui::Frame::new()
                .fill(th.surface_1)
                .stroke(Stroke::new(1.0, th.border_subtle))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::same(2))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.x = 1.0;
                    for filter in [
                        LibraryStatusFilter::All,
                        LibraryStatusFilter::Favorites,
                        LibraryStatusFilter::Recent,
                        LibraryStatusFilter::NeverPlayed,
                        LibraryStatusFilter::Installed,
                        LibraryStatusFilter::NotInstalled,
                    ] {
                        let sel = self.settings.status_filter == filter;
                        let bg = if sel {
                            accent.gamma_multiply(0.20)
                        } else {
                            Color32::TRANSPARENT
                        };
                        let border_stroke = if sel {
                            Stroke::new(1.0, accent.gamma_multiply(0.60))
                        } else {
                            Stroke::NONE
                        };
                        let fg = if sel { TEXT_PRIMARY } else { TEXT_SECONDARY };

                        let btn = egui::Button::new(RichText::new(filter.label()).size(11.5).strong().color(fg))
                            .fill(bg)
                            .stroke(border_stroke)
                            .corner_radius(6.0);

                        if ui
                            .add(btn)
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            self.settings.status_filter = filter;
                            self.settings.save();
                        }
                    }
                });

            // Segmented Control Ordre de Tri Enrichi
            egui::Frame::new()
                .fill(th.surface_1)
                .stroke(Stroke::new(1.0, th.border_subtle))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::same(2))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.x = 1.0;
                    for (mode, lbl) in [
                        (SortMode::Playtime, "Temps"),
                        (SortMode::Name, "A → Z"),
                        (SortMode::NameDesc, "Z → A"),
                        (SortMode::LastPlayed, "Récents"),
                        (SortMode::Favorites, "Favoris"),
                        (SortMode::SizeOnDisk, "Taille"),
                    ] {
                        let sel = self.sort == mode;
                        let bg = if sel {
                            th.surface_3
                        } else {
                            Color32::TRANSPARENT
                        };
                        let border_stroke = if sel {
                            Stroke::new(1.0, th.border_default)
                        } else {
                            Stroke::NONE
                        };
                        let fg = if sel { TEXT_PRIMARY } else { TEXT_TERTIARY };

                        let btn = egui::Button::new(RichText::new(lbl).size(11.5).color(fg))
                            .fill(bg)
                            .stroke(border_stroke)
                            .corner_radius(6.0);

                        if ui
                            .add(btn)
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            self.sort = mode;
                            self.settings.sort_mode = mode;
                            self.settings.save();
                        }
                    }
                });

            // Tailles de Cartes (S, M, L) pour le Mode Grille
            if self.view_mode == ViewMode::Grid {
                egui::Frame::new()
                    .fill(th.surface_1)
                    .stroke(Stroke::new(1.0, th.border_subtle))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::same(2))
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.x = 1.0;
                        for (sz, lbl) in [
                            (GridCardSize::Small, "S"),
                            (GridCardSize::Medium, "M"),
                            (GridCardSize::Large, "L"),
                        ] {
                            let sel = self.settings.grid_card_size == sz;
                            let bg = if sel { th.surface_3 } else { Color32::TRANSPARENT };
                            let fg = if sel { accent } else { TEXT_TERTIARY };
                            let btn = egui::Button::new(RichText::new(lbl).size(11.0).strong().color(fg))
                                .fill(bg)
                                .corner_radius(5.0);
                            if ui.add(btn).on_hover_text(format!("Taille de jaquette : {}", sz.label())).clicked() {
                                self.settings.grid_card_size = sz;
                                self.settings.save();
                            }
                        }
                    });
            }

            // Contrôles de vue alignés
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 3.0;
                if icon_square_button(
                    ui,
                    Icon::Grid,
                    "Vue Grille de jaquettes",
                    self.view_mode == ViewMode::Grid,
                    th,
                ) {
                    self.view_mode = ViewMode::Grid;
                    self.settings.view_mode = ViewMode::Grid;
                    self.settings.save();
                }
                if icon_square_button(
                    ui,
                    Icon::List,
                    "Vue Liste détaillée",
                    self.view_mode == ViewMode::List,
                    th,
                ) {
                    self.view_mode = ViewMode::List;
                    self.settings.view_mode = ViewMode::List;
                    self.settings.save();
                }
                if icon_square_button(
                    ui,
                    Icon::Gamepad,
                    "Mode Big Picture / Salon (F10)",
                    self.is_big_picture,
                    th,
                ) {
                    self.is_big_picture = true;
                }

                ui.add_space(6.0);

                // Slider de zoom fin
                ui.add(
                    egui::Slider::new(&mut self.settings.card_scale, 0.75..=1.35)
                        .show_value(false)
                        .trailing_fill(true),
                )
                .on_hover_text(format!("Échelle dynamique ({:.0}%)", self.settings.card_scale * 100.0));
            });
        });

        ui.add_space(14.0);

        // ── 4. ÉTAT VIDE SOIGNÉ (EMPTY STATE STYLE LINEAR / RAYCAST) ──
        if filtered.is_empty() {
            ui.add_space(60.0);
            ui.vertical_centered(|ui| {
                // Conteneur d'icône avec cercle épuré
                let (r, _) = ui.allocate_exact_size(vec2(56.0, 56.0), Sense::hover());
                ui.painter().circle_filled(r.center(), 28.0, th.surface_2);
                ui.painter().circle_stroke(r.center(), 28.5, Stroke::new(1.0, th.border_subtle));
                draw_icon(ui.painter(), Icon::Library, r.center(), accent);

                ui.add_space(14.0);
                ui.label(
                    RichText::new("Aucun titre ne correspond à vos filtres")
                        .size(16.0)
                        .strong()
                        .color(TEXT_PRIMARY),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Ajustez vos filtres de recherche ou ajoutez un nouvel exécutable.")
                        .size(12.5)
                        .color(TEXT_SECONDARY),
                );

                ui.add_space(14.0);

                ui.horizontal(|ui| {
                    let has_filters = !self.search.is_empty() || self.settings.quick_filter_mode != 0;
                    if has_filters {
                        let reset_btn = egui::Button::new(
                            RichText::new("Réinitialiser les filtres").size(12.0).color(TEXT_PRIMARY),
                        )
                        .fill(th.surface_2)
                        .stroke(Stroke::new(1.0, th.border_default))
                        .corner_radius(6.0);
                        if ui.add(reset_btn).clicked() {
                            self.search.clear();
                            self.settings.quick_filter_mode = 0;
                        }
                        ui.add_space(8.0);
                    }

                    if modern_action_button(ui, "Ajouter un jeu", Icon::Plus, accent, true, th) {
                        actions.push(Action::Add);
                    }
                });
            });
            return actions;
        }

        // ── 5. ZONE DÉFILABLE : BANNIÈRE HERO + GRILLE / LISTE ──
        egui::ScrollArea::vertical()
            .id_salt("library_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Bannière Hero immersive uniquement sur l'onglet principal sans recherche textuelle
                if self.search.is_empty() && self.tab == Tab::All && !filtered.is_empty() {
                    let featured_idx = self
                        .settings
                        .last_played
                        .as_ref()
                        .and_then(|id| {
                            filtered
                                .iter()
                                .copied()
                                .find(|&i| &self.games[i].id == id)
                        })
                        .unwrap_or(filtered[0]);

                    let fg_game = self.games[featured_idx].clone();
                    let fg_mins = self.total_playtime_mins(&fg_game);
                    let fg_tex = self.get_cover_texture(&ctx, &fg_game);

                    let (play_clicked, prop_clicked) =
                        hero_banner(ui, &fg_game, fg_mins, fg_tex.as_ref(), accent, th);
                    if play_clicked {
                        actions.push(Action::Launch(featured_idx));
                    }
                    if prop_clicked {
                        actions.push(Action::OpenDetails(fg_game.id.clone()));
                    }
                    ui.add_space(18.0);
                }

                match self.view_mode {
                    // ── Mode Grille de Jaquettes ──
                    ViewMode::Grid => {
                        let spacing = self.settings.grid_spacing.clamp(8.0, 36.0);
                        let (card_w, _) = self.settings.grid_card_size.dimensions(self.settings.card_scale);
                        let avail_w = ui.available_width();
                        let _cols = ((avail_w + spacing) / (card_w + spacing)).floor().max(1.0) as usize;

                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(spacing, spacing + 6.0);

                            for &i in &filtered {
                                let g = self.games[i].clone();
                                let fav = favs.contains(&g.id);
                                let ovr = self.settings.exe_overrides.get(&g.id).cloned();
                                let fixed = ovr.as_ref().map(|p| p.exists()).unwrap_or(false);
                                let broken = g.is_broken() && !fixed;
                                let is_running = self
                                    .active_game
                                    .as_ref()
                                    .map(|(name, _)| name == &g.name)
                                    .unwrap_or(false);
                                let play_mins = self.total_playtime_mins(&g);
                                let tex = self.get_cover_texture(&ctx, &g);

                                let (resp, star_clicked, details_clicked, play_clicked) = game_card(
                                    ui,
                                    &g,
                                    tex.as_ref(),
                                    play_mins,
                                    fav,
                                    broken,
                                    is_running,
                                    ovr.is_some(),
                                    accent,
                                    self.settings.card_scale,
                                    self.settings.grid_card_size,
                                    th,
                                );

                                if star_clicked {
                                    actions.push(Action::Fav(g.id.clone()));
                                } else if play_clicked || resp.double_clicked() {
                                    actions.push(Action::Launch(i));
                                } else if details_clicked || resp.clicked() {
                                    actions.push(Action::OpenDetails(g.id.clone()));
                                }

                                self.attach_game_context_menu(&resp, &g, i, fav, &mut actions);
                            }
                        });
                    }

                    // ── Mode Liste Détaillée ──
                    ViewMode::List => {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 6.0;
                            for &i in &filtered {
                                let g = self.games[i].clone();
                                let fav = favs.contains(&g.id);
                                let ovr = self.settings.exe_overrides.get(&g.id).cloned();
                                let fixed = ovr.as_ref().map(|p| p.exists()).unwrap_or(false);
                                let broken = g.is_broken() && !fixed;
                                let is_running = self
                                    .active_game
                                    .as_ref()
                                    .map(|(name, _)| name == &g.name)
                                    .unwrap_or(false);
                                let play_mins = self.total_playtime_mins(&g);
                                let last_session = self.settings.last_played_dates.get(&g.id).copied();
                                let tex = self.get_cover_texture(&ctx, &g);

                                let (resp, play_clicked, fav_clicked, opt_clicked) = game_list_row(
                                    ui,
                                    &g,
                                    tex.as_ref(),
                                    play_mins,
                                    last_session,
                                    fav,
                                    broken,
                                    is_running,
                                    accent,
                                    th,
                                );

                                if fav_clicked {
                                    actions.push(Action::Fav(g.id.clone()));
                                } else if opt_clicked || resp.clicked() {
                                    actions.push(Action::OpenDetails(g.id.clone()));
                                } else if play_clicked || resp.double_clicked() {
                                    actions.push(Action::Launch(i));
                                }

                                self.attach_game_context_menu(&resp, &g, i, fav, &mut actions);
                            }
                        });
                    }
                }
                ui.add_space(28.0);
            });

        actions
    }

    // ───────────────────────────── Menu Contextuel du Jeu (Clic Droit) ──────────
    pub fn attach_game_context_menu(
        &self,
        resp: &egui::Response,
        g: &Game,
        idx: usize,
        fav: bool,
        actions: &mut Vec<Action>,
    ) {
        let has_custom_cover = self.settings.cover_overrides.contains_key(&g.id);
        let is_hidden = self.settings.hidden_games.contains(&g.id);
        let accent = self.accent();

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

        resp.context_menu(|ui| {
            ui.set_min_width(240.0);
            ui.spacing_mut().item_spacing.y = 3.0;

            // En-tête : Titre + Badge Plateforme
            ui.add_space(2.0);
            ui.label(RichText::new(&g.name).size(13.5).strong().color(TEXT_PRIMARY));

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                let (dot_r, _) = ui.allocate_exact_size(vec2(6.0, 6.0), Sense::hover());
                ui.painter().circle_filled(dot_r.center(), 2.5, src_col);
                ui.label(
                    RichText::new(format!(
                        "{}  •  {}",
                        g.source.label(),
                        format_playtime(self.total_playtime_mins(g))
                    ))
                    .size(11.0)
                    .color(TEXT_SECONDARY),
                );
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Bouton Principal : Lancer
            let play_btn = egui::Button::new(
                RichText::new("▶  Lancer le jeu").size(12.5).strong().color(WHITE),
            )
            .fill(accent.gamma_multiply(0.25))
            .stroke(Stroke::new(1.0, accent.gamma_multiply(0.70)))
            .corner_radius(6.0);

            if ui.add(play_btn).clicked() {
                actions.push(Action::Launch(idx));
                ui.close();
            }

            ui.add_space(2.0);

            // Fiche détaillée
            if ui.button("📋  Fiche détaillée & Propriétés…").clicked() {
                actions.push(Action::OpenDetails(g.id.clone()));
                ui.close();
            }

            // Favori
            let fav_lbl = if fav {
                "⭐  Retirer des favoris"
            } else {
                "⭐  Ajouter aux favoris"
            };
            if ui.button(fav_lbl).clicked() {
                actions.push(Action::Fav(g.id.clone()));
                ui.close();
            }

            ui.separator();

            // Jaquette & Fichiers
            if ui.button("🎨  Modifier la jaquette HD…").clicked() {
                actions.push(Action::PickCover(g.id.clone()));
                ui.close();
            }
            if has_custom_cover && ui.button("🔄  Réinitialiser la jaquette").clicked() {
                actions.push(Action::ClearCover(g.id.clone()));
                ui.close();
            }
            if ui.button("💾  Sauvegarder les données (Backup)").clicked() {
                actions.push(Action::BackupSave(g.id.clone()));
                ui.close();
            }
            if let Some(d) = &g.install_dir {
                if d.exists() && ui.button("📂  Ouvrir le dossier local").clicked() {
                    actions.push(Action::OpenDir(d.clone()));
                    ui.close();
                }
            }

            ui.separator();

            // Masquer / Supprimer
            let hide_lbl = if is_hidden {
                "👁️  Restaurer dans la bibliothèque"
            } else {
                "👁️  Masquer ce titre"
            };
            if ui.button(hide_lbl).clicked() {
                actions.push(Action::ToggleHide(g.id.clone()));
                ui.close();
            }

            if g.source == Source::Manual
                && ui
                    .button(RichText::new("🗑️  Supprimer de la bibliothèque").color(RED))
                    .clicked()
            {
                actions.push(Action::Remove(g.id.clone()));
                ui.close();
            }
            ui.add_space(2.0);
        });
    }
}
