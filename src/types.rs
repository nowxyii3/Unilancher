use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Tab {
    All,
    Favorites,
    Steam,
    Epic,
    Gog,
    Ubisoft,
    Ea,
    Battlenet,
    Xbox,
    Manual,
    Emulators,
    Deals,
    Hidden,
    Stats,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Default)]
pub enum SortMode {
    #[default]
    Playtime,
    Name,
    NameDesc,
    LastPlayed,
    Favorites,
    SizeOnDisk,
}

impl SortMode {
    pub fn label(&self) -> &'static str {
        match self {
            SortMode::Playtime => "Temps de jeu",
            SortMode::Name => "Nom (A → Z)",
            SortMode::NameDesc => "Nom (Z → A)",
            SortMode::LastPlayed => "Dernière utilisation",
            SortMode::Favorites => "Favoris en premier",
            SortMode::SizeOnDisk => "Taille sur le disque",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Default)]
pub enum ViewMode {
    #[default]
    Grid,
    List,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Default)]
pub enum GridCardSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl GridCardSize {
    pub fn label(&self) -> &'static str {
        match self {
            GridCardSize::Small => "Petite",
            GridCardSize::Medium => "Moyenne",
            GridCardSize::Large => "Grande",
        }
    }

    pub fn dimensions(&self, scale: f32) -> (f32, f32) {
        match self {
            GridCardSize::Small => (148.0 * scale, 210.0 * scale),
            GridCardSize::Medium => (184.0 * scale, 258.0 * scale),
            GridCardSize::Large => (228.0 * scale, 318.0 * scale),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, Default)]
pub enum LibraryStatusFilter {
    #[default]
    All,
    Favorites,
    Recent,
    NeverPlayed,
    Installed,
    NotInstalled,
}

impl LibraryStatusFilter {
    pub fn label(&self) -> &'static str {
        match self {
            LibraryStatusFilter::All => "Tous les jeux",
            LibraryStatusFilter::Favorites => "⭐ Favoris",
            LibraryStatusFilter::Recent => "🕒 Récents",
            LibraryStatusFilter::NeverPlayed => "✨ Jamais lancés",
            LibraryStatusFilter::Installed => "🟢 Installés",
            LibraryStatusFilter::NotInstalled => "⚠️ Non installés",
        }
    }
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
    PickBanner(String),
    ClearBanner(String),
    BackupSave(String),
    OpenDir(PathBuf),
    Remove(String),
    OpenDetails(String),
    CloseDetails,
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

pub fn format_last_played_time(ts: Option<u64>) -> String {
    let Some(t) = ts else {
        return "Jamais joué".to_string();
    };
    if t == 0 {
        return "Jamais joué".to_string();
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if now < t {
        return "À l'instant".to_string();
    }
    let diff = now - t;
    if diff < 60 {
        "À l'instant".to_string()
    } else if diff < 3600 {
        format!("Il y a {} min", diff / 60)
    } else if diff < 86400 {
        format!("Il y a {} h", diff / 3600)
    } else if diff < 86400 * 30 {
        format!("Il y a {} j", diff / 86400)
    } else {
        format!("Il y a {} mois", diff / (86400 * 30))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_card_size_dimensions() {
        let (w_s, h_s) = GridCardSize::Small.dimensions(1.0);
        let (w_m, h_m) = GridCardSize::Medium.dimensions(1.0);
        let (w_l, h_l) = GridCardSize::Large.dimensions(1.0);

        assert!(w_s < w_m);
        assert!(w_m < w_l);
        assert!(h_s < h_m);
        assert!(h_m < h_l);

        // Aspect ratio is roughly 3:2 height to width
        assert!((h_m / w_m - 1.4).abs() < 0.1);

        // Test scaling
        let (w_scaled, h_scaled) = GridCardSize::Medium.dimensions(1.5);
        assert_eq!(w_scaled, 184.0 * 1.5);
        assert_eq!(h_scaled, 258.0 * 1.5);
    }

    #[test]
    fn test_format_last_played() {
        assert_eq!(format_last_played_time(None), "Jamais joué");
        assert_eq!(format_last_played_time(Some(0)), "Jamais joué");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // 10 seconds ago
        assert_eq!(format_last_played_time(Some(now - 10)), "À l'instant");
        // 10 minutes ago
        assert_eq!(format_last_played_time(Some(now - 600)), "Il y a 10 min");
        // 5 hours ago
        assert_eq!(format_last_played_time(Some(now - 18000)), "Il y a 5 h");
        // 3 days ago
        assert_eq!(format_last_played_time(Some(now - 86400 * 3)), "Il y a 3 j");
    }

    #[test]
    fn test_sort_mode_names() {
        assert_eq!(SortMode::Playtime.label(), "Temps de jeu");
        assert_eq!(SortMode::Name.label(), "Nom (A → Z)");
        assert_eq!(SortMode::NameDesc.label(), "Nom (Z → A)");
        assert_eq!(SortMode::LastPlayed.label(), "Dernière utilisation");
        assert_eq!(SortMode::Favorites.label(), "Favoris en premier");
        assert_eq!(SortMode::SizeOnDisk.label(), "Taille sur le disque");
    }

    #[test]
    fn test_library_status_filter_labels() {
        assert_eq!(LibraryStatusFilter::All.label(), "Tous les jeux");
        assert_eq!(LibraryStatusFilter::Favorites.label(), "⭐ Favoris");
        assert_eq!(LibraryStatusFilter::Recent.label(), "🕒 Récents");
        assert_eq!(LibraryStatusFilter::NeverPlayed.label(), "✨ Jamais lancés");
        assert_eq!(LibraryStatusFilter::Installed.label(), "🟢 Installés");
        assert_eq!(LibraryStatusFilter::NotInstalled.label(), "⚠️ Non installés");
    }
}
