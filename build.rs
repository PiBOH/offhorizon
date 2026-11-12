//! `version.txt` is the single version source (the same rule as vivi-music-de and flowonline2):
//! the release workflow reads it, and the binary carries it.

use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=version.txt");
    let version = std::fs::read_to_string("version.txt")
        .ok()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
    println!("cargo:rustc-env=OFFLINE_PATCH_VERSION={version}");
    if !Path::new("icon.png").exists() {
        println!("cargo:warning=icon.png is missing: the window gets no icon");
    }
}
