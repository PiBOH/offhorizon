//! OffHorizon: the portable, telemetry-free companion for Horizon — Minecraft: Java Edition in
//! Rust. With no argument it opens a window; with arguments it does what they say. Its game folder,
//! its backups and its settings live beside the executable, so a USB stick carries the whole setup.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod actions;
mod app;
mod cli;
mod download;
mod i18n;
mod instances;
mod launch;
mod launcher;
mod logo;
mod minecraft;
mod native;
mod nbt;
mod others;
mod portable;
mod store;
mod theme;
mod world;

/// The version from `version.txt`, read by `build.rs`: the single version source.
pub const VERSION: &str = env!("OFFHORIZON_VERSION");

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let window = args.is_empty() || (args.len() == 1 && args[0] == "gui");
    if !window {
        std::process::exit(cli::run(&args));
    }

    let portable = portable::Portable::find();
    let lang = i18n::resolve(&portable);
    i18n::init(&lang);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 820.0])
            .with_min_inner_size([940.0, 640.0])
            .with_icon(logo::icon())
            .with_title(format!("{} {}", i18n::t("app.name"), VERSION)),
        ..Default::default()
    };
    eframe::run_native(
        "OffHorizon",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
