//! OffHorizon's portable uninstaller: the release ships it beside the launcher, and it runs from
//! wherever the player keeps it, with no setup and no administrator rights.
//!
//! It takes away what OffHorizon and Horizon wrote on this computer and nothing else: the
//! launcher's own folders and copies, its shortcuts, its registry keys on Windows, and — only
//! when asked — the launcher's data folder and the players' worlds. Every folder goes through
//! [`ours`], a list of the names OffHorizon and Horizon use, so a `.minecraft`, a Prism or an
//! SKlauncher folder is never removed, even when it sits next to ours. A folder whose name this
//! program does not know (a data location the player moved by hand) is reported and left alone.
//!
//! Usage:
//!   offhorizon-uninstaller [--dry-run] [--remove-data] [--remove-worlds]
use std::fs;
use std::path::{Path, PathBuf};

const USAGE: &str = "\
OffHorizon uninstaller

  --dry-run        print what would be removed, change nothing
  --remove-data    also remove the launcher's data folder (settings, accounts, logs, prepared games)
  --remove-worlds  also remove the game folder (saves). Without this, worlds stay.

Only OffHorizon's and Horizon's own folders, shortcuts and registry keys are touched.";

/// The folder names this program may delete. Anything else is left alone.
const OURS: [&str; 6] = [
    "offhorizon",
    "OffHorizon",
    "OffHorizon Launcher",
    "Horizon Launcher",
    "horizon-launcher",
    "horizon",
];

fn ours(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| OURS.iter().any(|ours| n.eq_ignore_ascii_case(ours)))
        && path.parent().is_some()
}

/// The folders to remove, each already named for OffHorizon or Horizon.
fn targets(remove_data: bool, remove_worlds: bool) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if remove_data {
        out.push(horizon_launcher::backend::data_dir());
    }
    if remove_worlds {
        out.push(horizon_launcher::backend::game_home());
    }
    #[cfg(windows)]
    {
        // The portable copy in %APPDATA%\offhorizon and the folder of the 0.1.5-nightly setup.
        out.push(horizon_launcher::installer::home());
        out.push(horizon_launcher::installer::default_dir());
    }
    out
}

fn remove_dir(path: &Path, dry_run: bool, left: &mut Vec<String>) {
    if !path.exists() {
        return;
    }
    if dry_run {
        println!("Would remove {}", path.display());
        return;
    }
    match fs::remove_dir_all(path) {
        Ok(()) => println!("Removed {}", path.display()),
        Err(e) => {
            println!("Left {}: {e}", path.display());
            left.push(path.display().to_string());
        }
    }
}

fn remove_file(path: &Path, dry_run: bool, left: &mut Vec<String>) {
    if !path.is_file() {
        return;
    }
    if dry_run {
        println!("Would remove {}", path.display());
        return;
    }
    match fs::remove_file(path) {
        Ok(()) => println!("Removed {}", path.display()),
        Err(e) => {
            println!("Left {}: {e}", path.display());
            left.push(path.display().to_string());
        }
    }
}

/// The applications-menu entries OffHorizon wrote: a `.desktop` file named for it (or for the
/// Horizon launcher the fork started from), never anyone else's.
fn desktop_entries() -> Vec<PathBuf> {
    let Some(dir) = dirs::data_dir().map(|d| d.join("applications")) else {
        return Vec::new();
    };
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("desktop"))
                && path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .is_some_and(|stem| {
                        let stem = stem.to_ascii_lowercase();
                        stem.contains("offhorizon") || stem.starts_with("horizon-launcher")
                    })
        })
        .collect()
}

#[cfg(windows)]
fn remove_registry(left: &mut Vec<String>) {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    for key in [
        r"Software\Horizon\Launcher",
        r"Software\Microsoft\Windows\CurrentVersion\Uninstall\HorizonLauncher",
    ] {
        match hkcu.delete_subkey_all(key) {
            Ok(()) => println!("Removed HKCU\\{key}"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                println!("Left HKCU\\{key}: {e}");
                left.push(format!(r"HKCU\{key}"));
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{USAGE}");
        return;
    }
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let remove_data = args.iter().any(|a| a == "--remove-data");
    let remove_worlds = args.iter().any(|a| a == "--remove-worlds");
    let mut left: Vec<String> = Vec::new();

    #[cfg(windows)]
    {
        if dry_run {
            println!("Would remove OffHorizon's shortcuts and registry keys");
        } else {
            horizon_launcher::installer::remove_shortcuts(true);
            remove_registry(&mut left);
        }
    }
    for entry in desktop_entries() {
        remove_file(&entry, dry_run, &mut left);
    }
    for path in targets(remove_data, remove_worlds) {
        if !ours(&path) {
            println!(
                "Skipped {}: not an OffHorizon folder, so it stays.",
                path.display()
            );
            continue;
        }
        remove_dir(&path, dry_run, &mut left);
    }
    if !remove_data {
        println!("The launcher's data folder stays: pass --remove-data to remove it.");
    }
    if !remove_worlds {
        println!("Your worlds stay: pass --remove-worlds to remove them too.");
    }
    if !left.is_empty() {
        println!("{} item(s) could not be removed.", left.len());
        std::process::exit(1);
    }
}
