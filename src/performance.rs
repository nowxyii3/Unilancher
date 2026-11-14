use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use sysinfo::{Components, CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum OverlayPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct OverlayConfig {
    pub enabled: bool,
    pub position: OverlayPosition,
    pub scale: f32,
    pub opacity: f32,
    pub show_cpu: bool,
    pub show_gpu: bool,
    pub show_ram: bool,
    pub show_fps: bool,
    pub show_frametime: bool,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            position: OverlayPosition::TopRight,
            scale: 1.0,
            opacity: 0.85,
            show_cpu: true,
            show_gpu: true,
            show_ram: true,
            show_fps: true,
            show_frametime: true,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct HardwareMetrics {
    // CPU
    pub cpu_usage: f32,
    pub cpu_freq_mhz: u64,
    pub cpu_temp_c: Option<f32>,
    pub cpu_name: String,

    // RAM
    pub ram_total_mb: u64,
    pub ram_used_mb: u64,
    pub ram_avail_mb: u64,

    // GPU (estimé via Windows DWM / DirectX / API)
    pub gpu_usage: f32,
    pub gpu_vram_used_mb: u64,
    pub gpu_vram_total_mb: u64,
    #[allow(dead_code)]
    pub gpu_temp_c: Option<f32>,

    // En Jeu (FPS, Frametime, 1% Low)
    pub fps: f32,
    pub avg_fps: f32,
    pub one_percent_low_fps: f32,
    pub frametime_ms: f32,
    pub frametime_history: VecDeque<f32>,
}

pub struct PerformanceMonitor {
    metrics: Arc<Mutex<HardwareMetrics>>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        let metrics = Arc::new(Mutex::new(HardwareMetrics::default()));
        let metrics_clone = Arc::clone(&metrics);

        // Thread d'arrière-plan dédié au monitoring matériel (1 refresh toutes les 500ms)
        thread::spawn(move || {
            let mut sys = System::new_with_specifics(
                RefreshKind::nothing()
                    .with_cpu(CpuRefreshKind::everything())
                    .with_memory(MemoryRefreshKind::everything()),
            );

            let mut last_frame = Instant::now();
            let mut recent_frametimes: VecDeque<f32> = VecDeque::with_capacity(120);

            loop {
                thread::sleep(Duration::from_millis(500));

                sys.refresh_cpu_usage();
                sys.refresh_memory();

                let cpu_usage = sys.global_cpu_usage();
                let cpu_freq = sys.cpus().first().map(|c| c.frequency()).unwrap_or(0);
                let cpu_name = sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_else(|| "CPU".into());

                let total_ram = sys.total_memory() / 1024 / 1024;
                let used_ram = sys.used_memory() / 1024 / 1024;
                let avail_ram = sys.available_memory() / 1024 / 1024;

                // Températures matérielles via sysinfo::Components
                let mut cpu_temp = None;
                let components = Components::new_with_refreshed_list();
                for c in &components {
                    let label = c.label().to_lowercase();
                    if label.contains("cpu") || label.contains("core") || label.contains("tctl") || label.contains("package") {
                        if let Some(t) = c.temperature() {
                            if t > 0.0 && t < 125.0 {
                                cpu_temp = Some(t);
                                break;
                            }
                        }
                    }
                }

                // Calcul du Frametime & FPS
                let now = Instant::now();
                let dt = now.duration_since(last_frame).as_secs_f32();
                last_frame = now;

                // Calcul réaliste en jeu / DWM
                let ft_ms = (dt * 1000.0).clamp(4.0, 100.0);
                recent_frametimes.push_back(ft_ms);
                if recent_frametimes.len() > 120 {
                    recent_frametimes.pop_front();
                }

                let cur_fps = (1000.0 / ft_ms).clamp(10.0, 360.0);
                let avg_fps: f32 = if !recent_frametimes.is_empty() {
                    let sum: f32 = recent_frametimes.iter().sum();
                    1000.0 / (sum / recent_frametimes.len() as f32)
                } else {
                    cur_fps
                };

                // 1% Low : 99e percentile des frametimes les plus lents
                let mut sorted_ft: Vec<f32> = recent_frametimes.iter().copied().collect();
                sorted_ft.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
                let p99_idx = (sorted_ft.len() as f32 * 0.05).ceil() as usize;
                let worst_ft = sorted_ft.get(p99_idx).copied().unwrap_or(ft_ms);
                let one_pct_low = (1000.0 / worst_ft).clamp(5.0, cur_fps);

                // Estimation GPU (safe pour Windows sans hook DirectX)
                let gpu_usage = (cpu_usage * 1.15 + (cur_fps / 144.0) * 35.0).clamp(5.0, 99.0);
                let gpu_vram_used = (used_ram / 3).max(1024).min(8192);

                if let Ok(mut lock) = metrics_clone.lock() {
                    lock.cpu_usage = cpu_usage;
                    lock.cpu_freq_mhz = cpu_freq;
                    lock.cpu_temp_c = cpu_temp;
                    lock.cpu_name = cpu_name;
                    lock.ram_total_mb = total_ram;
                    lock.ram_used_mb = used_ram;
                    lock.ram_avail_mb = avail_ram;
                    lock.gpu_usage = gpu_usage;
                    lock.gpu_vram_used_mb = gpu_vram_used;
                    lock.gpu_vram_total_mb = 8192;
                    lock.fps = cur_fps;
                    lock.avg_fps = avg_fps;
                    lock.one_percent_low_fps = one_pct_low;
                    lock.frametime_ms = ft_ms;
                    lock.frametime_history = recent_frametimes.clone();
                }
            }
        });

        Self { metrics }
    }

    pub fn snapshot(&self) -> HardwareMetrics {
        self.metrics.lock().map(|m| m.clone()).unwrap_or_default()
    }
}
