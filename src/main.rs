#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod deals;
mod discord;
mod emulators;
mod process;
mod scan;
mod theme;
mod types;
mod ui;

use app::Launcher;
use config::Settings;
use eframe::egui::{self, IconData};
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    let initial_settings = Settings::load();
    let start_fullscreen = initial_settings.start_fullscreen;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("UniLauncher")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1024.0, 660.0])
            .with_fullscreen(start_fullscreen)
            .with_icon(build_app_icon()),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "UniLauncher",
        options,
        Box::new(|cc| Ok(Box::new(Launcher::new(cc, initial_settings)))),
    )
}

fn build_app_icon() -> Arc<IconData> {
    let size = 64u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - 31.5;
            let dy = y as f32 - 31.5;
            let dist = ((dx / 28.0).powi(4) + (dy / 28.0).powi(4)).powf(0.25);
            if dist <= 1.0 {
                let alpha = ((1.0 - dist) * 14.0).clamp(0.0, 1.0);
                let t = (x + y) as f32 / (size * 2) as f32;
                let mut r = (79.0 * (1.0 - t) + 99.0 * t) as u8;
                let mut g = (70.0 * (1.0 - t) + 102.0 * t) as u8;
                let mut b = (229.0 * (1.0 - t) + 241.0 * t) as u8;

                let in_u = ((6.0..=12.0).contains(&dx.abs()) && (-12.0..=6.0).contains(&dy))
                    || ((4.0..=11.0).contains(&dy) && dx.abs() <= 12.0);
                if in_u {
                    r = 255;
                    g = 255;
                    b = 255;
                }

                let idx = ((y * size + x) * 4) as usize;
                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
                rgba[idx + 3] = (alpha * 255.0) as u8;
            }
        }
    }
    Arc::new(IconData {
        rgba,
        width: size,
        height: size,
    })
}
