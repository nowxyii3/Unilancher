use crate::config::{Game, Settings, Source};
use crate::discord::{spawn_discord_rpc_worker, RpcEvent, RpcStatus};
use crate::process::{launch_game, open_uri};
use crate::scan;
use crate::theme::{
    apply_style, get_theme, ThemeColors, ACCENTS, DIM, DISCORD_C, EA_C, EPIC_C, GOG_C, GOLD,
    GREEN, RED, STEAM_C, UBI_C, WHITE, XBOX_C,
};
use crate::types::{Action, SortMode, Tab, ViewMode};
use crate::ui::components::{icon_square_button, sidebar_item, trunc, Icon};
use eframe::egui::{
    self, vec2, Align, Align2, Color32, ColorImage, CornerRadius, FontId, Id, Key, Layout, Margin,
    Rect, RichText, Sense, Stroke, TextureHandle, TextureOptions,
};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::mpsc,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[cfg(windows)]
use std::process::Command;

// ───────────────────────────── Structure Principale ─────────────────────────────
pub struct Launcher {
    pub(crate) settings: Settings,
    pub(crate) games: Vec<Game>,
    pub(crate) tab: Tab,
    pub(crate) sort: SortMode,
    pub(crate) view_mode: ViewMode,
    pub(crate) search: String,
    pub(crate) focus_search: bool,
    pub(crate) is_fullscreen: bool,
    pub(crate) status: Option<(String, Instant)>,
    pub(crate) applied_accent: usize,
    pub(crate) applied_theme: usize,
    pub(crate) is_big_picture: bool,
    pub(crate) big_picture_index: usize,

    // Chargement asynchrone des textures (0 freeze)
    pub(crate) covers: HashMap<String, Option<TextureHandle>>,
    pub(crate) loading_covers: HashSet<String>,
    pub(crate) tex_tx: mpsc::Sender<(String, ColorImage)>,
    pub(crate) tex_rx: mpsc::Receiver<(String, ColorImage)>,

    // Suivi de session et Discord RPC
    pub(crate) playtime_rx: mpsc::Receiver<(String, u32)>,
    pub(crate) playtime_tx: mpsc::Sender<(String, u32)>,
    pub(crate) rpc_tx: mpsc::Sender<RpcEvent>,
    pub(crate) rpc_status_rx: mpsc::Receiver<RpcStatus>,
    pub(crate) rpc_status: RpcStatus,
    pub(crate) active_game: Option<(String, Instant)>,

    // Panneau latéral / Modale de propriétés d'un jeu
    pub(crate) selected_game_id: Option<String>,
    pub(crate) properties_tab: usize,
    pub(crate) show_discord_guide: bool,
    pub(crate) new_tag_input: String,

    // Spotlight search
    pub(crate) spotlight_open: bool,
    pub(crate) spotlight_query: String,
    pub(crate) spotlight_selected_idx: usize,
    pub(crate) spotlight_focus_needed: bool,

    // Deals / Free Games tracker
    pub(crate) deals: Vec<crate::deals::FreeDeal>,
    pub(crate) loading_deals: bool,
    pub(crate) deals_tx: mpsc::Sender<Vec<crate::deals::FreeDeal>>,
    pub(crate) deals_rx: mpsc::Receiver<Vec<crate::deals::FreeDeal>>,
    pub(crate) centered_once: bool,
}

impl Launcher {
    pub fn new(cc: &eframe::CreationContext<'_>, settings: Settings) -> Self {
        let (playtime_tx, playtime_rx) = mpsc::channel();
        let (tex_tx, tex_rx) = mpsc::channel();
        let (rpc_tx, rpc_rx) = mpsc::channel();
        let (rpc_status_tx, rpc_status_rx) = mpsc::channel();
        let (deals_tx, deals_rx) = mpsc::channel();

        spawn_discord_rpc_worker(
            rpc_rx,
            rpc_status_tx,
            settings.discord_rpc,
            settings.discord_client_id.clone(),
        );

        let is_fullscreen = settings.start_fullscreen;
        let mut l = Self {
            settings,
            games: vec![],
            tab: Tab::All,
            sort: SortMode::Playtime,
            view_mode: ViewMode::Grid,
            search: String::new(),
            focus_search: false,
            is_fullscreen,
            status: None,
            applied_accent: usize::MAX,
            applied_theme: usize::MAX,
            is_big_picture: false,
            big_picture_index: 0,
            covers: HashMap::new(),
            loading_covers: HashSet::new(),
            tex_tx,
            tex_rx,
            playtime_rx,
            playtime_tx,
            rpc_tx,
            rpc_status_rx,
            rpc_status: RpcStatus::Connecting,
            active_game: None,
            selected_game_id: None,
            properties_tab: 0,
            show_discord_guide: false,
            new_tag_input: String::new(),
            spotlight_open: false,
            spotlight_query: String::new(),
            spotlight_selected_idx: 0,
            spotlight_focus_needed: false,
            deals: vec![],
            loading_deals: false,
            deals_tx,
            deals_rx,
            centered_once: false,
        };

        apply_style(&cc.egui_ctx, l.accent(), l.settings.theme);
        l.applied_accent = l.settings.accent;
        l.applied_theme = l.settings.theme;
        l.refresh();
        l.load_deals_async();
        l
    }

    pub fn theme(&self) -> ThemeColors {
        get_theme(self.settings.theme)
    }

    pub fn accent(&self) -> Color32 {
        ACCENTS[self.settings.accent.min(ACCENTS.len() - 1)]
    }

    pub fn total_playtime_mins(&self, g: &Game) -> u32 {
        let extra = self
            .settings
            .tracked_playtime
            .get(&g.id)
            .copied()
            .unwrap_or(0);
        g.playtime_mins + extra
    }

    pub fn update_rpc_menu(&self) {
        let visible_count = self
            .games
            .iter()
            .filter(|g| !self.settings.hidden_games.contains(&g.id))
            .count();
        let _ = self.rpc_tx.send(RpcEvent::InMenu(visible_count));
    }

    pub fn refresh(&mut self) {
        let mut g = scan::scan_all(&self.settings);
        for mut mg in self.settings.manual_games.clone() {
            if mg.cover_path.is_none() {
                if let Some(dir) = &mg.install_dir {
                    mg.cover_path = scan::find_folder_cover(dir);
                }
            }
            g.push(mg);
        }

        // Scanner également les dossiers de ROMs d'émulateurs
        let emus = crate::emulators::detect_installed_emulators();
        for folder in &self.settings.custom_scan_folders {
            let roms = crate::emulators::scan_roms_folder(folder, &emus);
            g.extend(roms);
        }

        self.games = g;
        self.covers.clear();
        self.loading_covers.clear();
        if self.active_game.is_none() {
            self.update_rpc_menu();
        }
    }

    pub fn rescan_roms(&mut self) {
        self.refresh();
        let rom_count = self.games.iter().filter(|g| g.id.starts_with("rom:")).count();
        self.toast(format!("{rom_count} ROM(s) rétrogaming indexée(s)"));
    }

    pub fn load_deals_async(&mut self) {
        if self.loading_deals {
            return;
        }
        self.loading_deals = true;
        let tx = self.deals_tx.clone();
        thread::spawn(move || {
            if let Ok(deals) = crate::deals::DealsTracker::fetch_epic_free_games() {
                let _ = tx.send(deals);
            } else {
                let _ = tx.send(vec![]);
            }
        });
    }

    /// Charge les jaquettes sur un thread d'arrière-plan pour garder l'UI à 144 FPS constants
    pub fn get_cover_texture(&mut self, ctx: &egui::Context, g: &Game) -> Option<TextureHandle> {
        if let Some(cached) = self.covers.get(&g.id) {
            return cached.clone();
        }
        if self.loading_covers.contains(&g.id) {
            return None;
        }

        let path = self
            .settings
            .cover_overrides
            .get(&g.id)
            .or(g.cover_path.as_ref())
            .cloned();

        if let Some(p) = path {
            self.loading_covers.insert(g.id.clone());
            let tx = self.tex_tx.clone();
            let id = g.id.clone();
            let ctx_clone = ctx.clone();

            thread::spawn(move || {
                if let Ok(bytes) = std::fs::read(&p) {
                    if let Ok(img) = image::load_from_memory(&bytes) {
                        let resized = img.thumbnail(420, 600);
                        let rgba = resized.to_rgba8();
                        let dims = [rgba.width() as usize, rgba.height() as usize];
                        let color_image = ColorImage::from_rgba_unmultiplied(
                            dims,
                            rgba.as_flat_samples().as_slice(),
                        );
                        let _ = tx.send((id, color_image));
                        ctx_clone.request_repaint();
                    }
                }
            });
        } else if self.settings.auto_download_covers {
            self.loading_covers.insert(g.id.clone());
            let g_clone = g.clone();
            let tx = self.tex_tx.clone();
            let id = g.id.clone();
            let ctx_clone = ctx.clone();
            let cache_dir = Settings::dir().join("covers_cache");

            thread::spawn(move || {
                if let Some(cover_file) = scan::download_cover_auto(&g_clone, &cache_dir) {
                    if let Ok(bytes) = std::fs::read(&cover_file) {
                        if let Ok(img) = image::load_from_memory(&bytes) {
                            let resized = img.thumbnail(420, 600);
                            let rgba = resized.to_rgba8();
                            let dims = [rgba.width() as usize, rgba.height() as usize];
                            let color_image = ColorImage::from_rgba_unmultiplied(
                                dims,
                                rgba.as_flat_samples().as_slice(),
                            );
                            let _ = tx.send((id, color_image));
                            ctx_clone.request_repaint();
                        }
                    }
                }
            });
        } else {
            self.covers.insert(g.id.clone(), None);
        }
        None
    }

    pub fn toast(&mut self, msg: impl Into<String>) {
        self.status = Some((msg.into(), Instant::now()));
    }

    pub fn filtered_indices(&self) -> Vec<usize> {
        let q = self.search.to_lowercase();
        let tab = self.tab;
        let favs = &self.settings.favorites;
        let hidden = &self.settings.hidden_games;

        let mut filtered: Vec<usize> = self
            .games
            .iter()
            .enumerate()
            .filter(|(_, g)| {
                let is_hidden = hidden.contains(&g.id);
                if tab == Tab::Hidden {
                    return is_hidden && (q.is_empty() || g.name.to_lowercase().contains(&q));
                }
                if is_hidden {
                    return false;
                }
                let in_tab = match tab {
                    Tab::All => true,
                    Tab::Favorites => favs.contains(&g.id),
                    Tab::Steam => g.source == Source::Steam,
                    Tab::Epic => g.source == Source::Epic,
                    Tab::Gog => g.source == Source::Gog,
                    Tab::Ubisoft => g.source == Source::Ubisoft,
                    Tab::Ea => g.source == Source::Ea,
                    Tab::Xbox => g.source == Source::Xbox,
                    Tab::Manual => g.source == Source::Manual && !g.id.starts_with("rom:"),
                    Tab::Emulators => g.id.starts_with("rom:"),
                    Tab::Deals | Tab::Hidden | Tab::Settings | Tab::Stats => false,
                };
                let in_quick_filter = match self.settings.quick_filter_mode {
                    1 => favs.contains(&g.id),
                    2 => self.total_playtime_mins(g) > 0,
                    3 => self.total_playtime_mins(g) == 0,
                    _ => true,
                };
                let matches_query = if q.is_empty() {
                    true
                } else {
                    let name_match = g.name.to_lowercase().contains(&q);
                    let tag_match = self
                        .settings
                        .game_tags
                        .get(&g.id)
                        .map(|tags| tags.iter().any(|t| t.to_lowercase().contains(&q)))
                        .unwrap_or(false);
                    let note_match = self
                        .settings
                        .game_notes
                        .get(&g.id)
                        .map(|n| n.to_lowercase().contains(&q))
                        .unwrap_or(false);
                    name_match || tag_match || note_match
                };
                in_tab && in_quick_filter && matches_query
            })
            .map(|(i, _)| i)
            .collect();

        match self.sort {
            SortMode::Playtime => {
                filtered.sort_by(|&a, &b| {
                    let pa = self.total_playtime_mins(&self.games[a]);
                    let pb = self.total_playtime_mins(&self.games[b]);
                    pb.cmp(&pa).then_with(|| {
                        self.games[a]
                            .name
                            .to_lowercase()
                            .cmp(&self.games[b].name.to_lowercase())
                    })
                });
            }
            SortMode::Name => {
                filtered.sort_by_key(|&i| self.games[i].name.to_lowercase());
            }
            SortMode::Favorites => {
                filtered.sort_by(|&a, &b| {
                    let fa = favs.contains(&self.games[a].id);
                    let fb = favs.contains(&self.games[b].id);
                    fb.cmp(&fa).then_with(|| {
                        let pa = self.total_playtime_mins(&self.games[a]);
                        let pb = self.total_playtime_mins(&self.games[b]);
                        pb.cmp(&pa)
                    })
                });
            }
            SortMode::SizeOnDisk => {
                filtered.sort_by(|&a, &b| {
                    self.games[b]
                        .size_bytes
                        .cmp(&self.games[a].size_bytes)
                        .then_with(|| {
                            self.games[a]
                                .name
                                .to_lowercase()
                                .cmp(&self.games[b].name.to_lowercase())
                        })
                });
            }
        }
        filtered
    }

    pub fn launch(&mut self, ctx: &egui::Context, i: usize) {
        if let Some(g) = self.games.get(i).cloned() {
            match launch_game(&g, &self.settings, self.playtime_tx.clone()) {
                Ok(()) => {
                    self.settings.last_played = Some(g.id.clone());
                    self.settings.save();
                    self.active_game = Some((g.name.clone(), Instant::now()));
                    let _ = self
                        .rpc_tx
                        .send(RpcEvent::Playing(g.name.clone(), g.source.label().to_string()));
                    self.toast(format!("Lancement de {} en cours...", g.name));
                    if self.settings.minimize_on_launch {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                }
                Err(e) => self.toast(format!("Erreur au lancement de {} : {}", g.name, e)),
            }
        }
    }

    pub fn apply(&mut self, ctx: &egui::Context, a: Action) {
        match a {
            Action::Launch(i) => self.launch(ctx, i),
            Action::LaunchRandom => {
                let candidates = self.filtered_indices();
                if !candidates.is_empty() {
                    let nanos = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .subsec_nanos() as usize;
                    let pick = candidates[nanos % candidates.len()];
                    self.launch(ctx, pick);
                } else {
                    self.toast("Aucun jeu disponible dans cette vue");
                }
            }
            Action::Fav(id) => {
                if !self.settings.favorites.remove(&id) {
                    self.settings.favorites.insert(id);
                }
                self.settings.save();
            }
            Action::ToggleHide(id) => {
                if !self.settings.hidden_games.remove(&id) {
                    self.settings.hidden_games.insert(id);
                    self.toast("Jeu masqué de la bibliothèque");
                } else {
                    self.toast("Jeu restauré dans la bibliothèque");
                }
                self.settings.save();
                self.update_rpc_menu();
            }
            Action::OpenDetails(id) => {
                self.selected_game_id = Some(id);
            }
            Action::PickExe(id) => {
                if let Some(p) = rfd::FileDialog::new()
                    .set_title("Choisir l'exécutable du jeu")
                    .pick_file()
                {
                    self.settings.exe_overrides.insert(id, p);
                    self.settings.save();
                    self.toast("Exécutable mis à jour");
                }
            }
            Action::ClearExe(id) => {
                self.settings.exe_overrides.remove(&id);
                self.settings.save();
            }
            Action::PickCover(id) => {
                if let Some(p) = rfd::FileDialog::new()
                    .set_title("Choisir une image de couverture (PNG/JPG/WEBP)")
                    .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                    .pick_file()
                {
                    self.settings.cover_overrides.insert(id.clone(), p);
                    self.covers.remove(&id);
                    self.loading_covers.remove(&id);
                    self.settings.save();
                    self.toast("Jaquette personnalisée appliquée");
                }
            }
            Action::ClearCover(id) => {
                self.settings.cover_overrides.remove(&id);
                self.covers.remove(&id);
                self.loading_covers.remove(&id);
                self.settings.save();
            }
            Action::DownloadCover(id) => {
                if let Some(game) = self.games.iter().find(|g| g.id == id).cloned() {
                    self.toast(format!("Téléchargement de l'illustration pour {}…", game.name));
                    self.covers.remove(&id);
                    self.loading_covers.insert(id.clone());
                    let tx = self.tex_tx.clone();
                    let ctx_clone = ctx.clone();
                    let cache_dir = Settings::dir().join("covers_cache");
                    let id_clone = id.clone();

                    thread::spawn(move || {
                        if let Some(cover_file) = scan::download_cover_auto(&game, &cache_dir) {
                            if let Ok(bytes) = std::fs::read(&cover_file) {
                                if let Ok(img) = image::load_from_memory(&bytes) {
                                    let resized = img.thumbnail(420, 600);
                                    let rgba = resized.to_rgba8();
                                    let dims = [rgba.width() as usize, rgba.height() as usize];
                                    let color_image = ColorImage::from_rgba_unmultiplied(
                                        dims,
                                        rgba.as_flat_samples().as_slice(),
                                    );
                                    let _ = tx.send((id_clone, color_image));
                                    ctx_clone.request_repaint();
                                }
                            }
                        }
                    });
                }
            }
            Action::BackupSave(id) => {
                if let Some(game) = self.games.iter().find(|g| g.id == id).cloned() {
                    let backup_root = dirs::document_dir()
                        .map(|d| d.join("UniLauncher_Backups"))
                        .unwrap_or_else(|| PathBuf::from("UniLauncher_Backups"));
                    match scan::create_save_backup(&game, &backup_root) {
                        Ok(dest) => {
                            let name = dest.file_name().unwrap_or_default().to_string_lossy();
                            self.toast(format!("Sauvegarde réussie : {name}"));
                        }
                        Err(e) => {
                            self.toast(format!("Échec sauvegarde : {e}"));
                        }
                    }
                }
            }
            Action::OpenDir(p) => {
                let _ = open_uri(&p.to_string_lossy());
            }
            Action::Remove(id) => {
                self.settings.manual_games.retain(|g| g.id != id);
                if self.selected_game_id.as_deref() == Some(&id) {
                    self.selected_game_id = None;
                }
                self.settings.save();
                self.refresh();
            }
            Action::Refresh => {
                self.refresh();
                self.toast(format!("Bibliothèque synchronisée ({} titres)", self.games.len()));
            }
            Action::Add => {
                if let Some(p) = rfd::FileDialog::new()
                    .set_title("Choisir l'exécutable du jeu")
                    .pick_file()
                {
                    let name = p
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Jeu".into());
                    let id = format!("manual:{}", p.display());
                    if self.settings.manual_games.iter().any(|g| g.id == id) {
                        self.toast("Ce jeu figure déjà dans la bibliothèque");
                    } else {
                        let parent = p.parent().map(|d| d.to_path_buf());
                        let cover = parent.as_deref().and_then(scan::find_folder_cover);
                        self.settings.manual_games.push(Game {
                            id,
                            name: name.clone(),
                            source: Source::Manual,
                            install_dir: parent,
                            exe: Some(p),
                            launch_uri: None,
                            cover_path: cover,
                            playtime_mins: 0,
                            size_bytes: 0,
                        });
                        self.settings.save();
                        self.refresh();
                        self.toast(format!("{name} ajouté à la bibliothèque"));
                    }
                }
            }
            Action::AddScanFolder => {
                if let Some(p) = rfd::FileDialog::new()
                    .set_title("Sélectionner un dossier contenant des jeux à scanner")
                    .pick_folder()
                {
                    if !self.settings.custom_scan_folders.contains(&p) {
                        self.settings.custom_scan_folders.push(p.clone());
                        self.settings.save();
                        self.refresh();
                        self.toast(format!("Dossier ajouté : {}", p.display()));
                    } else {
                        self.toast("Ce dossier est déjà configuré");
                    }
                }
            }
            Action::OpenUrl(url) => {
                let _ = open_uri(&url);
            }
            Action::ScanRoms => {
                self.rescan_roms();
            }
            Action::RefreshDeals => {
                self.load_deals_async();
            }
        }
    }

    // ───────────────────────────── Bandeau Titre Riot Style (Frameless) ─────────────────────────────
    pub fn render_riot_title_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let th = self.theme();
        let accent = self.accent();

        egui::Panel::top("riot_title_bar")
            .exact_size(38.0)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(th.panel)
                    .stroke(Stroke::new(1.0, th.panel_border))
                    .inner_margin(Margin::ZERO),
            )
            .show(ui, |ui| {
                let bar_rect = ui.max_rect();

                // Déplacement de la fenêtre par glisser-déposer sur toute la barre
                let drag_resp = ui.interact(
                    bar_rect,
                    Id::unique("riot_titlebar_drag"),
                    Sense::click_and_drag(),
                );
                if drag_resp.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                if drag_resp.double_clicked() {
                    let is_max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_max));
                }

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    // ── Gauche : Logo + Titre ──
                    ui.add_space(14.0);
                    let (icon_r, _) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::hover());
                    let p = ui.painter();
                    p.rect_filled(icon_r, CornerRadius::same(5), accent);
                    let c = icon_r.center();
                    let st = Stroke::new(1.8, WHITE);
                    p.line_segment([c + vec2(-3.5, -4.0), c + vec2(-3.5, 1.5)], st);
                    p.line_segment([c + vec2(3.5, -4.0), c + vec2(3.5, 1.5)], st);
                    p.line_segment([c + vec2(-3.0, 3.5), c + vec2(3.0, 3.5)], st);

                    ui.add_space(10.0);
                    ui.label(RichText::new("UniLauncher").size(13.5).strong().color(WHITE));
                    ui.add_space(8.0);
                    ui.label(RichText::new("Client").size(11.5).color(DIM));

                    // ── Centre : Raccourci recherche discret ──
                    let total_w = ui.available_width();
                    if total_w > 420.0 {
                        ui.add_space(total_w * 0.18);
                        let (search_pill, pill_resp) = ui.allocate_exact_size(vec2(230.0, 26.0), Sense::click());
                        let hov = pill_resp.hovered();
                        ui.painter().rect_filled(
                            search_pill,
                            CornerRadius::same(13),
                            if hov { th.card_hover } else { th.card },
                        );
                        ui.painter().rect_stroke(
                            search_pill,
                            CornerRadius::same(13),
                            Stroke::new(1.0, if hov { accent.gamma_multiply(0.6) } else { th.card_border }),
                            egui::StrokeKind::Inside,
                        );
                        let p = ui.painter();
                        p.text(
                            search_pill.center(),
                            Align2::CENTER_CENTER,
                            "🔍 Rechercher (Alt + Espace)",
                            FontId::proportional(11.5),
                            if hov { WHITE } else { DIM },
                        );
                        if pill_resp.clicked() {
                            self.spotlight_open = true;
                            self.spotlight_focus_needed = true;
                            self.spotlight_query.clear();
                        }
                    }

                    // ── Droite : Boutons de contrôle de fenêtre Riot Style ──
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // 1. Bouton Fermer (✕) - Devient ROUGE vif au survol
                        let (close_rect, close_resp) = ui.allocate_exact_size(vec2(48.0, 38.0), Sense::click());
                        let close_hov = close_resp.hovered();
                        if close_hov {
                            ui.painter().rect_filled(close_rect, CornerRadius::ZERO, Color32::from_rgb(232, 17, 35));
                        }
                        let close_col = if close_hov { WHITE } else { DIM };
                        let c = close_rect.center();
                        let st = Stroke::new(1.3, close_col);
                        ui.painter().line_segment([c + vec2(-4.5, -4.5), c + vec2(4.5, 4.5)], st);
                        ui.painter().line_segment([c + vec2(4.5, -4.5), c + vec2(-4.5, 4.5)], st);
                        if close_resp.clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }

                        // 2. Bouton Maximiser / Restaurer (□ / ❐)
                        let is_max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                        let (max_rect, max_resp) = ui.allocate_exact_size(vec2(44.0, 38.0), Sense::click());
                        let max_hov = max_resp.hovered();
                        if max_hov {
                            ui.painter().rect_filled(max_rect, CornerRadius::ZERO, th.card_hover);
                        }
                        let max_col = if max_hov { WHITE } else { DIM };
                        let c = max_rect.center();
                        if is_max {
                            ui.painter().rect_stroke(
                                Rect::from_center_size(c + vec2(-2.0, 2.0), vec2(8.0, 8.0)),
                                CornerRadius::ZERO,
                                Stroke::new(1.2, max_col),
                                egui::StrokeKind::Inside,
                            );
                            ui.painter().rect_stroke(
                                Rect::from_center_size(c + vec2(2.0, -2.0), vec2(8.0, 8.0)),
                                CornerRadius::ZERO,
                                Stroke::new(1.2, max_col),
                                egui::StrokeKind::Inside,
                            );
                        } else {
                            ui.painter().rect_stroke(
                                Rect::from_center_size(c, vec2(10.0, 10.0)),
                                CornerRadius::ZERO,
                                Stroke::new(1.2, max_col),
                                egui::StrokeKind::Inside,
                            );
                        }
                        if max_resp.clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_max));
                        }

                        // 3. Bouton Minimiser (—)
                        let (min_rect, min_resp) = ui.allocate_exact_size(vec2(44.0, 38.0), Sense::click());
                        let min_hov = min_resp.hovered();
                        if min_hov {
                            ui.painter().rect_filled(min_rect, CornerRadius::ZERO, th.card_hover);
                        }
                        let min_col = if min_hov { WHITE } else { DIM };
                        let c = min_rect.center();
                        ui.painter().line_segment(
                            [c + vec2(-5.0, 0.0), c + vec2(5.0, 0.0)],
                            Stroke::new(1.4, min_col),
                        );
                        if min_resp.clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                    });
                });
            });
    }
}

// ───────────────────────────── Boucle Principale eframe::App ─────────────────────────────
impl eframe::App for Launcher {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let th = self.theme();

        // 0. Centrage automatique au milieu de l'écran lors du lancement
        if !self.centered_once && !self.is_fullscreen {
            if let Some(cmd) = egui::ViewportCommand::center_on_screen(&ctx) {
                ctx.send_viewport_cmd(cmd);
                self.centered_once = true;
            }
        }

        // 1. Intégration des textures chargées en arrière-plan
        while let Ok((id, color_img)) = self.tex_rx.try_recv() {
            let tex = ctx.load_texture(format!("cover_{id}"), color_img, TextureOptions::LINEAR);
            self.covers.insert(id.clone(), Some(tex));
            self.loading_covers.remove(&id);
        }

        // 1b. Réception du statut Discord RPC en direct
        while let Ok(st) = self.rpc_status_rx.try_recv() {
            self.rpc_status = st;
        }

        // 1c. Réception des bons plans et jeux gratuits
        while let Ok(deals) = self.deals_rx.try_recv() {
            self.loading_deals = false;
            if !deals.is_empty() {
                self.deals = deals;
            }
        }

        // 2. Réception de la fin de session d'un jeu
        while let Ok((game_id, mins_added)) = self.playtime_rx.try_recv() {
            self.active_game = None;
            self.update_rpc_menu();
            if self.settings.performance_mode {
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    let mut cmd = Command::new("powercfg");
                    cmd.creation_flags(0x08000000);
                    let _ = cmd.args(["/setactive", "381b4222-f694-41f0-9685-ff5bb260df2e"]).spawn();
                }
            }
            if mins_added > 0 {
                let entry = self.settings.tracked_playtime.entry(game_id).or_insert(0);
                *entry += mins_added;
                self.settings.save();
                self.toast(format!("Session terminée (+{mins_added} min comptabilisées)"));
            }
        }

        // 3. Raccourcis clavier globaux (F11 = Plein écran, F10 = Big Picture, Ctrl+F = Recherche, Échap = Fermer tiroir/recherche)
        ctx.input(|i| {
            if (i.modifiers.alt || i.modifiers.command) && i.key_pressed(Key::Space) {
                self.spotlight_open = !self.spotlight_open;
                self.spotlight_focus_needed = true;
                self.spotlight_query.clear();
            }
            if i.key_pressed(Key::F11) {
                self.is_fullscreen = !self.is_fullscreen;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.is_fullscreen));
            }
            if i.key_pressed(Key::F10) {
                self.is_big_picture = !self.is_big_picture;
            }
            if i.modifiers.ctrl && i.key_pressed(Key::F) {
                self.focus_search = true;
            }
            if i.key_pressed(Key::Escape) {
                if self.is_big_picture {
                    self.is_big_picture = false;
                } else if self.selected_game_id.is_some() {
                    self.selected_game_id = None;
                } else if !self.search.is_empty() {
                    self.search.clear();
                }
            }
        });

        if self.applied_accent != self.settings.accent || self.applied_theme != self.settings.theme {
            apply_style(&ctx, self.accent(), self.settings.theme);
            self.applied_accent = self.settings.accent;
            self.applied_theme = self.settings.theme;
        }

        let accent = self.accent();

        // ── Mode Plein Écran Console / Salon (Big Picture) ──
        if self.is_big_picture {
            let mut actions = vec![];
            egui::CentralPanel::default()
                .frame(
                    egui::Frame::new()
                        .fill(th.bg)
                        .inner_margin(Margin::symmetric(36, 24)),
                )
                .show(ui, |ui| {
                    actions.extend(self.big_picture_ui(ui));
                });
            for a in actions {
                self.apply(&ctx, a);
            }
            return;
        }

        // ── Bandeau Titre Riot Games Style (Frameless) ──
        self.render_riot_title_bar(&ctx, ui);

        let compact = self.settings.compact_sidebar;
        let sidebar_width = if compact { 72.0 } else { 232.0 };

        // Compteurs dynamiques pour la barre latérale
        let hidden_set = &self.settings.hidden_games;
        let count_all = self.games.iter().filter(|g| !hidden_set.contains(&g.id)).count();
        let count_fav = self
            .games
            .iter()
            .filter(|g| !hidden_set.contains(&g.id) && self.settings.favorites.contains(&g.id))
            .count();
        let count_steam = self
            .games
            .iter()
            .filter(|g| !hidden_set.contains(&g.id) && g.source == Source::Steam)
            .count();
        let count_epic = self
            .games
            .iter()
            .filter(|g| !hidden_set.contains(&g.id) && g.source == Source::Epic)
            .count();
        let count_gog = self
            .games
            .iter()
            .filter(|g| !hidden_set.contains(&g.id) && g.source == Source::Gog)
            .count();
        let count_ubisoft = self
            .games
            .iter()
            .filter(|g| !hidden_set.contains(&g.id) && g.source == Source::Ubisoft)
            .count();
        let count_ea = self
            .games
            .iter()
            .filter(|g| !hidden_set.contains(&g.id) && g.source == Source::Ea)
            .count();
        let count_xbox = self
            .games
            .iter()
            .filter(|g| !hidden_set.contains(&g.id) && g.source == Source::Xbox)
            .count();
        let count_manual = self
            .games
            .iter()
            .filter(|g| !hidden_set.contains(&g.id) && g.source == Source::Manual && !g.id.starts_with("rom:"))
            .count();
        let count_roms = self
            .games
            .iter()
            .filter(|g| !hidden_set.contains(&g.id) && g.id.starts_with("rom:"))
            .count();
        let count_hidden = hidden_set.len();

        // ── Barre latérale repensée ──
        egui::Panel::left("sidebar")
            .exact_size(sidebar_width)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(th.panel)
                    .stroke(Stroke::new(1.0, th.panel_border))
                    .inner_margin(Margin::symmetric(12, 18)),
            )
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 6.0;

                // En-tête Logo + Bouton Replier
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(38.0, 38.0), Sense::hover());
                    let p = ui.painter();
                    p.rect_filled(rect, CornerRadius::same(10), accent);
                    let c = rect.center();
                    let st = Stroke::new(2.6, WHITE);
                    p.line_segment([c + vec2(-5.5, -6.0), c + vec2(-5.5, 2.0)], st);
                    p.line_segment([c + vec2(5.5, -6.0), c + vec2(5.5, 2.0)], st);
                    p.line_segment([c + vec2(-4.5, 5.5), c + vec2(4.5, 5.5)], st);

                    if !compact {
                        ui.add_space(6.0);
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            ui.label(RichText::new("UniLauncher").size(15.5).strong().color(WHITE));
                            ui.label(RichText::new("Bibliothèque PC").size(11.0).color(DIM));
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if icon_square_button(ui, Icon::SidebarToggle, "Réduire le menu", false, th)
                            {
                                self.settings.compact_sidebar = true;
                                self.settings.save();
                            }
                        });
                    }
                });

                if compact {
                    ui.vertical_centered(|ui| {
                        if icon_square_button(ui, Icon::SidebarToggle, "Étendre le menu", false, th) {
                            self.settings.compact_sidebar = false;
                            self.settings.save();
                        }
                    });
                }

                ui.add_space(12.0);

                let mut nav_items = vec![
                    (Tab::All, Icon::Library, accent, "Tous les jeux", Some(count_all)),
                    (Tab::Favorites, Icon::Star, GOLD, "Favoris", Some(count_fav)),
                ];

                if count_steam > 0 || self.tab == Tab::Steam {
                    nav_items.push((Tab::Steam, Icon::Steam, STEAM_C, "Steam", Some(count_steam)));
                }
                if count_epic > 0 || self.tab == Tab::Epic {
                    nav_items.push((Tab::Epic, Icon::Epic, EPIC_C, "Epic Games", Some(count_epic)));
                }
                if count_gog > 0 || self.tab == Tab::Gog {
                    nav_items.push((Tab::Gog, Icon::Gog, GOG_C, "GOG Galaxy", Some(count_gog)));
                }
                if count_ubisoft > 0 || self.tab == Tab::Ubisoft {
                    nav_items.push((Tab::Ubisoft, Icon::Ubisoft, UBI_C, "Ubisoft Connect", Some(count_ubisoft)));
                }
                if count_ea > 0 || self.tab == Tab::Ea {
                    nav_items.push((Tab::Ea, Icon::Ea, EA_C, "EA App", Some(count_ea)));
                }
                if count_xbox > 0 || self.tab == Tab::Xbox {
                    nav_items.push((Tab::Xbox, Icon::Xbox, XBOX_C, "Xbox", Some(count_xbox)));
                }
                if count_manual > 0 || self.tab == Tab::Manual {
                    nav_items.push((Tab::Manual, Icon::Manual, accent, "Jeux manuels", Some(count_manual)));
                }

                if count_roms > 0 || self.tab == Tab::Emulators {
                    nav_items.push((Tab::Emulators, Icon::Gamepad, STEAM_C, "Émulateurs & ROMs", Some(count_roms)));
                } else {
                    nav_items.push((Tab::Emulators, Icon::Gamepad, STEAM_C, "Émulateurs & ROMs", None));
                }

                nav_items.push((Tab::Deals, Icon::Gift, GREEN, "Bons Plans Gratuits", if self.deals.is_empty() { None } else { Some(self.deals.len()) }));
                nav_items.push((Tab::Stats, Icon::Chart, Color32::from_rgb(56, 189, 248), "Statistiques", None));

                for (tab, icon, col, label, count) in nav_items {
                    if sidebar_item(
                        ui,
                        Id::unique(("nav", tab)),
                        icon,
                        col,
                        self.tab == tab,
                        label,
                        count,
                        compact,
                        th,
                    ) {
                        self.tab = tab;
                    }
                }

                if (count_hidden > 0 || self.tab == Tab::Hidden)
                    && sidebar_item(
                        ui,
                        Id::unique("nav_hidden"),
                        Icon::EyeOff,
                        DIM,
                        self.tab == Tab::Hidden,
                        "Masqués",
                        Some(count_hidden),
                        compact,
                        th,
                    )
                {
                    self.tab = Tab::Hidden;
                }

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                if sidebar_item(
                    ui,
                    Id::unique("nav_big_picture"),
                    Icon::Gamepad,
                    accent,
                    self.is_big_picture,
                    "Mode Salon (F10)",
                    None,
                    compact,
                    th,
                ) {
                    self.is_big_picture = true;
                }

                if sidebar_item(
                    ui,
                    Id::unique("nav_fullscreen"),
                    Icon::Fullscreen,
                    DIM,
                    self.is_fullscreen,
                    if self.is_fullscreen {
                        "Mode fenêtré (F11)"
                    } else {
                        "Plein écran (F11)"
                    },
                    None,
                    compact,
                    th,
                ) {
                    self.is_fullscreen = !self.is_fullscreen;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.is_fullscreen));
                }

                if sidebar_item(
                    ui,
                    Id::unique("nav_settings"),
                    Icon::Gear,
                    accent,
                    self.tab == Tab::Settings,
                    "Paramètres",
                    None,
                    compact,
                    th,
                ) {
                    self.tab = Tab::Settings;
                }

                // ── Carte de Statut en bas de la barre latérale (Jeu actif & Discord RPC) ──
                let footer_h = if compact { 52.0 } else { 78.0 };
                let rem = (ui.available_height() - footer_h).max(8.0);
                ui.add_space(rem);

                egui::Frame::new()
                    .fill(th.card)
                    .stroke(Stroke::new(1.0, th.card_border))
                    .corner_radius(10.0)
                    .inner_margin(Margin::symmetric(10, 10))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        let (status_color, status_text, status_sub) = if let Some((ref name, started)) = self.active_game {
                            let mins = started.elapsed().as_secs() / 60;
                            (GREEN, format!("EN JEU: {}", trunc(name, 16)), format!("Session : {mins} min"))
                        } else {
                            match &self.rpc_status {
                                RpcStatus::Connected => (
                                    DISCORD_C,
                                    "Discord : Connecté".to_string(),
                                    "Rich Presence actif".to_string(),
                                ),
                                RpcStatus::WaitingDiscord => (
                                    GOLD,
                                    "Discord : En attente...".to_string(),
                                    "Lancez Discord pour lier".to_string(),
                                ),
                                RpcStatus::Connecting => (
                                    STEAM_C,
                                    "Discord : Connexion...".to_string(),
                                    "Recherche du socket".to_string(),
                                ),
                                RpcStatus::Error(_) => (
                                    RED,
                                    "Discord : Erreur ID".to_string(),
                                    "Vérifiez l'ID client".to_string(),
                                ),
                                RpcStatus::Disabled => (
                                    DIM,
                                    "Discord RPC désactivé".to_string(),
                                    "Prêt à lancer un titre".to_string(),
                                ),
                            }
                        };

                        if compact {
                            let (r, _) = ui.allocate_exact_size(vec2(24.0, 24.0), Sense::hover());
                            ui.painter().circle_filled(r.center(), 5.0, status_color);
                        } else {
                            ui.horizontal(|ui| {
                                let (r, _) =
                                    ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                                ui.painter().circle_filled(r.center(), 4.0, status_color);
                                ui.label(RichText::new(status_text).size(11.5).strong().color(WHITE));
                            });
                            ui.add_space(2.0);
                            ui.label(
                                RichText::new(status_sub)
                                    .size(11.0)
                                    .color(DIM),
                            );
                        }
                    });
            });

        let mut actions = vec![];

        // ── Contenu Central ──
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(th.bg)
                    .inner_margin(Margin::symmetric(30, 24)),
            )
            .show(ui, |ui| {
                if self.tab == Tab::Settings {
                    self.settings_ui(ui);
                } else if self.tab == Tab::Stats {
                    self.stats_ui(ui);
                } else if self.tab == Tab::Deals {
                    self.deals_ui(ui);
                } else if self.tab == Tab::Emulators {
                    actions.extend(self.emulators_ui(ui));
                } else {
                    actions.extend(self.library_ui(ui));
                }
            });

        // ── Modale de Propriétés Réactive (Style Riot / Steam) ──
        if self.selected_game_id.is_some() {
            actions.extend(self.properties_modal_ui(&ctx));
        }

        for a in actions {
            self.apply(&ctx, a);
        }

        // ── Spotlight (Recherche flottante instantanée Alt + Espace) ──
        if let Some(a) = self.spotlight_ui(&ctx) {
            self.apply(&ctx, a);
        }

        // ── Notification Toast sobre ──
        if self
            .status
            .as_ref()
            .map(|(_, t)| t.elapsed() > Duration::from_secs(3))
            .unwrap_or(false)
        {
            self.status = None;
        }
        if let Some((msg, t)) = &self.status {
            let age = t.elapsed().as_secs_f32();
            let alpha = if age < 0.15 {
                age / 0.15
            } else if age > 2.6 {
                ((3.0 - age) / 0.4).clamp(0.0, 1.0)
            } else {
                1.0
            };

            egui::Area::new(Id::unique("toast_area"))
                .anchor(Align2::RIGHT_BOTTOM, vec2(-24.0, -24.0))
                .order(egui::Order::Foreground)
                .show(&ctx, |ui| {
                    egui::Frame::new()
                        .fill(th.card_hover.gamma_multiply(alpha))
                        .corner_radius(10.0)
                        .stroke(Stroke::new(1.0, accent.gamma_multiply(alpha * 0.8)))
                        .inner_margin(Margin::symmetric(16, 10))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(msg)
                                    .size(13.0)
                                    .color(WHITE.gamma_multiply(alpha)),
                            );
                        });
                });
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }
}
