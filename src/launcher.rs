//! Where the launcher and the game keep what this tool patches: the launcher's data folder (its
//! `settings.json`, `accounts.json` and `instances.json`), the game folders with their worlds, and
//! the offline player name the game reads from `horizon-player.txt`.
//!
//! Pointing the launcher at the portable game folder is a change to a file outside the stick, so
//! it leaves `.offhorizon.bak` beside that file, a second copy in the launcher's own folder and a
//! third in the temporary folder, and `undo` puts every byte back.

use crate::portable::Portable;
use crate::store::{now, Store};
use std::path::{Path, PathBuf};

/// What was found on this computer.
#[derive(Clone, Debug, Default)]
pub struct Layout {
    /// The launcher's data folder, when one is there.
    pub launcher: Option<PathBuf>,
    /// Game folders, the portable one first.
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
pub fn launcher_dirs(portable: &Portable) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    // A launcher carried on the stick comes first: the whole setup is portable.
    dirs.push(portable.launcher());
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
pub fn game_home_candidates(portable: &Portable) -> Vec<PathBuf> {
    let mut homes = vec![portable.minecraft()];
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
pub fn detect(portable: &Portable) -> Layout {
    let mut homes: Vec<PathBuf> = Vec::new();
    let mut launcher = None;
    for dir in launcher_dirs(portable) {
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
    for home in game_home_candidates(portable) {
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

/// The launcher's settings file.
pub fn settings_file(launcher: &Path) -> PathBuf {
    launcher.join("settings.json")
}

/// The marker left beside the launcher's settings: what the file was before this tool wrote it.
pub fn marker(launcher: &Path) -> PathBuf {
    launcher.join("settings.json.offhorizon.bak")
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

/// What pointing the launcher here did, and where every copy of the old file is.
#[derive(Clone, Debug)]
pub struct LauncherChange {
    pub settings: PathBuf,
    pub marker: PathBuf,
    pub copies: Vec<PathBuf>,
}

/// Points the launcher's `minecraft_dir` at the portable game folder. The old settings are kept
/// four times over: the marker beside them, the tool's backup list, a dated copy in the launcher's
/// own `offhorizon-backups` folder and one in the temporary folder.
pub fn point_at(
    store: &mut Store,
    portable: &Portable,
    launcher: &Path,
    dir: &Path,
) -> Result<LauncherChange, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let settings = settings_file(launcher);
    let before = if settings.is_file() {
        let text = std::fs::read_to_string(&settings)
            .map_err(|e| format!("{}: {e}", settings.display()))?;
        let value = serde_json::from_str::<serde_json::Value>(&text)
            .map_err(|e| format!("{}: {e}", settings.display()))?;
        Some((text, value))
    } else {
        None
    };
    let when = now();
    let stamp = when.to_string();
    let marker = marker(launcher);
    let record = match &before {
        Some((_, value)) => serde_json::json!({
            "existed": true,
            "when": when,
            "settings": value,
        }),
        None => serde_json::json!({ "existed": false, "when": when }),
    };
    std::fs::write(&marker, format!("{record}\n"))
        .map_err(|e| format!("{}: {e}", marker.display()))?;

    let mut copies = vec![marker.clone()];
    if before.is_some() {
        // The tool's own list, the launcher's folder and the temporary folder each keep one.
        if let Ok(copy) = store.backup(&settings, "launcher folder") {
            copies.push(copy);
        }
        if let Some((text, _)) = &before {
            let bytes = text.as_bytes();
            let name = format!("settings.json.{stamp}.offhorizon.bak");
            if let Ok(copy) = store.copy_to(bytes, &launcher.join("offhorizon-backups"), &name) {
                copies.push(copy);
            }
            if let Ok(copy) = store.copy_to(bytes, &portable.temp_copy(), &name) {
                copies.push(copy);
            }
        }
    }

    let mut value = match before {
        Some((_, value)) => value,
        None => serde_json::json!({}),
    };
    if !value.is_object() {
        value = serde_json::json!({});
    }
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "minecraft_dir".into(),
            serde_json::json!(dir.display().to_string()),
        );
    }
    let text = format!("{value}\n");
    let tmp = settings.with_extension("json.offhorizon.tmp");
    std::fs::write(&tmp, text).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, &settings).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("{}: {e}", settings.display())
    })?;
    Ok(LauncherChange {
        settings,
        marker,
        copies,
    })
}

/// Puts the launcher's settings back as they were before [`point_at`].
pub fn undo(launcher: &Path) -> Result<PathBuf, String> {
    let settings = settings_file(launcher);
    let marker = marker(launcher);
    let text = std::fs::read_to_string(&marker)
        .map_err(|e| format!("{}: {e}", marker.display()))?;
    let record = serde_json::from_str::<serde_json::Value>(&text)
        .map_err(|e| format!("{}: {e}", marker.display()))?;
    let existed = record
        .get("existed")
        .and_then(|e| e.as_bool())
        .unwrap_or(false);
    if existed {
        let original = record
            .get("settings")
            .cloned()
            .ok_or_else(|| "The marker has no settings in it.".to_string())?;
        std::fs::write(&settings, format!("{original}\n"))
            .map_err(|e| format!("{}: {e}", settings.display()))?;
    } else {
        // There was no settings file: take the key back out, and the file with it when empty.
        let mut value = std::fs::read_to_string(&settings)
            .ok()
            .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
            .unwrap_or_else(|| serde_json::json!({}));
        if let Some(object) = value.as_object_mut() {
            object.remove("minecraft_dir");
        }
        let empty = value.as_object().is_none_or(|object| object.is_empty());
        if empty {
            let _ = std::fs::remove_file(&settings);
        } else {
            std::fs::write(&settings, format!("{value}\n"))
                .map_err(|e| format!("{}: {e}", settings.display()))?;
        }
    }
    let _ = std::fs::remove_file(&marker);
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("offhorizon-launcher-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn instances_are_read_from_the_file_the_launcher_writes() {
        let dir = scratch("instances");
        std::fs::write(
            dir.join("instances.json"),
            r#"{"instances":[{"id":"a","name":"Horizon","kind":"own","home":"G:/game/instances/a"},
                              {"id":"b","name":"Ported","kind":"port","home":"relative/b"}]}"#,
        )
        .unwrap();
        let found = instances(&dir);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].name, "Horizon");
        assert_eq!(found[0].home.file_name().and_then(|n| n.to_str()), Some("a"));
        assert_eq!(found[1].home, dir.join("relative/b"));
    }

    #[test]
    fn the_offline_name_is_read_and_written_with_a_backup() {
        let dir = scratch("name");
        std::fs::write(dir.join("horizon-player.txt"), "name:Steve\n").unwrap();
        assert_eq!(offline_name(&dir).as_deref(), Some("Steve"));
        let mut store = Store::at(dir.join("tool"));
        set_offline_name(&mut store, &dir, "Alex").unwrap();
        assert_eq!(offline_name(&dir).as_deref(), Some("Alex"));
        assert_eq!(store.entries.len(), 1);
    }

    #[test]
    fn a_name_minecraft_would_refuse_is_refused() {
        let dir = scratch("badname");
        let mut store = Store::at(dir.join("tool"));
        for bad in ["", " ", "a name", "thisnameiswaytoolong"] {
            assert!(set_offline_name(&mut store, &dir, bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn pointing_the_launcher_keeps_the_old_settings_and_undo_puts_them_back() {
        let dir = scratch("point");
        let launcher = dir.join("launcher");
        let portable = Portable::at(dir.join("stick"));
        std::fs::create_dir_all(&launcher).unwrap();
        std::fs::write(
            launcher.join("settings.json"),
            r#"{"minecraft_dir":"C:/old","language":"it"}"#,
        )
        .unwrap();
        let mut store = Store::at(portable.backups());
        let change = point_at(&mut store, &portable, &launcher, &portable.minecraft()).unwrap();
        let after: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&change.settings).unwrap()).unwrap();
        assert_eq!(
            after.get("minecraft_dir").and_then(|d| d.as_str()),
            Some(portable.minecraft().display().to_string().as_str())
        );
        assert_eq!(after.get("language").and_then(|l| l.as_str()), Some("it"));
        assert!(change.marker.is_file());
        // A dated copy in the launcher's folder and one in the temporary folder.
        assert!(launcher.join("offhorizon-backups").is_dir());
        assert!(portable.temp_copy().is_dir());

        undo(&launcher).unwrap();
        let back: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(launcher.join("settings.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(back.get("minecraft_dir").and_then(|d| d.as_str()), Some("C:/old"));
        assert!(!change.marker.exists());
    }

    #[test]
    fn undo_takes_the_key_out_when_there_was_no_settings_file() {
        let dir = scratch("point-new");
        let launcher = dir.join("launcher");
        let portable = Portable::at(dir.join("stick"));
        std::fs::create_dir_all(&launcher).unwrap();
        let mut store = Store::at(portable.backups());
        point_at(&mut store, &portable, &launcher, &portable.minecraft()).unwrap();
        undo(&launcher).unwrap();
        assert!(!launcher.join("settings.json").exists());
    }
}
