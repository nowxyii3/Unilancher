use crate::config::Settings;
use eframe::egui::Color32;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ═══════════════════════════════════════════════════════════════════════════════
//  UniLauncher — Deals & Freebies System (Modèle & Tracker)
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DealPlatform {
    Epic,
    Steam,
    Gog,
    Itch,
    Windows,
    OpenSource,
    Other(String),
}

impl DealPlatform {
    pub fn name(&self) -> &str {
        match self {
            DealPlatform::Epic => "Epic Games",
            DealPlatform::Steam => "Steam",
            DealPlatform::Gog => "GOG",
            DealPlatform::Itch => "itch.io",
            DealPlatform::Windows => "Windows / PC",
            DealPlatform::OpenSource => "Open Source",
            DealPlatform::Other(s) => s.as_str(),
        }
    }

    pub fn color(&self) -> Color32 {
        match self {
            DealPlatform::Epic => Color32::from_rgb(226, 232, 240),
            DealPlatform::Steam => Color32::from_rgb(110, 194, 244),
            DealPlatform::Gog => Color32::from_rgb(167, 139, 250),
            DealPlatform::Itch => Color32::from_rgb(251, 113, 133),
            DealPlatform::Windows => Color32::from_rgb(56, 189, 248),
            DealPlatform::OpenSource => Color32::from_rgb(52, 211, 153),
            DealPlatform::Other(_) => Color32::from_rgb(251, 191, 36),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DealCategory {
    Game,
    Dlc,
    Software,
    Productivity,
    Creative,
    DevTool,
    Indie,
}

impl DealCategory {
    pub fn label(&self) -> &'static str {
        match self {
            DealCategory::Game => "Jeu complet",
            DealCategory::Dlc => "DLC & Contenu",
            DealCategory::Software => "Logiciel & Utilitaire",
            DealCategory::Productivity => "Productivité",
            DealCategory::Creative => "Création & Design",
            DealCategory::DevTool => "Développement",
            DealCategory::Indie => "Pépite Indépendante",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            DealCategory::Game => "🎮",
            DealCategory::Dlc => "📦",
            DealCategory::Software => "💻",
            DealCategory::Productivity => "⚡",
            DealCategory::Creative => "🎨",
            DealCategory::DevTool => "🛠️",
            DealCategory::Indie => "💎",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DealType {
    /// Offre 100% gratuite à réclamer pour conserver à vie dans sa bibliothèque
    FreeToKeep,
    /// Offre d'essai temporaire, week-end gratuit ou période limitée
    LimitedTime,
    /// Jeu Free-to-play permanent
    FreeToPlay,
    /// Logiciel libre ou jeu gratuit permanent vérifié
    OpenSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DealsTab {
    All,
    Games,
    EndingSoon,
    ToKeep,
    Software,
    Discoveries,
    Favorites,
}

#[allow(dead_code)]
impl DealsTab {
    pub fn label(&self) -> &'static str {
        match self {
            DealsTab::All => "🔥 Tout découvrir",
            DealsTab::Games => "🎮 Jeux gratuits",
            DealsTab::EndingSoon => "⚡ Offres limitées",
            DealsTab::ToKeep => "🎁 -100% À garder",
            DealsTab::Software => "💻 Logiciels & Outils",
            DealsTab::Discoveries => "💎 Pépites & Découvertes",
            DealsTab::Favorites => "⭐ Mes Favoris",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DealsSortMode {
    Featured,
    EndingSoon,
    HighestValue,
    Newest,
    Alphabetical,
}

impl DealsSortMode {
    pub fn label(&self) -> &'static str {
        match self {
            DealsSortMode::Featured => "✨ Recommandés",
            DealsSortMode::EndingSoon => "⏳ Expire bientôt",
            DealsSortMode::HighestValue => "💰 Plus grande valeur",
            DealsSortMode::Newest => "🆕 Récemment ajoutés",
            DealsSortMode::Alphabetical => "🔤 Nom (A-Z)",
        }
    }
}

impl DealType {
    pub fn label(&self) -> &'static str {
        match self {
            DealType::FreeToKeep => "-100% À GARDER",
            DealType::LimitedTime => "OFFRE LIMITÉE",
            DealType::FreeToPlay => "FREE-TO-PLAY",
            DealType::OpenSource => "GRATUIT PERMANENT",
        }
    }

    pub fn color(&self) -> Color32 {
        match self {
            DealType::FreeToKeep => Color32::from_rgb(52, 211, 153), // Emerald green
            DealType::LimitedTime => Color32::from_rgb(251, 191, 36), // Amber warning
            DealType::FreeToPlay => Color32::from_rgb(96, 165, 250), // Sky blue
            DealType::OpenSource => Color32::from_rgb(167, 139, 250), // Violet
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FreeDeal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub image_url: Option<String>,
    pub store_url: String,
    pub is_active_now: bool,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub end_timestamp: Option<i64>,
    pub original_price: Option<String>,
    #[serde(default = "default_current_price")]
    pub current_price: String,
    #[serde(default = "default_platform")]
    pub platform: DealPlatform,
    #[serde(default = "default_category")]
    pub category: DealCategory,
    #[serde(default = "default_deal_type")]
    pub deal_type: DealType,
    #[serde(default)]
    pub claim_instructions: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub is_featured: bool,
    #[serde(default)]
    pub publisher_or_dev: Option<String>,
    #[serde(default)]
    pub rating_or_popularity: Option<f32>,
}

fn default_current_price() -> String {
    "0,00 €".to_string()
}
fn default_platform() -> DealPlatform {
    DealPlatform::Epic
}
fn default_category() -> DealCategory {
    DealCategory::Game
}
fn default_deal_type() -> DealType {
    DealType::FreeToKeep
}

impl FreeDeal {
    /// Vérifie si l'offre a dépassé sa date limite
    pub fn is_expired(&self) -> bool {
        if let Some(ts) = self.end_timestamp {
            let now = current_unix_timestamp();
            return ts <= now;
        }
        false
    }

    /// Indique si l'offre expire dans moins de X heures
    pub fn expires_soon(&self, hours_threshold: i64) -> bool {
        if !self.is_active_now || self.is_expired() {
            return false;
        }
        if let Some(ts) = self.end_timestamp {
            let now = current_unix_timestamp();
            let diff = ts - now;
            return diff > 0 && diff <= hours_threshold * 3600;
        }
        false
    }

    /// Retourne les composants du compte à rebours (jours, heures, minutes, secondes)
    pub fn remaining_time(&self) -> Option<(i64, i64, i64, i64)> {
        let ts = self.end_timestamp?;
        let now = current_unix_timestamp();
        if ts <= now {
            return None;
        }
        let diff = ts - now;
        let days = diff / 86400;
        let hours = (diff % 86400) / 3600;
        let mins = (diff % 3600) / 60;
        let secs = diff % 60;
        Some((days, hours, mins, secs))
    }

    /// Formate le compte à rebours de façon lisible et compacte
    pub fn remaining_time_str(&self) -> Option<String> {
        let (d, h, m, _) = self.remaining_time()?;
        if d > 0 {
            Some(format!("⏰ Reste {}j {}h", d, h))
        } else if h > 0 {
            Some(format!("⏰ Reste {}h {}m", h, m))
        } else {
            Some(format!("⏰ Reste {}m (Expire vite !)", m.max(1)))
        }
    }

    /// Économie estimée ou texte promotionnel
    pub fn discount_badge(&self) -> &'static str {
        match self.deal_type {
            DealType::FreeToKeep => "-100%",
            DealType::LimitedTime => "GRATUIT",
            DealType::FreeToPlay => "F2P",
            DealType::OpenSource => "LIBRE",
        }
    }
}

pub fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DealsCache {
    pub last_updated_secs: u64,
    pub deals: Vec<FreeDeal>,
}

pub struct DealsTracker;

impl DealsTracker {
    /// Dossier du cache des bons plans
    fn cache_file() -> PathBuf {
        Settings::dir().join("deals_cache.json")
    }

    /// Charge le cache depuis le disque ou génère les données initiales vérifiées
    pub fn load_cached_or_default() -> Vec<FreeDeal> {
        if let Some((_, cached)) = Self::load_cache_from_disk() {
            if !cached.is_empty() {
                return cached;
            }
        }
        Self::fallback_curated_deals()
    }

    /// Charge le cache depuis le disque avec le timestamp de dernière mise à jour
    pub fn load_cache_from_disk() -> Option<(u64, Vec<FreeDeal>)> {
        let path = Self::cache_file();
        let content = fs::read_to_string(path).ok()?;
        let cache: DealsCache = serde_json::from_str(&content).ok()?;
        Some((cache.last_updated_secs, cache.deals))
    }

    /// Sauvegarde atomique du cache sur le disque
    pub fn save_cache_to_disk(deals: &[FreeDeal]) {
        let dir = Settings::dir();
        let _ = fs::create_dir_all(&dir);
        let cache = DealsCache {
            last_updated_secs: current_unix_timestamp() as u64,
            deals: deals.to_vec(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&cache) {
            let tmp = dir.join("deals_cache.json.tmp");
            let final_f = Self::cache_file();
            if fs::write(&tmp, &json).is_ok() {
                let _ = fs::rename(&tmp, &final_f);
            }
        }
    }

    /// Récupération complète et fusion dédupliquée depuis toutes les sources fiables
    pub fn fetch_all_deals() -> Vec<FreeDeal> {
        let mut all_deals = Vec::new();

        // 1. Offres officielles Epic Games Store
        if let Ok(epic_deals) = Self::fetch_epic_free_games() {
            all_deals.extend(epic_deals);
        }

        // 2. Offres GamerPower (Steam, GOG, Itch, etc.)
        if let Ok(gp_deals) = Self::fetch_gamerpower_giveaways() {
            all_deals.extend(gp_deals);
        }

        // 3. Top Free-to-Play PC reconnus
        if let Ok(f2p_deals) = Self::fetch_freetogame_top() {
            all_deals.extend(f2p_deals);
        }

        // 4. Logiciels & Outils indispensables (Open source & vérifiés)
        all_deals.extend(Self::curated_software_and_tools());

        // 5. Découvertes & Pépites indépendantes
        all_deals.extend(Self::curated_indie_gems());

        // 6. Déduplication par titre normalisé
        let mut seen = HashSet::new();
        let mut deduped = Vec::new();

        for deal in all_deals {
            let norm_key = format!(
                "{}:{}",
                deal.platform.name(),
                deal.title
                    .to_lowercase()
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            );
            if seen.insert(norm_key) {
                deduped.push(deal);
            }
        }

        // S'assurer qu'au moins 1 ou 2 offres majeures actives sont marquées 'is_featured'
        let mut featured_count = 0;
        for d in &mut deduped {
            if d.is_active_now && d.deal_type == DealType::FreeToKeep && featured_count < 2 {
                d.is_featured = true;
                featured_count += 1;
            }
        }
        if featured_count == 0 && !deduped.is_empty() {
            deduped[0].is_featured = true;
        }

        // Sauvegarder dans le cache local
        Self::save_cache_to_disk(&deduped);

        deduped
    }

    /// 1. Source Epic Games Store officielle
    pub fn fetch_epic_free_games() -> Result<Vec<FreeDeal>, String> {
        let url = "https://store-site-backend-static-ipv4.ak.epicgames.com/freeGamesPromotions?locale=fr-FR&country=FR&allowCountries=FR";

        let response = ureq::get(url)
            .timeout(Duration::from_secs(8))
            .call()
            .map_err(|e| format!("Erreur réseau Epic Games: {e}"))?;

        let body = response
            .into_string()
            .map_err(|e| format!("Erreur lecture réponse: {e}"))?;

        let val: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| format!("Erreur parsing JSON: {e}"))?;

        let mut deals = vec![];

        if let Some(elements) = val["data"]["Catalog"]["searchStore"]["elements"].as_array() {
            for el in elements {
                let title = el["title"].as_str().unwrap_or("Jeu inconnu").to_string();
                let description = el["description"].as_str().unwrap_or("").to_string();
                let id = format!("epic_{}", el["id"].as_str().unwrap_or_else(|| &title));

                let mut image_url = None;
                if let Some(images) = el["keyImages"].as_array() {
                    for img in images {
                        let t = img["type"].as_str().unwrap_or("");
                        if t == "DieselStoreFrontWide" || t == "OfferImageWide" || t == "Thumbnail" {
                            image_url = img["url"].as_str().map(|s| s.to_string());
                            break;
                        }
                    }
                    if image_url.is_none() && !images.is_empty() {
                        image_url = images[0]["url"].as_str().map(|s| s.to_string());
                    }
                }

                let product_slug = el["productSlug"].as_str()
                    .or_else(|| el["urlSlug"].as_str())
                    .unwrap_or("");

                let store_url = if product_slug.is_empty() {
                    "https://store.epicgames.com/fr/free-games".to_string()
                } else {
                    format!("https://store.epicgames.com/fr/p/{product_slug}")
                };

                let original_price = el["price"]["totalPrice"]["fmtPrice"]["originalPrice"]
                    .as_str()
                    .map(|s| s.to_string());

                let mut is_active = false;
                let mut start_date = None;
                let mut end_date = None;
                let mut end_timestamp = None;

                // 1a. Offres actives en cours
                if let Some(promos) = el["promotions"]["promotionalOffers"].as_array() {
                    for group in promos {
                        if let Some(offers) = group["promotionalOffers"].as_array() {
                            for off in offers {
                                let disc_type = off["discountSetting"]["discountType"].as_str().unwrap_or("");
                                let disc_pct = off["discountSetting"]["discountPercentage"].as_i64().unwrap_or(0);
                                if disc_type == "PERCENTAGE" && disc_pct == 0 {
                                    is_active = true;
                                    start_date = off["startDate"].as_str().map(format_date);
                                    let raw_end = off["endDate"].as_str().unwrap_or("");
                                    end_date = Some(format_date(raw_end));
                                    end_timestamp = parse_iso_date_to_timestamp(raw_end);
                                    break;
                                }
                            }
                        }
                    }
                }

                // 1b. Prochaines offres gratuites
                if !is_active {
                    if let Some(upcomings) = el["promotions"]["upcomingPromotionalOffers"].as_array() {
                        for group in upcomings {
                            if let Some(offers) = group["promotionalOffers"].as_array() {
                                for off in offers {
                                    let disc_type = off["discountSetting"]["discountType"].as_str().unwrap_or("");
                                    let disc_pct = off["discountSetting"]["discountPercentage"].as_i64().unwrap_or(0);
                                    if disc_type == "PERCENTAGE" && disc_pct == 0 {
                                        start_date = off["startDate"].as_str().map(format_date);
                                        let raw_end = off["endDate"].as_str().unwrap_or("");
                                        end_date = Some(format_date(raw_end));
                                        end_timestamp = parse_iso_date_to_timestamp(raw_end);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                if is_active || start_date.is_some() {
                    deals.push(FreeDeal {
                        id,
                        title,
                        description,
                        image_url,
                        store_url,
                        is_active_now: is_active,
                        start_date,
                        end_date,
                        end_timestamp,
                        original_price: original_price.or_else(|| Some("Gratuit".to_string())),
                        current_price: "0,00 €".to_string(),
                        platform: DealPlatform::Epic,
                        category: DealCategory::Game,
                        deal_type: DealType::FreeToKeep,
                        claim_instructions: Some(
                            "Offert par l'Epic Games Store. Connectez-vous avec votre compte Epic Games pour l'ajouter définitivement à votre bibliothèque."
                                .to_string(),
                        ),
                        tags: vec!["Epic Games".to_string(), "Jeu Offert".to_string()],
                        is_featured: is_active,
                        publisher_or_dev: el["seller"]["name"].as_str().map(|s| s.to_string()),
                        rating_or_popularity: None,
                    });
                }
            }
        }

        deals.sort_by_key(|a| std::cmp::Reverse(a.is_active_now));
        Ok(deals)
    }

    /// 2. Source GamerPower Giveaways (Steam, GOG, Itch, etc.)
    pub fn fetch_gamerpower_giveaways() -> Result<Vec<FreeDeal>, String> {
        let url = "https://www.gamerpower.com/api/giveaways?platform=pc";

        let response = ureq::get(url)
            .timeout(Duration::from_secs(8))
            .call()
            .map_err(|e| format!("Erreur réseau GamerPower: {e}"))?;

        let body = response
            .into_string()
            .map_err(|e| format!("Erreur lecture réponse: {e}"))?;

        let val: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| format!("Erreur parsing JSON: {e}"))?;

        let mut deals = vec![];

        if let Some(items) = val.as_array() {
            for it in items {
                let id_val = it["id"].as_i64().unwrap_or(0);
                let title = it["title"].as_str().unwrap_or("").to_string();
                if title.is_empty() {
                    continue;
                }
                let desc = it["description"].as_str().unwrap_or("").to_string();
                let instructions = it["instructions"].as_str().unwrap_or("").to_string();
                let store_url = it["open_giveaway_url"]
                    .as_str()
                    .or_else(|| it["giveaway_url"].as_str())
                    .unwrap_or("")
                    .to_string();

                let img = it["image"]
                    .as_str()
                    .or_else(|| it["thumbnail"].as_str())
                    .map(|s| s.to_string());

                let worth = it["worth"].as_str().map(|s| s.to_string());

                let platforms_str = it["platforms"].as_str().unwrap_or("");
                let platform = if platforms_str.to_lowercase().contains("steam") {
                    DealPlatform::Steam
                } else if platforms_str.to_lowercase().contains("gog") {
                    DealPlatform::Gog
                } else if platforms_str.to_lowercase().contains("epic") {
                    DealPlatform::Epic
                } else if platforms_str.to_lowercase().contains("itch") {
                    DealPlatform::Itch
                } else {
                    DealPlatform::Windows
                };

                let type_str = it["type"].as_str().unwrap_or("");
                let category = if type_str.to_lowercase().contains("dlc") {
                    DealCategory::Dlc
                } else if type_str.to_lowercase().contains("software") || type_str.to_lowercase().contains("other") {
                    DealCategory::Software
                } else {
                    DealCategory::Game
                };

                let end_str = it["end_date"].as_str().unwrap_or("N/A");
                let (end_date, end_timestamp) = if end_str != "N/A" && !end_str.is_empty() {
                    (Some(format_date(end_str)), parse_iso_date_to_timestamp(end_str))
                } else {
                    (None, None)
                };

                let is_expired = if let Some(ts) = end_timestamp {
                    ts <= current_unix_timestamp()
                } else {
                    false
                };

                if is_expired {
                    continue;
                }

                deals.push(FreeDeal {
                    id: format!("gp_{id_val}"),
                    title,
                    description: desc,
                    image_url: img,
                    store_url,
                    is_active_now: true,
                    start_date: None,
                    end_date,
                    end_timestamp,
                    original_price: worth,
                    current_price: "0,00 €".to_string(),
                    platform,
                    category,
                    deal_type: DealType::FreeToKeep,
                    claim_instructions: if !instructions.is_empty() {
                        Some(instructions)
                    } else {
                        None
                    },
                    tags: vec!["Bons Plans PC".to_string(), "Offre Vérifiée".to_string()],
                    is_featured: false,
                    publisher_or_dev: None,
                    rating_or_popularity: None,
                });
            }
        }

        Ok(deals)
    }

    /// 3. Top Free-to-play PC reconnus via FreeToGame
    pub fn fetch_freetogame_top() -> Result<Vec<FreeDeal>, String> {
        let url = "https://www.freetogame.com/api/games?platform=pc&sort-by=popularity";

        let response = ureq::get(url)
            .timeout(Duration::from_secs(8))
            .call()
            .map_err(|e| format!("Erreur réseau FreeToGame: {e}"))?;

        let body = response
            .into_string()
            .map_err(|e| format!("Erreur lecture réponse: {e}"))?;

        let val: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| format!("Erreur parsing JSON: {e}"))?;

        let mut deals = vec![];

        if let Some(items) = val.as_array() {
            // Ne garder que le top 16 des jeux gratuits les plus populaires
            for it in items.iter().take(16) {
                let id_val = it["id"].as_i64().unwrap_or(0);
                let title = it["title"].as_str().unwrap_or("").to_string();
                let desc = it["short_description"].as_str().unwrap_or("").to_string();
                let img = it["thumbnail"].as_str().map(|s| s.to_string());
                let game_url = it["game_url"].as_str().unwrap_or("").to_string();
                let genre = it["genre"].as_str().unwrap_or("Action").to_string();
                let publisher = it["publisher"].as_str().map(|s| s.to_string());

                deals.push(FreeDeal {
                    id: format!("f2p_{id_val}"),
                    title,
                    description: desc,
                    image_url: img,
                    store_url: game_url,
                    is_active_now: true,
                    start_date: None,
                    end_date: None,
                    end_timestamp: None,
                    original_price: Some("Free-to-Play".to_string()),
                    current_price: "0,00 €".to_string(),
                    platform: DealPlatform::Steam,
                    category: DealCategory::Game,
                    deal_type: DealType::FreeToPlay,
                    claim_instructions: Some(
                        "Jeu gratuit Free-to-Play complet disponible sans abonnement ni achat préalable."
                            .to_string(),
                    ),
                    tags: vec![genre, "Free-to-Play".to_string(), "Multijoueur".to_string()],
                    is_featured: false,
                    publisher_or_dev: publisher,
                    rating_or_popularity: Some(4.8),
                });
            }
        }

        Ok(deals)
    }

    /// 4. Logiciels et applications indispensables 100% gratuits et vérifiés
    pub fn curated_software_and_tools() -> Vec<FreeDeal> {
        vec![
            FreeDeal {
                id: "soft_blender".to_string(),
                title: "Blender 3D".to_string(),
                description: "Suite complète de création 3D open source : modélisation, animation, simulation, rendu réaliste et compositing vidéo.".to_string(),
                image_url: Some("https://download.blender.org/branding/blender_logo_socket.png".to_string()),
                store_url: "https://www.blender.org/download/".to_string(),
                is_active_now: true,
                start_date: None,
                end_date: None,
                end_timestamp: None,
                original_price: Some("Open Source".to_string()),
                current_price: "0,00 €".to_string(),
                platform: DealPlatform::OpenSource,
                category: DealCategory::Creative,
                deal_type: DealType::OpenSource,
                claim_instructions: Some("Téléchargement officiel gratuit sans aucune licence payante requise.".to_string()),
                tags: vec!["3D".to_string(), "Création".to_string(), "Open Source".to_string(), "Animation".to_string()],
                is_featured: false,
                publisher_or_dev: Some("Blender Foundation".to_string()),
                rating_or_popularity: Some(4.9),
            },
            FreeDeal {
                id: "soft_obs".to_string(),
                title: "OBS Studio".to_string(),
                description: "Le logiciel de référence pour l'enregistrement vidéo haute fidélité et la diffusion en direct (Twitch, YouTube, Discord).".to_string(),
                image_url: Some("https://obsproject.com/assets/images/new_icon_large.png".to_string()),
                store_url: "https://obsproject.com/fr/download".to_string(),
                is_active_now: true,
                start_date: None,
                end_date: None,
                end_timestamp: None,
                original_price: Some("Open Source".to_string()),
                current_price: "0,00 €".to_string(),
                platform: DealPlatform::OpenSource,
                category: DealCategory::Creative,
                deal_type: DealType::OpenSource,
                claim_instructions: Some("Logiciel libre, sans filigrane ni publicité.".to_string()),
                tags: vec!["Streaming".to_string(), "Enregistrement".to_string(), "Open Source".to_string()],
                is_featured: false,
                publisher_or_dev: Some("OBS Project".to_string()),
                rating_or_popularity: Some(4.9),
            },
            FreeDeal {
                id: "soft_powertoys".to_string(),
                title: "Microsoft PowerToys".to_string(),
                description: "Ensemble d'utilitaires système indispensables développé par Microsoft : FancyZones, Text Extractor, PowerToys Run et ColorPicker.".to_string(),
                image_url: Some("https://raw.githubusercontent.com/microsoft/PowerToys/main/doc/images/overview/PowerToys_Overview_Hero.png".to_string()),
                store_url: "https://github.com/microsoft/PowerToys/releases".to_string(),
                is_active_now: true,
                start_date: None,
                end_date: None,
                end_timestamp: None,
                original_price: Some("Gratuit".to_string()),
                current_price: "0,00 €".to_string(),
                platform: DealPlatform::Windows,
                category: DealCategory::Productivity,
                deal_type: DealType::OpenSource,
                claim_instructions: Some("Disponible directement sur le GitHub officiel de Microsoft ou via winget.".to_string()),
                tags: vec!["Windows".to_string(), "Productivité".to_string(), "Utilitaire".to_string()],
                is_featured: false,
                publisher_or_dev: Some("Microsoft".to_string()),
                rating_or_popularity: Some(4.8),
            },
            FreeDeal {
                id: "soft_davinci".to_string(),
                title: "DaVinci Resolve".to_string(),
                description: "Solution hollywoodienne de montage vidéo 8K, étalonnage de couleur de renommée mondiale, effets visuels et post-production audio.".to_string(),
                image_url: Some("https://images.blackmagicdesign.com/images/products/davinciresolve/landing/hero.jpg".to_string()),
                store_url: "https://www.blackmagicdesign.com/fr/products/davinciresolve".to_string(),
                is_active_now: true,
                start_date: None,
                end_date: None,
                end_timestamp: None,
                original_price: Some("Valeur : 295 €".to_string()),
                current_price: "0,00 €".to_string(),
                platform: DealPlatform::Windows,
                category: DealCategory::Creative,
                deal_type: DealType::OpenSource,
                claim_instructions: Some("La version complète standard de DaVinci Resolve est 100% gratuite sans abonnement.".to_string()),
                tags: vec!["Montage Vidéo".to_string(), "Étalonnage".to_string(), "Professionnel".to_string()],
                is_featured: false,
                publisher_or_dev: Some("Blackmagic Design".to_string()),
                rating_or_popularity: Some(4.9),
            },
            FreeDeal {
                id: "soft_godot".to_string(),
                title: "Godot Engine 4".to_string(),
                description: "Moteur de jeu vidéo 2D et 3D complet, puissant et léger, distribué sous licence libre MIT sans aucune redevance sur vos ventes.".to_string(),
                image_url: Some("https://godotengine.org/assets/og_image.png".to_string()),
                store_url: "https://godotengine.org/download/windows/".to_string(),
                is_active_now: true,
                start_date: None,
                end_date: None,
                end_timestamp: None,
                original_price: Some("Open Source".to_string()),
                current_price: "0,00 €".to_string(),
                platform: DealPlatform::OpenSource,
                category: DealCategory::DevTool,
                deal_type: DealType::OpenSource,
                claim_instructions: Some("100% open source sous licence MIT, aucun compte requis.".to_string()),
                tags: vec!["Moteur de jeu".to_string(), "2D / 3D".to_string(), "Open Source".to_string()],
                is_featured: false,
                publisher_or_dev: Some("Godot Community".to_string()),
                rating_or_popularity: Some(4.9),
            },
            FreeDeal {
                id: "soft_sharex".to_string(),
                title: "ShareX".to_string(),
                description: "Outil ultra-rapide de capture d'écran, enregistrement GIF/vidéo, reconnaissance de texte OCR et téléversement automatisé.".to_string(),
                image_url: Some("https://getsharex.com/img/ShareX_Logo.png".to_string()),
                store_url: "https://getsharex.com/".to_string(),
                is_active_now: true,
                start_date: None,
                end_date: None,
                end_timestamp: None,
                original_price: Some("Open Source".to_string()),
                current_price: "0,00 €".to_string(),
                platform: DealPlatform::Windows,
                category: DealCategory::Productivity,
                deal_type: DealType::OpenSource,
                claim_instructions: Some("Téléchargement direct ou via Steam et Windows Store.".to_string()),
                tags: vec!["Capture".to_string(), "OCR".to_string(), "Utilitaire Windows".to_string()],
                is_featured: false,
                publisher_or_dev: Some("ShareX Team".to_string()),
                rating_or_popularity: Some(4.8),
            },
            FreeDeal {
                id: "soft_krita".to_string(),
                title: "Krita Digital Painting".to_string(),
                description: "Studio professionnel de peinture numérique et d'illustration 2D conçu pour les illustrateurs de bandes dessinées et concept artists.".to_string(),
                image_url: Some("https://krita.org/wp-content/themes/krita-org-theme/images/banner.jpg".to_string()),
                store_url: "https://krita.org/fr/telechargement/krita-desktop/".to_string(),
                is_active_now: true,
                start_date: None,
                end_date: None,
                end_timestamp: None,
                original_price: Some("Open Source".to_string()),
                current_price: "0,00 €".to_string(),
                platform: DealPlatform::OpenSource,
                category: DealCategory::Creative,
                deal_type: DealType::OpenSource,
                claim_instructions: Some("Logiciel libre et gratuit maintenu par la Fondation Krita.".to_string()),
                tags: vec!["Dessin".to_string(), "Illustration".to_string(), "Peinture 2D".to_string()],
                is_featured: false,
                publisher_or_dev: Some("Krita Foundation".to_string()),
                rating_or_popularity: Some(4.8),
            },
            FreeDeal {
                id: "soft_7zip".to_string(),
                title: "7-Zip".to_string(),
                description: "Archiveur de fichiers avec taux de compression exceptionnel au format 7z, prise en charge LZMA et chiffrement AES-256 fort.".to_string(),
                image_url: None,
                store_url: "https://www.7-zip.org/download.html".to_string(),
                is_active_now: true,
                start_date: None,
                end_date: None,
                end_timestamp: None,
                original_price: Some("Open Source".to_string()),
                current_price: "0,00 €".to_string(),
                platform: DealPlatform::Windows,
                category: DealCategory::Productivity,
                deal_type: DealType::OpenSource,
                claim_instructions: Some("Utilitaire système gratuit à usage personnel et commercial.".to_string()),
                tags: vec!["Compression".to_string(), "Archive".to_string(), "Utilitaire".to_string()],
                is_featured: false,
                publisher_or_dev: Some("Igor Pavlov".to_string()),
                rating_or_popularity: Some(4.9),
            },
        ]
    }

    /// 5. Découvertes et pépites indépendantes gratuites
    pub fn curated_indie_gems() -> Vec<FreeDeal> {
        vec![
            FreeDeal {
                id: "indie_openttd".to_string(),
                title: "OpenTTD".to_string(),
                description: "Simulation d'empire ferroviaire et de transport en temps réel mythique, hautement modulable et avec support multijoueur.".to_string(),
                image_url: Some("https://cdn.cloudflare.steamstatic.com/steam/apps/1536610/header.jpg".to_string()),
                store_url: "https://store.steampowered.com/app/1536610/OpenTTD/".to_string(),
                is_active_now: true,
                start_date: None,
                end_date: None,
                end_timestamp: None,
                original_price: Some("Gratuit".to_string()),
                current_price: "0,00 €".to_string(),
                platform: DealPlatform::Steam,
                category: DealCategory::Indie,
                deal_type: DealType::FreeToKeep,
                claim_instructions: Some("Ajoutez-le à votre bibliothèque Steam en un clic pour y jouer immédiatement.".to_string()),
                tags: vec!["Gestion".to_string(), "Trains".to_string(), "Indépendant".to_string(), "Stratégie".to_string()],
                is_featured: false,
                publisher_or_dev: Some("OpenTTD Team".to_string()),
                rating_or_popularity: Some(4.9),
            },
            FreeDeal {
                id: "indie_mindustry".to_string(),
                title: "Mindustry".to_string(),
                description: "Jeu hybride captivant mêlant automatisation d'usines avec convoyeurs à bande et défense de base contre des vagues d'ennemis.".to_string(),
                image_url: Some("https://cdn.cloudflare.steamstatic.com/steam/apps/1127400/header.jpg".to_string()),
                store_url: "https://anuke.itch.io/mindustry".to_string(),
                is_active_now: true,
                start_date: None,
                end_date: None,
                end_timestamp: None,
                original_price: Some("Prix libre".to_string()),
                current_price: "0,00 €".to_string(),
                platform: DealPlatform::Itch,
                category: DealCategory::Indie,
                deal_type: DealType::FreeToKeep,
                claim_instructions: Some("Téléchargement gratuit sans DRM sur itch.io.".to_string()),
                tags: vec!["Usine".to_string(), "Tower Defense".to_string(), "Automatisation".to_string()],
                is_featured: false,
                publisher_or_dev: Some("Anuke".to_string()),
                rating_or_popularity: Some(4.9),
            },
            FreeDeal {
                id: "indie_endless_sky".to_string(),
                title: "Endless Sky".to_string(),
                description: "Space opera 2D en monde ouvert inspiré d'Escape Velocity : commerce galactique, batailles spatiales et guerre civile extraterrestre.".to_string(),
                image_url: Some("https://cdn.cloudflare.steamstatic.com/steam/apps/404410/header.jpg".to_string()),
                store_url: "https://store.steampowered.com/app/404410/Endless_Sky/".to_string(),
                is_active_now: true,
                start_date: None,
                end_date: None,
                end_timestamp: None,
                original_price: Some("Gratuit".to_string()),
                current_price: "0,00 €".to_string(),
                platform: DealPlatform::Steam,
                category: DealCategory::Indie,
                deal_type: DealType::FreeToKeep,
                claim_instructions: Some("Jeu complet sans microtransactions, disponible sur Steam et GitHub.".to_string()),
                tags: vec!["Espace".to_string(), "RPG".to_string(), "Exploration".to_string(), "Bac à sable".to_string()],
                is_featured: false,
                publisher_or_dev: Some("Michael Zahniser".to_string()),
                rating_or_popularity: Some(4.8),
            },
            FreeDeal {
                id: "indie_shattered_pixel".to_string(),
                title: "Shattered Pixel Dungeon".to_string(),
                description: "Le rogue-like classique au tour par tour par excellence : 5 classes distinctes, donjons générés procéduralement et haute rejouabilité.".to_string(),
                image_url: Some("https://cdn.cloudflare.steamstatic.com/steam/apps/1769170/header.jpg".to_string()),
                store_url: "https://github.com/00-Evan/shattered-pixel-dungeon/releases".to_string(),
                is_active_now: true,
                start_date: None,
                end_date: None,
                end_timestamp: None,
                original_price: Some("Gratuit (GitHub)".to_string()),
                current_price: "0,00 €".to_string(),
                platform: DealPlatform::OpenSource,
                category: DealCategory::Indie,
                deal_type: DealType::OpenSource,
                claim_instructions: Some("Téléchargement gratuit de la version PC sur le dépôt GitHub officiel du développeur.".to_string()),
                tags: vec!["Rogue-like".to_string(), "Pixel Art".to_string(), "Donjon".to_string()],
                is_featured: false,
                publisher_or_dev: Some("00-Evan".to_string()),
                rating_or_popularity: Some(4.9),
            },
        ]
    }

    /// Données initiales riches pour affichage immédiat hors-ligne
    pub fn fallback_curated_deals() -> Vec<FreeDeal> {
        let mut list = Vec::new();

        // Une offre phare vedette
        list.push(FreeDeal {
            id: "featured_epic_highlight".to_string(),
            title: "Jeux Offerts de la Semaine".to_string(),
            description: "Chaque jeudi à 17h, l'Epic Games Store propose un ou plusieurs jeux PC complets gratuits à conserver à vie dans votre compte.".to_string(),
            image_url: Some("https://cdn1.epicgames.com/offer/d5241c76f178492ea1540fce4561677a/Diesel_Hero_Epic_Store_1920x1080.jpg".to_string()),
            store_url: "https://store.epicgames.com/fr/free-games".to_string(),
            is_active_now: true,
            start_date: None,
            end_date: Some("Chaque jeudi à 17h00".to_string()),
            end_timestamp: None,
            original_price: Some("Jusqu'à 59,99 €".to_string()),
            current_price: "0,00 €".to_string(),
            platform: DealPlatform::Epic,
            category: DealCategory::Game,
            deal_type: DealType::FreeToKeep,
            claim_instructions: Some("Connectez-vous à votre compte Epic Games Store et ajoutez les jeux du jeudi à votre bibliothèque sans frais.".to_string()),
            tags: vec!["Epic Games".to_string(), "Sélection Hebdo".to_string(), "À conserver".to_string()],
            is_featured: true,
            publisher_or_dev: Some("Epic Games Store".to_string()),
            rating_or_popularity: Some(5.0),
        });

        list.extend(Self::curated_software_and_tools());
        list.extend(Self::curated_indie_gems());
        list
    }
}

// ───────────────────────────── Utilitaires de dates ──────────────────────────

/// Convertit un format "2026-11-20T16:00:00.000Z" ou "2026-11-20 23:59:00" en "20/11 à 16h00"
pub fn format_date(iso: &str) -> String {
    let clean = iso.trim();
    if clean.len() >= 10 {
        let date_part = &clean[..10];
        let parts: Vec<&str> = date_part.split('-').collect();
        if parts.len() == 3 {
            let time = if clean.len() >= 16 {
                let t = &clean[11..16];
                format!(" à {}", t.replace(':', "h"))
            } else {
                String::new()
            };
            return format!("{}/{}{}", parts[2], parts[1], time);
        }
    }
    clean.to_string()
}

/// Convertit une chaîne de date ISO ou DateTime en timestamp UNIX en secondes
pub fn parse_iso_date_to_timestamp(s: &str) -> Option<i64> {
    let clean = s.trim();
    if clean.len() < 10 {
        return None;
    }
    let date_part = &clean[..10];
    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() != 3 {
        return None;
    }

    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;

    let (hour, min, sec) = if clean.len() >= 19 && (clean.as_bytes()[10] == b'T' || clean.as_bytes()[10] == b' ') {
        let time_part = &clean[11..19];
        let tparts: Vec<&str> = time_part.split(':').collect();
        if tparts.len() == 3 {
            (
                tparts[0].parse().unwrap_or(0),
                tparts[1].parse().unwrap_or(0),
                tparts[2].parse().unwrap_or(0),
            )
        } else {
            (0, 0, 0)
        }
    } else {
        (0, 0, 0)
    };

    Some(date_to_unix_timestamp(year, month, day, hour, min, sec))
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn date_to_unix_timestamp(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> i64 {
    let mut days: i64 = 0;
    for y in 1970..year {
        days += if is_leap_year(y) { 366 } else { 365 };
    }
    let month_days = [31, if is_leap_year(year) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for &d in month_days.iter().take((month.saturating_sub(1) as usize).min(11)) {
        days += d as i64;
    }
    days += (day.saturating_sub(1)) as i64;
    days * 86400 + (hour as i64) * 3600 + (min as i64) * 60 + (sec as i64)
}

// ───────────────────────────── Tests Unitaires ───────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_curated_deals_not_empty() {
        let deals = DealsTracker::fallback_curated_deals();
        assert!(!deals.is_empty(), "Les bons plans initiaux doivent contenir des offres");

        // Vérifie la présence d'au moins un logiciel et une pépite indépendante
        let has_software = deals.iter().any(|d| {
            d.category == DealCategory::Creative
                || d.category == DealCategory::Productivity
                || d.category == DealCategory::DevTool
        });
        assert!(has_software, "Doit inclure des logiciels indispensables");

        let has_indie = deals.iter().any(|d| d.category == DealCategory::Indie);
        assert!(has_indie, "Doit inclure des pépites indépendantes");
    }

    #[test]
    fn test_date_formatting() {
        let iso = "2026-11-20T16:00:00.000Z";
        let formatted = format_date(iso);
        assert_eq!(formatted, "20/11 à 16h00");

        let date_only = "2026-12-25";
        let formatted_short = format_date(date_only);
        assert_eq!(formatted_short, "25/12");
    }

    #[test]
    fn test_iso_parsing_to_timestamp() {
        let iso = "1970-01-01T01:00:00Z";
        let ts = parse_iso_date_to_timestamp(iso).expect("Parsing timestamp valide");
        assert_eq!(ts, 3600);

        let iso2 = "2026-11-20 18:30:00";
        let ts2 = parse_iso_date_to_timestamp(iso2).expect("Parsing timestamp space");
        assert!(ts2 > 1_700_000_000);
    }

    #[test]
    fn test_expiry_and_remaining_time() {
        let now = current_unix_timestamp();

        // Offre déjà expirée
        let expired_deal = FreeDeal {
            id: "test_exp".to_string(),
            title: "Jeu Expiré".to_string(),
            description: "".to_string(),
            image_url: None,
            store_url: "".to_string(),
            is_active_now: true,
            start_date: None,
            end_date: None,
            end_timestamp: Some(now - 3600),
            original_price: None,
            current_price: "0,00 €".to_string(),
            platform: DealPlatform::Steam,
            category: DealCategory::Game,
            deal_type: DealType::FreeToKeep,
            claim_instructions: None,
            tags: vec![],
            is_featured: false,
            publisher_or_dev: None,
            rating_or_popularity: None,
        };
        assert!(expired_deal.is_expired());
        assert!(!expired_deal.expires_soon(24));
        assert!(expired_deal.remaining_time().is_none());

        // Offre qui expire dans 2 heures
        let active_deal = FreeDeal {
            id: "test_act".to_string(),
            title: "Jeu Actif".to_string(),
            description: "".to_string(),
            image_url: None,
            store_url: "".to_string(),
            is_active_now: true,
            start_date: None,
            end_date: None,
            end_timestamp: Some(now + 7200),
            original_price: Some("19,99 €".to_string()),
            current_price: "0,00 €".to_string(),
            platform: DealPlatform::Epic,
            category: DealCategory::Game,
            deal_type: DealType::FreeToKeep,
            claim_instructions: None,
            tags: vec![],
            is_featured: true,
            publisher_or_dev: None,
            rating_or_popularity: None,
        };
        assert!(!active_deal.is_expired());
        assert!(active_deal.expires_soon(24));
        let rem = active_deal.remaining_time().expect("Temps restant valide");
        assert_eq!(rem.0, 0); // 0 jours
        assert_eq!(rem.1, 2); // 2 heures
        assert!(active_deal.remaining_time_str().is_some());
    }

    #[test]
    fn test_cache_serialization() {
        let deals = DealsTracker::fallback_curated_deals();
        let cache = DealsCache {
            last_updated_secs: 12345678,
            deals: deals.clone(),
        };
        let json = serde_json::to_string(&cache).expect("Sérialisation cache");
        let decoded: DealsCache = serde_json::from_str(&json).expect("Désérialisation cache");
        assert_eq!(decoded.last_updated_secs, 12345678);
        assert_eq!(decoded.deals.len(), deals.len());
    }
}
