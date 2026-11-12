//! Offline Patch: a reversible profile patcher for Horizon, the Minecraft: Java Edition client
//! written in Rust. It changes the offline player name and the player in a save (health, hunger,
//! experience, inventory), and keeps a copy of every file it writes so each change can be undone.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod items;
mod launcher;
mod nbt;
mod store;
mod theme;
mod world;

/// The version from `version.txt`, read by `build.rs`: the single version source.
pub const VERSION: &str = env!("OFFLINE_PATCH_VERSION");

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 800.0])
            .with_min_inner_size([900.0, 620.0])
            .with_icon(window_icon()),
        ..Default::default()
    };
    eframe::run_native(
        "Offline Patch",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}

/// The window and taskbar icon, from the project's `icon.png`.
fn window_icon() -> egui::IconData {
    let bytes = include_bytes!("../icon.png");
    match image::load_from_memory(bytes) {
        Ok(picture) => {
            let small = picture
                .resize(64, 64, image::imageops::FilterType::Lanczos3)
                .to_rgba8();
            let (width, height) = small.dimensions();
            egui::IconData {
                rgba: small.into_raw(),
                width,
                height,
            }
        }
        Err(_) => egui::IconData {
            rgba: vec![0; 64 * 64 * 4],
            width: 64,
            height: 64,
        },
    }
}
