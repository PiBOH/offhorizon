//! Instances: one version, played in a game folder of its own. Portability is a choice made when
//! the instance is created, not a rule: an instance can point at the folder beside this executable
//! or at a folder another launcher already filled. The list lives with the stick.

use crate::i18n;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Which game an instance plays: the Horizon client (Minecraft: Rust Edition), or Java.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edition {
    Rust,
    Java,
}

impl Edition {
    /// The word the file keeps: `rust` or `java`.
    pub fn key(self) -> &'static str {
        match self {
            Edition::Rust => "rust",
            Edition::Java => "java",
        }
    }

    /// The same word back into one. Anything but `rust` is Java: an instance made before the Rust
    /// client was supported plays Java.
    pub fn parse(text: &str) -> Self {
        if text.trim().eq_ignore_ascii_case("rust") {
            Edition::Rust
        } else {
            Edition::Java
        }
    }

    /// The edition's name, in the player's language.
    pub fn name(self) -> String {
        i18n::t(match self {
            Edition::Rust => "home.rust",
            Edition::Java => "home.java",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instance {
    pub name: String,
    /// The Rust client's tag, or the Java version's id.
    pub version: String,
    /// The game folder holding `saves/`, `libraries/` and `versions/`.
    pub game: PathBuf,
    /// Rust first: an instance made without one plays Java, as before.
    pub edition: Edition,
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
                edition: Edition::parse(entry.get("edition").and_then(Value::as_str).unwrap_or("java")),
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
                "edition": instance.edition.key(),
            })
        })
        .collect();
    std::fs::write(&path, format!("{}\\n", Value::Array(value)))
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_instance_survives_a_round_trip_with_its_own_folder_and_edition() {
        let tool =
            std::env::temp_dir().join(format!("offhorizon-instances-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tool);
        let list = vec![
            Instance {
                name: "Stick".into(),
                version: "1.21.1".into(),
                game: PathBuf::from("/stick/.minecraft"),
                edition: Edition::Java,
            },
            Instance {
                name: "Rust".into(),
                version: "26.3".into(),
                game: PathBuf::from("/stick/.minecraft"),
                edition: Edition::Rust,
            },
        ];
        save(&tool, &list).unwrap();
        assert_eq!(load(&tool), list);
    }

    #[test]
    fn an_instance_written_before_the_rust_client_still_plays_java() {
        let tool = std::env::temp_dir().join(format!("offhorizon-old-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tool);
        std::fs::create_dir_all(&tool).unwrap();
        std::fs::write(
            file(&tool),
            r#"[{"name":"Old","version":"1.20.1","game":"/games/old"}]"#,
        )
        .unwrap();
        let list = load(&tool);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].edition, Edition::Java);
    }
}
