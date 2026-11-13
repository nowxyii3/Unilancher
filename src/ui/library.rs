use crate::app::Launcher;
use crate::config::{Game, Source};
use crate::process::open_uri;
use crate::scan;
use crate::theme::{DIM, GOLD, RED, WHITE};
use crate::types::{format_playtime, Action, SortMode, Tab, ViewMode};
use crate::ui::components::{
    draw_icon, game_card, game_list_row, hero_banner, icon_square_button, modern_action_button,
    search_bar, stat_chip, trunc, Icon,
};
use eframe::egui::{
    self, vec2, Align, Color32, CornerRadius, Key, Layout, Margin, RichText, Sense, Stroke, Ui,
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

        ui.add_space(14.0);

        // ── Barre de tri, filtres rapides & Mode d'affichage (Grille / Liste) ──
        ui.horizontal(|ui| {
            ui.label(RichText::new("Filtre").size(12.0).color(DIM));
            ui.add_space(4.0);
            for (idx, lbl) in [
                (0, "Tous"),
                (1, "⭐ Favoris"),
                (2, "🕒 Joués"),
                (3, "✨ Non lancés"),
            ] {
                let sel = self.settings.quick_filter_mode == idx;
                let bg = if sel {
                    accent.gamma_multiply(0.22)
                } else {
                    th.card
                };
                let fg = if sel { WHITE } else { DIM };
                let btn = egui::Button::new(RichText::new(lbl).size(12.0).color(fg))
                    .fill(bg)
                    .stroke(Stroke::new(
                        1.0,
                        if sel {
                            accent.gamma_multiply(0.8)
                        } else {
                            th.card_border
                        },
                    ))
                    .corner_radius(7.0);
                if ui
                    .add(btn)
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    self.settings.quick_filter_mode = idx;
                }
            }

            ui.add_space(14.0);
            ui.label(RichText::new("Trier par").size(12.0).color(DIM));
            ui.add_space(4.0);
            for (mode, lbl) in [
                (SortMode::Playtime, "Temps de jeu"),
                (SortMode::Name, "Ordre alphabétique"),
                (SortMode::Favorites, "Favoris en tête"),
                (SortMode::SizeOnDisk, "Taille disque"),
            ] {
                let sel = self.sort == mode;
                let bg = if sel {
                    accent.gamma_multiply(0.18)
                } else {
                    th.card
                };
                let fg = if sel { WHITE } else { DIM };
                let btn = egui::Button::new(RichText::new(lbl).size(12.0).color(fg))
                    .fill(bg)
                    .stroke(Stroke::new(
                        1.0,
                        if sel {
                            accent.gamma_multiply(0.7)
                        } else {
                            th.card_border
                        },
                    ))
                    .corner_radius(7.0);
                if ui
                    .add(btn)
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    self.sort = mode;
                }
            }

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

                    if hero_banner(ui, &fg_game, fg_mins, fg_tex.as_ref(), accent, th) {
                        actions.push(Action::Launch(featured_idx));
                    }
                    ui.add_space(18.0);
                }

                match self.view_mode {
                    ViewMode::Grid => {
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = vec2(16.0, 18.0);
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
                                } else if resp.clicked() {
                                    actions.push(Action::Launch(i));
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
        let has_custom_exe = self.settings.exe_overrides.contains_key(&g.id);
        let is_hidden = self.settings.hidden_games.contains(&g.id);

        resp.context_menu(|ui| {
            ui.set_min_width(220.0);
            if ui.button("Lancer le jeu").clicked() {
                actions.push(Action::Launch(idx));
                ui.close();
            }
            if ui.button("Propriétés, notes & arguments…").clicked() {
                actions.push(Action::OpenDetails(g.id.clone()));
                ui.close();
            }
            let fav_lbl = if fav {
                "Retirer des favoris"
            } else {
                "Ajouter aux favoris"
            };
            if ui.button(fav_lbl).clicked() {
                actions.push(Action::Fav(g.id.clone()));
                ui.close();
            }
            ui.separator();
            if ui.button("Changer la jaquette…").clicked() {
                actions.push(Action::PickCover(g.id.clone()));
                ui.close();
            }
            if has_custom_cover && ui.button("Réinitialiser la jaquette").clicked() {
                actions.push(Action::ClearCover(g.id.clone()));
                ui.close();
            }
            if ui.button("Définir un exécutable personnalisé…").clicked() {
                actions.push(Action::PickExe(g.id.clone()));
                ui.close();
            }
            if has_custom_exe && ui.button("Réinitialiser l'exécutable").clicked() {
                actions.push(Action::ClearExe(g.id.clone()));
                ui.close();
            }
            if let Some(d) = &g.install_dir {
                if d.exists() && ui.button("Ouvrir le dossier d'installation").clicked() {
                    actions.push(Action::OpenDir(d.clone()));
                    ui.close();
                }
            }
            ui.separator();
            let hide_lbl = if is_hidden {
                "Restaurer dans la bibliothèque"
            } else {
                "Masquer ce jeu"
            };
            if ui.button(hide_lbl).clicked() {
                actions.push(Action::ToggleHide(g.id.clone()));
                ui.close();
            }
            if g.source == Source::Manual
                && ui
                    .button(RichText::new("Supprimer de la liste").color(RED))
                    .clicked()
            {
                actions.push(Action::Remove(g.id.clone()));
                ui.close();
            }
        });
    }

    pub fn details_drawer_ui(&mut self, ui: &mut egui::Ui) -> Vec<Action> {
        let mut actions = vec![];
        let Some(game_id) = self.selected_game_id.clone() else {
            return actions;
        };
        let Some((idx, game)) = self
            .games
            .iter()
            .enumerate()
            .find(|(_, g)| g.id == game_id)
            .map(|(i, g)| (i, g.clone()))
        else {
            self.selected_game_id = None;
            return actions;
        };

        let accent = self.accent();
        let th = self.theme();
        let mut close_drawer = false;
        let mut save_needed = false;

        egui::Panel::right("game_details_drawer")
            .exact_size(340.0)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(th.panel)
                    .stroke(Stroke::new(1.0, th.panel_border))
                    .inner_margin(Margin::symmetric(20, 20)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Propriétés du jeu").size(16.0).strong().color(WHITE));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button("✕").clicked() {
                            close_drawer = true;
                        }
                    });
                });
                ui.add_space(12.0);

                ui.label(RichText::new(&game.name).size(20.0).strong().color(WHITE));
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!(
                        "{}  •  {}",
                        game.source.label(),
                        format_playtime(self.total_playtime_mins(&game))
                    ))
                    .size(12.5)
                    .color(DIM),
                );
                if game.size_bytes > 0 {
                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(format!("Espace disque : {}", scan::format_size(game.size_bytes)))
                            .size(12.0)
                            .color(GOLD),
                    );
                }

                ui.add_space(16.0);
                if modern_action_button(ui, "Lancer maintenant", Icon::Play, accent, true, th) {
                    actions.push(Action::Launch(idx));
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(12.0);

                ui.label(RichText::new("Arguments de lancement").size(13.0).strong().color(WHITE));
                ui.label(
                    RichText::new("Options passées à l'exécutable (ex: -dx11 -windowed)")
                        .size(11.5)
                        .color(DIM),
                );
                ui.add_space(6.0);
                let mut args = self
                    .settings
                    .launch_args
                    .get(&game.id)
                    .cloned()
                    .unwrap_or_default();
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut args)
                            .hint_text("-novid -high")
                            .desired_width(f32::INFINITY)
                            .margin(Margin::symmetric(10, 8)),
                    )
                    .changed()
                {
                    if args.trim().is_empty() {
                        self.settings.launch_args.remove(&game.id);
                    } else {
                        self.settings.launch_args.insert(game.id.clone(), args);
                    }
                    save_needed = true;
                }

                ui.add_space(16.0);
                ui.label(RichText::new("Notes personnelles").size(13.0).strong().color(WHITE));
                ui.add_space(6.0);
                let mut notes = self
                    .settings
                    .game_notes
                    .get(&game.id)
                    .cloned()
                    .unwrap_or_default();
                if ui
                    .add(
                        egui::TextEdit::multiline(&mut notes)
                            .hint_text("Objectifs, mods installés, codes, configuration...")
                            .desired_rows(5)
                            .desired_width(f32::INFINITY)
                            .margin(Margin::symmetric(10, 8)),
                    )
                    .changed()
                {
                    if notes.trim().is_empty() {
                        self.settings.game_notes.remove(&game.id);
                    } else {
                        self.settings.game_notes.insert(game.id.clone(), notes);
                    }
                    save_needed = true;
                }

                ui.add_space(16.0);
                ui.label(RichText::new("Tags & Catégories").size(13.0).strong().color(WHITE));
                ui.label(
                    RichText::new("Permet de filtrer et classer tes jeux")
                        .size(11.5)
                        .color(DIM),
                );
                ui.add_space(6.0);

                let mut current_tags = self
                    .settings
                    .game_tags
                    .get(&game.id)
                    .cloned()
                    .unwrap_or_default();

                if !current_tags.is_empty() {
                    let mut to_remove = None;
                    ui.horizontal_wrapped(|ui| {
                        for (idx, tag) in current_tags.iter().enumerate() {
                            let label = format!("{tag} ✕");
                            if ui.button(RichText::new(label).size(11.0).color(WHITE)).clicked() {
                                to_remove = Some(idx);
                            }
                        }
                    });
                    if let Some(i) = to_remove {
                        current_tags.remove(i);
                        if current_tags.is_empty() {
                            self.settings.game_tags.remove(&game.id);
                        } else {
                            self.settings.game_tags.insert(game.id.clone(), current_tags.clone());
                        }
                        save_needed = true;
                    }
                }

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let w = (ui.available_width() - 80.0).max(80.0);
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.new_tag_input)
                            .hint_text("Ajouter un tag (ex: RPG)...")
                            .desired_width(w)
                            .margin(Margin::symmetric(8, 6)),
                    );
                    let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
                    if ui.button("Ajouter").clicked() || enter {
                        let trimmed = self.new_tag_input.trim().to_string();
                        if !trimmed.is_empty() && !current_tags.iter().any(|t| t.eq_ignore_ascii_case(&trimmed)) {
                            current_tags.push(trimmed);
                            self.settings.game_tags.insert(game.id.clone(), current_tags.clone());
                            self.new_tag_input.clear();
                            save_needed = true;
                        }
                    }
                });

                ui.horizontal_wrapped(|ui| {
                    for sugg in ["RPG", "Action", "Solo", "Coop", "Terminé", "En cours", "Favori"] {
                        if !current_tags.iter().any(|t| t.eq_ignore_ascii_case(sugg))
                            && ui.small_button(format!("+ {sugg}")).clicked()
                        {
                            current_tags.push(sugg.to_string());
                            self.settings.game_tags.insert(game.id.clone(), current_tags.clone());
                            save_needed = true;
                        }
                    }
                });

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(12.0);

                ui.label(RichText::new("Raccourcis communautaires").size(13.0).strong().color(WHITE));
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    let enc_name = game.name.replace(' ', "%20");
                    if ui.button("🌐 PCGamingWiki").on_hover_text("Fixes, résolutions ultrawide & patchs").clicked() {
                        let _ = open_uri(&format!("https://www.pcgamingwiki.com/w/index.php?search={enc_name}"));
                    }
                    if ui.button("🧩 NexusMods").on_hover_text("Mods et utilitaires").clicked() {
                        let _ = open_uri(&format!("https://www.nexusmods.com/search/?gsearch={enc_name}"));
                    }
                    if ui.button("⏱️ HowLongToBeat").on_hover_text("Estimation de durée de vie").clicked() {
                        let _ = open_uri(&format!("https://howlongtobeat.com/?q={enc_name}"));
                    }
                    if ui.button("🐧 ProtonDB").on_hover_text("Compatibilité Linux / Steam Deck").clicked() {
                        let _ = open_uri(&format!("https://www.protondb.com/search?q={enc_name}"));
                    }
                });

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(12.0);

                ui.label(RichText::new("Sauvegarde des Données (Save Backup)").size(13.0).strong().color(WHITE));
                ui.add_space(4.0);
                if let Some(save_p) = scan::find_save_folder(&game) {
                    ui.label(RichText::new(format!("📁 {}", trunc(&save_p.to_string_lossy(), 34))).size(11.0).color(DIM));
                    ui.add_space(4.0);
                    if ui.button("💾 Créer une sauvegarde de secours").clicked() {
                        actions.push(Action::BackupSave(game.id.clone()));
                    }
                } else {
                    ui.label(RichText::new("Dossier de sauvegarde non détecté automatiquement").size(11.0).color(DIM));
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(12.0);

                ui.label(RichText::new("Fichiers & Raccourcis").size(13.0).strong().color(WHITE));
                ui.add_space(8.0);
                if ui.button("🔄 Télécharger jaquette HD en ligne").clicked() {
                    actions.push(Action::DownloadCover(game.id.clone()));
                }
                if ui.button("Modifier la jaquette…").clicked() {
                    actions.push(Action::PickCover(game.id.clone()));
                }
                if ui.button("Sélectionner un exécutable (.exe)…").clicked() {
                    actions.push(Action::PickExe(game.id.clone()));
                }
                if let Some(dir) = &game.install_dir {
                    if dir.exists() && ui.button("Ouvrir le dossier local").clicked() {
                        actions.push(Action::OpenDir(dir.clone()));
                    }
                }
            });

        if save_needed {
            self.settings.save();
        }
        if close_drawer {
            self.selected_game_id = None;
        }
        actions
    }
}
