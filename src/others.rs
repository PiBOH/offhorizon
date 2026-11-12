//! The other Minecraft launchers on the computer: Rust ones, plain Java ones and modded ones.
//! When the game runs from a stick, this is how its worlds come along.

use std::path::{Path, PathBuf};

/// One launcher this tool knows how to look for.
pub struct Launcher {
    pub id: &'static str,
    pub name: &'static str,
    /// `Rust`, `Java` or `Java, modded`.
    pub kind: &'static str,
    /// Folders that launcher keeps, with `~`, `%APPDATA%`, `%LOCALAPPDATA%` and `%USERPROFILE%`.
    pub roots: &'static [&'static str],
}

/// The launchers players use most, Horizon included. The list only says where to look: nothing is
/// started, and a folder that isn't there is skipped.
pub const LAUNCHERS: &[Launcher] = &[
    Launcher {
        id: "horizon",
        name: "Horizon",
        kind: "Rust",
        roots: &[
            "%APPDATA%/horizon",
            "%APPDATA%/horizon-launcher",
            "~/.local/share/horizon",
            "~/.horizon",
        ],
    },
    Launcher {
        id: "sklauncher",
        name: "SKLauncher",
        kind: "Java",
        roots: &["%APPDATA%/SKlauncher", "%APPDATA%/.sklauncher", "~/.sklauncher"],
    },
    Launcher {
        id: "minecraft",
        name: "Minecraft Launcher",
        kind: "Java",
        roots: &["%APPDATA%/.minecraft", "~/Library/Application Support/minecraft", "~/.minecraft"],
    },
    Launcher {
        id: "tlauncher",
        name: "TLauncher",
        kind: "Java",
        roots: &["%APPDATA%/.minecraft", "%APPDATA%/TLauncher"],
    },
    Launcher {
        id: "prism",
        name: "Prism Launcher",
        kind: "Java, modded",
        roots: &[
            "%APPDATA%/PrismLauncher",
            "~/.local/share/PrismLauncher",
            "~/Library/Application Support/PrismLauncher",
        ],
    },
    Launcher {
        id: "multimc",
        name: "MultiMC",
        kind: "Java, modded",
        roots: &["%APPDATA%/MultiMC", "~/.local/share/multimc"],
    },
    Launcher {
        id: "polymc",
        name: "PolyMC",
        kind: "Java, modded",
        roots: &["%APPDATA%/PolyMC", "~/.local/share/PolyMC"],
    },
    Launcher {
        id: "atlauncher",
        name: "ATLauncher",
        kind: "Java, modded",
        roots: &["%APPDATA%/ATLauncher", "~/.local/share/ATLauncher"],
    },
    Launcher {
        id: "gdlauncher",
        name: "GDLauncher",
        kind: "Java, modded",
        roots: &["%APPDATA%/gdlauncher_next", "%APPDATA%/gdlauncher"],
    },
    Launcher {
        id: "curseforge",
        name: "CurseForge",
        kind: "Java, modded",
        roots: &[
            "%USERPROFILE%/curseforge/minecraft/Instances",
            "%APPDATA%/CurseForge",
        ],
    },
    Launcher {
        id: "modrinth",
        name: "Modrinth App",
        kind: "Java, modded",
        roots: &["%APPDATA%/com.modrinth.theseus", "~/.local/share/com.modrinth.theseus"],
    },
    Launcher {
        id: "technic",
        name: "Technic Launcher",
        kind: "Java, modded",
        roots: &["%APPDATA%/.technic", "~/.technic"],
    },
    Launcher {
        id: "ftb",
        name: "FTB App",
        kind: "Java, modded",
        roots: &["%APPDATA%/FTBApp", "%LOCALAPPDATA%/FTBApp"],
    },
    Launcher {
        id: "lunar",
        name: "Lunar Client",
        kind: "Java",
        roots: &["%USERPROFILE%/.lunarclient"],
    },
    Launcher {
        id: "badlion",
        name: "Badlion Client",
        kind: "Java",
        roots: &["%APPDATA%/Badlion Client"],
    },
    Launcher {
        id: "xmcl",
        name: "XMCL",
        kind: "Rust",
        roots: &["%APPDATA%/xmcl"],
    },
];

/// A game folder one of them left here.
#[derive(Clone, Debug)]
pub struct Found {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: &'static str,
    /// The launcher's folder.
    pub root: PathBuf,
    /// The folder holding `saves/` and `mods/`.
    pub game: PathBuf,
    pub worlds: usize,
    pub modded: bool,
}

/// `%APPDATA%/x` and `~/x` become real folders; a token the system doesn't have gives nothing.
fn expand(root: &str) -> Option<PathBuf> {
    let mut text = root.to_string();
    let mut home = false;
    for (token, variable) in [
        ("%APPDATA%", "APPDATA"),
        ("%LOCALAPPDATA%", "LOCALAPPDATA"),
        ("%USERPROFILE%", "USERPROFILE"),
    ] {
        if text.starts_with(token) {
            let value = std::env::var_os(variable).filter(|v| !v.is_empty())?;
            text = text.replacen(token, &value.to_string_lossy(), 1);
        }
    }
    if let Some(rest) = text.strip_prefix("~/") {
        let value = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .filter(|v| !v.is_empty())?;
        text = format!("{}/{}", PathBuf::from(value).display(), rest);
        home = true;
    }
    let _ = home;
    let path = PathBuf::from(text);
    path.is_dir().then_some(path)
}

/// The game folders under one launcher folder: the folder itself, its `.minecraft`, or each of its
/// `instances/`.
fn game_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut push = |dir: PathBuf| {
        if dir.join("saves").is_dir() && !dirs.contains(&dir) {
            dirs.push(dir);
        }
    };
    push(root.to_path_buf());
    push(root.join(".minecraft"));
    if let Ok(list) = std::fs::read_dir(root.join("instances")) {
        for entry in list.flatten() {
            let dir = entry.path();
            push(dir.join(".minecraft"));
            push(dir);
        }
    }
    dirs
}

/// Every game folder the known launchers left on this computer.
pub fn found() -> Vec<Found> {
    let mut all: Vec<Found> = Vec::new();
    for launcher in LAUNCHERS {
        for root in launcher.roots {
            let Some(root) = expand(root) else { continue };
            for game in game_dirs(&root) {
                if all.iter().any(|f| f.game == game) {
                    continue;
                }
                let worlds = crate::world::worlds(&game).len();
                all.push(Found {
                    id: launcher.id,
                    name: launcher.name,
                    kind: launcher.kind,
                    root: root.clone(),
                    modded: game.join("mods").is_dir(),
                    game,
                    worlds,
                });
            }
        }
    }
    all
}

/// The launchers this build knows, for an empty list that still says what it looks for.
pub fn catalogue() -> Vec<(&'static str, &'static str, &'static str)> {
    LAUNCHERS
        .iter()
        .map(|l| (l.id, l.name, l.kind))
        .collect()
}

/// Copies every world of a found install into this stick's `saves/`, never over one already there.
pub fn import(found: &Found, minecraft: &Path) -> Result<usize, String> {
    let target = minecraft.join("saves");
    std::fs::create_dir_all(&target).map_err(|e| format!("{}: {e}", target.display()))?;
    let Ok(list) = std::fs::read_dir(found.game.join("saves")) else {
        return Ok(0);
    };
    let mut copied = 0;
    for entry in list.flatten() {
        let from = entry.path();
        if !from.is_dir() || !from.join("level.dat").is_file() {
            continue;
        }
        let name = entry.file_name();
        let to = target.join(&name);
        if to.exists() {
            continue;
        }
        copy_tree(&from, &to)?;
        copied += 1;
    }
    Ok(copied)
}

fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    let list = std::fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))?;
    for entry in list.flatten() {
        let source = entry.path();
        let target = to.join(entry.file_name());
        match entry.file_type() {
            Ok(kind) if kind.is_dir() => copy_tree(&source, &target)?,
            Ok(kind) if kind.is_file() => {
                std::fs::copy(&source, &target)
                    .map_err(|e| format!("{}: {e}", target.display()))?;
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("offhorizon-others-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_launcher_folder_with_instances_lists_each_game_folder() {
        let root = scratch("instances");
        let one = root.join("instances/one/.minecraft");
        std::fs::create_dir_all(one.join("saves")).unwrap();
        std::fs::create_dir_all(one.join("mods")).unwrap();
        let dirs = game_dirs(&root);
        assert_eq!(dirs.len(), 1);
        assert!(dirs[0].ends_with("one/.minecraft") || dirs[0].ends_with("one\\.minecraft"));
    }

    #[test]
    fn importing_copies_a_world_and_leaves_the_one_already_there() {
        let dir = scratch("import");
        let game = dir.join("sklauncher");
        let world = game.join("saves/New World");
        std::fs::create_dir_all(&world).unwrap();
        std::fs::write(world.join("level.dat"), b"nbt").unwrap();
        std::fs::create_dir_all(game.join("saves/Old World")).unwrap();
        std::fs::write(game.join("saves/Old World/level.dat"), b"nbt").unwrap();
        let minecraft = dir.join("stick/.minecraft");
        std::fs::create_dir_all(minecraft.join("saves/Old World")).unwrap();
        std::fs::write(minecraft.join("saves/Old World/level.dat"), b"mine").unwrap();

        let found = Found {
            id: "sklauncher",
            name: "SKLauncher",
            kind: "Java",
            root: game.clone(),
            game,
            worlds: 2,
            modded: false,
        };
        let copied = import(&found, &minecraft).unwrap();
        assert_eq!(copied, 1);
        assert_eq!(
            std::fs::read(minecraft.join("saves/New World/level.dat")).unwrap(),
            b"nbt"
        );
        assert_eq!(
            std::fs::read(minecraft.join("saves/Old World/level.dat")).unwrap(),
            b"mine"
        );
    }
}
