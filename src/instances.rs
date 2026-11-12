//! Instances: one version, played in a game folder of its own. Portability is a choice made when
//! the instance is created, not a rule: an instance can point at the folder beside this executable
//! or at a folder another launcher already filled. The list lives with the stick.

use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instance {
    pub name: String,
    pub version: String,
    /// The game folder holding `saves/`, `libraries/` and `versions/`.
    pub game: PathBuf,
}

/// Where the instance list is kept.
pub fn file(tool: &Path) -> PathBuf {
    tool.join("instances.json")
}

/// The instances on this stick, in the order they were made.
pub fn load(tool: &Path) -> Vec<Instance> {
    let Ok(text) = std::fs::read_to_string(file(tool)) else {
        return Vec::new();
    };
    let Ok(Value::Array(list)) = serde_json::from_str::<Value>(&text) else {
        return Vec::new();
    };
    list.iter()
        .filter_map(|entry| {
            Some(Instance {
                name: entry.get("name")?.as_str()?.to_string(),
                version: entry.get("version")?.as_str()?.to_string(),
                game: PathBuf::from(entry.get("game")?.as_str()?),
            })
        })
        .collect()
}

/// Writes the list back, beside the executable.
pub fn save(tool: &Path, list: &[Instance]) -> Result<PathBuf, String> {
    let path = file(tool);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let value: Vec<Value> = list
        .iter()
        .map(|instance| {
            json!({
                "name": instance.name,
                "version": instance.version,
                "game": instance.game.display().to_string(),
            })
        })
        .collect();
    std::fs::write(&path, format!("{}\n", Value::Array(value)))
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_instance_survives_a_round_trip_with_its_own_folder() {
        let tool =
            std::env::temp_dir().join(format!("offhorizon-instances-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tool);
        let list = vec![
            Instance {
                name: "Stick".into(),
                version: "1.21.1".into(),
                game: PathBuf::from("/stick/.minecraft"),
            },
            Instance {
                name: "Modded".into(),
                version: "fabric-loader-0.16-1.21.1".into(),
                game: PathBuf::from("/games/modded"),
            },
        ];
        save(&tool, &list).unwrap();
        assert_eq!(load(&tool), list);
    }
}
