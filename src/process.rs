use crate::config::{Game, Settings, Source};
use std::{
    path::Path,
    process::Command,
    sync::mpsc,
    thread,
    time::Instant,
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

pub fn launch_game(
    g: &Game,
    s: &Settings,
    tx: mpsc::Sender<(String, u32)>,
) -> Result<(), String> {
    if s.performance_mode {
        #[cfg(windows)]
        {
            let mut cmd = Command::new("powercfg");
            cmd.creation_flags(0x08000000);
            let _ = cmd.args(["/setactive", "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c"]).spawn();
        }
    }

    let custom_args = s.launch_args.get(&g.id).cloned().unwrap_or_default();

    if let Some(exe) = s.exe_overrides.get(&g.id) {
        return spawn_and_track(exe, &custom_args, g.id.clone(), tx);
    }
    match g.source {
        Source::Manual | Source::Gog | Source::Ubisoft | Source::Ea | Source::Xbox => {
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
    #[cfg(windows)]
    let mut c = Command::new("explorer");
    #[cfg(target_os = "macos")]
    let mut c = Command::new("open");
    #[cfg(all(unix, not(target_os = "macos")))]
    let mut c = Command::new("xdg-open");
    c.arg(u).spawn().map(|_| ()).map_err(|e| e.to_string())
}
