use crate::config::{Game, Source};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct Emulator {
    pub name: &'static str,
    pub system: &'static str,
    pub exe_path: Option<PathBuf>,
    pub extensions: &'static [&'static str],
}

pub fn detect_installed_emulators() -> Vec<Emulator> {
    let mut list = vec![
        Emulator {
            name: "Dolphin",
            system: "Nintendo GameCube / Wii",
            exe_path: find_exe(&["Dolphin-x64/Dolphin.exe", "Dolphin/Dolphin.exe", "Dolphin.exe"]),
            extensions: &["iso", "rvz", "gcm", "wbfs"],
        },
        Emulator {
            name: "RPCS3",
            system: "Sony PlayStation 3",
            exe_path: find_exe(&["rpcs3/rpcs3.exe", "RPCS3/rpcs3.exe", "rpcs3.exe"]),
            extensions: &["iso", "pkg", "bin"],
        },
        Emulator {
            name: "PCSX2",
            system: "Sony PlayStation 2",
            exe_path: find_exe(&["PCSX2/pcsx2-qtx64.exe", "pcsx2.exe", "PCSX2.exe"]),
            extensions: &["iso", "chd", "cso", "bin"],
        },
        Emulator {
            name: "DuckStation",
            system: "Sony PlayStation 1",
            exe_path: find_exe(&["duckstation/duckstation-qt-x64-ReleaseLTCG.exe", "duckstation.exe"]),
            extensions: &["chd", "cue", "iso", "bin", "pbp"],
        },
        Emulator {
            name: "RetroArch",
            system: "Multi-Systèmes (Arcade & Rétro)",
            exe_path: find_exe(&["RetroArch-Win64/retroarch.exe", "RetroArch/retroarch.exe", "retroarch.exe"]),
            extensions: &["sfc", "smc", "nes", "gba", "gbc", "gb", "md", "z64", "n64"],
        },
        Emulator {
            name: "Cemu",
            system: "Nintendo Wii U",
            exe_path: find_exe(&["cemu/Cemu.exe", "Cemu/Cemu.exe", "Cemu.exe"]),
            extensions: &["wua", "rpx", "wud", "wux"],
        },
    ];

    list.retain(|e| e.exe_path.is_some());
    list
}

fn find_exe(candidates: &[&str]) -> Option<PathBuf> {
    let mut check_roots = vec![];
    if let Some(p) = dirs::config_dir() {
        check_roots.push(p);
    }
    if let Some(p) = dirs::data_local_dir() {
        check_roots.push(p.clone());
        check_roots.push(p.join("Programs"));
    }
    check_roots.push(PathBuf::from(r"C:\Emulators"));
    check_roots.push(PathBuf::from(r"D:\Emulators"));
    check_roots.push(PathBuf::from(r"C:\Program Files"));
    check_roots.push(PathBuf::from(r"C:\Program Files (x86)"));

    for root in check_roots {
        for c in candidates {
            let p = root.join(c);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

/// Scanne un dossier de ROMs et génère les fiches de jeux associées
pub fn scan_roms_folder(roms_dir: &Path, emulators: &[Emulator]) -> Vec<Game> {
    let mut games = vec![];
    if !roms_dir.is_dir() {
        return games;
    }

    if let Ok(entries) = fs::read_dir(roms_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Some(ext) = p.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()) {
                    for emu in emulators {
                        if emu.extensions.contains(&ext.as_str()) {
                            let title = p.file_stem().unwrap_or_default().to_string_lossy().to_string();
                            let clean_title = clean_rom_title(&title);
                            let id = format!("rom:{}:{}", emu.name.to_lowercase(), p.display());
                            let size = fs::metadata(&p).map(|m| m.len()).unwrap_or(0);

                            games.push(Game {
                                id,
                                name: format!("{clean_title} [{}]", emu.name),
                                source: Source::Manual,
                                install_dir: p.parent().map(|d| d.to_path_buf()),
                                exe: emu.exe_path.clone(),
                                launch_uri: None,
                                cover_path: None,
                                playtime_mins: 0,
                                size_bytes: size,
                            });
                            break;
                        }
                    }
                }
            }
        }
    }

    games
}

fn clean_rom_title(raw: &str) -> String {
    // Retire les tags classiques de ROMs : (USA), [En], (v1.0), etc.
    let mut out = String::new();
    let mut inside_paren = false;
    let mut inside_bracket = false;

    for ch in raw.chars() {
        match ch {
            '(' => inside_paren = true,
            ')' => inside_paren = false,
            '[' => inside_bracket = true,
            ']' => inside_bracket = false,
            _ if !inside_paren && !inside_bracket => out.push(ch),
            _ => {}
        }
    }

    let trimmed = out.trim().to_string();
    if trimmed.is_empty() {
        raw.to_string()
    } else {
        trimmed
    }
}
