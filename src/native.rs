//! Minecraft: Rust Edition — the Horizon client itself. The official launcher keeps one client per
//! version under `versions/<tag>/`, runs it with `--resources` pointing at the prepared resources
//! and the player's name, and this module finds those clients beside the stick and builds the same
//! command. Rust comes first everywhere: when both editions can play, this one does.

use std::path::{Path, PathBuf};
use std::process::Command;

/// One client found: its version, the program, and the resources it reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Native {
    /// The version's tag, from its folder (the official `versions/<tag>` layout).
    pub tag: String,
    /// `horizon-client(.exe)`, the game itself.
    pub program: PathBuf,
    /// The prepared resources folder (`--resources`).
    pub resources: PathBuf,
}

/// The command that runs one client.
#[derive(Clone, Debug)]
pub struct Plan {
    pub tag: String,
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    /// `HORIZON_HOME`: logs, crash reports and the game's own settings, kept with the stick.
    pub home: PathBuf,
}

impl Plan {
    /// Starts the game and comes back at once: the window keeps running.
    pub fn spawn(&self) -> Result<(), String> {
        let mut command = Command::new(&self.program);
        command
            .args(&self.args)
            .current_dir(&self.cwd)
            .env("HORIZON_HOME", &self.home);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // The game keeps its own window; no console flashes up beside it.
            command.creation_flags(0x0800_0000);
        }
        if let Some(dir) = self.home.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        command
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("{}: {e}", self.program.display()))
    }
}

/// The game's file name on this machine.
pub fn program_name() -> &'static str {
    if cfg!(windows) {
        "horizon-client.exe"
    } else {
        "horizon-client"
    }
}

/// Every client found beside the executable, the newest first.
///
/// The official launcher lays a client out as `versions/<tag>/horizon-client` with its resources in
/// `versions/<tag>/.local/horizon-client/<minecraft version>`. A client dropped straight beside this
/// launcher is found too, with its resources in `horizon-client/` or `.local/horizon-client/`.
pub fn found(base: &Path) -> Vec<Native> {
    let mut search: Vec<PathBuf> = vec![base.to_path_buf()];
    for folder in [base.join("versions"), base.join(".offhorizon").join("versions")] {
        if let Ok(list) = std::fs::read_dir(&folder) {
            let mut tags: Vec<PathBuf> = list.flatten().map(|entry| entry.path()).collect();
            tags.sort();
            search.extend(tags);
        }
    }
    let mut clients: Vec<Native> = Vec::new();
    for dir in search {
        let program = dir.join(program_name());
        if !program.is_file() {
            continue;
        }
        let Some(resources) = resources_for(&dir, base) else {
            continue;
        };
        let tag = tag_of(&dir, base, &resources);
        let client = Native {
            tag,
            program,
            resources,
        };
        if !clients.iter().any(|known| known == &client) {
            clients.push(client);
        }
    }
    clients.sort_by(|a, b| b.tag.cmp(&a.tag).then(a.program.cmp(&b.program)));
    clients
}

/// The newest prepared resources folder that belongs to a client.
fn resources_for(dir: &Path, base: &Path) -> Option<PathBuf> {
    for parent in [
        dir.join(".local").join("horizon-client"),
        dir.join("horizon-client"),
        base.join(".local").join("horizon-client"),
        base.join("horizon-client"),
    ] {
        let Ok(list) = std::fs::read_dir(&parent) else {
            continue;
        };
        let mut versions: Vec<PathBuf> = list
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect();
        if versions.is_empty() {
            continue;
        }
        versions.sort_by_key(|path| version_key(path));
        if let Some(newest) = versions.pop() {
            return Some(newest);
        }
    }
    None
}

/// Orders `1.9` before `1.21` and `26.3` after both: the numbers, not the text.
fn version_key(path: &Path) -> Vec<u64> {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    name.split(|c: char| !c.is_ascii_digit())
        .filter_map(|part| part.parse::<u64>().ok())
        .collect()
}

/// What the instance list shows: the tag when the folder is the version's, else the game's version.
fn tag_of(dir: &Path, base: &Path, resources: &Path) -> String {
    let name = dir
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    if dir != base && !name.is_empty() {
        return name;
    }
    resources
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "Horizon".to_string())
}

/// The vanilla version the client's resources are prepared for.
pub fn minecraft_of(client: &Native) -> String {
    client
        .resources
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The command that runs one client as `player`, playing in `game_dir`.
pub fn plan(client: &Native, player: &str, game_dir: &Path, home: &Path) -> Plan {
    let args = vec![
        "--resources".to_string(),
        client.resources.display().to_string(),
        "--game-dir".to_string(),
        game_dir.display().to_string(),
        "--name".to_string(),
        player.to_string(),
    ];
    Plan {
        tag: client.tag.clone(),
        program: client.program.clone(),
        args,
        cwd: game_dir.to_path_buf(),
        home: home.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("offhorizon-native-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_client_beside_the_stick_is_found_with_its_resources() {
        let base = scratch("found");
        let tag = base.join("versions").join("26.3");
        let resources = tag
            .join(".local")
            .join("horizon-client")
            .join("26.3");
        std::fs::create_dir_all(&resources).unwrap();
        std::fs::write(tag.join(program_name()), b"").unwrap();
        // An older client is found too, and listed after the newer one.
        let older = base.join("versions").join("25.1");
        let older_resources = older
            .join(".local")
            .join("horizon-client")
            .join("25.1");
        std::fs::create_dir_all(&older_resources).unwrap();
        std::fs::write(older.join(program_name()), b"").unwrap();

        let clients = found(&base);
        assert_eq!(clients.len(), 2);
        assert_eq!(clients[0].tag, "26.3");
        assert_eq!(clients[0].resources, resources);
        assert_eq!(minecraft_of(&clients[0]), "26.3");
        assert_eq!(clients[1].tag, "25.1");
    }

    #[test]
    fn the_plan_hands_the_game_the_resources_the_folder_and_the_name() {
        let base = scratch("plan");
        let tag = base.join("versions").join("26.3");
        let resources = tag
            .join(".local")
            .join("horizon-client")
            .join("26.3");
        std::fs::create_dir_all(&resources).unwrap();
        std::fs::write(tag.join(program_name()), b"").unwrap();
        let clients = found(&base);
        let plan = plan(&clients[0], "Steve", &base.join("game"), &base.join("home"));

        let value = |flag: &str| {
            let at = plan.args.iter().position(|arg| arg == flag).unwrap();
            plan.args[at + 1].clone()
        };
        assert_eq!(value("--resources"), resources.display().to_string());
        assert_eq!(value("--game-dir"), base.join("game").display().to_string());
        assert_eq!(value("--name"), "Steve");
        assert_eq!(plan.cwd, base.join("game"));
        assert_eq!(plan.home, base.join("home"));
    }

    #[test]
    fn versions_compare_as_numbers() {
        let key = |name: &str| version_key(Path::new(name));
        assert!(key("1.9") < key("1.21"));
        assert!(key("1.21") < key("26.3"));
    }
}
