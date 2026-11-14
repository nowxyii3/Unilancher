use crate::app::Launcher;
use crate::config::{Game, Source};
use crate::theme::{DIM, GOLD, RED, WHITE};
use crate::types::{format_playtime, Action, SortMode, Tab, ViewMode};
use crate::ui::components::{
    draw_icon, game_card, game_list_row, hero_banner, icon_square_button, modern_action_button,
    search_bar, stat_chip, Icon,
};
use eframe::egui::{
    self, vec2, Align, Color32, CornerRadius, Layout, Margin, RichText, Sense, Stroke, Ui,
};
use std::collections::HashSet;

impl Launcher {
    pub fn library_ui(&mut self, ui: &mut Ui) -> Vec<Action> {
        let ctx = ui.ctx().clone();
        let accent = self.accent();
        let th = self.theme();
        let mut actions: Vec<Action> = vec![];

        let (title, subtitle) = match self.tab {
            Tab::All => ("Bibliothèque", "Ensemble de tes titres détectés et ajoutés"),
            Tab::Favorites => ("Favoris", "Accès rapide à tes titres prioritaires"),
            Tab::Steam => ("Steam", "Titres installés via ton client Steam local"),
            Tab::Epic => ("Epic Games", "Titres détectés depuis les manifestes Epic Games"),
            Tab::Gog => ("GOG Galaxy", "Titres détectés via GOG Galaxy"),
            Tab::Ubisoft => ("Ubisoft Connect", "Titres détectés via Ubisoft Connect"),
            Tab::Ea => ("EA App / Origin", "Titres détectés via EA App / Origin"),
            Tab::Battlenet => ("Battle.net", "Titres détectés via l'écosystème Battle.net / Blizzard"),
            Tab::Xbox => ("Xbox / Game Pass", "Titres détectés via Xbox PC"),
            Tab::Manual => ("Jeux manuels", "Exécutables autonomes et jeux indépendants"),
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

        // ── En-tête + Barre d'actions ──
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(title).size(26.0).strong().color(WHITE));
                    ui.add_space(8.0);
                    stat_chip(ui, &format!("{} titres", filtered.len()), accent);
                    if total_mins > 0 {
                        stat_chip(
                            ui,
                            &format!("{:.1} h cumulées", total_mins as f32 / 60.0),
                            Color32::from_rgb(56, 189, 248),
                        );
                    }
                    if platform_count > 1 {
                        stat_chip(
                            ui,
                            &format!("{} plateformes", platform_count),
                            GOLD,
                        );
                    }
                });
                ui.add_space(2.0);
                ui.label(RichText::new(subtitle).size(12.5).color(DIM));
            });

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

        // ── Indicateur de Progression du Scan Multi-Plateformes en Temps Réel ──
        if let Some(prog) = &self.scan_progress {
            ui.add_space(8.0);
            egui::Frame::new()
                .fill(accent.gamma_multiply(0.12))
                .stroke(Stroke::new(1.0, accent.gamma_multiply(0.40)))
                .corner_radius(10.0)
                .inner_margin(Margin::symmetric(14, 8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🔄").size(13.0));
                        let desc = if prog.games_found > 0 {
                            format!("{} ({} trouvés)", prog.stage, prog.games_found)
                        } else {
                            prog.stage.clone()
                        };
                        ui.label(RichText::new(desc).size(12.5).strong().color(WHITE));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let pct_str = format!("{:.0}%", prog.percent * 100.0);
                            ui.label(RichText::new(pct_str).size(12.0).strong().color(accent));
                            let pb = egui::ProgressBar::new(prog.percent)
                                .desired_width(120.0)
                                .fill(accent);
                            ui.add(pb);
                        });
                    });
                });
        }

        ui.add_space(14.0);

        // ── Segmented Control pour les Filtres rapides (Style React / Tailwind Tabs) ──
        ui.horizontal(|ui| {
            egui::Frame::new()
                .fill(th.panel)
                .stroke(Stroke::new(1.0, th.card_border))
                .corner_radius(10.0)
                .inner_margin(Margin::same(3))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    for (idx, lbl) in [
                        (0, "Tous"),
                        (1, "⭐ Favoris"),
                        (2, "🕒 Récents"),
                        (3, "✨ Non lancés"),
                    ] {
                        let sel = self.settings.quick_filter_mode == idx;
                        let bg = if sel {
                            accent.gamma_multiply(0.28)
                        } else {
                            Color32::TRANSPARENT
                        };
                        let fg = if sel { WHITE } else { DIM };
                        let btn = egui::Button::new(RichText::new(lbl).size(12.0).strong().color(fg))
                            .fill(bg)
                            .stroke(Stroke::new(
                                1.0,
                                if sel { accent.gamma_multiply(0.8) } else { Color32::TRANSPARENT },
                            ))
                            .corner_radius(8.0);
                        if ui
                            .add(btn)
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            self.settings.quick_filter_mode = idx;
                        }
                    }
                });

            ui.add_space(8.0);

            // Segmented Control pour le Tri
            egui::Frame::new()
                .fill(th.panel)
                .stroke(Stroke::new(1.0, th.card_border))
                .corner_radius(10.0)
                .inner_margin(Margin::same(3))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    for (mode, lbl) in [
                        (SortMode::Playtime, "Temps de jeu"),
                        (SortMode::Name, "A → Z"),
                        (SortMode::Favorites, "Favoris"),
                        (SortMode::SizeOnDisk, "Taille disque"),
                    ] {
                        let sel = self.sort == mode;
                        let bg = if sel {
                            accent.gamma_multiply(0.25)
                        } else {
                            Color32::TRANSPARENT
                        };
                        let fg = if sel { WHITE } else { DIM };
                        let btn = egui::Button::new(RichText::new(lbl).size(12.0).strong().color(fg))
                            .fill(bg)
                            .stroke(Stroke::new(
                                1.0,
                                if sel { accent.gamma_multiply(0.7) } else { Color32::TRANSPARENT },
                            ))
                            .corner_radius(8.0);
                        if ui
                            .add(btn)
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            self.sort = mode;
                        }
                    }
                });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if icon_square_button(
                    ui,
                    Icon::Gamepad,
                    "Mode Big Picture / Salon (F10)",
                    self.is_big_picture,
                    th,
                ) {
                    self.is_big_picture = true;
                }
                ui.add_space(4.0);
                if icon_square_button(
                    ui,
                    Icon::List,
                    "Vue Liste détaillée",
                    self.view_mode == ViewMode::List,
                    th,
                ) {
                    self.view_mode = ViewMode::List;
                }
                if icon_square_button(
                    ui,
                    Icon::Grid,
                    "Vue Grille de jaquettes",
                    self.view_mode == ViewMode::Grid,
                    th,
                ) {
                    self.view_mode = ViewMode::Grid;
                }
                ui.add_space(8.0);
                ui.add(
                    egui::Slider::new(&mut self.settings.card_scale, 0.75..=1.35)
                        .show_value(false)
                        .trailing_fill(true),
                )
                .on_hover_text(format!("Zoom de la grille ({:.0}%)", self.settings.card_scale * 100.0));
            });
        });

        ui.add_space(14.0);

        if filtered.is_empty() {
            ui.add_space(90.0);
            ui.vertical_centered(|ui| {
                let (r, _) = ui.allocate_exact_size(vec2(60.0, 60.0), Sense::hover());
                ui.painter().rect_filled(r, CornerRadius::same(16), th.card);
                draw_icon(ui.painter(), Icon::Library, r.center(), DIM);
                ui.add_space(12.0);
                ui.label(RichText::new("Aucun titre trouvé").size(18.0).strong().color(WHITE));
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Modifie ta recherche ou ajoute un exécutable via le bouton « Ajouter un jeu ».")
                        .size(13.0)
                        .color(DIM),
                );
            });
            return actions;
        }

        egui::ScrollArea::vertical()
            .id_salt("library_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Bannière de reprise rapide uniquement sur l'onglet principal sans filtre texte
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
                    ViewMode::Grid => {
                        ui.horizontal_wrapped(|ui| {
                            let spacing = self.settings.grid_spacing.clamp(8.0, 36.0);
                            ui.spacing_mut().item_spacing = vec2(spacing, spacing + 2.0);
                            for &i in &filtered {
                                let g = self.games[i].clone();
                                let fav = favs.contains(&g.id);
                                let ovr = self.settings.exe_overrides.get(&g.id).cloned();
                                let fixed = ovr.as_ref().map(|p| p.exists()).unwrap_or(false);
                                let broken = g.is_broken() && !fixed;
                                let play_mins = self.total_playtime_mins(&g);
                                let tex = self.get_cover_texture(&ctx, &g);

                                let (resp, star_clicked) = game_card(
                                    ui,
                                    &g,
                                    tex.as_ref(),
                                    play_mins,
                                    fav,
                                    broken,
                                    ovr.is_some(),
                                    accent,
                                    self.settings.card_scale,
                                    th,
                                );

                                if star_clicked {
                                    actions.push(Action::Fav(g.id.clone()));
                                } else if resp.double_clicked() {
                                    actions.push(Action::Launch(i));
                                } else if resp.clicked() {
                                    actions.push(Action::OpenDetails(g.id.clone()));
                                }

                                self.attach_game_context_menu(&resp, &g, i, fav, &mut actions);
                            }
                        });
                    }
                    ViewMode::List => {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 8.0;
                            for &i in &filtered {
                                let g = self.games[i].clone();
                                let fav = favs.contains(&g.id);
                                let play_mins = self.total_playtime_mins(&g);
                                let tex = self.get_cover_texture(&ctx, &g);

                                let (resp, play_clicked, fav_clicked, opt_clicked) =
                                    game_list_row(ui, &g, tex.as_ref(), play_mins, fav, accent, th);

                                if fav_clicked {
                                    actions.push(Action::Fav(g.id.clone()));
                                } else if opt_clicked {
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

        resp.context_menu(|ui| {
            ui.set_min_width(240.0);
            ui.spacing_mut().item_spacing.y = 3.0;

            // En-tête avec nom du jeu et plateforme
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(&g.name).size(13.0).strong().color(WHITE));
            });
            ui.label(
                RichText::new(format!(
                    "{}  •  {}",
                    g.source.label(),
                    format_playtime(self.total_playtime_mins(g))
                ))
                .size(11.0)
                .color(DIM),
            );
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Bouton Jouer
            let play_btn = egui::Button::new(
                RichText::new("▶  Lancer le jeu").size(12.5).strong().color(WHITE),
            )
            .fill(accent.gamma_multiply(0.22))
            .stroke(Stroke::new(1.0, accent.gamma_multiply(0.7)))
            .corner_radius(6.0);
            if ui.add(play_btn).clicked() {
                actions.push(Action::Launch(idx));
                ui.close();
            }

            // Bouton Fiche détaillée & Propriétés
            if ui.button("📋  Fiche détaillée du jeu & Propriétés…").clicked() {
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

            // Jaquette & Sauvegarde
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
                if d.exists() && ui.button("📂  Ouvrir l'emplacement local").clicked() {
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
