//! The Minecraft Java Edition a Horizon client is prepared from: one already on the computer, in
//! the official launcher's folder or in a Prism, MultiMC or Modrinth instance, and — when there is
//! none — one this tool fetches from Mojang.
//!
//! OffHorizon is an alternative to the official launcher, not an add-on: nothing here reads
//! Horizon's files, needs it installed, or asks for an account. The game plays offline, and its
//! resources come from any copy of the game that is here, or from the pins in the client's
//! `research-lock.json`.

use crate::others;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Mojang's list of every version, with the address of each one's JSON.
const MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
/// The asset store: an object is served under the first two characters of its hash.
const OBJECTS: &str = "https://resources.download.minecraft.net";

/// The launchers that rewrite the version JSON and read their own layout: the preparer takes them
/// as `--prism-dir`, root or instance.
const PRISM_LIKE: &[&str] = &[
    "prism",
    "multimc",
    "polymc",
    "modrinth",
    "gdlauncher",
    "curseforge",
];

/// One place a Minecraft of the pinned version can be read from.
#[derive(Clone, Debug)]
pub struct Source {
    pub dir: PathBuf,
    /// A launcher whose versions live outside the instance: `--prism-dir` instead of
    /// `--minecraft-dir`.
    pub prism: bool,
    /// What the log calls it.
    pub name: String,
}

impl Source {
    /// The flag the preparer takes this folder as.
    pub fn flag(&self) -> &'static str {
        if self.prism {
            "--prism-dir"
        } else {
            "--minecraft-dir"
        }
    }
}

/// Every install of `version` on this computer, the stick's own folder first.
///
/// `OFFHORIZON_MINECRAFT` names the one folder to use, which is how a player points at a copy this
/// tool doesn't know, and how the download path is reached on a machine that has Minecraft.
pub fn sources(version: &str, base: &Path) -> Vec<Source> {
    let mut all: Vec<Source> = Vec::new();
    if let Some(named) = std::env::var_os("OFFHORIZON_MINECRAFT").filter(|value| !value.is_empty()) {
        let dir = PathBuf::from(named);
        if holds(&dir, version) {
            all.push(Source {
                dir,
                prism: false,
                name: "OFFHORIZON_MINECRAFT".into(),
            });
        }
        return all;
    }
    let portable = base.join(".minecraft");
    if holds(&portable, version) {
        all.push(Source {
            dir: portable,
            prism: false,
            name: crate::i18n::t("home.rust_portable"),
        });
    }
    for found in others::found() {
        let prism = PRISM_LIKE.contains(&found.id);
        let dir = if prism {
            found.root.clone()
        } else {
            found.game.clone()
        };
        if !holds(&dir, version) && !holds(&found.game, version) {
            continue;
        }
        if all.iter().any(|source| source.dir == dir) {
            continue;
        }
        all.push(Source {
            dir,
            prism,
            name: found.name.to_string(),
        });
    }
    all
}

/// Whether `dir` has this version: a real Minecraft folder keeps `versions/<version>/<version>.json`.
fn holds(dir: &Path, version: &str) -> bool {
    dir.join("versions")
        .join(version)
        .join(format!("{version}.json"))
        .is_file()
}

/// Fetches `version` from Mojang into `dir` as a Minecraft folder the preparer reads: the version
/// JSON, the client and server jars, the libraries and the asset store.
///
/// `progress` takes the fraction done and what is being fetched. Nothing is downloaded twice: a
/// file already there is kept, so an interrupted download picks up where it stopped.
pub fn from_mojang(
    version: &str,
    dir: &Path,
    progress: &mut dyn FnMut(f32, &str),
) -> Result<PathBuf, String> {
    progress(0.0, "version");
    let manifest = json(MANIFEST)?;
    let url = manifest
        .get("versions")
        .and_then(Value::as_array)
        .and_then(|list| {
            list.iter()
                .find(|entry| entry.get("id").and_then(Value::as_str) == Some(version))
        })
        .and_then(|entry| entry.get("url").and_then(Value::as_str))
        .ok_or_else(|| format!("{version}"))?;
    let meta = json(url)?;

    let folder = dir.join("versions").join(version);
    std::fs::create_dir_all(&folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    std::fs::write(
        folder.join(format!("{version}.json")),
        serde_json::to_vec_pretty(&meta).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("{}: {e}", folder.display()))?;

    // The jars and the libraries the preparer's import reads.
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    if let Some(url) = meta.pointer("/downloads/client/url").and_then(Value::as_str) {
        files.push((url.to_string(), folder.join(format!("{version}.jar"))));
    }
    if let Some(url) = meta.pointer("/downloads/server/url").and_then(Value::as_str) {
        files.push((
            url.to_string(),
            dir.join("server").join(format!("{version}.jar")),
        ));
    }
    let libraries: &[Value] = meta
        .get("libraries")
        .and_then(Value::as_array)
        .map(|list| list.as_slice())
        .unwrap_or(&[]);
    for library in libraries {
        if !allowed(library) {
            continue;
        }
        let artifact = library.pointer("/downloads/artifact");
        let (Some(path), Some(url)) = (
            artifact.and_then(|a| a.get("path")).and_then(Value::as_str),
            artifact.and_then(|a| a.get("url")).and_then(Value::as_str),
        ) else {
            continue;
        };
        files.push((url.to_string(), dir.join("libraries").join(path)));
    }
    let total = files.len().max(1) as f32;
    for (at, (url, to)) in files.iter().enumerate() {
        fetch(url, to)?;
        progress(0.05 + 0.35 * (at as f32 / total), "files");
    }

    // The asset store: the index names every object by its hash.
    let index = meta.pointer("/assetIndex");
    let index_url = index
        .and_then(|i| i.get("url"))
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{version}"))?;
    let index_id = index
        .and_then(|i| i.get("id"))
        .and_then(Value::as_str)
        .or_else(|| meta.get("assets").and_then(Value::as_str))
        .unwrap_or(version)
        .to_string();
    let index = json(index_url)?;
    let indexes = dir.join("assets").join("indexes");
    std::fs::create_dir_all(&indexes).map_err(|e| format!("{}: {e}", indexes.display()))?;
    std::fs::write(
        indexes.join(format!("{index_id}.json")),
        serde_json::to_vec_pretty(&index).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("{}: {e}", indexes.display()))?;

    let mut objects: Vec<(String, PathBuf)> = Vec::new();
    for object in index
        .get("objects")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .map(|(_, object)| object)
    {
        let Some(hash) = object.get("hash").and_then(Value::as_str) else {
            continue;
        };
        if hash.len() < 2 {
            continue;
        }
        let to = dir
            .join("assets")
            .join("objects")
            .join(&hash[..2])
            .join(hash);
        if to.is_file() {
            continue;
        }
        objects.push((format!("{OBJECTS}/{}/{}", &hash[..2], hash), to));
    }
    let total_objects = objects.len().max(1) as f32;
    let mut done = 0.0_f32;
    for chunk in objects.chunks(250) {
        fetch_many(chunk)?;
        done += chunk.len() as f32;
        progress(0.4 + 0.6 * (done / total_objects).min(1.0), "assets");
    }
    progress(1.0, "done");
    Ok(dir.to_path_buf())
}

/// Whether a library is for this machine: the same rules the Java launcher applies.
fn allowed(library: &Value) -> bool {
    let Some(rules) = library.get("rules").and_then(Value::as_array) else {
        return true;
    };
    let mut allowed = false;
    for rule in rules {
        if let Some(name) = rule.pointer("/os/name").and_then(Value::as_str) {
            if name != os_name() {
                continue;
            }
        }
        if let Some(arch) = rule.pointer("/os/arch").and_then(Value::as_str) {
            let machine = if cfg!(target_arch = "x86_64") { "x86_64" } else { "arm64" };
            if arch != machine && !machine.starts_with(arch) {
                continue;
            }
        }
        if let Some(features) = rule.get("features").and_then(Value::as_object) {
            if features.values().any(|on| on.as_bool().unwrap_or(false)) {
                continue;
            }
        }
        let action = rule.get("action").and_then(Value::as_str).unwrap_or("allow");
        allowed = action == "allow";
    }
    allowed
}

fn os_name() -> &'static str {
    if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    }
}

fn json(url: &str) -> Result<Value, String> {
    let text = curl(&["-L", "--fail", "--silent", "--show-error", url])?;
    serde_json::from_str(&text).map_err(|e| format!("{url}: {e}"))
}

fn fetch(url: &str, to: &Path) -> Result<(), String> {
    if to.is_file() {
        return Ok(());
    }
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let file = text(to);
    curl(&[
        "-L",
        "--fail",
        "--silent",
        "--show-error",
        "--retry",
        "2",
        "-o",
        file.as_str(),
        url,
    ])?;
    Ok(())
}

/// Fetches a batch of files with one `curl`, eight at a time: a process per asset would cost more
/// than the downloads do.
fn fetch_many(files: &[(String, PathBuf)]) -> Result<(), String> {
    let config = std::env::temp_dir().join(format!(
        "offhorizon-assets-{}-{:x}.cfg",
        std::process::id(),
        files.len()
    ));
    let mut body = String::new();
    for (url, to) in files {
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        body.push_str(&format!("url = \"{url}\"\noutput = \"{}\"\n", text(to)));
    }
    std::fs::write(&config, body).map_err(|e| format!("{}: {e}", config.display()))?;
    let from = text(&config);
    let result = curl(&[
        "-L",
        "--fail",
        "--silent",
        "--show-error",
        "--retry",
        "2",
        "--parallel",
        "--parallel-max",
        "8",
        "-K",
        from.as_str(),
    ]);
    let _ = std::fs::remove_file(&config);
    result.map(|_| ())
}

fn curl(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("curl")
        .args(args)
        .output()
        .map_err(|e| format!("curl: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn text(path: &Path) -> String {
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("offhorizon-minecraft-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn version_in(dir: &Path, version: &str) {
        let folder = dir.join("versions").join(version);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join(format!("{version}.json")), b"{}").unwrap();
    }

    #[test]
    fn the_sticks_own_folder_comes_first_when_it_has_the_version() {
        let base = scratch("stick");
        let portable = base.join(".minecraft");
        version_in(&portable, "26.3");
        let found = sources("26.3", &base);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].dir, portable);
        assert!(!found[0].prism);
        assert_eq!(found[0].flag(), "--minecraft-dir");
    }

    #[test]
    fn a_folder_without_the_version_is_not_a_source() {
        let base = scratch("other");
        version_in(&base.join(".minecraft"), "1.20.1");
        assert!(sources("26.3", &base).is_empty());
    }

    #[test]
    fn a_launcher_that_rewrites_its_versions_is_read_as_a_prism_folder() {
        let source = Source {
            dir: PathBuf::from("/prism"),
            prism: true,
            name: "Prism".into(),
        };
        assert_eq!(source.flag(), "--prism-dir");
    }

    #[test]
    fn a_library_rule_for_another_machine_is_refused() {
        let other = serde_json::json!({ "rules": [ { "action": "allow", "os": { "name": "plan9" } } ] });
        assert!(!allowed(&other));
        let plain = serde_json::json!({ "name": "com.example:lib:1.0" });
        assert!(allowed(&plain));
    }
}
