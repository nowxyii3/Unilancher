use crate::config::{Game, Settings, Source};
use std::{
    path::Path,
    process::Command,
    sync::mpsc,
    thread,
    time::Instant,
};

/// Liste blanche stricte des protocoles autorisés pour éviter toute injection ou exécution arbitraire
const ALLOWED_URI_SCHEMES: &[&str] = &[
    "steam://",
    "com.epicgames.launcher://",
    "uplay://",
    "origin2://",
    "ea://",
    "battlenet://",
    "xbox://",
    "ms-windows-store://",
    "ms-settings:",
    "https://",
    "http://",
];

/// Vérifie si une URI est conforme à la liste blanche de sécurité
pub fn is_safe_uri(u: &str) -> bool {
    let lower = u.trim().to_lowercase();
    ALLOWED_URI_SCHEMES.iter().any(|scheme| lower.starts_with(scheme))
}

#[cfg(windows)]
#[repr(C)]
#[derive(Clone, Copy)]
struct WinGuid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

#[cfg(windows)]
#[link(name = "powrprof")]
extern "system" {
    fn PowerSetActiveScheme(
        rootpowerkey: *const std::ffi::c_void,
        schemeguid: *const WinGuid,
    ) -> u32;
}

#[cfg(windows)]
pub fn set_high_performance_power_scheme() {
    // GUID pour Performance Élevée : 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c
    let high_perf = WinGuid {
        data1: 0x8c5e7fda,
        data2: 0xe8bf,
        data3: 0x4a96,
        data4: [0x9a, 0x85, 0xa6, 0xe2, 0x3a, 0x8c, 0x63, 0x5c],
    };
    unsafe {
        let _ = PowerSetActiveScheme(std::ptr::null(), &high_perf);
    }
}

#[cfg(not(windows))]
pub fn set_high_performance_power_scheme() {}

#[cfg(windows)]
pub fn set_balanced_power_scheme() {
    // GUID pour Mode Normal / Équilibré : 381b4222-f694-41f0-9685-ff5bb260df2e
    let balanced = WinGuid {
        data1: 0x381b4222,
        data2: 0xf694,
        data3: 0x41f0,
        data4: [0x96, 0x85, 0xff, 0x5b, 0xb2, 0x60, 0xdf, 0x2e],
    };
    unsafe {
        let _ = PowerSetActiveScheme(std::ptr::null(), &balanced);
    }
}

#[cfg(not(windows))]
pub fn set_balanced_power_scheme() {}

pub fn launch_game(
    g: &Game,
    s: &Settings,
    tx: mpsc::Sender<(String, u32)>,
) -> Result<(), String> {
    if s.performance_mode {
        set_high_performance_power_scheme();
    }

    let custom_args = s.launch_args.get(&g.id).cloned().unwrap_or_default();

    if let Some(exe) = s.exe_overrides.get(&g.id) {
        return spawn_and_track(exe, &custom_args, g.id.clone(), tx);
    }
    match g.source {
        Source::Manual | Source::Gog | Source::Ubisoft | Source::Ea | Source::Battlenet | Source::Xbox => {
            if let Some(exe) = &g.exe {
                if exe.exists() {
                    return spawn_and_track(exe, &custom_args, g.id.clone(), tx);
                }
            }
            if let Some(uri) = &g.launch_uri {
                return open_uri(uri);
            }
            Err("Aucun moyen de lancement disponible (ni exécutable ni URI)".into())
        }
        Source::Steam => open_uri(g.launch_uri.as_deref().ok_or("URI Steam manquante")?),
        Source::Epic => {
            if s.epic_direct {
                if let Some(exe) = &g.exe {
                    if exe.exists()
                        && spawn_and_track(exe, &custom_args, g.id.clone(), tx.clone()).is_ok()
                    {
                        return Ok(());
                    }
                }
            }
            open_uri(g.launch_uri.as_deref().ok_or("URI Epic manquante")?)
        }
    }
}

pub fn spawn_and_track(
    exe: &Path,
    raw_args: &str,
    game_id: String,
    tx: mpsc::Sender<(String, u32)>,
) -> Result<(), String> {
    let mut c = Command::new(exe);
    if let Some(dir) = exe.parent() {
        c.current_dir(dir);
    }
    for arg in raw_args.split_whitespace() {
        c.arg(arg);
    }
    let mut child = c.spawn().map_err(|e| e.to_string())?;
    thread::spawn(move || {
        let start = Instant::now();
        let _ = child.wait();
        let mins = (start.elapsed().as_secs() / 60) as u32;
        let _ = tx.send((game_id, mins.max(1)));
    });
    Ok(())
}

pub fn open_uri(u: &str) -> Result<(), String> {
    if !is_safe_uri(u) {
        return Err(format!("Protocole URI rejeté par sécurité : {u}"));
    }

    #[cfg(windows)]
    let mut c = Command::new("explorer");
    #[cfg(target_os = "macos")]
    let mut c = Command::new("open");
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut c = Command::new("xdg-open");
    c.arg(u).spawn().map(|_| ()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_safe_uri() {
        assert!(is_safe_uri("steam://rungameid/12345"));
        assert!(is_safe_uri("com.epicgames.launcher://apps?action=open"));
        assert!(is_safe_uri("https://store.steampowered.com"));
        assert!(is_safe_uri("http://example.com"));
        assert!(is_safe_uri("ms-settings:appsfeatures"));
        assert!(is_safe_uri("uplay://launch/123"));
        assert!(is_safe_uri("xbox://launch/123"));

        // Reject dangerous/unauthorized schemes
        assert!(!is_safe_uri("cmd.exe /c calc"));
        assert!(!is_safe_uri("powershell -Command calc"));
        assert!(!is_safe_uri("javascript:alert(1)"));
        assert!(!is_safe_uri("file:///C:/Windows/System32/calc.exe"));
    }
}
