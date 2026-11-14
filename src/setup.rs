use eframe::egui::{self, vec2, Align2, Color32, CornerRadius, FontId, Layout, Margin, Pos2, Rect, RichText, Sense, Stroke, Ui};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

pub const APP_NAME: &str = "UniLauncher";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PUBLISHER: &str = "nowxyii3";
pub const GITHUB_URL: &str = "https://github.com/nowxyii3/Unilancher";

// Palette thématique sombre moderne (Riot Client / Steam Dark gaming)
const BG_DARK: Color32 = Color32::from_rgb(11, 15, 25);
const SURFACE_1: Color32 = Color32::from_rgb(18, 24, 38);
const SURFACE_2: Color32 = Color32::from_rgb(26, 34, 52);
const BORDER_DEFAULT: Color32 = Color32::from_rgb(38, 48, 70);
const ACCENT_INDIGO: Color32 = Color32::from_rgb(99, 102, 241);
const ACCENT_CYAN: Color32 = Color32::from_rgb(56, 189, 248);
const TEXT_PRIMARY: Color32 = Color32::from_rgb(241, 245, 249);
const TEXT_SECONDARY: Color32 = Color32::from_rgb(148, 163, 184);
const TEXT_MUTED: Color32 = Color32::from_rgb(100, 116, 139);
const SUCCESS_EMERALD: Color32 = Color32::from_rgb(16, 185, 129);
const ERROR_ROSE: Color32 = Color32::from_rgb(244, 63, 94);

#[derive(Debug, Clone, PartialEq)]
pub enum SetupStep {
    Welcome,
    Installing,
    Finished,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum UninstallStep {
    Confirm,
    Uninstalling,
    Finished,
    Error(String),
}

pub enum InstallMessage {
    Status { step_name: String, progress: f32 },
    Success,
    Failure(String),
}

// ─────────────────────────────────────────────────────────────────────────────
// Fonctions Système Windows (Registre & Raccourcis)
// ─────────────────────────────────────────────────────────────────────────────

pub fn default_install_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files"))
        .join("Programs")
        .join(APP_NAME)
}

pub fn get_start_menu_shortcut_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| {
        d.join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join(format!("{}.lnk", APP_NAME))
    })
}

pub fn get_desktop_shortcut_path() -> Option<PathBuf> {
    dirs::desktop_dir().map(|d| d.join(format!("{}.lnk", APP_NAME)))
}

/// Enregistre l'application dans Windows (Ajout/Suppression de programmes) sans élévation
pub fn register_in_windows_registry(install_dir: &Path, exe_path: &Path) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key_path = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\UniLauncher";
    let (key, _) = hkcu
        .create_subkey(key_path)
        .map_err(|e| format!("Impossible d'accéder au registre Windows : {e}"))?;

    let setup_path = install_dir.join("UniLauncherSetup.exe");
    let uninst_cmd = format!("\"{}\" --uninstall", setup_path.to_string_lossy());
    let quiet_cmd = format!("\"{}\" --uninstall --silent", setup_path.to_string_lossy());
    let icon_val = format!("\"{}\",0", exe_path.to_string_lossy());

    key.set_value("DisplayName", &APP_NAME).ok();
    key.set_value("DisplayVersion", &APP_VERSION).ok();
    key.set_value("Publisher", &PUBLISHER).ok();
    key.set_value("InstallLocation", &install_dir.to_string_lossy().as_ref()).ok();
    key.set_value("DisplayIcon", &icon_val).ok();
    key.set_value("UninstallString", &uninst_cmd).ok();
    key.set_value("QuietUninstallString", &quiet_cmd).ok();
    key.set_value("HelpLink", &GITHUB_URL).ok();
    key.set_value("URLInfoAbout", &GITHUB_URL).ok();
    key.set_value("NoModify", &1u32).ok();
    key.set_value("NoRepair", &1u32).ok();
    key.set_value("EstimatedSize", &35000u32).ok(); // ~35 Mo

    let now = chrono_free_date();
    key.set_value("InstallDate", &now).ok();

    Ok(())
}

fn chrono_free_date() -> String {
    "20261115".to_string()
}

/// Supprime l'enregistrement de l'application dans le Registre Windows
pub fn unregister_from_windows_registry() -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key_path = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\UniLauncher";
    let _ = hkcu.delete_subkey_all(key_path);
    Ok(())
}

/// Crée un raccourci Windows .lnk propre
pub fn create_windows_shortcut(shortcut_path: &Path, target_exe: &Path, description: &str) -> Result<(), String> {
    if let Some(parent) = shortcut_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let script = format!(
        r#"$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('{lnk}'); $s.TargetPath = '{target}'; $s.WorkingDirectory = '{workdir}'; $s.Description = '{desc}'; $s.IconLocation = '{target},0'; $s.Save()"#,
        lnk = shortcut_path.to_string_lossy().replace('\'', "''"),
        target = target_exe.to_string_lossy().replace('\'', "''"),
        workdir = target_exe.parent().unwrap_or(target_exe).to_string_lossy().replace('\'', "''"),
        desc = description.replace('\'', "''")
    );

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .map_err(|e| format!("Échec PowerShell : {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Exécutant d'installation en arrière-plan
// ─────────────────────────────────────────────────────────────────────────────

fn perform_installation(
    target_dir: PathBuf,
    create_desktop: bool,
    create_start_menu: bool,
    tx: Sender<InstallMessage>,
) {
    thread::spawn(move || {
        // Étape 1 : Préparation
        let _ = tx.send(InstallMessage::Status {
            step_name: "Initialisation du dossier cible...".to_string(),
            progress: 0.15,
        });
        thread::sleep(Duration::from_millis(200));

        if let Err(e) = std::fs::create_dir_all(&target_dir) {
            let _ = tx.send(InstallMessage::Failure(format!("Erreur création de dossier: {e}")));
            return;
        }

        // Étape 2 : Copie des fichiers
        let _ = tx.send(InstallMessage::Status {
            step_name: "Déploiement des fichiers exécutables UniLauncher...".to_string(),
            progress: 0.40,
        });
        thread::sleep(Duration::from_millis(250));

        let current_exe = match std::env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                let _ = tx.send(InstallMessage::Failure(format!("Impossible de localiser l'exécutable actuel : {e}")));
                return;
            }
        };

        let target_exe = target_dir.join("UniLauncher.exe");
        let target_setup = target_dir.join("UniLauncherSetup.exe");

        // Copie l'exécutable
        if let Err(e) = std::fs::copy(&current_exe, &target_exe) {
            let _ = tx.send(InstallMessage::Failure(format!("Échec de la copie de UniLauncher.exe : {e}")));
            return;
        }

        // Copie également comme setup/uninstalleur local
        let _ = std::fs::copy(&current_exe, &target_setup);

        // Crée README.txt d'accompagnement
        let readme_content = format!(
            "================================================================================\r\n\
             UniLauncher — Lanceur Universel de Jeux Vidéo pour Windows\r\n\
             Version : v{}\r\n\
             Éditeur : {}\r\n\
             Dépôt officiel : {}\r\n\
             ================================================================================\r\n\r\n\
             Application installée avec succès dans : {}\r\n\
             Pour désinstaller, utilisez les Paramètres Windows ou lancez : {}\r\n",
            APP_VERSION,
            PUBLISHER,
            GITHUB_URL,
            target_dir.display(),
            target_setup.display()
        );
        let _ = std::fs::write(target_dir.join("README.txt"), readme_content);

        // Étape 3 : Raccourcis
        let _ = tx.send(InstallMessage::Status {
            step_name: "Configuration des raccourcis Windows...".to_string(),
            progress: 0.70,
        });
        thread::sleep(Duration::from_millis(200));

        if create_desktop {
            if let Some(desktop) = get_desktop_shortcut_path() {
                let _ = create_windows_shortcut(&desktop, &target_exe, "UniLauncher — Lanceur Universel");
            }
        }

        if create_start_menu {
            if let Some(start_menu) = get_start_menu_shortcut_path() {
                let _ = create_windows_shortcut(&start_menu, &target_exe, "UniLauncher — Lanceur Universel");
            }
        }

        // Étape 4 : Registre Windows
        let _ = tx.send(InstallMessage::Status {
            step_name: "Enregistrement dans les Applications Windows...".to_string(),
            progress: 0.90,
        });
        thread::sleep(Duration::from_millis(200));

        if let Err(e) = register_in_windows_registry(&target_dir, &target_exe) {
            let _ = tx.send(InstallMessage::Failure(e));
            return;
        }

        // Finalisation
        let _ = tx.send(InstallMessage::Status {
            step_name: "Installation terminée avec succès !".to_string(),
            progress: 1.0,
        });
        thread::sleep(Duration::from_millis(150));
        let _ = tx.send(InstallMessage::Success);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Exécutant de désinstallation
// ─────────────────────────────────────────────────────────────────────────────

fn perform_uninstallation(delete_user_data: bool, tx: Sender<InstallMessage>) {
    thread::spawn(move || {
        let _ = tx.send(InstallMessage::Status {
            step_name: "Arrêt des processus UniLauncher en cours...".to_string(),
            progress: 0.20,
        });
        thread::sleep(Duration::from_millis(300));

        // Ferme UniLauncher s'il tourne
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/IM", "UniLauncher.exe"])
            .output();

        let _ = tx.send(InstallMessage::Status {
            step_name: "Suppression des raccourcis Bureau et Menu Démarrer...".to_string(),
            progress: 0.50,
        });
        thread::sleep(Duration::from_millis(200));

        if let Some(desktop) = get_desktop_shortcut_path() {
            let _ = std::fs::remove_file(desktop);
        }
        if let Some(start_menu) = get_start_menu_shortcut_path() {
            let _ = std::fs::remove_file(start_menu);
        }

        let _ = tx.send(InstallMessage::Status {
            step_name: "Suppression de l'entrée dans le Registre Windows...".to_string(),
            progress: 0.75,
        });
        thread::sleep(Duration::from_millis(200));
        let _ = unregister_from_windows_registry();

        // Dossier d'installation
        let _ = tx.send(InstallMessage::Status {
            step_name: "Nettoyage des fichiers du programme...".to_string(),
            progress: 0.90,
        });
        let install_dir = default_install_dir();
        if install_dir.exists() {
            // Supprime les fichiers sauf l'exécutable actuel s'il est en cours d'exécution
            if let Ok(entries) = std::fs::read_dir(&install_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.file_name().and_then(|n| n.to_str()) != Some("UniLauncherSetup.exe") {
                        let _ = std::fs::remove_file(&p);
                    }
                }
            }
        }

        if delete_user_data {
            if let Some(config_dir) = dirs::config_dir() {
                let user_data = config_dir.join(APP_NAME);
                let _ = std::fs::remove_dir_all(user_data);
            }
        }

        let _ = tx.send(InstallMessage::Status {
            step_name: "Désinstallation terminée.".to_string(),
            progress: 1.0,
        });
        thread::sleep(Duration::from_millis(200));
        let _ = tx.send(InstallMessage::Success);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Exécutant de mise à jour transparente (--update-apply)
// ─────────────────────────────────────────────────────────────────────────────

pub fn run_update_worker(target_dir_str: &str, parent_pid_str: &str) {
    let pid: u32 = parent_pid_str.parse().unwrap_or(0);
    let target_dir = PathBuf::from(target_dir_str);

    // Attente que le processus parent quitte
    if pid > 0 {
        for _ in 0..60 {
            let check = std::process::Command::new("tasklist")
                .args(["/FI", &format!("PID eq {pid}")])
                .output();
            if let Ok(out) = check {
                let s = String::from_utf8_lossy(&out.stdout);
                if !s.contains(&pid.to_string()) {
                    break;
                }
            }
            thread::sleep(Duration::from_millis(150));
        }
    }

    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return,
    };

    let target_exe = target_dir.join("UniLauncher.exe");
    let backup_exe = target_dir.join("UniLauncher.exe.old");

    // Renommage de sécurité
    if target_exe.exists() {
        let _ = std::fs::rename(&target_exe, &backup_exe);
    }

    match std::fs::copy(&current_exe, &target_exe) {
        Ok(_) => {
            let _ = std::fs::remove_file(backup_exe);
            // Relance UniLauncher
            let _ = std::process::Command::new(&target_exe).spawn();
        }
        Err(_) => {
            // Restauration en cas d'échec
            if backup_exe.exists() {
                let _ = std::fs::rename(&backup_exe, &target_exe);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Interface Graphique du Setup (eframe)
// ─────────────────────────────────────────────────────────────────────────────

pub struct SetupApp {
    pub is_uninstaller: bool,
    pub step: SetupStep,
    pub uninst_step: UninstallStep,

    pub install_dir: String,
    pub create_desktop_shortcut: bool,
    pub create_start_menu_shortcut: bool,
    pub launch_after_install: bool,
    pub delete_user_data: bool,

    pub current_status_text: String,
    pub progress: f32,
    pub rx: Option<Receiver<InstallMessage>>,
}

impl SetupApp {
    pub fn new(is_uninstaller: bool) -> Self {
        let default_dir = default_install_dir().to_string_lossy().to_string();
        Self {
            is_uninstaller,
            step: SetupStep::Welcome,
            uninst_step: UninstallStep::Confirm,
            install_dir: default_dir,
            create_desktop_shortcut: true,
            create_start_menu_shortcut: true,
            launch_after_install: true,
            delete_user_data: false,
            current_status_text: String::new(),
            progress: 0.0,
            rx: None,
        }
    }
}

impl eframe::App for SetupApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Consommation des messages du worker
        if let Some(ref rx) = self.rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    InstallMessage::Status { step_name, progress } => {
                        self.current_status_text = step_name;
                        self.progress = progress;
                    }
                    InstallMessage::Success => {
                        if self.is_uninstaller {
                            self.uninst_step = UninstallStep::Finished;
                        } else {
                            self.step = SetupStep::Finished;
                        }
                    }
                    InstallMessage::Failure(err) => {
                        if self.is_uninstaller {
                            self.uninst_step = UninstallStep::Error(err);
                        } else {
                            self.step = SetupStep::Error(err);
                        }
                    }
                }
            }
            ctx.request_repaint_after(Duration::from_millis(50));
        }

        let bg_rect = ui.max_rect();
        ui.painter().rect_filled(bg_rect, CornerRadius::same(12), BG_DARK);
        ui.painter().rect_stroke(bg_rect, CornerRadius::same(12), Stroke::new(1.0, BORDER_DEFAULT), egui::StrokeKind::Inside);

        // Barre de titre personnalisée frameless
        self.render_custom_titlebar(ui, &ctx);

        ui.add_space(8.0);

        // Corps de l'assistant
        egui::Frame::new()
            .fill(SURFACE_1)
            .stroke(Stroke::new(1.0, BORDER_DEFAULT))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(24))
            .show(ui, |ui| {
                if self.is_uninstaller {
                    self.render_uninstaller(ui, &ctx);
                } else {
                    self.render_installer(ui, &ctx);
                }
            });
    }
}

impl SetupApp {
    fn render_custom_titlebar(&self, ui: &mut Ui, ctx: &egui::Context) {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::click_and_drag());
        if response.dragged() {
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        let p = ui.painter();
        let c = rect.center();

        // Icône et titre
        let icon_r = Rect::from_min_size(rect.min + vec2(8.0, 6.0), vec2(20.0, 20.0));
        p.rect_filled(icon_r, CornerRadius::same(5), ACCENT_INDIGO);
        p.text(
            icon_r.center(),
            Align2::CENTER_CENTER,
            "U",
            FontId::proportional(12.0),
            Color32::WHITE,
        );

        let title_text = if self.is_uninstaller {
            "UniLauncher — Assistant de Désinstallation"
        } else {
            "UniLauncher — Assistant d'Installation Professionnel"
        };
        p.text(
            Pos2::new(rect.min.x + 36.0, c.y),
            Align2::LEFT_CENTER,
            title_text,
            FontId::proportional(12.5),
            TEXT_PRIMARY,
        );

        // Bouton Fermer (✕)
        let close_rect = Rect::from_min_size(Pos2::new(rect.max.x - 32.0, rect.min.y + 4.0), vec2(24.0, 24.0));
        let close_hover = ui.rect_contains_pointer(close_rect);
        if close_hover {
            p.rect_filled(close_rect, CornerRadius::same(4), ERROR_ROSE.gamma_multiply(0.2));
            if ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary)) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        p.text(
            close_rect.center(),
            Align2::CENTER_CENTER,
            "✕",
            FontId::proportional(12.0),
            if close_hover { ERROR_ROSE } else { TEXT_SECONDARY },
        );
    }

    fn render_installer(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        let cur_step = self.step.clone();
        match &cur_step {
            SetupStep::Welcome => {
                ui.horizontal(|ui| {
                    // Logo UniLauncher moderne
                    let (logo_r, _) = ui.allocate_exact_size(vec2(60.0, 60.0), Sense::hover());
                    let p = ui.painter();
                    // Glow subtil
                    p.circle_filled(logo_r.center(), 32.0, ACCENT_INDIGO.gamma_multiply(0.2));
                    p.rect_filled(logo_r, CornerRadius::same(14), ACCENT_INDIGO);
                    p.text(
                        logo_r.center(),
                        Align2::CENTER_CENTER,
                        "U",
                        FontId::proportional(32.0),
                        Color32::WHITE,
                    );

                    ui.add_space(16.0);

                    ui.vertical(|ui| {
                        ui.label(RichText::new("Installation de UniLauncher").size(20.0).strong().color(TEXT_PRIMARY));
                        ui.add_space(3.0);
                        ui.label(
                            RichText::new(format!("Version {} — Production Stable (Windows 10/11 x64)", APP_VERSION))
                                .size(12.0)
                                .color(ACCENT_CYAN),
                        );
                        ui.label(
                            RichText::new("Centralisez l'ensemble de vos jeux PC dans une interface ultra-rapide.")
                                .size(12.0)
                                .color(TEXT_SECONDARY),
                        );
                    });
                });

                ui.add_space(18.0);
                ui.separator();
                ui.add_space(14.0);

                // Sélection du dossier d'installation
                ui.label(RichText::new("Dossier de destination :").size(13.0).strong().color(TEXT_PRIMARY));
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    let text_edit = egui::TextEdit::singleline(&mut self.install_dir)
                        .margin(Margin::symmetric(10, 8))
                        .desired_width(ui.available_width() - 110.0);
                    ui.add(text_edit);

                    if ui.button(RichText::new("Parcourir...").size(12.0).color(TEXT_PRIMARY)).clicked() {
                        if let Some(folder) = rfd::FileDialog::new()
                            .set_title("Sélectionner le dossier d'installation")
                            .pick_folder()
                        {
                            self.install_dir = folder.join(APP_NAME).to_string_lossy().to_string();
                        }
                    }
                });

                ui.add_space(6.0);
                ui.label(
                    RichText::new("Emplacement recommandé sans droits administrateur requis (%LOCALAPPDATA%).")
                        .size(11.0)
                        .color(TEXT_MUTED),
                );

                ui.add_space(16.0);

                // Options
                ui.label(RichText::new("Options de raccourcis :").size(13.0).strong().color(TEXT_PRIMARY));
                ui.add_space(6.0);

                ui.checkbox(&mut self.create_desktop_shortcut, RichText::new("Créer un raccourci sur le Bureau").size(12.5).color(TEXT_SECONDARY));
                ui.checkbox(&mut self.create_start_menu_shortcut, RichText::new("Ajouter au Menu Démarrer de Windows").size(12.5).color(TEXT_SECONDARY));
                ui.checkbox(&mut self.launch_after_install, RichText::new("Lancer UniLauncher dès la fin de l'installation").size(12.5).color(TEXT_SECONDARY));

                ui.add_space(24.0);

                // Boutons d'action
                ui.with_layout(Layout::right_to_left(Align2::CENTER_CENTER.y()), |ui| {
                    let install_btn = egui::Button::new(RichText::new("  Installer maintenant  ").size(13.5).strong().color(Color32::WHITE))
                        .fill(ACCENT_INDIGO)
                        .corner_radius(CornerRadius::same(6))
                        .min_size(vec2(160.0, 36.0));

                    if ui.add(install_btn).clicked() {
                        let (tx, rx) = channel();
                        self.rx = Some(rx);
                        self.step = SetupStep::Installing;
                        perform_installation(
                            PathBuf::from(&self.install_dir),
                            self.create_desktop_shortcut,
                            self.create_start_menu_shortcut,
                            tx,
                        );
                    }

                    ui.add_space(12.0);

                    if ui.button(RichText::new("Annuler").size(13.0).color(TEXT_MUTED)).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            }

            SetupStep::Installing => {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(RichText::new("Installation de UniLauncher en cours...").size(18.0).strong().color(TEXT_PRIMARY));
                    ui.add_space(6.0);
                    ui.label(RichText::new(&self.current_status_text).size(13.0).color(ACCENT_CYAN));

                    ui.add_space(32.0);

                    // Barre de progression élégante
                    let bar_width = 540.0;
                    let bar_height = 14.0;
                    let (bar_rect, _) = ui.allocate_exact_size(vec2(bar_width, bar_height), Sense::hover());
                    let p = ui.painter();

                    // Fond de barre
                    p.rect_filled(bar_rect, CornerRadius::same(7), SURFACE_2);
                    p.rect_stroke(bar_rect, CornerRadius::same(7), Stroke::new(1.0, BORDER_DEFAULT), egui::StrokeKind::Inside);

                    // Remplissage progressif avec gradient
                    let fill_w = bar_width * self.progress.clamp(0.0, 1.0);
                    if fill_w > 0.0 {
                        let fill_rect = Rect::from_min_size(bar_rect.min, vec2(fill_w, bar_height));
                        p.rect_filled(fill_rect, CornerRadius::same(7), ACCENT_INDIGO);
                        // Halo sur la tête de progression
                        p.circle_filled(fill_rect.right_center(), 7.0, ACCENT_CYAN.gamma_multiply(0.4));
                    }

                    ui.add_space(12.0);
                    ui.label(RichText::new(format!("{:.0}%", self.progress * 100.0)).size(14.0).strong().color(TEXT_PRIMARY));

                    ui.add_space(30.0);
                    ui.label(RichText::new("Veuillez patienter pendant la configuration des composants Windows.").size(12.0).color(TEXT_MUTED));
                });
            }

            SetupStep::Finished => {
                ui.vertical_centered(|ui| {
                    ui.add_space(16.0);

                    // Badge succès
                    let (chk_rect, _) = ui.allocate_exact_size(vec2(52.0, 52.0), Sense::hover());
                    let p = ui.painter();
                    p.circle_filled(chk_rect.center(), 26.0, SUCCESS_EMERALD.gamma_multiply(0.2));
                    p.circle_filled(chk_rect.center(), 20.0, SUCCESS_EMERALD);
                    p.text(chk_rect.center(), Align2::CENTER_CENTER, "✓", FontId::proportional(22.0), Color32::WHITE);

                    ui.add_space(16.0);
                    ui.label(RichText::new("UniLauncher est installé avec succès !").size(20.0).strong().color(TEXT_PRIMARY));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!("L'application est prête à l'emploi dans : {}", self.install_dir))
                            .size(12.0)
                            .color(TEXT_SECONDARY),
                    );

                    ui.add_space(20.0);
                    ui.checkbox(&mut self.launch_after_install, RichText::new("Lancer UniLauncher maintenant").size(13.0).color(TEXT_PRIMARY));

                    ui.add_space(28.0);

                    let finish_btn = egui::Button::new(RichText::new("  Terminer  ").size(14.0).strong().color(Color32::WHITE))
                        .fill(SUCCESS_EMERALD)
                        .corner_radius(CornerRadius::same(6))
                        .min_size(vec2(160.0, 38.0));

                    if ui.add(finish_btn).clicked() {
                        if self.launch_after_install {
                            let target_exe = PathBuf::from(&self.install_dir).join("UniLauncher.exe");
                            let _ = std::process::Command::new(target_exe).spawn();
                        }
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            }

            SetupStep::Error(err) => {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(RichText::new("Une erreur est survenue lors de l'installation").size(18.0).strong().color(ERROR_ROSE));
                    ui.add_space(12.0);
                    ui.label(RichText::new(err).size(12.5).color(TEXT_SECONDARY));

                    ui.add_space(30.0);

                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Réessayer").size(13.0).color(TEXT_PRIMARY)).clicked() {
                            self.step = SetupStep::Welcome;
                        }
                        if ui.button(RichText::new("Quitter").size(13.0).color(TEXT_MUTED)).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
            }
        }
    }

    fn render_uninstaller(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        let cur_uninst = self.uninst_step.clone();
        match &cur_uninst {
            UninstallStep::Confirm => {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Désinstallation de UniLauncher").size(20.0).strong().color(TEXT_PRIMARY));
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("Cette opération supprimera UniLauncher de votre ordinateur.")
                            .size(12.5)
                            .color(TEXT_SECONDARY),
                    );

                    ui.add_space(18.0);
                    ui.separator();
                    ui.add_space(14.0);

                    ui.label(RichText::new("Éléments qui seront supprimés :").size(13.0).strong().color(TEXT_PRIMARY));
                    ui.add_space(4.0);
                    ui.label(RichText::new("• Les fichiers de l'application dans %LOCALAPPDATA%\\Programs\\UniLauncher").size(12.0).color(TEXT_SECONDARY));
                    ui.label(RichText::new("• Le raccourci sur le Bureau et dans le Menu Démarrer").size(12.0).color(TEXT_SECONDARY));
                    ui.label(RichText::new("• L'enregistrement dans les Applications installées de Windows").size(12.0).color(TEXT_SECONDARY));

                    ui.add_space(16.0);

                    // Option de suppression des sauvegardes (désactivée par défaut)
                    ui.checkbox(
                        &mut self.delete_user_data,
                        RichText::new("Supprimer également mes jaquettes en cache et données personnelles (%APPDATA%\\UniLauncher)")
                            .size(12.0)
                            .color(ERROR_ROSE),
                    );

                    ui.add_space(28.0);

                    ui.with_layout(Layout::right_to_left(Align2::CENTER_CENTER.y()), |ui| {
                        let uninst_btn = egui::Button::new(RichText::new("  Désinstaller définitivement  ").size(13.5).strong().color(Color32::WHITE))
                            .fill(ERROR_ROSE)
                            .corner_radius(CornerRadius::same(6))
                            .min_size(vec2(190.0, 36.0));

                        if ui.add(uninst_btn).clicked() {
                            let (tx, rx) = channel();
                            self.rx = Some(rx);
                            self.uninst_step = UninstallStep::Uninstalling;
                            perform_uninstallation(self.delete_user_data, tx);
                        }

                        ui.add_space(12.0);

                        if ui.button(RichText::new("Annuler").size(13.0).color(TEXT_MUTED)).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
            }

            UninstallStep::Uninstalling => {
                ui.vertical_centered(|ui| {
                    ui.add_space(30.0);
                    ui.label(RichText::new("Désinstallation en cours...").size(18.0).strong().color(TEXT_PRIMARY));
                    ui.add_space(8.0);
                    ui.label(RichText::new(&self.current_status_text).size(13.0).color(ACCENT_CYAN));

                    ui.add_space(32.0);

                    let bar_width = 540.0;
                    let (bar_rect, _) = ui.allocate_exact_size(vec2(bar_width, 12.0), Sense::hover());
                    ui.painter().rect_filled(bar_rect, CornerRadius::same(6), SURFACE_2);
                    let fill_w = bar_width * self.progress.clamp(0.0, 1.0);
                    if fill_w > 0.0 {
                        let fill_rect = Rect::from_min_size(bar_rect.min, vec2(fill_w, 12.0));
                        ui.painter().rect_filled(fill_rect, CornerRadius::same(6), ERROR_ROSE);
                    }

                    ui.add_space(12.0);
                    ui.label(RichText::new(format!("{:.0}%", self.progress * 100.0)).size(13.0).color(TEXT_PRIMARY));
                });
            }

            UninstallStep::Finished => {
                ui.vertical_centered(|ui| {
                    ui.add_space(30.0);
                    ui.label(RichText::new("UniLauncher a été désinstallé").size(20.0).strong().color(TEXT_PRIMARY));
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("L'ensemble des fichiers du programme ont été retirés de votre système.")
                            .size(12.5)
                            .color(TEXT_SECONDARY),
                    );

                    ui.add_space(36.0);

                    if ui.button(RichText::new("  Fermer  ").size(13.5).color(Color32::WHITE)).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            }

            UninstallStep::Error(err) => {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(RichText::new("Erreur lors de la désinstallation").size(18.0).strong().color(ERROR_ROSE));
                    ui.add_space(8.0);
                    ui.label(RichText::new(err).size(12.5).color(TEXT_SECONDARY));

                    ui.add_space(24.0);
                    if ui.button(RichText::new("Fermer").size(13.0).color(TEXT_MUTED)).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            }
        }
    }
}

/// Point d'entrée de l'assistant graphique d'installation
pub fn run_setup(is_uninstaller: bool) -> eframe::Result<()> {
    let title = if is_uninstaller {
        "UniLauncher — Désinstallation"
    } else {
        "UniLauncher — Installation"
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(title)
            .with_decorations(false)
            .with_resizable(false)
            .with_inner_size([680.0, 440.0])
            .with_min_inner_size([680.0, 440.0]),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        title,
        options,
        Box::new(move |_cc| Ok(Box::new(SetupApp::new(is_uninstaller)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_paths_valid() {
        let install_dir = default_install_dir();
        assert!(install_dir.to_string_lossy().contains("UniLauncher"));
        assert!(install_dir.to_string_lossy().contains("Programs"));

        if let Some(desktop) = get_desktop_shortcut_path() {
            assert!(desktop.to_string_lossy().ends_with("UniLauncher.lnk"));
        }

        if let Some(start_menu) = get_start_menu_shortcut_path() {
            assert!(start_menu.to_string_lossy().ends_with("UniLauncher.lnk"));
        }
    }

    #[test]
    fn test_setup_app_initial_state() {
        let app = SetupApp::new(false);
        assert!(!app.is_uninstaller);
        assert_eq!(app.step, SetupStep::Welcome);
        assert!(app.create_desktop_shortcut);
        assert!(app.create_start_menu_shortcut);
        assert!(app.launch_after_install);

        let uninst = SetupApp::new(true);
        assert!(uninst.is_uninstaller);
        assert_eq!(uninst.uninst_step, UninstallStep::Confirm);
        assert!(!uninst.delete_user_data);
    }
}
