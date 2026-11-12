//! Where the launcher and the game keep what this tool patches: the launcher's data folder (its
//! `accounts.json` and `instances.json`), the game folders with their worlds, and the offline
//! player name the game reads from `horizon-player.txt`.

use crate::store::Store;
use std::path::{Path, PathBuf};

/// What was found on this computer.
#[derive(Clone, Debug, Default)]
pub struct Layout {
    /// The launcher's data folder, when one is there.
    pub launcher: Option<PathBuf>,
    /// Game folders that hold `saves/` or a player name.
    pub homes: Vec<PathBuf>,
}

/// An instance the launcher knows: its name and its game folder.
#[derive(Clone, Debug)]
pub struct Instance {
    pub name: String,
    pub home: PathBuf,
}

fn exists(path: &Path) -> bool {
    path.is_dir()
}

/// The folders Horizon's launcher uses for its own data, most likely first.
pub fn launcher_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = std::env::var_os("HORIZON_LAUNCHER_HOME").filter(|h| !h.is_empty()) {
        dirs.push(PathBuf::from(home));
    }
    if cfg!(windows) {
        if let Some(appdata) = std::env::var_os("APPDATA").filter(|h| !h.is_empty()) {
            let appdata = PathBuf::from(appdata);
            dirs.push(appdata.join("horizon-launcher"));
            dirs.push(appdata.join("horizon"));
        }
        if let Some(local) = std::env::var_os("LOCALAPPDATA").filter(|h| !h.is_empty()) {
            dirs.push(PathBuf::from(local).join("horizon-launcher"));
        }
    } else if cfg!(target_os = "macos") {
        if let Some(home) = std::env::var_os("HOME") {
            let home = PathBuf::from(home);
            dirs.push(home.join("Library/Application Support/horizon-launcher"));
            dirs.push(home.join("Library/Application Support/horizon"));
        }
    } else {
        if let Some(data) = std::env::var_os("XDG_DATA_HOME").filter(|h| !h.is_empty()) {
            dirs.push(PathBuf::from(data).join("horizon-launcher"));
        }
        if let Some(home) = std::env::var_os("HOME") {
            let home = PathBuf::from(home);
            dirs.push(home.join(".local/share/horizon-launcher"));
            dirs.push(home.join(".horizon"));
        }
    }
    dirs
}

/// The game folders a copy of the game might have left here.
pub fn game_home_candidates() -> Vec<PathBuf> {
    let mut homes = Vec::new();
    if let Some(home) = std::env::var_os("HORIZON_HOME").filter(|h| !h.is_empty()) {
        homes.push(PathBuf::from(home));
    }
    if let Ok(cwd) = std::env::current_dir() {
        homes.push(cwd.clone());
        homes.push(cwd.join(".minecraft"));
    }
    if let Some(appdata) = std::env::var_os("APPDATA").filter(|h| !h.is_empty()) {
        homes.push(PathBuf::from(appdata).join(".minecraft"));
    }
    if let Some(home) = std::env::var_os("HOME") {
        homes.push(PathBuf::from(home).join(".horizon"));
    }
    homes
}

/// Finds the launcher folder and every game folder on this computer.
pub fn detect() -> Layout {
    let mut homes: Vec<PathBuf> = Vec::new();
    let mut launcher = None;
    for dir in launcher_dirs() {
        if !exists(&dir) {
            continue;
        }
        if launcher.is_none() {
            launcher = Some(dir.clone());
        }
        for instance in instances(&dir) {
            push_unique(&mut homes, instance.home);
        }
        if let Some(game) = settings_game_dir(&dir) {
            push_unique(&mut homes, game);
        }
    }
    for home in game_home_candidates() {
        if home.join("saves").is_dir() || home.join("horizon-player.txt").is_file() {
            push_unique(&mut homes, home);
        }
    }
    Layout { launcher, homes }
}

fn push_unique(list: &mut Vec<PathBuf>, path: PathBuf) {
    if !list.iter().any(|p| p == &path) {
        list.push(path);
    }
}

/// The instances the launcher keeps in `instances.json`, read leniently: the file's shape has
/// changed across versions, so every string that looks like a folder with a name beside it counts.
pub fn instances(launcher: &Path) -> Vec<Instance> {
    let Ok(text) = std::fs::read_to_string(launcher.join("instances.json")) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Vec::new();
    };
    let list = value
        .get("instances")
        .and_then(|i| i.as_array())
        .or_else(|| value.as_array())
        .cloned()
        .unwrap_or_default();
    let base = launcher;
    list.iter()
        .filter_map(|entry| {
            let name = entry
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string();
            let home = ["home", "dir", "path", "game"]
                .iter()
                .find_map(|key| entry.get(*key).and_then(|h| h.as_str()))
                .map(PathBuf::from)?;
            let home = if home.is_absolute() { home } else { base.join(home) };
            Some(Instance { name, home })
        })
        .collect()
}

/// The game folder `settings.json` names (`minecraft_dir`), when it is there.
pub fn settings_game_dir(launcher: &Path) -> Option<PathBuf> {
    let text = std::fs::read_to_string(launcher.join("settings.json")).ok()?;
    let value = serde_json::from_str::<serde_json::Value>(&text).ok()?;
    let dir = value
        .get("minecraft_dir")
        .and_then(|d| d.as_str())
        .map(PathBuf::from)?;
    dir.is_dir().then_some(dir)
}

/// The name file the game keeps its offline player in.
pub fn player_name_file(home: &Path) -> PathBuf {
    home.join("horizon-player.txt")
}

/// The offline player name, as `horizon-player.txt` holds it (`name:Steve`).
pub fn offline_name(home: &Path) -> Option<String> {
    let text = std::fs::read_to_string(player_name_file(home)).ok()?;
    text.lines()
        .find_map(|line| line.trim().strip_prefix("name:"))
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
}

/// Writes the offline player name, backing the file up first.
pub fn set_offline_name(store: &mut Store, home: &Path, name: &str) -> Result<PathBuf, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("The name can't be empty.".into());
    }
    if name.len() > 16 {
        return Err("Minecraft names are at most 16 characters long.".into());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err("A Minecraft name uses letters, numbers and underscores only.".into());
    }
    std::fs::create_dir_all(home).map_err(|e| format!("{}: {e}", home.display()))?;
    let file = player_name_file(home);
    if file.is_file() {
        store.backup(&file, "offline name")?;
    }
    std::fs::write(&file, format!("name:{name}\n"))
        .map_err(|e| format!("{}: {e}", file.display()))?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("offline-patch-launcher-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn instances_are_read_from_the_file_the_launcher_writes() {
        let dir = scratch("instances");
        std::fs::write(
            dir.join("instances.json"),
            r#"{"instances":[{"id":"a","name":"Horizon","kind":"own","home":"G:\\game\\instances\\a"},
                              {"id":"b","name":"Ported","kind":"port","home":"relative/b"}]}"#,
        )
        .unwrap();
        let found = instances(&dir);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].name, "Horizon");
        assert!(found[0].home.ends_with("game\\instances\\a") || found[0].home.ends_with("game/instances/a"));
        assert_eq!(found[1].home, dir.join("relative/b"));
    }

    #[test]
    fn the_offline_name_is_read_and_written_with_a_backup() {
        let dir = scratch("name");
        std::env::set_var("OFFLINE_PATCH_HOME", dir.join("tool"));
        std::fs::write(dir.join("horizon-player.txt"), "name:Steve\n").unwrap();
        assert_eq!(offline_name(&dir).as_deref(), Some("Steve"));
        let mut store = Store::open();
        set_offline_name(&mut store, &dir, "Alex").unwrap();
        assert_eq!(offline_name(&dir).as_deref(), Some("Alex"));
        assert_eq!(store.entries.len(), 1);
    }

    #[test]
    fn a_name_minecraft_would_refuse_is_refused() {
        let dir = scratch("badname");
        std::env::set_var("OFFLINE_PATCH_HOME", dir.join("tool"));
        let mut store = Store::open();
        for bad in ["", " ", "a name", "thisnameiswaytoolong"] {
            assert!(set_offline_name(&mut store, &dir, bad).is_err(), "{bad:?}");
        }
    }
}
