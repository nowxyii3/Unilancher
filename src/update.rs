use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub const GITHUB_REPO_OWNER: &str = "nowxyii3";
pub const GITHUB_REPO_NAME: &str = "Unilancher";
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub size: u64,
    pub browser_download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub published_at: Option<String>,
    #[serde(default)]
    pub assets: Vec<ReleaseAsset>,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub prerelease: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AvailableUpdate {
    pub version: String,
    pub tag_name: String,
    pub title: String,
    pub changelog: String,
    pub published_at: String,
    pub download_url: String,
    pub asset_name: String,
    pub asset_size: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateStatus {
    Idle,
    Checking,
    UpToDate {
        version: String,
    },
    UpdateAvailable(AvailableUpdate),
    Downloading {
        version: String,
        percent: f32,
        downloaded_bytes: u64,
        total_bytes: u64,
    },
    ReadyToRestart {
        version: String,
        installer_path: PathBuf,
    },
    Error(String),
}

pub struct UpdateManager {
    pub status: Arc<Mutex<UpdateStatus>>,
    event_rx: Option<Receiver<UpdateEvent>>,
    download_rx: Option<Receiver<DownloadEvent>>,
}

pub enum UpdateEvent {
    Found(AvailableUpdate),
    UpToDate(String),
    Failed(String),
}

pub enum DownloadEvent {
    Progress {
        percent: f32,
        downloaded: u64,
        total: u64,
    },
    Completed {
        path: PathBuf,
        version: String,
    },
    Failed(String),
}

impl Default for UpdateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateManager {
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(UpdateStatus::Idle)),
            event_rx: None,
            download_rx: None,
        }
    }

    /// Appelé à chaque frame UI pour consommer les événements asynchrones
    pub fn update(&mut self) {
        // Traitement des résultats de vérification
        if let Some(ref rx) = self.event_rx {
            if let Ok(event) = rx.try_recv() {
                let mut status = self.status.lock().unwrap();
                match event {
                    UpdateEvent::Found(update) => {
                        *status = UpdateStatus::UpdateAvailable(update);
                    }
                    UpdateEvent::UpToDate(v) => {
                        *status = UpdateStatus::UpToDate { version: v };
                    }
                    UpdateEvent::Failed(err) => {
                        *status = UpdateStatus::Error(err);
                    }
                }
                self.event_rx = None;
            }
        }

        // Traitement de la progression de téléchargement
        if let Some(ref rx) = self.download_rx {
            while let Ok(event) = rx.try_recv() {
                let mut status = self.status.lock().unwrap();
                match event {
                    DownloadEvent::Progress {
                        percent,
                        downloaded,
                        total,
                    } => {
                        let cur_ver = match &*status {
                            UpdateStatus::Downloading { version, .. } => version.clone(),
                            UpdateStatus::UpdateAvailable(u) => u.version.clone(),
                            _ => CURRENT_VERSION.to_string(),
                        };
                        *status = UpdateStatus::Downloading {
                            version: cur_ver,
                            percent,
                            downloaded_bytes: downloaded,
                            total_bytes: total,
                        };
                    }
                    DownloadEvent::Completed { path, version } => {
                        *status = UpdateStatus::ReadyToRestart {
                            version,
                            installer_path: path,
                        };
                        self.download_rx = None;
                        break;
                    }
                    DownloadEvent::Failed(err) => {
                        *status = UpdateStatus::Error(err);
                        self.download_rx = None;
                        break;
                    }
                }
            }
        }
    }

    /// Déclenche la recherche asynchrone d'une mise à jour sur GitHub Releases
    pub fn check_for_updates(&mut self) {
        {
            let mut status = self.status.lock().unwrap();
            *status = UpdateStatus::Checking;
        }

        let (tx, rx) = channel();
        self.event_rx = Some(rx);

        thread::spawn(move || {
            match fetch_latest_release(GITHUB_REPO_OWNER, GITHUB_REPO_NAME) {
                Ok(release) => {
                    let clean_target = clean_version_str(&release.tag_name);
                    if is_newer_version(CURRENT_VERSION, &clean_target) {
                        // Chercher en priorité l'installateur setup .exe
                        let asset = release
                            .assets
                            .iter()
                            .find(|a| {
                                let name = a.name.to_lowercase();
                                name.ends_with(".exe") && (name.contains("setup") || name.contains("installer") || name.contains("unilauncher"))
                            })
                            .or_else(|| {
                                // Fallback : archive ZIP
                                release.assets.iter().find(|a| {
                                    let name = a.name.to_lowercase();
                                    name.ends_with(".zip") && (name.contains("windows") || name.contains("win"))
                                })
                            });

                        if let Some(asset) = asset {
                            let title = release.name.unwrap_or_else(|| format!("UniLauncher {}", clean_target));
                            let info = AvailableUpdate {
                                version: clean_target,
                                tag_name: release.tag_name,
                                title,
                                changelog: release.body.unwrap_or_else(|| "Aucune note de mise à jour fournie.".to_string()),
                                published_at: release.published_at.unwrap_or_default(),
                                download_url: asset.browser_download_url.clone(),
                                asset_name: asset.name.clone(),
                                asset_size: asset.size,
                            };
                            let _ = tx.send(UpdateEvent::Found(info));
                        } else {
                            let _ = tx.send(UpdateEvent::Failed(
                                "Mise à jour détectée mais aucun installateur Windows n'a été trouvé dans les fichiers de la release.".to_string(),
                            ));
                        }
                    } else {
                        let _ = tx.send(UpdateEvent::UpToDate(CURRENT_VERSION.to_string()));
                    }
                }
                Err(err) => {
                    let _ = tx.send(UpdateEvent::Failed(format!("Impossible de vérifier les mises à jour : {err}")));
                }
            }
        });
    }

    /// Déclenche le téléchargement du package d'installation
    pub fn start_download(&mut self, update: AvailableUpdate) {
        let (tx, rx) = channel();
        self.download_rx = Some(rx);

        let version = update.version.clone();
        let url = update.download_url.clone();
        let total_size = update.asset_size;

        {
            let mut status = self.status.lock().unwrap();
            *status = UpdateStatus::Downloading {
                version: version.clone(),
                percent: 0.0,
                downloaded_bytes: 0,
                total_bytes: total_size,
            };
        }

        thread::spawn(move || {
            let temp_dir = std::env::temp_dir();
            let dest_file = temp_dir.join(format!("UniLauncherSetup-v{}.exe", version));

            match download_file_with_progress(&url, &dest_file, total_size, &tx) {
                Ok(_) => {
                    let _ = tx.send(DownloadEvent::Completed {
                        path: dest_file,
                        version,
                    });
                }
                Err(e) => {
                    let _ = tx.send(DownloadEvent::Failed(e));
                }
            }
        });
    }

    /// Redémarre et applique la mise à jour
    pub fn apply_update_and_restart(&self, installer_path: &Path) -> Result<(), String> {
        let current_exe = std::env::current_exe().map_err(|e| format!("Impossible d'obtenir l'exécutable actuel: {e}"))?;
        let current_dir = current_exe
            .parent()
            .ok_or_else(|| "Impossible de déterminer le dossier d'installation".to_string())?;

        let current_pid = std::process::id();

        // Lance le setup avec les arguments d'auto-update
        std::process::Command::new(installer_path)
            .args([
                "--update-apply",
                "--target-dir",
                &current_dir.to_string_lossy(),
                "--pid",
                &current_pid.to_string(),
            ])
            .spawn()
            .map_err(|e| format!("Impossible de lancer le processus de mise à jour: {e}"))?;

        // Fermeture propre de l'application
        std::process::exit(0);
    }
}

/// Interroge l'API GitHub Releases publique
pub fn fetch_latest_release(owner: &str, repo: &str) -> Result<GitHubRelease, String> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");
    let resp = ureq::get(&url)
        .set("User-Agent", "UniLauncher-Client")
        .set("Accept", "application/vnd.github.v3+json")
        .timeout(Duration::from_secs(10))
        .call()
        .map_err(|e| format!("Erreur réseau GitHub ({e})"))?;

    let release: GitHubRelease = resp
        .into_json()
        .map_err(|e| format!("Impossible d'analyser la réponse GitHub: {e}"))?;

    Ok(release)
}

/// Télécharge un fichier HTTPS avec calcul de progression
fn download_file_with_progress(
    url: &str,
    destination: &Path,
    total_hint: u64,
    tx: &Sender<DownloadEvent>,
) -> Result<(), String> {
    use std::fs::File;
    use std::io::{Read, Write};

    let resp = ureq::get(url)
        .set("User-Agent", "UniLauncher-Downloader")
        .timeout(Duration::from_secs(60))
        .call()
        .map_err(|e| format!("Échec du téléchargement HTTPS: {e}"))?;

    let content_len = resp
        .header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(total_hint);

    let mut reader = resp.into_reader();
    let mut file = File::create(destination).map_err(|e| format!("Impossible de créer le fichier temporaire: {e}"))?;

    let mut buffer = [0u8; 64 * 1024]; // 64 KB chunk
    let mut downloaded = 0u64;
    let mut last_percent = -1.0f32;

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| format!("Erreur lors de la lecture du flux: {e}"))?;
        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .map_err(|e| format!("Erreur d'écriture sur le disque: {e}"))?;
        downloaded += bytes_read as u64;

        if content_len > 0 {
            let percent = (downloaded as f32 / content_len as f32).clamp(0.0, 1.0);
            if (percent - last_percent).abs() >= 0.01 || downloaded == content_len {
                last_percent = percent;
                let _ = tx.send(DownloadEvent::Progress {
                    percent,
                    downloaded,
                    total: content_len,
                });
            }
        }
    }

    file.flush()
        .map_err(|e| format!("Erreur lors de la finalisation du fichier: {e}"))?;

    Ok(())
}

/// Nettoie une chaîne de version pour enlever les 'v' ou préfixes
pub fn clean_version_str(v: &str) -> String {
    v.trim().trim_start_matches(|c| c == 'v' || c == 'V').to_string()
}

/// Analyse une chaîne SemVer en triplet (majeur, mineur, patch)
pub fn parse_version(v: &str) -> Option<(u32, u32, u32)> {
    let clean = clean_version_str(v);
    let parts: Vec<&str> = clean.split('.').collect();
    if parts.is_empty() {
        return None;
    }

    let major = parts.first()?.split('-').next()?.parse().ok()?;
    let minor = parts
        .get(1)
        .and_then(|s| s.split('-').next()?.parse().ok())
        .unwrap_or(0);
    let patch = parts
        .get(2)
        .and_then(|s| s.split('-').next()?.parse().ok())
        .unwrap_or(0);

    Some((major, minor, patch))
}

/// Compare deux versions : renvoie true si target est strictement plus récente que current
pub fn is_newer_version(current: &str, target: &str) -> bool {
    match (parse_version(current), parse_version(target)) {
        (Some(c), Some(t)) => t > c,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        assert_eq!(parse_version("1.0.0"), Some((1, 0, 0)));
        assert_eq!(parse_version("v1.0.2"), Some((1, 0, 2)));
        assert_eq!(parse_version("V2.15.3"), Some((2, 15, 3)));
        assert_eq!(parse_version("1.2"), Some((1, 2, 0)));
        assert_eq!(parse_version("1.0.0-beta"), Some((1, 0, 0)));
    }

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("1.0.0", "1.0.1"));
        assert!(is_newer_version("1.0.0", "1.1.0"));
        assert!(is_newer_version("1.0.0", "2.0.0"));
        assert!(is_newer_version("v1.0.0", "v1.0.1"));
        assert!(!is_newer_version("1.0.0", "1.0.0"));
        assert!(!is_newer_version("1.0.1", "1.0.0"));
        assert!(!is_newer_version("2.0.0", "1.9.9"));
    }
}
