use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum Source {
    Steam,
    Epic,
    Gog,
    Ubisoft,
    Ea,
    Battlenet,
    Xbox,
    Manual,
}

impl Source {
    pub fn label(&self) -> &'static str {
        match self {
            Source::Steam => "STEAM",
            Source::Epic => "EPIC",
            Source::Gog => "GOG",
            Source::Ubisoft => "UBISOFT",
            Source::Ea => "EA",
            Source::Battlenet => "BATTLE.NET",
            Source::Xbox => "XBOX",
            Source::Manual => "MANUEL",
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub source: Source,
    pub install_dir: Option<PathBuf>,
    pub exe: Option<PathBuf>,
    pub launch_uri: Option<String>,
    /// Chemin local vers la jaquette détectée automatiquement
    #[serde(default)]
    pub cover_path: Option<PathBuf>,
    /// Temps de jeu détecté (en minutes)
    #[serde(default)]
    pub playtime_mins: u32,
    /// Taille occupée sur le disque en octets
    #[serde(default)]
    pub size_bytes: u64,
}

impl Game {
    /// Vrai si le jeu semble cassé / déplacé (dossier ou exécutable introuvable).
    pub fn is_broken(&self) -> bool {
        let dir_missing = self
            .install_dir
            .as_ref()
            .map(|d| !d.exists())
            .unwrap_or(false);
        match self.source {
            Source::Steam => dir_missing,
            Source::Epic | Source::Gog | Source::Ubisoft | Source::Ea | Source::Battlenet | Source::Xbox => {
                dir_missing || self.exe.as_ref().map(|e| !e.exists()).unwrap_or(false)
            }
            Source::Manual => self.exe.as_ref().map(|e| !e.exists()).unwrap_or(true),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    /// Lancer directement l'exécutable des jeux Epic au lieu de passer par l'URI
    pub epic_direct: bool,
    /// Minimiser UniLauncher dans la barre des tâches au lancement d'un jeu
    pub minimize_on_launch: bool,
    /// Activer ou désactiver le statut Discord Rich Presence en temps réel
    #[serde(default = "default_true")]
    pub discord_rpc: bool,
    /// Identifiant d'application Discord pour le Rich Presence (Client ID)
    #[serde(default = "default_discord_client_id")]
    pub discord_client_id: String,
    /// Ouvrir automatiquement l'application en plein écran au démarrage
    #[serde(default = "default_true")]
    pub start_fullscreen: bool,
    /// Réduire la barre latérale aux icônes uniquement
    pub compact_sidebar: bool,
    /// Chemin personnalisé vers Steam
    pub steam_path: String,
    /// Chemin personnalisé vers les manifestes Epic Games
    pub epic_manifests_path: String,
    /// Dossiers supplémentaires à scanner automatiquement
    pub custom_scan_folders: Vec<PathBuf>,
    /// Index de la couleur d'accentuation choisie
    pub accent: usize,
    /// Mode de filtre rapide (0=Tous, 1=Favoris, 2=Récents, 3=Non joués)
    pub quick_filter_mode: usize,
    /// Échelle de zoom des cartes de la grille (0.75 à 1.4)
    #[serde(default = "default_card_scale")]
    pub card_scale: f32,
    /// Thème visuel de l'interface (0=Obsidian, 1=OLED Noir Pur, 2=Cyberpunk, 3=Midnight Blue)
    #[serde(default)]
    pub theme: usize,
    /// Télécharger automatiquement les jaquettes manquantes via Steam CDN / Store
    #[serde(default = "default_true")]
    pub auto_download_covers: bool,
    /// Mode haute performance au lancement d'un jeu (optimisation plan d'alimentation)
    #[serde(default)]
    pub performance_mode: bool,
    /// Mode d'affichage mémorisé (Grille ou Liste)
    #[serde(default)]
    pub view_mode: crate::types::ViewMode,
    /// Taille prédéfinie des cartes en grille (Petite, Moyenne, Grande)
    #[serde(default)]
    pub grid_card_size: crate::types::GridCardSize,
    /// Filtre de statut avancé mémorisé
    #[serde(default)]
    pub status_filter: crate::types::LibraryStatusFilter,
    /// Mode de tri mémorisé
    #[serde(default)]
    pub sort_mode: crate::types::SortMode,
    /// Bannière personnalisée choisie à la main (clé = id du jeu)
    #[serde(default)]
    pub banner_overrides: HashMap<String, PathBuf>,
    /// Liste des jeux ajoutés manuellement
    pub manual_games: Vec<Game>,
    /// Exécutable choisi à la main pour un jeu (clé = id du jeu)
    pub exe_overrides: HashMap<String, PathBuf>,
    /// Jaquette personnalisée choisie à la main (clé = id du jeu)
    pub cover_overrides: HashMap<String, PathBuf>,
    /// Temps de jeu mesuré par UniLauncher en minutes (clé = id du jeu)
    pub tracked_playtime: HashMap<String, u32>,
    /// Dernier jeu lancé (id du jeu)
    pub last_played: Option<String>,
    /// Identifiants des jeux mis en favoris
    pub favorites: HashSet<String>,
    /// Identifiants des bons plans mis en favoris
    #[serde(default)]
    pub favorite_deals: HashSet<String>,
    /// Activer les alertes et notifications de nouveaux bons plans
    #[serde(default = "default_true")]
    pub deals_notifications: bool,
    /// Identifiants des jeux masqués de la bibliothèque principale
    pub hidden_games: HashSet<String>,
    /// Arguments de lancement personnalisés par jeu (ex: "-dx11 -novid")
    pub launch_args: HashMap<String, String>,
    /// Notes personnelles associées à chaque jeu
    pub game_notes: HashMap<String, String>,
    /// Tags / Catégories associés à chaque jeu
    pub game_tags: HashMap<String, Vec<String>>,

    /// Nombre de sessions de jeu lancées (clé = id du jeu)
    #[serde(default)]
    pub session_counts: HashMap<String, u32>,
    /// Timestamp Unix de la dernière session terminée (clé = id du jeu)
    #[serde(default)]
    pub last_played_dates: HashMap<String, u64>,
    /// Descriptions / synopsis du jeu (clé = id du jeu)
    #[serde(default)]
    pub game_descriptions: HashMap<String, String>,
    /// Date de sortie du jeu (clé = id du jeu)
    #[serde(default)]
    pub game_release_dates: HashMap<String, String>,
    /// Clé API Steam Web pour les succès
    #[serde(default)]
    pub steam_api_key: String,
    /// SteamID de l'utilisateur pour les succès
    #[serde(default)]
    pub steam_user_id: String,
    /// Configuration de l'overlay de performances
    #[serde(default)]
    pub overlay: crate::performance::OverlayConfig,
    /// Espacement de la grille de jeux
    #[serde(default = "default_grid_spacing")]
    pub grid_spacing: f32,
    /// Afficher le temps de jeu sur les cartes
    #[serde(default = "default_true")]
    pub show_card_playtime: bool,
    /// Afficher les badges de plateformes sur les cartes
    #[serde(default = "default_true")]
    pub show_card_badges: bool,
    /// Animations et transitions activées
    #[serde(default = "default_true")]
    pub enable_animations: bool,
    /// Canal de mise à jour (Stable vs BETA)
    #[serde(default)]
    pub update_channel: UpdateChannel,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum UpdateChannel {
    Stable,
    Beta,
}

impl Default for UpdateChannel {
    fn default() -> Self {
        if env!("CARGO_PKG_VERSION").contains("beta") {
            UpdateChannel::Beta
        } else {
            UpdateChannel::Stable
        }
    }
}

impl UpdateChannel {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            UpdateChannel::Stable => "Stable",
            UpdateChannel::Beta => "BETA",
        }
    }
}

fn default_grid_spacing() -> f32 {
    16.0
}

fn default_card_scale() -> f32 {
    1.0
}

fn default_true() -> bool {
    true
}

fn default_discord_client_id() -> String {
    // ID d'application Discord par défaut
    "1198684210695467068".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            epic_direct: false,
            minimize_on_launch: true,
            discord_rpc: true,
            discord_client_id: default_discord_client_id(),
            start_fullscreen: true,
            compact_sidebar: false,
            steam_path: String::new(),
            epic_manifests_path: String::new(),
            custom_scan_folders: Vec::new(),
            accent: 0,
            quick_filter_mode: 0,
            card_scale: 1.0,
            theme: 0,
            auto_download_covers: true,
            performance_mode: false,
            view_mode: crate::types::ViewMode::Grid,
            grid_card_size: crate::types::GridCardSize::Medium,
            status_filter: crate::types::LibraryStatusFilter::All,
            sort_mode: crate::types::SortMode::Playtime,
            banner_overrides: HashMap::new(),
            manual_games: Vec::new(),
            exe_overrides: HashMap::new(),
            cover_overrides: HashMap::new(),
            tracked_playtime: HashMap::new(),
            last_played: None,
            favorites: HashSet::new(),
            favorite_deals: HashSet::new(),
            deals_notifications: true,
            hidden_games: HashSet::new(),
            launch_args: HashMap::new(),
            game_notes: HashMap::new(),
            game_tags: HashMap::new(),
            session_counts: HashMap::new(),
            last_played_dates: HashMap::new(),
            game_descriptions: HashMap::new(),
            game_release_dates: HashMap::new(),
            steam_api_key: String::new(),
            steam_user_id: String::new(),
            overlay: crate::performance::OverlayConfig::default(),
            grid_spacing: default_grid_spacing(),
            show_card_playtime: true,
            show_card_badges: true,
            enable_animations: true,
            update_channel: UpdateChannel::default(),
        }
    }
}

impl Settings {
    /// Dossier de stockage de la configuration (%APPDATA%/UniLauncher ou %APPDATA%/UniLauncher-Beta sous Windows)
    pub fn dir() -> PathBuf {
        let folder = if env!("CARGO_PKG_VERSION").contains("beta") {
            "UniLauncher-Beta"
        } else {
            "UniLauncher"
        };
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(folder)
    }

    /// Dossier de configuration de la version stable pour importation sécurisée en mode BETA
    pub fn stable_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("UniLauncher")
    }

    /// Chemin complet du fichier JSON de configuration
    pub fn file() -> PathBuf {
        Self::dir().join("config.json")
    }

    /// Charge la configuration depuis le disque ou renvoie les valeurs par défaut
    pub fn load() -> Self {
        let current_file = Self::file();
        let json_text = if current_file.exists() {
            fs::read_to_string(&current_file).ok()
        } else if env!("CARGO_PKG_VERSION").contains("beta") {
            // En mode BETA, si aucune config beta n'existe encore, on importe en lecture seule
            // la configuration stable existante sans jamais modifier cette dernière.
            let stable_file = Self::stable_dir().join("config.json");
            fs::read_to_string(&stable_file).ok()
        } else {
            None
        };

        let mut settings: Self = json_text
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();

        settings.cleanup_empty_entries();
        settings
    }

    /// Sauvegarde atomique sur le disque (évite de corrompre le fichier en cas de crash)
    pub fn save(&self) {
        let dir = Self::dir();
        if fs::create_dir_all(&dir).is_err() {
            return;
        }

        let mut clean_clone = self.clone();
        clean_clone.cleanup_empty_entries();

        if let Ok(json) = serde_json::to_string_pretty(&clean_clone) {
            let tmp_file = dir.join("config.json.tmp");
            let final_file = Self::file();

            if fs::write(&tmp_file, &json).is_ok()
                && fs::rename(&tmp_file, &final_file).is_err()
            {
                let _ = fs::write(&final_file, json);
                let _ = fs::remove_file(&tmp_file);
            }
        }
    }

    /// Nettoie les chaînes vides pour garder un fichier JSON propre
    fn cleanup_empty_entries(&mut self) {
        self.launch_args.retain(|_, v| !v.trim().is_empty());
        self.game_notes.retain(|_, v| !v.trim().is_empty());
        self.game_tags.retain(|_, v| !v.is_empty());
    }
}