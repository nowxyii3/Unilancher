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
    pub expected_sha256: Option<String>,
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
                    }
                }
            }
        }
    }

    /// Déclenche la recherche asynchrone d'une mise à jour selon le canal configuré
    pub fn check_for_updates(&mut self, channel_type: crate::config::UpdateChannel) {
        {
            let mut status = self.status.lock().unwrap();
            *status = UpdateStatus::Checking;
        }

        let (tx, rx) = channel();
        self.event_rx = Some(rx);

        thread::spawn(move || {
            match fetch_release_for_channel(GITHUB_REPO_OWNER, GITHUB_REPO_NAME, channel_type) {
                Ok(release) => {
                    let clean_target = clean_version_str(&release.tag_name);
                    if is_newer_version(CURRENT_VERSION, &clean_target) {
                        // Chercher en priorité l'installateur setup .exe
                        let asset = release
                            .assets
                            .iter()
                            .find(|a| {
                                let name = a.name.to_lowercase();
                                name.ends_with(".exe")
                                    && (name.contains("setup") || name.contains("installer") || name.contains("unilauncher"))
                            })
                            .or_else(|| {
                                // Fallback : archive ZIP
                                release.assets.iter().find(|a| {
                                    let name = a.name.to_lowercase();
                                    name.ends_with(".zip") && (name.contains("windows") || name.contains("win"))
                                })
                            });

                        if let Some(asset) = asset {
                            // Récupère l'empreinte SHA-256 attendue depuis checksums.txt si disponible
                            let expected_sha256 = fetch_expected_checksum(&release, &asset.name);

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
                                expected_sha256,
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

    /// Déclenche le téléchargement et la vérification cryptographique du package d'installation
    pub fn start_download(&mut self, update: AvailableUpdate) {
        let (tx, rx) = channel();
        self.download_rx = Some(rx);

        let version = update.version.clone();
        let url = update.download_url.clone();
        let total_size = update.asset_size;
        let expected_sha256 = update.expected_sha256.clone();

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

            match download_and_verify_file(&url, &dest_file, total_size, expected_sha256.as_deref(), &tx) {
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

/// Récupère la release cible selon le canal (Stable vs BETA)
pub fn fetch_release_for_channel(
    owner: &str,
    repo: &str,
    channel: crate::config::UpdateChannel,
) -> Result<GitHubRelease, String> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases");
    let resp = ureq::get(&url)
        .set("User-Agent", "UniLauncher-Client")
        .set("Accept", "application/vnd.github.v3+json")
        .timeout(Duration::from_secs(10))
        .call()
        .map_err(|e| format!("Erreur réseau GitHub ({e})"))?;

    let releases: Vec<GitHubRelease> = resp
        .into_json()
        .map_err(|e| format!("Impossible d'analyser la liste des releases: {e}"))?;

    let filtered = match channel {
        crate::config::UpdateChannel::Stable => {
            // Canal Stable : releases non-draft, non-prerelease et sans tag "beta"
            releases
                .into_iter()
                .find(|r| !r.draft && !r.prerelease && !r.tag_name.to_lowercase().contains("beta"))
        }
        crate::config::UpdateChannel::Beta => {
            // Canal BETA : première release publiée (stable ou beta)
            releases.into_iter().find(|r| !r.draft)
        }
    };

    filtered.ok_or_else(|| "Aucune version correspondante trouvée sur le dépôt.".to_string())
}

/// Extrait l'empreinte SHA-256 d'un asset depuis `checksums.txt`
fn fetch_expected_checksum(release: &GitHubRelease, asset_name: &str) -> Option<String> {
    use std::io::Read;

    let c_asset = release.assets.iter().find(|a| a.name.to_lowercase() == "checksums.txt")?;
    let resp = ureq::get(&c_asset.browser_download_url)
        .set("User-Agent", "UniLauncher-Verifier")
        .timeout(Duration::from_secs(5))
        .call()
        .ok()?;

    let mut content = String::new();
    resp.into_reader().read_to_string(&mut content).ok()?;

    for line in content.lines() {
        let mut parts = line.split_whitespace();
        let hash = parts.next()?;
        let name = parts.next()?;
        if name.eq_ignore_ascii_case(asset_name) {
            return Some(hash.to_lowercase());
        }
    }

    None
}

/// Télécharge un fichier HTTPS avec vérification de l'URL et validation cryptographique SHA-256
fn download_and_verify_file(
    url: &str,
    destination: &Path,
    total_hint: u64,
    expected_sha256: Option<&str>,
    tx: &Sender<DownloadEvent>,
) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    use std::fs::File;
    use std::io::{Read, Write};

    // 1. Validation de sécurité de l'URL source
    if !url.starts_with("https://github.com/nowxyii3/Unilancher/")
        && !url.starts_with("https://objects.githubusercontent.com/")
    {
        return Err(format!("URL de téléchargement rejetée par sécurité : {url}"));
    }

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
    let mut file = File::create(destination)
        .map_err(|e| format!("Impossible de créer le fichier temporaire: {e}"))?;

    let mut hasher = Sha256::new();
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
        hasher.update(&buffer[..bytes_read]);
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

    // 2. Vérification cryptographique de l'intégrité SHA-256
    let calculated_sha256 = format!("{:x}", hasher.finalize()).to_lowercase();
    if let Some(expected) = expected_sha256 {
        let clean_expected = expected.trim().to_lowercase();
        if !clean_expected.is_empty() && calculated_sha256 != clean_expected {
            let _ = std::fs::remove_file(destination);
            return Err(format!(
                "Échec de validation d'intégrité SHA-256 ! Empreinte calculée : {}, attendue : {}",
                calculated_sha256, clean_expected
            ));
        }
    }

    Ok(())
}

/// Nettoie une chaîne de version pour enlever les 'v' ou préfixes
pub fn clean_version_str(v: &str) -> String {
    v.trim().trim_start_matches(['v', 'V']).to_string()
}

/// Analyse SemVer avec prise en charge complète des tags pré-release / beta
pub fn parse_version_semver(v: &str) -> Option<(u32, u32, u32, Option<u32>)> {
    let clean = clean_version_str(v);
    let mut parts = clean.split('-');
    let core = parts.next()?;
    let prerelease = parts.next();

    let mut core_nums = core.split('.');
    let major: u32 = core_nums.next()?.parse().ok()?;
    let minor: u32 = core_nums.next().unwrap_or("0").parse().ok()?;
    let patch: u32 = core_nums.next().unwrap_or("0").parse().ok()?;

    let beta_num: Option<u32> = prerelease.and_then(|pr| {
        let digits: String = pr.chars().filter(|c| c.is_ascii_digit()).collect();
        digits.parse().ok().or(Some(0))
    });

    Some((major, minor, patch, beta_num))
}

/// Analyse une chaîne SemVer en triplet (majeur, mineur, patch)
#[allow(dead_code)]
pub fn parse_version(v: &str) -> Option<(u32, u32, u32)> {
    let (maj, min, pat, _) = parse_version_semver(v)?;
    Some((maj, min, pat))
}

/// Compare deux versions : renvoie true si target est strictement plus récente que current
pub fn is_newer_version(current: &str, target: &str) -> bool {
    let (c_maj, c_min, c_pat, c_beta) = match parse_version_semver(current) {
        Some(v) => v,
        None => return false,
    };
    let (t_maj, t_min, t_pat, t_beta) = match parse_version_semver(target) {
        Some(v) => v,
        None => return false,
    };

    if (t_maj, t_min, t_pat) > (c_maj, c_min, c_pat) {
        return true;
    }
    if (t_maj, t_min, t_pat) < (c_maj, c_min, c_pat) {
        return false;
    }

    // Même triplet majeur.mineur.patch
    match (c_beta, t_beta) {
        (Some(_), None) => true,       // Target est une version finale stable -> mise à jour recommandée
        (None, Some(_)) => false,      // Target est une beta, current est stable -> ne pas proposer de régression
        (Some(cb), Some(tb)) => tb > cb, // Les deux sont beta -> compare le numéro beta
        (None, None) => false,         // Versions stables identiques
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
        assert_eq!(parse_version("1.0.0-beta.1"), Some((1, 0, 0)));
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

        // Tests beta
        assert!(is_newer_version("1.1.0-beta.1", "1.1.0-beta.2"));
        assert!(is_newer_version("1.1.0-beta.1", "1.1.0"));
        assert!(!is_newer_version("1.1.0", "1.1.0-beta.1"));
        assert!(!is_newer_version("1.1.0-beta.2", "1.1.0-beta.1"));
        assert!(is_newer_version("1.0.0", "1.1.0-beta.1"));
    }
}
