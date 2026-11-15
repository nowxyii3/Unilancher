use crate::config::{Game, Settings, Source};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};

/// Lit une ligne VDF du type:   "clé"   "valeur"
fn kv(line: &str) -> Option<(String, String)> {
    let p: Vec<&str> = line.split('"').collect();
    if p.len() >= 5 {
        Some((p[1].to_string(), p[3].replace("\\\\", "\\")))
    } else {
        None
    }
}

/// Détecte tous les disques durs et SSD disponibles sur le système Windows (C:\, D:\, E:\, etc.)
pub fn get_available_drives() -> Vec<PathBuf> {
    let mut drives = Vec::new();
    for letter in b'C'..=b'Z' {
        let p = PathBuf::from(format!("{}:\\", letter as char));
        if p.exists() {
            drives.push(p);
        }
    }
    if drives.is_empty() {
        drives.push(PathBuf::from(r"C:\"));
    }
    drives
}

#[derive(Clone, Debug)]
pub struct ScanProgress {
    pub stage: String,
    pub games_found: usize,
    pub percent: f32,
    pub finished: bool,
    pub games: Option<Vec<Game>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PlatformStatus {
    pub source: Source,
    pub name: &'static str,
    pub is_installed: bool,
    pub path: Option<PathBuf>,
    pub game_count: usize,
}

pub fn detect_installed_platforms(s: &Settings, games: &[Game]) -> Vec<PlatformStatus> {
    let mut platforms = Vec::new();

    // Steam
    let steam_path = steam_root(s);
    platforms.push(PlatformStatus {
        source: Source::Steam,
        name: "Steam",
        is_installed: steam_path.is_some(),
        path: steam_path,
        game_count: games.iter().filter(|g| g.source == Source::Steam).count(),
    });

    // Epic Games
    let epic_installed = Path::new(r"C:\Program Files (x86)\Epic Games\Launcher\Portal\Binaries\Win64\EpicGamesLauncher.exe").exists()
        || Path::new(r"C:\Program Files\Epic Games\Launcher\Portal\Binaries\Win64\EpicGamesLauncher.exe").exists()
        || Path::new(r"C:\ProgramData\Epic\EpicGamesLauncher").exists();
    platforms.push(PlatformStatus {
        source: Source::Epic,
        name: "Epic Games",
        is_installed: epic_installed,
        path: None,
        game_count: games.iter().filter(|g| g.source == Source::Epic).count(),
    });

    // GOG Galaxy
    let gog_installed = Path::new(r"C:\Program Files (x86)\GOG Galaxy\GalaxyClient.exe").exists()
        || Path::new(r"C:\Program Files\GOG Galaxy\GalaxyClient.exe").exists();
    platforms.push(PlatformStatus {
        source: Source::Gog,
        name: "GOG Galaxy",
        is_installed: gog_installed,
        path: None,
        game_count: games.iter().filter(|g| g.source == Source::Gog).count(),
    });

    // Ubisoft Connect
    let ubi_installed = Path::new(r"C:\Program Files (x86)\Ubisoft\Ubisoft Game Launcher\UbisoftConnect.exe").exists()
        || Path::new(r"C:\Program Files\Ubisoft\Ubisoft Game Launcher\UbisoftConnect.exe").exists();
    platforms.push(PlatformStatus {
        source: Source::Ubisoft,
        name: "Ubisoft Connect",
        is_installed: ubi_installed,
        path: None,
        game_count: games.iter().filter(|g| g.source == Source::Ubisoft).count(),
    });

    // EA App
    let ea_installed = Path::new(r"C:\Program Files\Electronic Arts\EA Desktop\EA Desktop\EADesktop.exe").exists()
        || Path::new(r"C:\Program Files (x86)\Origin\Origin.exe").exists();
    platforms.push(PlatformStatus {
        source: Source::Ea,
        name: "EA App",
        is_installed: ea_installed,
        path: None,
        game_count: games.iter().filter(|g| g.source == Source::Ea).count(),
    });

    // Battle.net
    let bnet_installed = Path::new(r"C:\Program Files (x86)\Battle.net\Battle.net.exe").exists()
        || Path::new(r"C:\Program Files\Battle.net\Battle.net.exe").exists();
    platforms.push(PlatformStatus {
        source: Source::Battlenet,
        name: "Battle.net",
        is_installed: bnet_installed,
        path: None,
        game_count: games.iter().filter(|g| g.source == Source::Battlenet).count(),
    });

    // Xbox PC
    let xbox_installed = Path::new(r"C:\XboxGames").exists() || Path::new(r"D:\XboxGames").exists();
    platforms.push(PlatformStatus {
        source: Source::Xbox,
        name: "Xbox PC",
        is_installed: xbox_installed,
        path: None,
        game_count: games.iter().filter(|g| g.source == Source::Xbox).count(),
    });

    platforms
}

pub fn scan_all_progressive<F>(s: &Settings, mut on_progress: F) -> Vec<Game>
where
    F: FnMut(ScanProgress),
{
    let mut all_games: Vec<Game> = Vec::new();
    let mut seen_ids = HashSet::new();

    on_progress(ScanProgress {
        stage: "Scan des bibliothèques Steam (multi-disques)...".into(),
        games_found: 0,
        percent: 0.15,
        finished: false,
        games: None,
    });
    for g in scan_steam(s) {
        if !g.is_broken() && seen_ids.insert(g.id.clone()) {
            all_games.push(g);
        }
    }

    on_progress(ScanProgress {
        stage: format!("Scan Epic Games Store ({} jeux trouvés)...", all_games.len()),
        games_found: all_games.len(),
        percent: 0.32,
        finished: false,
        games: None,
    });
    for g in scan_epic(s) {
        if !g.is_broken() && seen_ids.insert(g.id.clone()) {
            all_games.push(g);
        }
    }

    on_progress(ScanProgress {
        stage: format!("Scan GOG Galaxy ({} jeux trouvés)...", all_games.len()),
        games_found: all_games.len(),
        percent: 0.45,
        finished: false,
        games: None,
    });
    for g in scan_gog(s) {
        if !g.is_broken() && seen_ids.insert(g.id.clone()) {
            all_games.push(g);
        }
    }

    on_progress(ScanProgress {
        stage: format!("Scan Ubisoft Connect ({} jeux trouvés)...", all_games.len()),
        games_found: all_games.len(),
        percent: 0.58,
        finished: false,
        games: None,
    });
    for g in scan_ubisoft(s) {
        if !g.is_broken() && seen_ids.insert(g.id.clone()) {
            all_games.push(g);
        }
    }

    on_progress(ScanProgress {
        stage: format!("Scan EA App & Origin ({} jeux trouvés)...", all_games.len()),
        games_found: all_games.len(),
        percent: 0.70,
        finished: false,
        games: None,
    });
    for g in scan_ea(s) {
        if !g.is_broken() && seen_ids.insert(g.id.clone()) {
            all_games.push(g);
        }
    }

    on_progress(ScanProgress {
        stage: format!("Scan Battle.net & Blizzard ({} jeux trouvés)...", all_games.len()),
        games_found: all_games.len(),
        percent: 0.82,
        finished: false,
        games: None,
    });
    for g in scan_battlenet(s) {
        if !g.is_broken() && seen_ids.insert(g.id.clone()) {
            all_games.push(g);
        }
    }

    on_progress(ScanProgress {
        stage: format!("Scan Xbox & dossiers configurés ({} jeux trouvés)...", all_games.len()),
        games_found: all_games.len(),
        percent: 0.94,
        finished: false,
        games: None,
    });
    for g in scan_xbox(s) {
        if !g.is_broken() && seen_ids.insert(g.id.clone()) {
            all_games.push(g);
        }
    }
    for g in scan_custom_folders(s) {
        if !g.is_broken() && seen_ids.insert(g.id.clone()) {
            all_games.push(g);
        }
    }

    all_games.sort_by_key(|g| g.name.to_lowercase());

    on_progress(ScanProgress {
        stage: format!("Scan complet terminé : {} jeux synchronisés", all_games.len()),
        games_found: all_games.len(),
        percent: 1.0,
        finished: true,
        games: Some(all_games.clone()),
    });

    all_games
}

#[allow(dead_code)]
pub fn scan_all(s: &Settings) -> Vec<Game> {
    scan_all_progressive(s, |_| {})
}

pub fn start_progressive_scan(s: Settings, tx: std::sync::mpsc::Sender<ScanProgress>) {
    std::thread::spawn(move || {
        scan_all_progressive(&s, |prog| {
            let _ = tx.send(prog);
        });
    });
}

#[cfg(windows)]
pub fn steam_root_registry() -> Option<PathBuf> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey(r"Software\Valve\Steam") {
        if let Ok(path) = key.get_value::<String, _>("SteamPath") {
            let p = PathBuf::from(path);
            if p.join("steamapps").exists() {
                return Some(p);
            }
        }
    }
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey(r"SOFTWARE\WOW6432Node\Valve\Steam") {
        if let Ok(path) = key.get_value::<String, _>("InstallPath") {
            let p = PathBuf::from(path);
            if p.join("steamapps").exists() {
                return Some(p);
            }
        }
    }
    None
}

pub fn steam_root(s: &Settings) -> Option<PathBuf> {
    let custom = s.steam_path.trim();
    if !custom.is_empty() {
        let p = PathBuf::from(custom);
        if p.exists() {
            return Some(p);
        }
    }
    #[cfg(windows)]
    if let Some(p) = steam_root_registry() {
        return Some(p);
    }
    let mut c: Vec<PathBuf> = vec![
        r"C:\Program Files (x86)\Steam".into(),
        r"C:\Program Files\Steam".into(),
    ];
    if let Some(h) = dirs::home_dir() {
        c.push(h.join(".local/share/Steam"));
        c.push(h.join(".steam/steam"));
        c.push(h.join("Library/Application Support/Steam"));
    }
    c.into_iter().find(|p| p.join("steamapps").exists())
}

/// Parcourt `Steam/userdata/<id>/config/localconfig.vdf` pour extraire le temps de jeu (en minutes) de chaque AppID
fn parse_steam_playtimes(steam_root: &Path) -> HashMap<String, u32> {
    let mut map = HashMap::new();
    let userdata = steam_root.join("userdata");
    let Ok(users) = fs::read_dir(userdata) else {
        return map;
    };

    for user in users.flatten() {
        let cfg = user.path().join("config").join("localconfig.vdf");
        let Ok(content) = fs::read_to_string(cfg) else {
            continue;
        };

        let mut stack: Vec<String> = Vec::new();
        let mut last_key = String::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "{" {
                stack.push(last_key.clone());
            } else if trimmed == "}" {
                stack.pop();
            } else if let Some((k, v)) = kv(trimmed) {
                if k.eq_ignore_ascii_case("Playtime") {
                    if let Ok(mins) = v.parse::<u32>() {
                        if let Some(appid) = stack.last() {
                            if appid.chars().all(|c| c.is_ascii_digit()) && !appid.is_empty() {
                                let entry = map.entry(appid.clone()).or_insert(0);
                                *entry = (*entry).max(mins);
                            }
                        }
                    }
                }
            } else {
                let parts: Vec<&str> = trimmed.split('"').collect();
                if parts.len() == 3 {
                    last_key = parts[1].to_string();
                }
            }
        }
    }
    map
}

/// Cherche automatiquement la jaquette d'un jeu Steam dans `librarycache` et `userdata/.../grid`
fn find_steam_cover(steam_root: &Path, appid: &str) -> Option<PathBuf> {
    let cache_dir = steam_root.join("appcache").join("librarycache");

    // 1. Formats directs dans appcache/librarycache
    let direct_candidates = [
        format!("{appid}_library_600x900.jpg"),
        format!("{appid}_library_600x900.png"),
        format!("{appid}_header.jpg"),
        format!("{appid}_capsule_616x353.jpg"),
    ];
    for cand in &direct_candidates {
        let p = cache_dir.join(cand);
        if p.exists() {
            return Some(p);
        }
    }

    // 2. Nouveau format Steam (sous-dossier par AppID dans appcache/librarycache/<appid>/)
    let sub_dir = cache_dir.join(appid);
    if sub_dir.is_dir() {
        let sub_candidates = [
            "library_600x900.jpg",
            "library_600x900.png",
            "library_capsule.jpg",
            "header.jpg",
        ];
        for cand in &sub_candidates {
            let p = sub_dir.join(cand);
            if p.exists() {
                return Some(p);
            }
        }
        // Recherche dans les sous-dossiers de hash éventuels
        if let Ok(entries) = fs::read_dir(&sub_dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    for cand in &sub_candidates {
                        let hp = p.join(cand);
                        if hp.exists() {
                            return Some(hp);
                        }
                    }
                }
            }
        }
    }

    // 3. Grilles personnalisées dans userdata/<id>/config/grid
    if let Ok(users) = fs::read_dir(steam_root.join("userdata")) {
        for user in users.flatten() {
            let grid = user.path().join("config").join("grid");
            for ext in ["png", "jpg"] {
                let p = grid.join(format!("{appid}p.{ext}"));
                if p.exists() {
                    return Some(p);
                }
                let p2 = grid.join(format!("{appid}.{ext}"));
                if p2.exists() {
                    return Some(p2);
                }
            }
        }
    }

    None
}

/// Cherche une image de couverture éventuelle dans le dossier d'installation
pub fn find_folder_cover(install_dir: &Path) -> Option<PathBuf> {
    if !install_dir.exists() {
        return None;
    }
    let candidates = [
        "cover.jpg",
        "cover.png",
        "poster.jpg",
        "poster.png",
        "splash.png",
        "icon.png",
        "logo.png",
        "header.jpg",
        "goggame.ico",
        "EasyAntiCheat/SplashScreen.png",
    ];
    for c in candidates {
        let p = install_dir.join(c);
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(rd) = fs::read_dir(install_dir) {
        for entry in rd.flatten() {
            let p = entry.path();
            if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                if ext.eq_ignore_ascii_case("ico") || ext.eq_ignore_ascii_case("png") || ext.eq_ignore_ascii_case("jpg") {
                    let fname = entry.file_name().to_string_lossy().to_lowercase();
                    if fname.contains("icon") || fname.contains("cover") || fname.contains("logo") || fname.contains("goggame") {
                        return Some(p);
                    }
                }
            }
        }
    }
    None
}

/// Cherche l'exécutable principal d'un jeu dans son dossier en évitant les utilitaires et crash reporters.
pub fn find_main_executable(dir: &Path) -> Option<PathBuf> {
    if !dir.exists() || !dir.is_dir() {
        return None;
    }

    let folder_name = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut candidates: Vec<(PathBuf, u64, bool)> = Vec::new();

    let mut dirs_to_check = vec![dir.to_path_buf()];
    if let Ok(rd) = fs::read_dir(dir) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name == "bin" || name == "binaries" || name == "x64" || name == "win64" || name == "content" || name == "game" {
                    dirs_to_check.push(p);
                }
            }
        }
    }

    for d in dirs_to_check {
        let Ok(rd) = fs::read_dir(&d) else { continue };
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_file() && p.extension().and_then(|s| s.to_str()).map(|e| e.eq_ignore_ascii_case("exe")).unwrap_or(false) {
                let fname = entry.file_name().to_string_lossy().to_lowercase();
                if fname.starts_with("unins")
                    || fname.starts_with("uninstall")
                    || fname.contains("setup")
                    || fname.contains("crash")
                    || fname.contains("report")
                    || fname.contains("helper")
                    || fname.contains("updater")
                    || fname.contains("redist")
                    || fname.contains("config")
                    || fname.contains("launcher_")
                    || fname == "dxsetup.exe"
                {
                    continue;
                }
                let meta = p.metadata().ok();
                let size = meta.map(|m| m.len()).unwrap_or(0);
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                let matches_name = !folder_name.is_empty() && (stem.contains(&folder_name) || folder_name.contains(&stem));
                candidates.push((p, size, matches_name));
            }
        }
    }

    candidates.sort_by(|a, b| {
        b.2.cmp(&a.2)
            .then_with(|| b.1.cmp(&a.1))
    });

    candidates.into_iter().next().map(|(p, _, _)| p)
}

pub fn scan_steam(s: &Settings) -> Vec<Game> {
    let Some(root) = steam_root(s) else {
        return vec![];
    };

    let playtimes = parse_steam_playtimes(&root);

    let mut libs = vec![root.clone()];
    if let Ok(t) = fs::read_to_string(root.join("steamapps/libraryfolders.vdf")) {
        for l in t.lines() {
            if let Some((k, v)) = kv(l) {
                if k == "path" {
                    let p = PathBuf::from(v);
                    if !libs.contains(&p) {
                        libs.push(p);
                    }
                }
            }
        }
    }

    // Détecte également les bibliothèques SteamLibrary sur tous les lecteurs du système
    for d in get_available_drives() {
        let p1 = d.join("SteamLibrary");
        let p2 = d.join("Steam");
        if p1.join("steamapps").exists() && !libs.contains(&p1) {
            libs.push(p1);
        }
        if p2.join("steamapps").exists() && !libs.contains(&p2) {
            libs.push(p2);
        }
    }

    let mut out = vec![];
    for lib in libs {
        let apps = lib.join("steamapps");
        let Ok(rd) = fs::read_dir(&apps) else {
            continue;
        };
        for e in rd.flatten() {
            let fname = e.file_name().to_string_lossy().to_string();
            if !(fname.starts_with("appmanifest_") && fname.ends_with(".acf")) {
                continue;
            }
            let Ok(t) = fs::read_to_string(e.path()) else {
                continue;
            };

            let (mut appid, mut name, mut dir, mut size_bytes) = (String::new(), String::new(), String::new(), 0u64);
            for l in t.lines() {
                if let Some((k, v)) = kv(l) {
                    match k.as_str() {
                        "appid" if appid.is_empty() => appid = v,
                        "name" if name.is_empty() => name = v,
                        "installdir" if dir.is_empty() => dir = v,
                        "SizeOnDisk" => size_bytes = v.parse::<u64>().unwrap_or(0),
                        _ => {}
                    }
                }
            }
            if appid.is_empty() || name.is_empty() {
                continue;
            }
            if appid == "228980"
                || name.starts_with("Proton")
                || name.starts_with("Steam Linux Runtime")
                || name.starts_with("Steamworks")
            {
                continue;
            }

            let cover_path = find_steam_cover(&root, &appid);
            let playtime_mins = playtimes.get(&appid).copied().unwrap_or(0);

            out.push(Game {
                id: format!("steam:{appid}"),
                name,
                source: Source::Steam,
                install_dir: Some(apps.join("common").join(dir)),
                exe: None,
                launch_uri: Some(format!("steam://rungameid/{appid}")),
                cover_path,
                playtime_mins,
                size_bytes,
            });
        }
    }
    out
}

pub fn epic_manifests_dir(s: &Settings) -> PathBuf {
    let custom = s.epic_manifests_path.trim();
    if !custom.is_empty() {
        return PathBuf::from(custom);
    }
    PathBuf::from(r"C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests")
}

pub fn scan_epic(s: &Settings) -> Vec<Game> {
    let Ok(rd) = fs::read_dir(epic_manifests_dir(s)) else {
        return vec![];
    };
    let mut out = vec![];
    for e in rd.flatten() {
        if e.path().extension().and_then(|x| x.to_str()) != Some("item") {
            continue;
        }
        let Ok(t) = fs::read_to_string(e.path()) else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&t) else {
            continue;
        };
        let get = |k: &str| {
            v.get(k)
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string()
        };

        let name = get("DisplayName");
        let launch = get("LaunchExecutable");
        let loc = get("InstallLocation");
        if name.is_empty() || launch.is_empty() {
            continue;
        }
        let install = PathBuf::from(&loc);
        let app = get("AppName");
        let uri = format!(
            "com.epicgames.launcher://apps/{}%3A{}%3A{}?action=launch&silent=true",
            get("CatalogNamespace"),
            get("CatalogItemId"),
            app
        );
        let cover_path = find_folder_cover(&install);
        let size_bytes = v.get("InstalledSize").and_then(|x| x.as_u64()).unwrap_or(0);

        out.push(Game {
            id: format!("epic:{app}"),
            name,
            source: Source::Epic,
            exe: Some(install.join(launch)),
            install_dir: Some(install),
            launch_uri: Some(uri),
            cover_path,
            playtime_mins: 0,
            size_bytes,
        });
    }
    out
}

pub fn scan_gog(_s: &Settings) -> Vec<Game> {
    let mut out = Vec::new();
    let candidates = [
        r"C:\Program Files (x86)\GOG Galaxy\Games",
        r"C:\Program Files\GOG Galaxy\Games",
        r"C:\GOG Games",
        r"D:\GOG Games",
        r"E:\GOG Games",
    ];
    let mut seen_ids = HashSet::new();

    for root_str in candidates {
        let root = Path::new(root_str);
        let Ok(rd) = fs::read_dir(root) else { continue };
        for entry in rd.flatten() {
            let p = entry.path();
            if !p.is_dir() { continue };

            let mut game_name = entry.file_name().to_string_lossy().to_string();
            let mut game_id = format!("gog:{}", game_name);
            let mut main_exe = None;
            let cover_path = find_folder_cover(&p);

            // Vérifie si un fichier goggame-*.info existe pour extraire l'ID et l'exécutable exacts
            if let Ok(sub_rd) = fs::read_dir(&p) {
                for sub_entry in sub_rd.flatten() {
                    let sub_p = sub_entry.path();
                    let sub_fname = sub_entry.file_name().to_string_lossy().to_string();
                    if sub_fname.starts_with("goggame-") && sub_fname.ends_with(".info") {
                        if let Ok(txt) = fs::read_to_string(&sub_p) {
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) {
                                if let Some(gid) = v.get("gameId").and_then(|x| x.as_str()) {
                                    game_id = format!("gog:{gid}");
                                }
                                if let Some(gname) = v.get("name").and_then(|x| x.as_str()) {
                                    game_name = gname.to_string();
                                }
                                if let Some(tasks) = v.get("playTasks").and_then(|x| x.as_array()) {
                                    for task in tasks {
                                        if let Some(task_path) = task.get("path").and_then(|x| x.as_str()) {
                                            let exe_path = p.join(task_path);
                                            if exe_path.exists() {
                                                main_exe = Some(exe_path);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        break;
                    }
                }
            }

            if main_exe.is_none() {
                main_exe = find_main_executable(&p);
            }

            if main_exe.is_some() && !seen_ids.contains(&game_id) {
                seen_ids.insert(game_id.clone());
                let raw_gid = game_id.strip_prefix("gog:").unwrap_or(&game_name);
                let launch_uri = format!("goggalaxy://openGameView/{raw_gid}");
                let size_bytes = calculate_folder_size(&p);
                out.push(Game {
                    id: game_id,
                    name: game_name,
                    source: Source::Gog,
                    install_dir: Some(p),
                    exe: main_exe,
                    launch_uri: Some(launch_uri),
                    cover_path,
                    playtime_mins: 0,
                    size_bytes,
                });
            }
        }
    }
    out
}

pub fn scan_ubisoft(_s: &Settings) -> Vec<Game> {
    let mut out = Vec::new();
    let candidates = [
        r"C:\Program Files (x86)\Ubisoft\Ubisoft Game Launcher\games",
        r"C:\Program Files\Ubisoft\Ubisoft Game Launcher\games",
        r"D:\Ubisoft Game Launcher\games",
    ];
    let mut seen_ids = HashSet::new();

    for root_str in candidates {
        let root = Path::new(root_str);
        let Ok(rd) = fs::read_dir(root) else { continue };
        for entry in rd.flatten() {
            let p = entry.path();
            if !p.is_dir() { continue };

            let folder_name = entry.file_name().to_string_lossy().to_string();
            let id = format!("ubisoft:{folder_name}");
            if seen_ids.contains(&id) { continue };

            if let Some(exe) = find_main_executable(&p) {
                seen_ids.insert(id.clone());
                let cover = find_folder_cover(&p);
                let size_bytes = calculate_folder_size(&p);
                out.push(Game {
                    id,
                    name: folder_name,
                    source: Source::Ubisoft,
                    install_dir: Some(p),
                    exe: Some(exe),
                    launch_uri: None,
                    cover_path: cover,
                    playtime_mins: 0,
                    size_bytes,
                });
            }
        }
    }
    out
}

pub fn scan_ea(_s: &Settings) -> Vec<Game> {
    let mut out = Vec::new();
    let candidates = [
        r"C:\Program Files\EA Games",
        r"C:\Program Files (x86)\Origin Games",
        r"D:\EA Games",
        r"D:\Origin Games",
    ];
    let mut seen_ids = HashSet::new();

    for root_str in candidates {
        let root = Path::new(root_str);
        let Ok(rd) = fs::read_dir(root) else { continue };
        for entry in rd.flatten() {
            let p = entry.path();
            if !p.is_dir() { continue };

            let folder_name = entry.file_name().to_string_lossy().to_string();
            let id = format!("ea:{folder_name}");
            if seen_ids.contains(&id) { continue };

            if let Some(exe) = find_main_executable(&p) {
                seen_ids.insert(id.clone());
                let cover = find_folder_cover(&p);
                let size_bytes = calculate_folder_size(&p);
                out.push(Game {
                    id,
                    name: folder_name,
                    source: Source::Ea,
                    install_dir: Some(p),
                    exe: Some(exe),
                    launch_uri: None,
                    cover_path: cover,
                    playtime_mins: 0,
                    size_bytes,
                });
            }
        }
    }
    out
}

pub fn scan_battlenet(_s: &Settings) -> Vec<Game> {
    let mut out = Vec::new();
    let mut candidates = vec![
        PathBuf::from(r"C:\Program Files (x86)\Battle.net"),
        PathBuf::from(r"C:\Program Files\Battle.net"),
    ];

    for d in get_available_drives() {
        let d_bnet = d.join("Battle.net");
        let d_games = d.join("Games");
        let d_blizzard = d.join("Blizzard");
        if d_bnet.exists() && !candidates.contains(&d_bnet) {
            candidates.push(d_bnet);
        }
        if d_games.exists() && !candidates.contains(&d_games) {
            candidates.push(d_games);
        }
        if d_blizzard.exists() && !candidates.contains(&d_blizzard) {
            candidates.push(d_blizzard);
        }
    }

    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(uninstall_key) = hklm.open_subkey(r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall") {
            for sub in uninstall_key.enum_keys().flatten() {
                if let Ok(sub_k) = uninstall_key.open_subkey(&sub) {
                    let publisher: String = sub_k.get_value("Publisher").unwrap_or_default();
                    if publisher.to_lowercase().contains("blizzard") {
                        if let Ok(loc) = sub_k.get_value::<String, _>("InstallLocation") {
                            let loc_p = PathBuf::from(loc);
                            if loc_p.is_dir() && !candidates.contains(&loc_p) {
                                candidates.push(loc_p);
                            }
                        }
                    }
                }
            }
        }
    }

    let mut seen_ids = HashSet::new();
    for dir in candidates {
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        for entry in rd.flatten() {
            let p = entry.path();
            if !p.is_dir() { continue };
            let folder_name = entry.file_name().to_string_lossy().to_string();
            if folder_name.eq_ignore_ascii_case("Battle.net") || folder_name.eq_ignore_ascii_case("Agent") {
                continue;
            }
            let id = format!("bnet:{folder_name}");
            if seen_ids.contains(&id) { continue };

            if let Some(exe) = find_main_executable(&p) {
                seen_ids.insert(id.clone());
                let cover = find_folder_cover(&p);
                let size_bytes = calculate_folder_size(&p);
                out.push(Game {
                    id,
                    name: folder_name,
                    source: Source::Battlenet,
                    install_dir: Some(p),
                    exe: Some(exe),
                    launch_uri: None,
                    cover_path: cover,
                    playtime_mins: 0,
                    size_bytes,
                });
            }
        }
    }

    out
}

pub fn scan_xbox(_s: &Settings) -> Vec<Game> {
    let mut out = Vec::new();
    let candidates = [
        r"C:\XboxGames",
        r"D:\XboxGames",
        r"E:\XboxGames",
    ];
    let mut seen_ids = HashSet::new();

    for root_str in candidates {
        let root = Path::new(root_str);
        let Ok(rd) = fs::read_dir(root) else { continue };
        for entry in rd.flatten() {
            let p = entry.path();
            if !p.is_dir() { continue };

            let folder_name = entry.file_name().to_string_lossy().to_string();
            let id = format!("xbox:{folder_name}");
            if seen_ids.contains(&id) { continue };

            let content_dir = p.join("Content");
            let search_dir = if content_dir.is_dir() { &content_dir } else { &p };

            if let Some(exe) = find_main_executable(search_dir) {
                seen_ids.insert(id.clone());
                let cover = find_folder_cover(&p).or_else(|| find_folder_cover(search_dir));
                let size_bytes = calculate_folder_size(&p);
                out.push(Game {
                    id,
                    name: folder_name,
                    source: Source::Xbox,
                    install_dir: Some(p),
                    exe: Some(exe),
                    launch_uri: None,
                    cover_path: cover,
                    playtime_mins: 0,
                    size_bytes,
                });
            }
        }
    }
    out
}

pub fn scan_custom_folders(s: &Settings) -> Vec<Game> {
    let mut out = Vec::new();
    let mut seen_ids = HashSet::new();

    for root in &s.custom_scan_folders {
        if !root.is_dir() { continue };
        let Ok(rd) = fs::read_dir(root) else { continue };
        for entry in rd.flatten() {
            let p = entry.path();
            if !p.is_dir() { continue };

            let folder_name = entry.file_name().to_string_lossy().to_string();
            if let Some(exe) = find_main_executable(&p) {
                let id = format!("custom:{}", exe.display());
                if !seen_ids.contains(&id) {
                    seen_ids.insert(id.clone());
                    let cover = find_folder_cover(&p);
                    let size_bytes = calculate_folder_size(&p);
                    out.push(Game {
                        id,
                        name: folder_name,
                        source: Source::Manual,
                        install_dir: Some(p),
                        exe: Some(exe),
                        launch_uri: None,
                        cover_path: cover,
                        playtime_mins: 0,
                        size_bytes,
                    });
                }
            }
        }
    }
    out
}

/// Calcule la taille occupée sur le disque (en octets) d'un dossier
pub fn calculate_folder_size(dir: &Path) -> u64 {
    if !dir.exists() {
        return 0;
    }
    let mut total = 0u64;
    let mut stack = vec![dir.to_path_buf()];
    let mut count = 0;

    while let Some(current) = stack.pop() {
        count += 1;
        if count > 3000 {
            break;
        }
        if let Ok(rd) = fs::read_dir(&current) {
            for entry in rd.flatten() {
                let p = entry.path();
                if p.is_file() {
                    if let Ok(meta) = p.metadata() {
                        total += meta.len();
                    }
                } else if p.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_lowercase();
                    if name != ".git" && name != "crashdumps" {
                        stack.push(p);
                    }
                }
            }
        }
    }
    total
}

/// Formate une taille en octets en format lisible (Go / Mo / Ko)
pub fn format_size(bytes: u64) -> String {
    if bytes == 0 {
        return "--".to_string();
    }
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} Go", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0} Mo", bytes as f64 / MB as f64)
    } else {
        format!("{:.0} Ko", bytes as f64 / KB as f64)
    }
}

/// Cherche l'emplacement des sauvegardes du jeu sur le système
pub fn find_save_folder(g: &Game) -> Option<PathBuf> {
    let clean_name = g.name.trim();
    let candidates = [
        dirs::home_dir().map(|d| d.join("Saved Games").join(clean_name)),
        dirs::document_dir().map(|d| d.join("My Games").join(clean_name)),
        dirs::document_dir().map(|d| d.join(clean_name)),
        dirs::data_local_dir().map(|d| d.join(clean_name)),
        dirs::config_dir().map(|d| d.join(clean_name)),
        dirs::data_local_dir().map(|d| d.join(clean_name).join("Saved")),
        g.install_dir.as_ref().map(|d| d.join("saves")),
        g.install_dir.as_ref().map(|d| d.join("Saved")),
    ];

    candidates.into_iter().flatten().find(|c| c.exists() && c.is_dir())
}

/// Crée une sauvegarde locale datée du dossier de sauvegarde du jeu
pub fn create_save_backup(g: &Game, dest_root: &Path) -> Result<PathBuf, String> {
    let save_dir = find_save_folder(g).ok_or("Aucun dossier de sauvegarde détecté automatiquement pour ce jeu")?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let safe_name = g.name.replace([':', '/', '\\', ' '], "_");
    let backup_dir = dest_root.join(format!("{safe_name}_{timestamp}"));

    copy_dir_all(&save_dir, &backup_dir).map_err(|e| e.to_string())?;
    Ok(backup_dir)
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Télécharge automatiquement une jaquette HD depuis Steam CDN ou Steam Store Search
pub fn download_cover_auto(g: &Game, cache_dir: &Path) -> Option<PathBuf> {
    let _ = fs::create_dir_all(cache_dir);
    let safe_id = g.id.replace([':', '\\', '/', ' '], "_");
    let target_file = cache_dir.join(format!("{safe_id}.jpg"));

    if target_file.exists() && target_file.metadata().map(|m| m.len() > 1000).unwrap_or(false) {
        return Some(target_file);
    }

    // 1. Jeu Steam : CDN officiel haute résolution
    if let Some(appid) = g.id.strip_prefix("steam:") {
        let url = format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{appid}/library_600x900_2x.jpg");
        if let Ok(resp) = ureq::get(&url).timeout(Duration::from_secs(4)).call() {
            if resp.status() == 200 {
                let mut bytes = Vec::new();
                if resp.into_reader().read_to_end(&mut bytes).is_ok()
                    && bytes.len() > 1000
                    && fs::write(&target_file, &bytes).is_ok()
                {
                    return Some(target_file);
                }
            }
        }
        let url_header = format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{appid}/header.jpg");
        if let Ok(resp) = ureq::get(&url_header).timeout(Duration::from_secs(4)).call() {
            if resp.status() == 200 {
                let mut bytes = Vec::new();
                if resp.into_reader().read_to_end(&mut bytes).is_ok()
                    && bytes.len() > 1000
                    && fs::write(&target_file, &bytes).is_ok()
                {
                    return Some(target_file);
                }
            }
        }
    }

    // 2. Recherche sur le Steam Store pour les autres plateformes
    let encoded_name = g.name.replace(' ', "%20");
    let search_url = format!("https://store.steampowered.com/api/storesearch/?term={encoded_name}&l=french&cc=FR");
    if let Ok(resp) = ureq::get(&search_url).timeout(Duration::from_secs(4)).call() {
        if let Ok(json_txt) = resp.into_string() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_txt) {
                if let Some(items) = val.get("items").and_then(|x| x.as_array()) {
                    if let Some(first) = items.first() {
                        if let Some(appid) = first.get("id").and_then(|x| x.as_u64()) {
                            let cover_url = format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{appid}/library_600x900_2x.jpg");
                            if let Ok(c_resp) = ureq::get(&cover_url).timeout(Duration::from_secs(4)).call() {
                                if c_resp.status() == 200 {
                                    let mut bytes = Vec::new();
                                    if c_resp.into_reader().read_to_end(&mut bytes).is_ok()
                                        && bytes.len() > 1000
                                        && fs::write(&target_file, &bytes).is_ok()
                                    {
                                        return Some(target_file);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Recherche les captures d'écran associées à un jeu (Steam userdata, dossier du jeu, captures Windows)
pub fn find_game_screenshots(g: &Game, s: &Settings) -> Vec<PathBuf> {
    let mut screenshots = Vec::new();
    let mut seen = HashSet::new();

    // 1. Steam Screenshots dans userdata/<user>/760/remote/<appid>/screenshots/
    if let Some(appid) = g.id.strip_prefix("steam:") {
        if let Some(root) = steam_root(s) {
            let userdata = root.join("userdata");
            if let Ok(entries) = fs::read_dir(userdata) {
                for u in entries.flatten() {
                    let s_dir = u.path().join("760").join("remote").join(appid).join("screenshots");
                    if let Ok(rd) = fs::read_dir(s_dir) {
                        for pic in rd.flatten() {
                            let p = pic.path();
                            if p.is_file() {
                                if let Some(ext) = p.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()) {
                                    if (ext == "jpg" || ext == "png" || ext == "jpeg") && seen.insert(p.clone()) {
                                        screenshots.push(p);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Dossier Screenshots / Captures dans le dossier d'installation du jeu
    if let Some(ref install_dir) = g.install_dir {
        for sub in &["Screenshots", "screenshots", "Captures", "captures", "Screen", "screens"] {
            let sub_path = install_dir.join(sub);
            if let Ok(rd) = fs::read_dir(sub_path) {
                for pic in rd.flatten() {
                    let p = pic.path();
                    if p.is_file() {
                        if let Some(ext) = p.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()) {
                            if (ext == "jpg" || ext == "png" || ext == "jpeg") && seen.insert(p.clone()) {
                                screenshots.push(p);
                            }
                        }
                    }
                }
            }
        }
    }

    // Tri pour avoir les plus récentes en premier si possible
    screenshots.sort_by(|a, b| {
        let ma = fs::metadata(a).and_then(|m| m.modified()).ok();
        let mb = fs::metadata(b).and_then(|m| m.modified()).ok();
        mb.cmp(&ma)
    });

    screenshots
}

/// Récupère la description et la date de sortie d'un jeu Steam depuis le Store officiel
pub fn fetch_steam_game_details(appid: &str) -> Option<(String, String)> {
    let url = format!("https://store.steampowered.com/api/appdetails?appids={appid}&l=french");
    let resp = ureq::get(&url).timeout(Duration::from_secs(4)).call().ok()?;
    let val = resp.into_json::<serde_json::Value>().ok()?;
    let data = val.get(appid)?.get("data")?;

    let raw_desc = data
        .get("short_description")
        .and_then(|v| v.as_str())
        .or_else(|| data.get("about_the_game").and_then(|v| v.as_str()))
        .unwrap_or("");

    let description = raw_desc
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
        .trim()
        .to_string();

    let release_date = data
        .get("release_date")
        .and_then(|v| v.get("date"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Some((description, release_date))
}

/// Désinstalle un jeu via son launcher officiel (ou la désinstallation Windows enregistrée)
pub fn uninstall_game(g: &Game) -> Result<(), String> {
    match g.source {
        Source::Steam => {
            if let Some(appid) = g.id.strip_prefix("steam:") {
                let uri = format!("steam://uninstall/{appid}");
                return crate::process::open_uri(&uri);
            }
        }
        Source::Epic => {
            return crate::process::open_uri("com.epicgames.launcher://apps?action=open");
        }
        _ => {}
    }

    crate::process::open_uri("ms-settings:appsfeatures")
}
