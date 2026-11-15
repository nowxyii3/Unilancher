use crate::config::{Game, Settings, Source};
use crate::discord::{spawn_discord_rpc_worker, RpcEvent, RpcStatus};
use crate::process::{launch_game, open_uri};
use crate::scan;
use crate::theme::{
    apply_style, get_theme, ThemeColors, ACCENTS, BNET_C, DISCORD_C, EA_C, EPIC_C, GOG_C, GOLD,
    GREEN, RED, STEAM_C, SUCCESS, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_TERTIARY, UBI_C, WHITE, XBOX_C,
};
use crate::types::{Action, SortMode, Tab, ViewMode};
use crate::ui::components::{icon_square_button, sidebar_item, trunc, Icon};
use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, ColorImage, CornerRadius, FontId, Id, Key, Layout,
    Margin, Rect, RichText, Sense, Stroke, TextureHandle, TextureOptions,
};
use std::{
    collections::{HashMap, HashSet},
    io::Read,
    path::PathBuf,
    sync::mpsc,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};



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

    // Chargement asynchrone des bannières hero HD
    pub(crate) banners: HashMap<String, Option<TextureHandle>>,
    pub(crate) loading_banners: HashSet<String>,
    pub(crate) banner_tx: mpsc::Sender<(String, ColorImage)>,
    pub(crate) banner_rx: mpsc::Receiver<(String, ColorImage)>,

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
    #[allow(dead_code)]
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
    pub(crate) deals_search: String,
    pub(crate) deals_active_tab: crate::deals::DealsTab,
    pub(crate) deals_platform_filter: Option<crate::deals::DealPlatform>,
    pub(crate) deals_sort_mode: crate::deals::DealsSortMode,
    pub(crate) selected_deal_modal: Option<crate::deals::FreeDeal>,
    pub(crate) hero_featured_index: usize,
    pub(crate) last_deals_sync: Option<u64>,
    pub(crate) deal_textures: HashMap<String, Option<TextureHandle>>,
    pub(crate) loading_deal_textures: HashSet<String>,
    pub(crate) deal_tex_tx: mpsc::Sender<(String, ColorImage)>,
    pub(crate) deal_tex_rx: mpsc::Receiver<(String, ColorImage)>,
    pub(crate) centered_once: bool,

    // Progression du scan multi-plateformes
    pub(crate) scan_rx: mpsc::Receiver<crate::scan::ScanProgress>,
    pub(crate) scan_tx: mpsc::Sender<crate::scan::ScanProgress>,
    pub(crate) scan_progress: Option<crate::scan::ScanProgress>,
    pub(crate) is_scanning: bool,

    // Succès & Trophées
    pub(crate) cached_achievements: HashMap<String, Option<crate::achievements::GameAchievements>>,
    pub(crate) loading_achievements: HashSet<String>,
    pub(crate) ach_tx: mpsc::Sender<(String, Option<crate::achievements::GameAchievements>)>,
    pub(crate) ach_rx: mpsc::Receiver<(String, Option<crate::achievements::GameAchievements>)>,
    pub(crate) achievements_filter: usize,

    // Détails du jeu (description, date de sortie)
    pub(crate) details_tx: mpsc::Sender<(String, Option<(String, String)>)>,
    pub(crate) details_rx: mpsc::Receiver<(String, Option<(String, String)>)>,
    pub(crate) loading_details: HashSet<String>,

    // Captures d'écran & Lightbox
    pub(crate) cached_screenshots: HashMap<String, Vec<PathBuf>>,
    pub(crate) selected_screenshot: Option<PathBuf>,

    // Performance & Overlay en jeu
    pub(crate) perf_monitor: crate::performance::PerformanceMonitor,

    // Édition d'informations d'un jeu
    pub(crate) edit_info_open: bool,
    pub(crate) edit_name_input: String,
    pub(crate) edit_desc_input: String,
    pub(crate) edit_date_input: String,

    // Gestionnaire de mise à jour automatique
    pub(crate) update_mgr: crate::update::UpdateManager,
}

impl Launcher {
    pub fn new(cc: &eframe::CreationContext<'_>, settings: Settings) -> Self {
        let (playtime_tx, playtime_rx) = mpsc::channel();
        let (tex_tx, tex_rx) = mpsc::channel();
        let (rpc_tx, rpc_rx) = mpsc::channel();
        let (rpc_status_tx, rpc_status_rx) = mpsc::channel();
        let (deals_tx, deals_rx) = mpsc::channel();
        let (deal_tex_tx, deal_tex_rx) = mpsc::channel();
        let (scan_tx, scan_rx) = mpsc::channel();
        let (ach_tx, ach_rx) = mpsc::channel();
        let (details_tx, details_rx) = mpsc::channel();
        let (banner_tx, banner_rx) = mpsc::channel();
        let perf_monitor = crate::performance::PerformanceMonitor::new();
        let initial_deals = crate::deals::DealsTracker::load_cached_or_default();
        let initial_sync = crate::deals::DealsTracker::load_cache_from_disk().map(|(ts, _)| ts);

        spawn_discord_rpc_worker(
            rpc_rx,
            rpc_status_tx,
            settings.discord_rpc,
            settings.discord_client_id.clone(),
        );

        let is_fullscreen = settings.start_fullscreen;
        let update_channel = settings.update_channel;
        let mut l = Self {
            sort: settings.sort_mode,
            view_mode: settings.view_mode,
            settings,
            games: vec![],
            tab: if std::env::args().any(|a| a == "deals" || a == "--tab=deals" || a == "--deals") {
                Tab::Deals
            } else {
                Tab::All
            },
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
            banners: HashMap::new(),
            loading_banners: HashSet::new(),
            banner_tx,
            banner_rx,
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
            deals: initial_deals,
            loading_deals: false,
            deals_tx,
            deals_rx,
            deals_search: String::new(),
            deals_active_tab: crate::deals::DealsTab::All,
            deals_platform_filter: None,
            deals_sort_mode: crate::deals::DealsSortMode::Featured,
            selected_deal_modal: None,
            hero_featured_index: 0,
            last_deals_sync: initial_sync,
            deal_textures: HashMap::new(),
            loading_deal_textures: HashSet::new(),
            deal_tex_tx,
            deal_tex_rx,
            centered_once: false,
            scan_rx,
            scan_tx,
            scan_progress: None,
            is_scanning: false,
            cached_achievements: HashMap::new(),
            loading_achievements: HashSet::new(),
            ach_tx,
            ach_rx,
            achievements_filter: 0,
            details_tx,
            details_rx,
            loading_details: HashSet::new(),
            cached_screenshots: HashMap::new(),
            selected_screenshot: None,
            perf_monitor,
            edit_info_open: false,
            edit_name_input: String::new(),
            edit_desc_input: String::new(),
            edit_date_input: String::new(),
            update_mgr: {
                let mut m = crate::update::UpdateManager::new();
                m.check_for_updates(update_channel);
                m
            },
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

    pub fn start_scan(&mut self) {
        if self.is_scanning {
            return;
        }
        self.is_scanning = true;
        self.scan_progress = Some(crate::scan::ScanProgress {
            stage: "Démarrage de la détection multi-plateformes...".into(),
            games_found: self.games.len(),
            percent: 0.05,
            finished: false,
            games: None,
        });
        crate::scan::start_progressive_scan(self.settings.clone(), self.scan_tx.clone());
    }

    pub fn refresh(&mut self) {
        let mut g = self.settings.manual_games.clone();
        for mg in &mut g {
            if mg.cover_path.is_none() {
                if let Some(dir) = &mg.install_dir {
                    mg.cover_path = scan::find_folder_cover(dir);
                }
            }
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
        self.start_scan();
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
            let deals = crate::deals::DealsTracker::fetch_all_deals();
            let _ = tx.send(deals);
        });
    }

    /// Charge la jaquette/bannière d'un bon plan de manière asynchrone avec cache disque
    pub fn get_deal_texture(
        &mut self,
        ctx: &egui::Context,
        deal: &crate::deals::FreeDeal,
    ) -> Option<TextureHandle> {
        if let Some(cached) = self.deal_textures.get(&deal.id) {
            return cached.clone();
        }
        if self.loading_deal_textures.contains(&deal.id) {
            return None;
        }
        let url = deal.image_url.clone()?;
        self.loading_deal_textures.insert(deal.id.clone());

        let tx = self.deal_tex_tx.clone();
        let id = deal.id.clone();
        let ctx_clone = ctx.clone();
        let cache_dir = Settings::dir().join("deals_images");
        let _ = std::fs::create_dir_all(&cache_dir);
        let safe_name = format!("{}.bin", id.replace(|c: char| !c.is_alphanumeric(), "_"));
        let cache_file = cache_dir.join(safe_name);

        thread::spawn(move || {
            let bytes = if cache_file.exists() {
                std::fs::read(&cache_file).ok()
            } else if let Ok(resp) = ureq::get(&url).timeout(Duration::from_secs(6)).call() {
                let mut reader = resp.into_reader();
                let mut buffer = Vec::new();
                if std::io::copy(&mut reader, &mut buffer).is_ok() {
                    let _ = std::fs::write(&cache_file, &buffer);
                    Some(buffer)
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(raw) = bytes {
                if let Ok(img) = image::load_from_memory(&raw) {
                    let resized = img.thumbnail(640, 360);
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

        None
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

    pub fn get_banner_texture(&mut self, ctx: &egui::Context, g: &Game) -> Option<TextureHandle> {
        if let Some(cached) = self.banners.get(&g.id) {
            return cached.clone();
        }
        if self.loading_banners.contains(&g.id) {
            return None;
        }

        let custom_banner = self.settings.banner_overrides.get(&g.id).cloned();
        if let Some(p) = custom_banner {
            self.loading_banners.insert(g.id.clone());
            let tx = self.banner_tx.clone();
            let id = g.id.clone();
            let ctx_clone = ctx.clone();
            thread::spawn(move || {
                if let Ok(bytes) = std::fs::read(&p) {
                    if let Ok(img) = image::load_from_memory(&bytes) {
                        let resized = img.thumbnail(1280, 500);
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
            return None;
        }

        if let Some(appid) = g.id.strip_prefix("steam:") {
            self.loading_banners.insert(g.id.clone());
            let tx = self.banner_tx.clone();
            let id = g.id.clone();
            let appid_s = appid.to_string();
            let ctx_clone = ctx.clone();
            let steam_path = crate::scan::steam_root(&self.settings);

            thread::spawn(move || {
                let mut banner_bytes: Option<Vec<u8>> = None;
                if let Some(ref s_root) = steam_path {
                    let candidates = [
                        s_root.join("appcache").join("librarycache").join(format!("{appid_s}_library_hero.jpg")),
                        s_root.join("appcache").join("librarycache").join(format!("{appid_s}_header.jpg")),
                        s_root.join("appcache").join("librarycache").join(&appid_s).join("library_hero.jpg"),
                        s_root.join("appcache").join("librarycache").join(&appid_s).join("header.jpg"),
                    ];
                    for c in candidates {
                        if c.exists() {
                            if let Ok(b) = std::fs::read(&c) {
                                banner_bytes = Some(b);
                                break;
                            }
                        }
                    }
                }

                if banner_bytes.is_none() {
                    let cache_dir = Settings::dir().join("banners_cache");
                    let _ = std::fs::create_dir_all(&cache_dir);
                    let cached_file = cache_dir.join(format!("{appid_s}_hero.jpg"));
                    if cached_file.exists() {
                        banner_bytes = std::fs::read(&cached_file).ok();
                    } else {
                        let cdn_url = format!("https://cdn.akamai.steamstatic.com/steam/apps/{appid_s}/library_hero.jpg");
                        if let Ok(resp) = ureq::get(&cdn_url).timeout(Duration::from_secs(4)).call() {
                            let mut buf = Vec::new();
                            if resp.into_reader().read_to_end(&mut buf).is_ok() && buf.len() > 1000 {
                                let _ = std::fs::write(&cached_file, &buf);
                                banner_bytes = Some(buf);
                            }
                        }
                    }
                }

                if let Some(bytes) = banner_bytes {
                    if let Ok(img) = image::load_from_memory(&bytes) {
                        let resized = img.thumbnail(1280, 500);
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
            return None;
        }

        self.banners.insert(g.id.clone(), None);
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
                    Tab::Battlenet => g.source == Source::Battlenet,
                    Tab::Xbox => g.source == Source::Xbox,
                    Tab::Manual => g.source == Source::Manual && !g.id.starts_with("rom:"),
                    Tab::Emulators => g.id.starts_with("rom:"),
                    Tab::Deals | Tab::Hidden | Tab::Settings | Tab::Stats => false,
                };
                let in_status_filter = match self.settings.status_filter {
                    crate::types::LibraryStatusFilter::All => match self.settings.quick_filter_mode {
                        1 => favs.contains(&g.id),
                        2 => {
                            self.total_playtime_mins(g) > 0
                                || self.settings.last_played_dates.contains_key(&g.id)
                        }
                        3 => {
                            self.total_playtime_mins(g) == 0
                                && !self.settings.last_played_dates.contains_key(&g.id)
                        }
                        _ => true,
                    },
                    crate::types::LibraryStatusFilter::Favorites => favs.contains(&g.id),
                    crate::types::LibraryStatusFilter::Recent => {
                        self.total_playtime_mins(g) > 0
                            || self.settings.last_played_dates.contains_key(&g.id)
                    }
                    crate::types::LibraryStatusFilter::NeverPlayed => {
                        self.total_playtime_mins(g) == 0
                            && !self.settings.last_played_dates.contains_key(&g.id)
                    }
                    crate::types::LibraryStatusFilter::Installed => !g.is_broken(),
                    crate::types::LibraryStatusFilter::NotInstalled => g.is_broken(),
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
                in_tab && in_status_filter && matches_query
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
            SortMode::NameDesc => {
                filtered.sort_by(|&a, &b| {
                    self.games[b]
                        .name
                        .to_lowercase()
                        .cmp(&self.games[a].name.to_lowercase())
                });
            }
            SortMode::LastPlayed => {
                filtered.sort_by(|&a, &b| {
                    let ta = self
                        .settings
                        .last_played_dates
                        .get(&self.games[a].id)
                        .copied()
                        .unwrap_or(0);
                    let tb = self
                        .settings
                        .last_played_dates
                        .get(&self.games[b].id)
                        .copied()
                        .unwrap_or(0);
                    tb.cmp(&ta).then_with(|| {
                        let pa = self.total_playtime_mins(&self.games[a]);
                        let pb = self.total_playtime_mins(&self.games[b]);
                        pb.cmp(&pa)
                    })
                });
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
                    let count = self.settings.session_counts.entry(g.id.clone()).or_insert(0);
                    *count += 1;
                    let now_sec = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    self.settings.last_played_dates.insert(g.id.clone(), now_sec);
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
            Action::CloseDetails => {
                self.selected_game_id = None;
            }
            Action::PickBanner(id) => {
                if let Some(p) = rfd::FileDialog::new()
                    .set_title("Choisir une bannière HD (PNG/JPG/WEBP)")
                    .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                    .pick_file()
                {
                    self.settings.banner_overrides.insert(id.clone(), p);
                    self.banners.remove(&id);
                    self.loading_banners.remove(&id);
                    self.settings.save();
                    self.toast("Bannière personnalisée appliquée");
                }
            }
            Action::ClearBanner(id) => {
                self.settings.banner_overrides.remove(&id);
                self.banners.remove(&id);
                self.loading_banners.remove(&id);
                self.settings.save();
                self.toast("Bannière réinitialisée");
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
            .exact_size(36.0)
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

                    // ── Gauche : Logo + Titre + Badge version ──
                    ui.add_space(12.0);
                    let (icon_r, _) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
                    let p = ui.painter();
                    p.rect_filled(icon_r, CornerRadius::same(6), accent);
                    let c = icon_r.center();
                    let st = Stroke::new(1.6, WHITE);
                    p.line_segment([c + vec2(-3.5, -4.0), c + vec2(-3.5, 1.5)], st);
                    p.line_segment([c + vec2(3.5, -4.0), c + vec2(3.5, 1.5)], st);
                    p.line_segment([c + vec2(-3.0, 3.5), c + vec2(3.0, 3.5)], st);

                    ui.add_space(9.0);
                    ui.label(RichText::new("UniLauncher").size(13.0).strong().color(TEXT_PRIMARY));
                    ui.add_space(6.0);

                    // Badge "v1.0.0"
                    let ver_text = format!("v{}", crate::update::CURRENT_VERSION);
                    let (ver_r, _) = ui.allocate_exact_size(vec2(46.0, 16.0), Sense::hover());
                    ui.painter().rect_filled(ver_r, CornerRadius::same(4), th.surface_2);
                    ui.painter().text(
                        ver_r.center(),
                        Align2::CENTER_CENTER,
                        ver_text,
                        FontId::proportional(9.5),
                        TEXT_TERTIARY,
                    );

                    // Badge notification si nouvelle mise à jour disponible
                    if let Ok(st) = self.update_mgr.status.lock() {
                        if let crate::update::UpdateStatus::UpdateAvailable(ref u) = *st {
                            ui.add_space(6.0);
                            let (badge_r, badge_resp) = ui.allocate_exact_size(vec2(108.0, 18.0), Sense::click());
                            let hov = badge_resp.hovered();
                            ui.painter().rect_filled(badge_r, CornerRadius::same(4), if hov { SUCCESS.gamma_multiply(0.28) } else { SUCCESS.gamma_multiply(0.18) });
                            ui.painter().rect_stroke(badge_r, CornerRadius::same(4), Stroke::new(1.0, SUCCESS.gamma_multiply(0.6)), egui::StrokeKind::Inside);
                            ui.painter().text(
                                badge_r.center(),
                                Align2::CENTER_CENTER,
                                format!("★ v{} DISPONIBLE", u.version),
                                FontId::proportional(9.0),
                                WHITE,
                            );
                            if badge_resp.clicked() {
                                self.tab = crate::types::Tab::Settings;
                            }
                        }
                    }

                    // ── Centre : Raccourci recherche centré (Raycast/Arc style) ──
                    let total_w = ui.available_width();
                    if total_w > 400.0 {
                        let pill_w = 250.0;
                        let offset = (total_w - pill_w - 140.0) / 2.0;
                        if offset > 10.0 {
                            ui.add_space(offset);
                        }

                        let (search_pill, pill_resp) = ui.allocate_exact_size(vec2(pill_w, 24.0), Sense::click());
                        let hov = pill_resp.hovered();

                        ui.painter().rect_filled(
                            search_pill,
                            CornerRadius::same(12),
                            if hov { th.surface_3 } else { th.surface_2 },
                        );
                        ui.painter().rect_stroke(
                            search_pill,
                            CornerRadius::same(12),
                            Stroke::new(1.0, if hov { accent.gamma_multiply(0.65) } else { th.border_subtle }),
                            egui::StrokeKind::Inside,
                        );

                        // Texte de recherche
                        let p = ui.painter();
                        p.text(
                            pos2(search_pill.left() + 10.0, search_pill.center().y),
                            Align2::LEFT_CENTER,
                            "🔍  Rechercher...",
                            FontId::proportional(11.5),
                            if hov { TEXT_PRIMARY } else { TEXT_SECONDARY },
                        );

                        // Badge raccourci clavier "Alt + Espace"
                        let kbd_rect = Rect::from_center_size(
                            pos2(search_pill.right() - 44.0, search_pill.center().y),
                            vec2(72.0, 16.0),
                        );
                        p.rect_filled(kbd_rect, CornerRadius::same(4), th.surface_1);
                        p.rect_stroke(kbd_rect, CornerRadius::same(4), Stroke::new(1.0, th.border_subtle), egui::StrokeKind::Inside);
                        p.text(
                            kbd_rect.center(),
                            Align2::CENTER_CENTER,
                            "Alt + Espace",
                            FontId::proportional(9.5),
                            TEXT_TERTIARY,
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
                        let (close_rect, close_resp) = ui.allocate_exact_size(vec2(46.0, 36.0), Sense::click());
                        let close_hov = close_resp.hovered();
                        if close_hov {
                            ui.painter().rect_filled(close_rect, CornerRadius::ZERO, Color32::from_rgb(232, 17, 35));
                        }
                        let close_col = if close_hov { WHITE } else { TEXT_SECONDARY };
                        let c = close_rect.center();
                        let st = Stroke::new(1.2, close_col);
                        ui.painter().line_segment([c + vec2(-4.0, -4.0), c + vec2(4.0, 4.0)], st);
                        ui.painter().line_segment([c + vec2(4.0, -4.0), c + vec2(-4.0, 4.0)], st);
                        if close_resp.clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }

                        // 2. Bouton Maximiser / Restaurer (□ / ❐)
                        let is_max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                        let (max_rect, max_resp) = ui.allocate_exact_size(vec2(40.0, 36.0), Sense::click());
                        let max_hov = max_resp.hovered();
                        if max_hov {
                            ui.painter().rect_filled(max_rect, CornerRadius::ZERO, th.surface_2);
                        }
                        let max_col = if max_hov { TEXT_PRIMARY } else { TEXT_SECONDARY };
                        let c = max_rect.center();
                        if is_max {
                            ui.painter().rect_stroke(
                                Rect::from_center_size(c + vec2(-2.0, 2.0), vec2(7.5, 7.5)),
                                CornerRadius::ZERO,
                                Stroke::new(1.1, max_col),
                                egui::StrokeKind::Inside,
                            );
                            ui.painter().rect_stroke(
                                Rect::from_center_size(c + vec2(2.0, -2.0), vec2(7.5, 7.5)),
                                CornerRadius::ZERO,
                                Stroke::new(1.1, max_col),
                                egui::StrokeKind::Inside,
                            );
                        } else {
                            ui.painter().rect_stroke(
                                Rect::from_center_size(c, vec2(9.0, 9.0)),
                                CornerRadius::ZERO,
                                Stroke::new(1.1, max_col),
                                egui::StrokeKind::Inside,
                            );
                        }
                        if max_resp.clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_max));
                        }

                        // 3. Bouton Minimiser (—)
                        let (min_rect, min_resp) = ui.allocate_exact_size(vec2(40.0, 36.0), Sense::click());
                        let min_hov = min_resp.hovered();
                        if min_hov {
                            ui.painter().rect_filled(min_rect, CornerRadius::ZERO, th.surface_2);
                        }
                        let min_col = if min_hov { TEXT_PRIMARY } else { TEXT_SECONDARY };
                        let c = min_rect.center();
                        ui.painter().line_segment(
                            [c + vec2(-4.5, 0.0), c + vec2(4.5, 0.0)],
                            Stroke::new(1.2, min_col),
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

        // -1. Traitement des événements de mise à jour
        self.update_mgr.update();

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

        // 1-banner. Intégration des bannières hero HD chargées en arrière-plan
        while let Ok((id, color_img)) = self.banner_rx.try_recv() {
            let tex = ctx.load_texture(format!("banner_{id}"), color_img, TextureOptions::LINEAR);
            self.banners.insert(id.clone(), Some(tex));
            self.loading_banners.remove(&id);
        }

        // 1-deals. Intégration des jaquettes et bannières de bons plans
        while let Ok((id, color_img)) = self.deal_tex_rx.try_recv() {
            let tex = ctx.load_texture(format!("deal_{id}"), color_img, TextureOptions::LINEAR);
            self.deal_textures.insert(id.clone(), Some(tex));
            self.loading_deal_textures.remove(&id);
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
                self.last_deals_sync = Some(crate::deals::current_unix_timestamp() as u64);
                if self.settings.deals_notifications {
                    let expiring_favs = self
                        .deals
                        .iter()
                        .filter(|d| self.settings.favorite_deals.contains(&d.id) && d.expires_soon(48))
                        .count();
                    if expiring_favs > 0 {
                        self.toast(format!("⚠️ {} offre(s) dans vos favoris expirent bientôt !", expiring_favs));
                    }
                }
            }
        }

        // 1d. Réception de la progression du scan multi-plateformes en direct
        while let Ok(prog) = self.scan_rx.try_recv() {
            if prog.finished {
                self.is_scanning = false;
                if let Some(mut games) = prog.games {
                    for mg in &self.settings.manual_games {
                        if !games.iter().any(|g| g.id == mg.id) {
                            games.push(mg.clone());
                        }
                    }
                    let emus = crate::emulators::detect_installed_emulators();
                    for folder in &self.settings.custom_scan_folders {
                        let roms = crate::emulators::scan_roms_folder(folder, &emus);
                        for r in roms {
                            if !games.iter().any(|g| g.id == r.id) {
                                games.push(r);
                            }
                        }
                    }
                    games.sort_by_key(|g| g.name.to_lowercase());
                    self.games = games;
                }
                self.toast(format!("Bibliothèque synchronisée : {} jeux détectés", self.games.len()));
                self.scan_progress = None;
            } else {
                self.scan_progress = Some(prog);
            }
        }

        // 2. Réception de la fin de session d'un jeu
        while let Ok((game_id, mins_added)) = self.playtime_rx.try_recv() {
            self.active_game = None;
            self.update_rpc_menu();
            if self.settings.performance_mode {
                crate::process::set_balanced_power_scheme();
            }
            if mins_added > 0 {
                let entry = self.settings.tracked_playtime.entry(game_id.clone()).or_insert(0);
                *entry += mins_added;
                let now_sec = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                self.settings.last_played_dates.insert(game_id, now_sec);
                self.settings.save();
                self.toast(format!("Session terminée (+{mins_added} min comptabilisées)"));
            }
        }

        // Réception des succès en arrière-plan
        while let Ok((id, achs)) = self.ach_rx.try_recv() {
            self.loading_achievements.remove(&id);
            self.cached_achievements.insert(id, achs);
        }

        // Réception des métadonnées du store (description, date de sortie)
        while let Ok((id, details)) = self.details_rx.try_recv() {
            self.loading_details.remove(&id);
            if let Some((desc, date)) = details {
                if !desc.is_empty() && !self.settings.game_descriptions.contains_key(&id) {
                    self.settings.game_descriptions.insert(id.clone(), desc);
                }
                if !date.is_empty() && !self.settings.game_release_dates.contains_key(&id) {
                    self.settings.game_release_dates.insert(id.clone(), date);
                }
                self.settings.save();
            }
        }

        // 3. Raccourcis clavier globaux (F11 = Plein écran, F10 = Big Picture, Ctrl+F = Recherche, Échap = Fermer tiroir/recherche)
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
            if i.key_pressed(Key::F9) {
                self.settings.overlay.enabled = !self.settings.overlay.enabled;
                self.settings.save();
                let state = if self.settings.overlay.enabled { "activé" } else { "désactivé" };
                self.toast(format!("Overlay de performance {state} (F9)"));
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
        let sidebar_width = if compact { 68.0 } else { 236.0 };

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
        let count_bnet = self
            .games
            .iter()
            .filter(|g| !hidden_set.contains(&g.id) && g.source == Source::Battlenet)
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

        // ── Barre latérale repensée avec hiérarchie claire ──
        egui::Panel::left("sidebar")
            .exact_size(sidebar_width)
            .resizable(false)
            .frame(
                egui::Frame::new()
                    .fill(th.surface_1)
                    .stroke(Stroke::new(1.0, th.border_subtle))
                    .inner_margin(Margin::symmetric(10, 14)),
            )
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;

                // ── En-tête Logo + Bouton Replier ──
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::hover());
                    let p = ui.painter();
                    p.rect_filled(rect, CornerRadius::same(8), accent);
                    let c = rect.center();
                    let st = Stroke::new(2.2, WHITE);
                    p.line_segment([c + vec2(-5.0, -5.0), c + vec2(-5.0, 2.0)], st);
                    p.line_segment([c + vec2(5.0, -5.0), c + vec2(5.0, 2.0)], st);
                    p.line_segment([c + vec2(-4.0, 4.5), c + vec2(4.0, 4.5)], st);

                    if !compact {
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            ui.label(RichText::new("UniLauncher").size(14.0).strong().color(TEXT_PRIMARY));
                            ui.label(RichText::new("Bibliothèque PC").size(10.5).color(TEXT_TERTIARY));
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
                    ui.add_space(2.0);
                    ui.vertical_centered(|ui| {
                        if icon_square_button(ui, Icon::SidebarToggle, "Étendre le menu", false, th) {
                            self.settings.compact_sidebar = false;
                            self.settings.save();
                        }
                    });
                }

                ui.add_space(10.0);

                // Helper pour les séparateurs / en-têtes de catégorie
                let render_cat_header = |ui: &mut egui::Ui, title: &str| {
                    if !compact {
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(title)
                                .size(9.5)
                                .strong()
                                .color(TEXT_TERTIARY),
                        );
                        ui.add_space(2.0);
                    } else {
                        ui.add_space(4.0);
                        let (sep_r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
                        ui.painter().rect_filled(sep_r, CornerRadius::ZERO, th.border_subtle);
                        ui.add_space(4.0);
                    }
                };

                // ── GROUPE 1 : BIBLIOTHÈQUE ──
                render_cat_header(ui, "BIBLIOTHÈQUE");

                let library_items = [
                    (Tab::All, Icon::Library, accent, "Tous les jeux", Some(count_all)),
                    (Tab::Favorites, Icon::Star, GOLD, "Favoris", Some(count_fav)),
                ];
                for (tab, icon, col, label, count) in library_items {
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

                // ── GROUPE 2 : PLATEFORMES ──
                let mut platform_items = vec![];
                if count_steam > 0 || self.tab == Tab::Steam {
                    platform_items.push((Tab::Steam, Icon::Steam, STEAM_C, "Steam", Some(count_steam)));
                }
                if count_epic > 0 || self.tab == Tab::Epic {
                    platform_items.push((Tab::Epic, Icon::Epic, EPIC_C, "Epic Games", Some(count_epic)));
                }
                if count_gog > 0 || self.tab == Tab::Gog {
                    platform_items.push((Tab::Gog, Icon::Gog, GOG_C, "GOG Galaxy", Some(count_gog)));
                }
                if count_ubisoft > 0 || self.tab == Tab::Ubisoft {
                    platform_items.push((Tab::Ubisoft, Icon::Ubisoft, UBI_C, "Ubisoft Connect", Some(count_ubisoft)));
                }
                if count_ea > 0 || self.tab == Tab::Ea {
                    platform_items.push((Tab::Ea, Icon::Ea, EA_C, "EA App", Some(count_ea)));
                }
                if count_bnet > 0 || self.tab == Tab::Battlenet {
                    platform_items.push((Tab::Battlenet, Icon::Battlenet, BNET_C, "Battle.net", Some(count_bnet)));
                }
                if count_xbox > 0 || self.tab == Tab::Xbox {
                    platform_items.push((Tab::Xbox, Icon::Xbox, XBOX_C, "Xbox", Some(count_xbox)));
                }
                if count_manual > 0 || self.tab == Tab::Manual {
                    platform_items.push((Tab::Manual, Icon::Manual, accent, "Jeux manuels", Some(count_manual)));
                }

                if !platform_items.is_empty() {
                    render_cat_header(ui, "PLATEFORMES");
                    for (tab, icon, col, label, count) in platform_items {
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
                }

                // ── GROUPE 3 : RETRO & DÉCOUVERTES ──
                render_cat_header(ui, "DÉCOUVRIR & RÉTRO");

                let retro_items = [
                    (Tab::Emulators, Icon::Gamepad, STEAM_C, "Émulateurs & ROMs", if count_roms > 0 { Some(count_roms) } else { None }),
                    (Tab::Deals, Icon::Gift, GREEN, "Bons Plans Gratuits", if self.deals.is_empty() { None } else { Some(self.deals.len()) }),
                    (Tab::Stats, Icon::Chart, Color32::from_rgb(56, 189, 248), "Statistiques", None),
                ];
                for (tab, icon, col, label, count) in retro_items {
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
                        TEXT_TERTIARY,
                        self.tab == Tab::Hidden,
                        "Masqués",
                        Some(count_hidden),
                        compact,
                        th,
                    )
                {
                    self.tab = Tab::Hidden;
                }

                // ── GROUPE 4 : SYSTÈME ──
                render_cat_header(ui, "SYSTÈME");

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
                    TEXT_SECONDARY,
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

                // ── Carte de Statut en bas de la barre latérale ──
                let footer_h = if compact { 48.0 } else { 66.0 };
                let rem = (ui.available_height() - footer_h).max(8.0);
                ui.add_space(rem);

                egui::Frame::new()
                    .fill(th.surface_2)
                    .stroke(Stroke::new(1.0, th.border_subtle))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        let (status_color, status_text, status_sub) = if let Some((ref name, started)) = self.active_game {
                            let mins = started.elapsed().as_secs() / 60;
                            (GREEN, format!("EN JEU: {}", trunc(name, 15)), format!("Session active : {mins} min"))
                        } else {
                            match &self.rpc_status {
                                RpcStatus::Connected => (
                                    DISCORD_C,
                                    "Discord RPC".to_string(),
                                    "Connecté (statut actif)".to_string(),
                                ),
                                RpcStatus::WaitingDiscord => (
                                    GOLD,
                                    "Discord RPC".to_string(),
                                    "En attente de Discord...".to_string(),
                                ),
                                RpcStatus::Connecting => (
                                    STEAM_C,
                                    "Discord RPC".to_string(),
                                    "Connexion au socket...".to_string(),
                                ),
                                RpcStatus::Error(_) => (
                                    RED,
                                    "Discord RPC".to_string(),
                                    "Erreur Application ID".to_string(),
                                ),
                                RpcStatus::Disabled => (
                                    TEXT_TERTIARY,
                                    "Discord RPC".to_string(),
                                    "Désactivé".to_string(),
                                ),
                            }
                        };

                        if compact {
                            let (r, resp) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
                            ui.painter().circle_filled(r.center(), 4.0, status_color);
                            ui.painter().circle_stroke(r.center(), 6.5, Stroke::new(1.0, status_color.gamma_multiply(0.35)));
                            resp.on_hover_text(format!("{}\n{}", status_text, status_sub));
                        } else {
                            ui.horizontal(|ui| {
                                let (r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                                ui.painter().circle_filled(r.center(), 3.5, status_color);
                                ui.label(RichText::new(status_text).size(11.0).strong().color(TEXT_PRIMARY));
                            });
                            ui.add_space(1.5);
                            ui.label(
                                RichText::new(status_sub)
                                    .size(10.5)
                                    .color(TEXT_SECONDARY),
                            );
                        }
                    });
            });

        let mut actions = vec![];

        // ── Contenu Central ──
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(th.surface_0)
                    .inner_margin(Margin::symmetric(28, 22)),
            )
            .show(ui, |ui| {
                if let Some(ref gid) = self.selected_game_id.clone() {
                    actions.extend(self.game_details_page_ui(ui, gid));
                } else if self.tab == Tab::Settings {
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

        // ── Fiche Détaillée d'un Bon Plan (Modale Plein Écran) ──
        if self.selected_deal_modal.is_some() {
            self.deal_details_modal_ui(&ctx);
        }

        for a in actions {
            self.apply(&ctx, a);
        }

        // ── Spotlight (Recherche flottante instantanée Alt + Espace) ──
        if let Some(a) = self.spotlight_ui(&ctx) {
            self.apply(&ctx, a);
        }

        // ── Notification Toast Moderne ──
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
                .anchor(Align2::RIGHT_BOTTOM, vec2(-28.0, -28.0))
                .order(egui::Order::Foreground)
                .show(&ctx, |ui| {
                    egui::Frame::new()
                        .fill(th.surface_3.gamma_multiply(alpha))
                        .corner_radius(CornerRadius::same(8))
                        .stroke(Stroke::new(1.0, th.border_default.gamma_multiply(alpha)))
                        .shadow(egui::epaint::Shadow {
                            offset: [0, 8],
                            blur: 16,
                            spread: 0,
                            color: Color32::from_black_alpha((120.0 * alpha) as u8),
                        })
                        .inner_margin(Margin::symmetric(14, 9))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 8.0;
                                let (bar_rect, _) = ui.allocate_exact_size(vec2(2.5, 13.0), Sense::hover());
                                ui.painter().rect_filled(bar_rect, CornerRadius::same(1), accent.gamma_multiply(alpha));
                                ui.label(
                                    RichText::new(msg)
                                        .size(12.5)
                                        .color(TEXT_PRIMARY.gamma_multiply(alpha)),
                                );
                            });
                        });
                });
            ctx.request_repaint_after(Duration::from_millis(16));
        }

        // ── Overlay In-Game de Performances (F9) ──
        if self.settings.overlay.enabled {
            self.render_performance_overlay(&ctx);
        }

        // ── Lightbox de Capture d'écran Plein Écran ──
        if self.selected_screenshot.is_some() {
            self.render_screenshot_lightbox(&ctx);
        }
    }
}

impl Launcher {
    // ───────────────────────────── Overlay In-Game de Performances ─────────────────────────────
    pub fn render_performance_overlay(&self, ctx: &egui::Context) {
        let metrics = self.perf_monitor.snapshot();
        let cfg = &self.settings.overlay;
        let screen = ctx.content_rect();

        let base_w = 205.0 * cfg.scale;
        let pos = match cfg.position {
            crate::performance::OverlayPosition::TopLeft => egui::pos2(screen.min.x + 20.0, screen.min.y + 48.0),
            crate::performance::OverlayPosition::TopRight => egui::pos2(screen.max.x - base_w - 20.0, screen.min.y + 48.0),
            crate::performance::OverlayPosition::BottomLeft => egui::pos2(screen.min.x + 20.0, screen.max.y - 180.0 * cfg.scale),
            crate::performance::OverlayPosition::BottomRight => egui::pos2(screen.max.x - base_w - 20.0, screen.max.y - 180.0 * cfg.scale),
        };

        let alpha = (cfg.opacity.clamp(0.15, 1.0) * 255.0) as u8;
        let bg = Color32::from_black_alpha(alpha);
        let border = Color32::from_white_alpha((alpha as f32 * 0.25) as u8);
        let accent = self.accent();

        egui::Area::new(Id::unique("perf_ingame_overlay"))
            .order(egui::Order::Tooltip)
            .fixed_pos(pos)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(bg)
                    .stroke(Stroke::new(1.0, border))
                    .corner_radius(8.0)
                    .inner_margin(Margin::symmetric((10.0 * cfg.scale).round() as i8, (8.0 * cfg.scale).round() as i8))
                    .show(ui, |ui| {
                        ui.set_width(base_w);
                        ui.spacing_mut().item_spacing.y = 3.0 * cfg.scale;

                        // En-tête HUD discret
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("● HUD PERF").size(9.5 * cfg.scale).strong().color(accent));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new("F9").size(9.0 * cfg.scale).color(TEXT_TERTIARY));
                            });
                        });
                        ui.add_space(2.0);

                        // FPS & Frametime
                        if cfg.show_fps {
                            ui.horizontal(|ui| {
                                let fps_col = if metrics.fps >= 60.0 { GREEN } else if metrics.fps >= 30.0 { GOLD } else { RED };
                                ui.label(RichText::new(format!("{:.0}", metrics.fps)).size(21.0 * cfg.scale).strong().color(fps_col));
                                ui.label(RichText::new("FPS").size(10.0 * cfg.scale).strong().color(TEXT_PRIMARY));

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(
                                        RichText::new(format!("1% Low: {:.0}", metrics.one_percent_low_fps))
                                            .size(9.5 * cfg.scale)
                                            .color(TEXT_SECONDARY),
                                    );
                                    ui.label(
                                        RichText::new(format!("Avg: {:.0}", metrics.avg_fps))
                                            .size(9.5 * cfg.scale)
                                            .color(TEXT_SECONDARY),
                                    );
                                });
                            });
                        }

                        if cfg.show_frametime {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("Frametime : {:.1} ms", metrics.frametime_ms)).size(10.0 * cfg.scale).monospace().color(TEXT_TERTIARY));
                            });
                        }

                        // CPU
                        if cfg.show_cpu {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("CPU").size(10.0 * cfg.scale).strong().color(Color32::from_rgb(56, 189, 248)));
                                let temp_str = metrics.cpu_temp_c.map(|t| format!("{:.0}°C", t)).unwrap_or_else(|| "".into());
                                let val_str = if temp_str.is_empty() {
                                    format!("{:.0}%  •  {:.2} GHz", metrics.cpu_usage, metrics.cpu_freq_mhz as f32 / 1000.0)
                                } else {
                                    format!("{:.0}%  •  {}  •  {:.2} GHz", metrics.cpu_usage, temp_str, metrics.cpu_freq_mhz as f32 / 1000.0)
                                };
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(RichText::new(val_str).size(9.5 * cfg.scale).color(TEXT_PRIMARY));
                                });
                            });
                        }

                        // GPU
                        if cfg.show_gpu {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("GPU").size(10.0 * cfg.scale).strong().color(GREEN));
                                let vram_gb = metrics.gpu_vram_used_mb as f32 / 1024.0;
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(RichText::new(format!("{:.0}%  •  {:.1} Go VRAM", metrics.gpu_usage, vram_gb)).size(9.5 * cfg.scale).color(TEXT_PRIMARY));
                                });
                            });
                        }

                        // RAM
                        if cfg.show_ram {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("RAM").size(10.0 * cfg.scale).strong().color(GOLD));
                                let used_gb = metrics.ram_used_mb as f32 / 1024.0;
                                let tot_gb = metrics.ram_total_mb as f32 / 1024.0;
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(RichText::new(format!("{:.1} / {:.0} Go", used_gb, tot_gb)).size(9.5 * cfg.scale).color(TEXT_PRIMARY));
                                });
                            });
                        }
                    });
            });
    }

    // ───────────────────────────── Lightbox Capture d'écran Plein Écran ─────────────────────────────
    pub fn render_screenshot_lightbox(&mut self, ctx: &egui::Context) {
        let Some(p) = self.selected_screenshot.clone() else {
            return;
        };
        let screen = ctx.content_rect();

        // 1. Backdrop assombri
        egui::Area::new(Id::unique("screenshot_lightbox_backdrop"))
            .order(egui::Order::Foreground)
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                let (r, resp) = ui.allocate_exact_size(screen.size(), Sense::click());
                ui.painter().rect_filled(r, CornerRadius::ZERO, Color32::from_black_alpha(225));
                if resp.clicked() || ctx.input(|i| i.key_pressed(Key::Escape)) {
                    self.selected_screenshot = None;
                }
            });

        // 2. Modale
        let modal_w = (screen.width() - 80.0).min(1100.0);
        let modal_h = (screen.height() - 80.0).min(750.0);
        let pos = egui::pos2(
            (screen.width() - modal_w) / 2.0,
            (screen.height() - modal_h) / 2.0,
        );

        egui::Area::new(Id::unique("screenshot_lightbox_content"))
            .order(egui::Order::Tooltip)
            .fixed_pos(pos)
            .show(ctx, |ui| {
                let th = self.theme();
                egui::Frame::new()
                    .fill(th.surface_2)
                    .stroke(Stroke::new(1.0, th.border_default))
                    .corner_radius(CornerRadius::same(12))
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 16],
                        blur: 32,
                        spread: 0,
                        color: Color32::from_black_alpha(200),
                    })
                    .inner_margin(Margin::same(16))
                    .show(ui, |ui| {
                        ui.set_width(modal_w);
                        ui.set_height(modal_h);

                        ui.horizontal(|ui| {
                            let fname = p.file_name().and_then(|f| f.to_str()).unwrap_or("Capture");
                            ui.label(RichText::new(fname).size(13.5).strong().color(TEXT_PRIMARY));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.button("✕ Fermer (Échap)").clicked() {
                                    self.selected_screenshot = None;
                                }
                                if ui.button("📂 Ouvrir le fichier").clicked() {
                                    let _ = crate::process::open_uri(&p.to_string_lossy());
                                }
                            });
                        });
                        ui.add_space(10.0);

                        let img_uri = format!("file://{}", p.to_string_lossy().replace('\\', "/"));
                        ui.add(
                            egui::Image::new(img_uri)
                                .max_width(modal_w - 32.0)
                                .max_height(modal_h - 70.0)
                                .corner_radius(8.0)
                                .fit_to_exact_size(vec2(modal_w - 32.0, modal_h - 70.0)),
                        );
                    });
            });
    }
}
