use serde::{Deserialize, Serialize};
use std::time::Duration;

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
    pub original_price: Option<String>,
}

pub struct DealsTracker;

impl DealsTracker {
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
                let id = el["id"].as_str().unwrap_or_else(|| &title).to_string();

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

                // Détecter si l'offre est active maintenant ou à venir
                let mut is_active = false;
                let mut start_date = None;
                let mut end_date = None;

                // 1. Offres actives en cours
                if let Some(promos) = el["promotions"]["promotionalOffers"].as_array() {
                    for group in promos {
                        if let Some(offers) = group["promotionalOffers"].as_array() {
                            for off in offers {
                                let disc_type = off["discountSetting"]["discountType"].as_str().unwrap_or("");
                                let disc_pct = off["discountSetting"]["discountPercentage"].as_i64().unwrap_or(0);
                                if disc_type == "PERCENTAGE" && disc_pct == 0 {
                                    is_active = true;
                                    start_date = off["startDate"].as_str().map(format_date);
                                    end_date = off["endDate"].as_str().map(format_date);
                                    break;
                                }
                            }
                        }
                    }
                }

                // 2. Prochaines offres gratuites
                if !is_active {
                    if let Some(upcomings) = el["promotions"]["upcomingPromotionalOffers"].as_array() {
                        for group in upcomings {
                            if let Some(offers) = group["promotionalOffers"].as_array() {
                                for off in offers {
                                    let disc_type = off["discountSetting"]["discountType"].as_str().unwrap_or("");
                                    let disc_pct = off["discountSetting"]["discountPercentage"].as_i64().unwrap_or(0);
                                    if disc_type == "PERCENTAGE" && disc_pct == 0 {
                                        start_date = off["startDate"].as_str().map(format_date);
                                        end_date = off["endDate"].as_str().map(format_date);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                // Ne retenir que les offres avec promotions gratuites valides
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
                        original_price,
                    });
                }
            }
        }

        // Trier : d'abord les gratuits actuellement, puis les prochains
        deals.sort_by_key(|a| std::cmp::Reverse(a.is_active_now));

        Ok(deals)
    }
}

fn format_date(iso: &str) -> String {
    // Convertir format "2026-11-13T16:00:00.000Z" -> "13/11 à 16h00"
    if iso.len() >= 16 {
        let parts: Vec<&str> = iso[..10].split('-').collect();
        let time = &iso[11..16];
        if parts.len() == 3 {
            return format!("{}/{} à {}", parts[2], parts[1], time);
        }
    }
    iso.to_string()
}
