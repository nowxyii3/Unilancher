use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Tab {
    All,
    Favorites,
    Steam,
    Epic,
    Gog,
    Ubisoft,
    Ea,
    Xbox,
    Manual,
    Emulators,
    Deals,
    Hidden,
    Stats,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SortMode {
    Playtime,
    Name,
    Favorites,
    SizeOnDisk,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ViewMode {
    Grid,
    List,
}

#[allow(dead_code)]
pub enum Action {
    Launch(usize),
    Fav(String),
    ToggleHide(String),
    PickExe(String),
    ClearExe(String),
    PickCover(String),
    ClearCover(String),
    DownloadCover(String),
    BackupSave(String),
    OpenDir(PathBuf),
    Remove(String),
    OpenDetails(String),
    Add,
    AddScanFolder,
    Refresh,
    LaunchRandom,
    OpenUrl(String),
    ScanRoms,
    RefreshDeals,
}

pub fn format_playtime(mins: u32) -> String {
    if mins == 0 {
        "Jamais joué".into()
    } else if mins < 60 {
        format!("{mins} min")
    } else {
        let h = mins / 60;
        let m = mins % 60;
        if m == 0 {
            format!("{h} h")
        } else {
            format!("{h} h {m:02}")
        }
    }
}
