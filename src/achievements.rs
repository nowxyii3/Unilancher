use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub unlocked: bool,
    pub unlock_time: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GameAchievements {
    pub appid: String,
    pub total: usize,
    pub unlocked_count: usize,
    pub progress_percent: f32,
    pub list: Vec<Achievement>,
}

/// Récupère les succès officiels d'un jeu Steam
pub fn fetch_steam_achievements(
    appid: &str,
    api_key: Option<&str>,
    steam_id: Option<&str>,
) -> Option<GameAchievements> {
    // 1. Si une clé API Steam et un SteamID sont configurés, interroge l'API officielle des joueurs
    if let (Some(key), Some(sid)) = (api_key, steam_id) {
        if !key.trim().is_empty() && !sid.trim().is_empty() {
            let url = format!(
                "https://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v0001/?appid={appid}&key={key}&steamid={sid}&l=french"
            );
            if let Ok(resp) = ureq::get(&url).timeout(Duration::from_secs(4)).call() {
                if let Ok(val) = resp.into_json::<serde_json::Value>() {
                    if let Some(playerstats) = val.get("playerstats") {
                        if let Some(ach_array) = playerstats.get("achievements").and_then(|a| a.as_array()) {
                            let mut list = Vec::new();
                            let mut unlocked_count = 0;
                            for item in ach_array {
                                let id = item.get("apiname").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or(&id).to_string();
                                let desc = item.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let achieved = item.get("achieved").and_then(|v| v.as_i64()).unwrap_or(0) == 1;
                                let unlock_time = item.get("unlocktime").and_then(|v| v.as_u64()).filter(|&t| t > 0);
                                if achieved {
                                    unlocked_count += 1;
                                }
                                list.push(Achievement {
                                    id,
                                    name,
                                    description: desc,
                                    icon_url: None,
                                    unlocked: achieved,
                                    unlock_time,
                                });
                            }
                            let total = list.len();
                            let pct = if total > 0 {
                                (unlocked_count as f32 / total as f32) * 100.0
                            } else {
                                0.0
                            };
                            return Some(GameAchievements {
                                appid: appid.to_string(),
                                total,
                                unlocked_count,
                                progress_percent: pct,
                                list,
                            });
                        }
                    }
                }
            }
        }
    }

    // 2. Récupère le schéma public officiel de la boutique Steam
    let schema_url = format!("https://store.steampowered.com/api/appdetails?appids={appid}&l=french");
    if let Ok(resp) = ureq::get(&schema_url).timeout(Duration::from_secs(4)).call() {
        if let Ok(val) = resp.into_json::<serde_json::Value>() {
            if let Some(data) = val.get(appid).and_then(|v| v.get("data")) {
                if let Some(ach_data) = data.get("achievements") {
                    let total = ach_data.get("total").and_then(|t| t.as_u64()).unwrap_or(0) as usize;
                    if total > 0 {
                        let mut list = Vec::new();
                        if let Some(highlighted) = ach_data.get("highlighted").and_then(|h| h.as_array()) {
                            for item in highlighted {
                                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let icon = item.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
                                list.push(Achievement {
                                    id: name.clone(),
                                    name,
                                    description: "Succès Steam officiel".into(),
                                    icon_url: icon,
                                    unlocked: false,
                                    unlock_time: None,
                                });
                            }
                        }
                        return Some(GameAchievements {
                            appid: appid.to_string(),
                            total,
                            unlocked_count: 0,
                            progress_percent: 0.0,
                            list,
                        });
                    }
                }
            }
        }
    }

    None
}
