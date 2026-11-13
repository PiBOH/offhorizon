//! The terminal. Every command here is a button in the window, and every button in the window is a
//! command here: both front ends call [`crate::actions::Session`].

use crate::actions::Session;
use crate::instances::Edition;
use crate::world::{self, Weather};
use crate::i18n;

// shortcut: the help text is English only, translate it when the commands grow.
const USAGE: &str = "\
OffHorizon — portable companion for Horizon

  offhorizon                       open the window
  offhorizon status                where everything is, what it found
  offhorizon versions              the Horizon clients and Java versions found here
  offhorizon instance list         the instances on this stick
  offhorizon instance new <NAME> <VERSION> [--dir PATH] [--rust|--java]
                                   one version in a game folder of its own
                                   (no --dir: the folder beside this file)
  offhorizon instance remove <NAME>
  offhorizon play [VERSION]        start the game as the offline name
                                   Rust first: --rust and --java pick the edition,
                                   --instance <NAME>, --dry-run: print the command instead
  offhorizon worlds                the saves, with health and inventory
  offhorizon name <NAME>           the offline player name
  offhorizon heal                  full health, air, fire out
  offhorizon feed                  full hunger
  offhorizon level <N>             experience level
  offhorizon item add <ID> [N]     put an item in the inventory
  offhorizon item remove <SLOT>    take a stack out
  offhorizon time <TICKS|day|noon|night|midnight>
  offhorizon weather <clear|rain|thunder> [TICKS]
  offhorizon launcher point        point the installed launcher at this stick
  offhorizon launcher undo         put the launcher's settings back
  offhorizon launchers             the other launchers found here
  offhorizon import <ID>           copy a launcher's worlds onto this stick
  offhorizon backups               every file this tool copied
  offhorizon restore <ID>          put a backup back
  offhorizon lang <CODE>|list      the language of the window and the terminal
  offhorizon help | version

Every world command takes --world <NAME>; without it, the world played last.";

/// Runs the terminal front end, answering the process exit code.
pub fn run(args: &[String]) -> i32 {
    let mut session = Session::open();
    let mut world_name: Option<String> = None;
    let mut instance: Option<String> = None;
    let mut dir: Option<String> = None;
    let mut edition: Option<Edition> = None;
    let mut dry_run = false;
    let mut words: Vec<String> = Vec::new();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--world" | "-w" => {
                world_name = args.get(index + 1).cloned();
                index += 2;
            }
            "--instance" | "-i" => {
                instance = args.get(index + 1).cloned();
                index += 2;
            }
            "--dir" | "-d" => {
                dir = args.get(index + 1).cloned();
                index += 2;
            }
            "--rust" => {
                edition = Some(Edition::Rust);
                index += 1;
            }
            "--java" => {
                edition = Some(Edition::Java);
                index += 1;
            }
            "--dry-run" => {
                dry_run = true;
                index += 1;
            }
            other => {
                words.push(other.to_string());
                index += 1;
            }
        }
    }
    let word = |at: usize| words.get(at).map(String::as_str).unwrap_or("");
    match word(0) {
        "" => {
            println!("{USAGE}");
            2
        }
        "help" | "--help" | "-h" => {
            println!("{USAGE}");
            0
        }
        "version" | "--version" | "-V" => {
            println!("{} {}", i18n::t("app.name"), crate::VERSION);
            0
        }
        "versions" => {
            let mut lines: Vec<String> = Vec::new();
            // The Rust clients first: that is the edition this tool plays when it can.
            for client in session.natives() {
                lines.push(format!(
                    "{:<24} {:<16} {:<12} {}",
                    client.tag,
                    i18n::t("home.rust"),
                    crate::native::minecraft_of(&client),
                    client.program.display()
                ));
            }
            for version in session.versions() {
                lines.push(format!(
                    "{:<24} {:<16} {:<12} {}",
                    version.id,
                    version.loader,
                    version.release_time,
                    version.game.display()
                ));
            }
            if lines.is_empty() {
                return done(vec![i18n::t("home.no_version")]);
            }
            done(lines)
        }
        "instance" => match word(1) {
            "" | "list" => {
                let list = session.instances();
                if list.is_empty() {
                    return done(vec![i18n::t("home.no_instances")]);
                }
                done(list
                    .iter()
                    .map(|i| {
                        format!(
                            "{:<20} {:<16} {:<26} {}",
                            i.name,
                            i.edition.name(),
                            i.version,
                            i.game.display()
                        )
                    })
                    .collect())
            }
            "new" => {
                if word(2).is_empty() || word(3).is_empty() {
                    return usage("instance new <NAME> <VERSION> [--dir PATH] [--rust|--java]");
                }
                let name = word(2).to_string();
                let version = word(3).to_string();
                let game = dir.clone().map(std::path::PathBuf::from);
                // Rust first: a version the Rust clients carry plays in Rust, unless --java says
                // otherwise.
                let edition = edition.unwrap_or_else(|| {
                    if session.native(Some(version.as_str())).is_some() {
                        Edition::Rust
                    } else {
                        Edition::Java
                    }
                });
                result(session.create_instance(&name, &version, game, edition))
            }
            "remove" => {
                if word(2).is_empty() {
                    return usage("instance remove <NAME>");
                }
                let name = word(2).to_string();
                result(session.remove_instance(&name))
            }
            _ => usage("instance list|new|remove"),
        },
        "play" => {
            let id = (!word(1).is_empty()).then(|| word(1).to_string());
            let planned = match &instance {
                Some(name) => session.plan_instance(name),
                None => match edition {
                    Some(edition) => session.plan_edition(id.as_deref(), edition),
                    None => session.plan(id.as_deref()),
                },
            };
            if dry_run {
                return match planned {
                    Ok(plan) => done(plan.lines()),
                    Err(why) => fail(why),
                };
            }
            match &instance {
                Some(name) => result(session.play_instance(name)),
                None => result(session.play_edition(id.as_deref(), edition)),
            }
        }
        "status" => done(session.status()),
        "worlds" => {
            let snapshot = session.snapshot();
            if snapshot.previews.is_empty() {
                done(vec![i18n::t("worlds.none")])
            } else {
                let lines = snapshot
                    .previews
                    .iter()
                    .map(|p| {
                        format!(
                            "{}  {:.1}/20  {} {}/20  {} {}  {}  {}",
                            p.label,
                            p.health.unwrap_or(0.0),
                            i18n::t("worlds.food"),
                            p.food.unwrap_or(0),
                            i18n::t("worlds.level"),
                            p.xp_level.unwrap_or(0),
                            format_bytes(p.bytes),
                            p.items
                                .iter()
                                .map(|i| format!("{} x{}", short(&i.id), i.count))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    })
                    .collect();
                done(lines)
            }
        }
        "name" => {
            if word(1).is_empty() {
                return usage("name <NAME>");
            }
            let name = word(1).to_string();
            result(session.set_name(&name))
        }
        "heal" | "feed" => {
            let world = match session.find_world(world_name.as_deref()) {
                Ok(world) => world,
                Err(why) => return fail(why),
            };
            if word(0) == "heal" {
                result(session.heal(&world))
            } else {
                result(session.feed(&world))
            }
        }
        "level" => {
            let Ok(level) = word(1).parse::<i32>() else {
                return usage("level <N>");
            };
            let world = match session.find_world(world_name.as_deref()) {
                Ok(world) => world,
                Err(why) => return fail(why),
            };
            result(session.set_level(&world, level))
        }
        "item" => {
            let world = match session.find_world(world_name.as_deref()) {
                Ok(world) => world,
                Err(why) => return fail(why),
            };
            match word(1) {
                "add" => {
                    if word(2).is_empty() {
                        return usage("item add <ID> [N]");
                    }
                    let count = word(3).parse::<i32>().unwrap_or(1);
                    let id = word(2).to_string();
                    result(session.add_item(&world, &id, count))
                }
                "remove" => {
                    let Ok(slot) = word(2).parse::<i32>() else {
                        return usage("item remove <SLOT>");
                    };
                    result(session.remove_slot(&world, slot))
                }
                _ => usage("item add|remove"),
            }
        }
        "time" => {
            let ticks = match word(1).to_lowercase().as_str() {
                "day" | "morning" => world::TICKS_DAY,
                "noon" => world::TICKS_NOON,
                "night" => world::TICKS_NIGHT,
                "midnight" => world::TICKS_MIDNIGHT,
                other => match other.parse::<i64>() {
                    Ok(ticks) => ticks,
                    Err(_) => return usage("time <TICKS|day|noon|night|midnight>"),
                },
            };
            let world = match session.find_world(world_name.as_deref()) {
                Ok(world) => world,
                Err(why) => return fail(why),
            };
            result(session.set_time(&world, ticks))
        }
        "weather" => {
            let Some(weather) = Weather::parse(word(1)) else {
                return usage("weather <clear|rain|thunder> [TICKS]");
            };
            let ticks = word(2).parse::<i32>().unwrap_or(6_000);
            let world = match session.find_world(world_name.as_deref()) {
                Ok(world) => world,
                Err(why) => return fail(why),
            };
            result(session.set_weather(&world, weather, ticks))
        }
        "launcher" => match word(1) {
            "point" => match session.point_launcher() {
                Ok(change) => done(vec![
                    format!("{}: {}", i18n::t("portable.launcher"), change.settings.display()),
                    format!(
                        "{}: {}",
                        i18n::t("portable.point"),
                        session.home().display()
                    ),
                    format!("{} backups", change.copies.len()),
                ]),
                Err(why) => fail(why),
            },
            "undo" => match session.undo_launcher() {
                Ok(file) => done(vec![format!(
                    "{} {}",
                    i18n::t("portable.unpoint"),
                    file.display()
                )]),
                Err(why) => fail(why),
            },
            _ => usage("launcher point|undo"),
        },
        "launchers" => {
            let found = session.launchers();
            let mut lines = Vec::new();
            for install in found {
                lines.push(format!(
                    "{:<10} {:<20} {:<14} {:>3} {}  {}",
                    install.id,
                    install.name,
                    install.kind,
                    install.worlds,
                    install.game.display(),
                    if install.modded { "modded" } else { "" }
                ));
            }
            if lines.is_empty() {
                lines.push(i18n::t("launchers.none"));
            }
            done(lines)
        }
        "import" => {
            if word(1).is_empty() {
                return usage("import <ID>");
            }
            let id = word(1).to_string();
            result(session.import(&id))
        }
        "backups" => {
            session.store.reload();
            let entries = session.backups().to_vec();
            if entries.is_empty() {
                return done(vec![i18n::t("backups.none")]);
            }
            done(entries
                .iter()
                .map(|b| {
                    format!(
                        "{}  {}  {}  {}",
                        b.id,
                        b.tool,
                        b.original.display(),
                        format_bytes(b.bytes)
                    )
                })
                .collect())
        }
        "restore" => {
            if word(1).is_empty() {
                return usage("restore <ID>");
            }
            let id = word(1).to_string();
            result(session.restore(&id))
        }
        "lang" => match word(1) {
            "" | "list" => done(
                i18n::LANGUAGES
                    .iter()
                    .map(|(code, name)| format!("{code:<3} {name}"))
                    .collect(),
            ),
            code => {
                let code = code.to_string();
                result(session.set_language(&code))
            }
        },
        other => usage(other),
    }
}

fn short(id: &str) -> &str {
    id.strip_prefix("minecraft:").unwrap_or(id)
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MiB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KiB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

fn done(lines: Vec<String>) -> i32 {
    for line in lines {
        println!("{line}");
    }
    0
}

fn result(what: Result<String, String>) -> i32 {
    match what {
        Ok(said) => {
            println!("{said}");
            0
        }
        Err(why) => fail(why),
    }
}

fn fail(why: String) -> i32 {
    eprintln!("{}", i18n::tf("status.error", &[("why", why)]));
    1
}

fn usage(what: &str) -> i32 {
    eprintln!("offhorizon {what}\n\n{USAGE}");
    2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_short_name_of_a_vanilla_item_drops_the_namespace() {
        assert_eq!(short("minecraft:bread"), "bread");
        assert_eq!(short("mymod:thing"), "mymod:thing");
    }

    #[test]
    fn sizes_read_the_way_a_player_expects() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2_048), "2.0 KiB");
        assert_eq!(format_bytes(3 * 1_048_576), "3.0 MiB");
    }
}
