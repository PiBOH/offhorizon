//! Starting the game: the versions found in a game folder, and the command that runs one.
//!
//! A version is a folder under `versions/` holding `<id>.json`; the JSON names the jars, the Java
//! arguments and the main class, and this module turns that into a command line. Nothing here talks
//! to a server: the game plays as the name on the stick, with no account and no ownership check.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

/// One installed version: where it plays from, its JSON, and what made it.
#[derive(Clone, Debug)]
pub struct Version {
    pub id: String,
    /// The game folder holding `libraries/`, `assets/` and `versions/`.
    pub game: PathBuf,
    pub json: PathBuf,
    /// `Vanilla`, or the loader that built it.
    pub loader: String,
    /// The release date from the JSON, `yyyy-mm-dd`.
    pub release_time: String,
}

/// The command that runs one version.
#[derive(Clone, Debug)]
pub struct Plan {
    pub java: PathBuf,
    pub jvm: Vec<String>,
    pub main_class: String,
    pub game_args: Vec<String>,
    pub cwd: PathBuf,
}

impl Plan {
    /// Starts the game and comes back at once: the window keeps running.
    pub fn spawn(&self) -> Result<(), String> {
        let mut command = Command::new(&self.java);
        command
            .args(&self.jvm)
            .arg(&self.main_class)
            .args(&self.game_args)
            .current_dir(&self.cwd);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // The game keeps its own window; no console flashes up beside it.
            command.creation_flags(0x0800_0000);
        }
        command
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("{}: {e}", self.java.display()))
    }
}

/// Every version installed in one game folder, newest first.
pub fn versions(game: &Path) -> Vec<Version> {
    let mut found = Vec::new();
    let Ok(list) = std::fs::read_dir(game.join("versions")) else {
        return found;
    };
    for entry in list.flatten() {
        let id = entry.file_name().to_string_lossy().into_owned();
        let json = entry.path().join(format!("{id}.json"));
        if !json.is_file() {
            continue;
        }
        let Ok(value) = merged(&json) else { continue };
        let release_time = value
            .get("releaseTime")
            .and_then(Value::as_str)
            .map(|text| text.chars().take(10).collect())
            .unwrap_or_default();
        found.push(Version {
            loader: loader_of(&id, &value),
            id,
            game: game.to_path_buf(),
            json,
            release_time,
        });
    }
    found.sort_by(|a, b| b.release_time.cmp(&a.release_time).then(b.id.cmp(&a.id)));
    found
}

fn loader_of(id: &str, value: &Value) -> String {
    let lower = id.to_lowercase();
    for (needle, name) in [
        ("fabric", "Fabric"),
        ("quilt", "Quilt"),
        ("neoforge", "NeoForge"),
        ("forge", "Forge"),
        ("optifine", "OptiFine"),
    ] {
        if lower.contains(needle) {
            return name.to_string();
        }
    }
    if value.get("inheritedFrom").is_some() {
        "Modded".to_string()
    } else {
        "Vanilla".to_string()
    }
}

/// The version's JSON with everything it inherits folded in, so one file describes the whole run.
fn merged(json: &Path) -> Result<Value, String> {
    let value = read_json(json)?;
    let Some(parent) = value.get("inheritedFrom").and_then(Value::as_str) else {
        return Ok(value);
    };
    let folder = json
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| json.display().to_string())?;
    let parent_json = folder.join(parent).join(format!("{parent}.json"));
    Ok(merge(merged(&parent_json)?, value))
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// The parent first, then the child's own libraries and arguments on top of it.
fn merge(base: Value, child: Value) -> Value {
    let mut out = match base {
        Value::Object(map) => map,
        other => return other,
    };
    let child = match child {
        Value::Object(map) => map,
        _ => return Value::Object(out),
    };
    for (key, value) in child {
        match key.as_str() {
            "libraries" => {
                let list = out
                    .entry("libraries")
                    .or_insert_with(|| Value::Array(Vec::new()));
                if let (Some(list), Some(more)) = (list.as_array_mut(), value.as_array()) {
                    list.extend(more.iter().cloned());
                }
            }
            "arguments" => {
                let into = out
                    .entry("arguments")
                    .or_insert_with(|| Value::Object(serde_json::Map::new()));
                if let (Some(into), Some(more)) = (into.as_object_mut(), value.as_object()) {
                    for (kind, values) in more {
                        let list = into
                            .entry(kind.clone())
                            .or_insert_with(|| Value::Array(Vec::new()));
                        if let (Some(list), Some(more)) = (list.as_array_mut(), values.as_array()) {
                            list.extend(more.iter().cloned());
                        }
                    }
                }
            }
            "inheritedFrom" => {}
            _ => {
                out.insert(key, value);
            }
        }
    }
    Value::Object(out)
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

/// Whether the rules let this library or argument through on this machine.
fn rules_allow(rules: &[Value]) -> bool {
    if rules.is_empty() {
        return true;
    }
    let mut allowed = false;
    for rule in rules {
        if !rule_matches(rule) {
            continue;
        }
        let action = rule.get("action").and_then(Value::as_str).unwrap_or("allow");
        allowed = action == "allow";
    }
    allowed
}

fn rule_matches(rule: &Value) -> bool {
    if let Some(features) = rule.get("features").and_then(Value::as_object) {
        // Demo mode and a custom resolution are never asked for here.
        if features.values().any(|on| on.as_bool().unwrap_or(false)) {
            return false;
        }
    }
    if let Some(name) = rule.pointer("/os/name").and_then(Value::as_str) {
        if name != os_name() {
            return false;
        }
    }
    if let Some(arch) = rule.pointer("/os/arch").and_then(Value::as_str) {
        if arch != std::env::consts::ARCH {
            return false;
        }
    }
    true
}

/// The jar a library entry names, relative to `libraries/`.
fn library_path(lib: &Value) -> Option<PathBuf> {
    let rules = lib
        .get("rules")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    if !rules_allow(rules) {
        return None;
    }
    // shortcut: versions before 1.13 keep their natives in a jar to unpack; those play from
    // another launcher until this one unpacks jars too.
    if lib.get("natives").is_some() {
        return None;
    }
    if let Some(path) = lib.pointer("/downloads/artifact/path").and_then(Value::as_str) {
        return Some(PathBuf::from(path));
    }
    let name = lib.get("name").and_then(Value::as_str)?;
    Some(PathBuf::from(name_to_path(name)))
}

/// `group:artifact:version[:classifier]` → `group/artifact/version/artifact-version[-classifier].jar`.
fn name_to_path(name: &str) -> String {
    let parts: Vec<&str> = name.split(':').collect();
    let (Some(group), Some(artifact), Some(version)) =
        (parts.first(), parts.get(1), parts.get(2))
    else {
        return format!("{name}.jar");
    };
    let classifier = parts
        .get(3)
        .map(|part| format!("-{part}"))
        .unwrap_or_default();
    format!(
        "{}/{artifact}/{version}/{artifact}-{version}{classifier}.jar",
        group.replace('.', "/")
    )
}

/// The strings and the rule-bound entries of an `arguments` list.
fn arg_list(value: Option<&Value>) -> Vec<String> {
    let mut out = Vec::new();
    let Some(list) = value.and_then(Value::as_array) else {
        return out;
    };
    for item in list {
        match item {
            Value::String(text) => out.push(text.clone()),
            Value::Object(_) => {
                let rules = item
                    .get("rules")
                    .and_then(Value::as_array)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                if !rules_allow(rules) {
                    continue;
                }
                match item.get("value") {
                    Some(Value::String(text)) => out.push(text.clone()),
                    Some(Value::Array(more)) => {
                        out.extend(more.iter().filter_map(Value::as_str).map(str::to_string));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    out
}

/// The Java the game runs on: the one the game folder carries, then `JAVA_HOME`, then `PATH`.
fn java_for(game: &Path) -> Option<PathBuf> {
    let exe = if cfg!(windows) { "java.exe" } else { "java" };
    let mut bundled = Vec::new();
    collect_java(&game.join("runtime"), exe, 5, &mut bundled);
    if let Some(found) = bundled.into_iter().next() {
        return Some(found);
    }
    for variable in ["JAVA_HOME", "JDK_HOME"] {
        if let Some(home) = std::env::var_os(variable).filter(|v| !v.is_empty()) {
            let candidate = PathBuf::from(home).join("bin").join(exe);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(exe))
        .find(|candidate| candidate.is_file())
}

fn collect_java(dir: &Path, exe: &str, depth: u32, into: &mut Vec<PathBuf>) {
    if depth == 0 {
        return;
    }
    let Ok(list) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in list.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_java(&path, exe, depth - 1, into);
            continue;
        }
        let named = path
            .file_name()
            .map(|name| name.eq_ignore_ascii_case(exe))
            .unwrap_or(false);
        let in_bin = path
            .parent()
            .and_then(Path::file_name)
            .map(|name| name == "bin")
            .unwrap_or(false);
        if named && in_bin {
            into.push(path);
        }
    }
}

/// A name's own fixed id: the game wants one per name, and nothing checks where it came from.
fn offline_uuid(name: &str) -> String {
    let mut low: u64 = 0xcbf2_9ce4_8422_2325;
    let mut high: u64 = 0x8422_2325_cbf2_9ce4;
    for byte in name.bytes() {
        low = (low ^ byte as u64).wrapping_mul(0x0000_0100_0000_01b3);
        high = (high ^ (byte as u64).rotate_left(13)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    let hex = format!("{high:016x}{low:016x}");
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

/// Turns one version into the command that runs it as `name`.
pub fn plan(game: &Path, version: &Version, name: &str) -> Result<Plan, String> {
    let value = merged(&version.json)?;
    let main_class = value
        .get("mainClass")
        .and_then(Value::as_str)
        .ok_or_else(|| version.json.display().to_string())?
        .to_string();
    let java = java_for(game).ok_or_else(|| crate::i18n::t("home.java_missing"))?;

    let natives = game.join("versions").join(&version.id).join("natives");
    std::fs::create_dir_all(&natives).map_err(|e| format!("{}: {e}", natives.display()))?;

    let mut jars: Vec<String> = Vec::new();
    if let Some(list) = value.get("libraries").and_then(Value::as_array) {
        for lib in list {
            let Some(path) = library_path(lib) else { continue };
            let full = game.join("libraries").join(path).display().to_string();
            if !jars.contains(&full) {
                jars.push(full);
            }
        }
    }
    let separator = if cfg!(windows) { ";" } else { ":" };
    let classpath = jars.join(separator);
    let assets = value
        .pointer("/assetIndex/id")
        .and_then(Value::as_str)
        .or_else(|| value.get("assets").and_then(Value::as_str))
        .unwrap_or("legacy")
        .to_string();

    let table: [(&str, String); 20] = [
        ("${auth_player_name}", name.to_string()),
        ("${version_name}", version.id.clone()),
        ("${game_directory}", game.display().to_string()),
        ("${assets_root}", game.join("assets").display().to_string()),
        ("${assets_index_name}", assets),
        ("${auth_uuid}", offline_uuid(name)),
        ("${auth_access_token}", "0".to_string()),
        ("${auth_session}", "0".to_string()),
        ("${clientid}", String::new()),
        ("${auth_xuid}", String::new()),
        ("${user_type}", "legacy".to_string()),
        (
            "${version_type}",
            value
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("release")
                .to_string(),
        ),
        ("${natives_directory}", natives.display().to_string()),
        ("${launcher_name}", "OffHorizon".to_string()),
        ("${launcher_version}", crate::VERSION.to_string()),
        ("${classpath}", classpath.clone()),
        ("${classpath_separator}", separator.to_string()),
        (
            "${library_directory}",
            game.join("libraries").display().to_string(),
        ),
        ("${resolution_width}", "1280".to_string()),
        ("${resolution_height}", "720".to_string()),
    ];
    let fill = |args: Vec<String>| -> Vec<String> {
        args.into_iter()
            .map(|arg| {
                table
                    .iter()
                    .fold(arg, |text, (key, value)| text.replace(key, value))
            })
            .collect()
    };

    let mut jvm = fill(arg_list(value.pointer("/arguments/jvm")));
    let mut game_args = fill(arg_list(value.pointer("/arguments/game")));
    if let Some(legacy) = value.get("minecraftArguments").and_then(Value::as_str) {
        game_args.extend(fill(legacy.split_whitespace().map(str::to_string).collect()));
    }
    if !jvm.iter().any(|arg| arg == "-cp" || arg == "-classpath") {
        jvm.push("-cp".to_string());
        jvm.push(classpath);
    }
    if !jvm.iter().any(|arg| arg.starts_with("-Djava.library.path")) {
        jvm.push(format!("-Djava.library.path={}", natives.display()));
    }
    Ok(Plan {
        java,
        jvm,
        main_class,
        game_args,
        cwd: game.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("offhorizon-launch-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_version_json_becomes_a_command_with_the_offline_name() {
        let game = scratch("plan");
        let folder = game.join("versions/1.20.1");
        std::fs::create_dir_all(&folder).unwrap();
        let jar = game.join("libraries/com/example/lib/1.0/lib-1.0.jar");
        std::fs::create_dir_all(jar.parent().unwrap()).unwrap();
        std::fs::write(&jar, b"jar").unwrap();
        std::fs::write(
            folder.join("1.20.1.json"),
            serde_json::json!({
                "id": "1.20.1",
                "type": "release",
                "releaseTime": "2023-06-12T12:00:00+00:00",
                "mainClass": "net.minecraft.client.main.Main",
                "assetIndex": { "id": "5" },
                "libraries": [
                    { "name": "com.example:lib:1.0" },
                    { "name": "com.example:elsewhere:2.0",
                      "rules": [ { "action": "allow", "os": { "name": "plan9" } } ] }
                ],
                "arguments": {
                    "jvm": [ "-cp", "${classpath}" ],
                    "game": [ "--username", "${auth_player_name}", "--uuid", "${auth_uuid}" ]
                }
            })
            .to_string(),
        )
        .unwrap();
        let exe = if cfg!(windows) { "java.exe" } else { "java" };
        let runtime = game.join("runtime/java-runtime/bin").join(exe);
        std::fs::create_dir_all(runtime.parent().unwrap()).unwrap();
        std::fs::write(&runtime, b"").unwrap();

        let all = versions(&game);
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].loader, "Vanilla");
        let plan = plan(&game, &all[0], "Steve").unwrap();
        assert_eq!(plan.java, runtime);
        assert_eq!(plan.main_class, "net.minecraft.client.main.Main");
        let name_at = plan.game_args.iter().position(|a| a == "--username").unwrap();
        assert_eq!(plan.game_args[name_at + 1], "Steve");
        let separator = if cfg!(windows) { ";" } else { ":" };
        let classpath = plan
            .jvm
            .iter()
            .find(|arg| arg.contains("lib-1.0.jar"))
            .expect("the library is on the classpath");
        assert!(classpath.contains(separator));
        assert!(!classpath.contains("elsewhere"));
        let uuid = offline_uuid("Steve");
        assert_eq!(uuid.len(), 36);
        assert_eq!(offline_uuid("Steve"), uuid);
        assert_ne!(uuid, offline_uuid("Alex"));
    }
}
