//! Every action the window and the terminal can take. Both front ends call the same functions, so
//! anything the window does the terminal does, and the other way round.

use crate::instances::{self, Instance};
use crate::launch;
use crate::launcher::{self, LauncherChange, Layout};
use crate::nbt::Root;
use crate::others::{self, Found};
use crate::portable::Portable;
use crate::store::{Backup, Store};
use crate::world::{self, Preview, Weather, World};
use crate::{i18n, VERSION};
use std::path::PathBuf;

/// The tool as it is right now: where it lives, what it keeps, and what it found.
pub struct Session {
    pub portable: Portable,
    pub store: Store,
    pub layout: Layout,
    pub lang: String,
}

/// The game folders, worlds and previews a page shows.
#[derive(Clone, Default)]
pub struct Snapshot {
    pub homes: Vec<PathBuf>,
    pub worlds: Vec<World>,
    pub previews: Vec<Preview>,
}

impl Session {
    /// Opens the tool, reads the language and looks around.
    pub fn open() -> Self {
        let portable = Portable::find();
        let lang = i18n::resolve(&portable);
        i18n::init(&lang);
        let store = Store::at(portable.backups());
        let layout = launcher::detect(&portable);
        Self {
            portable,
            store,
            layout,
            lang,
        }
    }

    /// Looks again: a game folder, a launcher or a world may have appeared.
    pub fn rescan(&mut self) {
        self.layout = launcher::detect(&self.portable);
        self.store.reload();
    }

    /// The game folder that plays from this stick.
    pub fn home(&self) -> PathBuf {
        self.portable.minecraft()
    }

    /// Every game folder, the portable one first.
    pub fn homes(&self) -> Vec<PathBuf> {
        let mut homes = vec![self.home()];
        for home in &self.layout.homes {
            if !homes.contains(home) {
                homes.push(home.clone());
            }
        }
        homes
    }

    /// Every world of every game folder, the most recently played first.
    pub fn snapshot(&self) -> Snapshot {
        let homes = self.homes();
        let mut worlds: Vec<World> = Vec::new();
        for home in &homes {
            for world in world::worlds(home) {
                if !worlds.iter().any(|w| w.level == world.level) {
                    worlds.push(world);
                }
            }
        }
        let previews: Vec<Preview> = worlds.iter().map(world::preview).collect();
        let mut order: Vec<usize> = (0..worlds.len()).collect();
        order.sort_by_key(|i| std::cmp::Reverse(previews[*i].modified));
        let worlds: Vec<World> = order.iter().map(|i| worlds[*i].clone()).collect();
        let previews: Vec<Preview> = order.iter().map(|i| previews[*i].clone()).collect();
        Snapshot {
            homes,
            worlds,
            previews,
        }
    }

    /// The world a request names, or the one played last.
    pub fn find_world(&self, name: Option<&str>) -> Result<World, String> {
        let snapshot = self.snapshot();
        match name {
            Some(name) => snapshot
                .worlds
                .iter()
                .find(|w| w.name.eq_ignore_ascii_case(name))
                .cloned()
                .ok_or_else(|| {
                    i18n::tf("status.world_missing", &[("name", name.to_string())])
                }),
            // A world on the stick first: the system install is only read, and only when the
            // stick has nothing to play.
            None => {
                let home = self.home();
                snapshot
                    .worlds
                    .iter()
                    .find(|w| w.level.starts_with(&home))
                    .or_else(|| snapshot.worlds.first())
                    .cloned()
                    .ok_or_else(|| i18n::t("status.no_world"))
            }
        }
    }

    /// Copies the save, changes it and writes it back: the one path every edit takes.
    fn edit(
        &mut self,
        world: &World,
        what: &str,
        edit: impl FnOnce(&mut Root) -> Result<String, String>,
    ) -> Result<String, String> {
        let mut root = world::load(&world.level)?;
        let said = edit(&mut root)?;
        self.store.backup(&world.level, what).map_err(|why| {
            i18n::tf(
                "status.backup_failed",
                &[("why", why)],
            )
        })?;
        world::save(&world.level, &root)?;
        Ok(i18n::tf(
            "status.done",
            &[("what", format!("{what}: {said}"))],
        ))
    }

    pub fn set_name(&mut self, name: &str) -> Result<String, String> {
        let home = self.home();
        launcher::set_offline_name(&mut self.store, &home, name)?;
        Ok(format!("name:{}", name.trim()))
    }

    /// Every version installed in the game folders this tool reads, the stick's first.
    pub fn versions(&self) -> Vec<launch::Version> {
        let mut all: Vec<launch::Version> = Vec::new();
        for home in self.homes() {
            for version in launch::versions(&home) {
                if all
                    .iter()
                    .any(|known| known.id == version.id && known.game == version.game)
                {
                    continue;
                }
                all.push(version);
            }
        }
        all
    }

    /// The name the game plays under: the file on the stick, or `Player` until one is set.
    pub fn player_name(&self) -> String {
        launcher::offline_name(&self.home()).unwrap_or_else(|| "Player".to_string())
    }

    /// The instances kept with this stick.
    pub fn instances(&self) -> Vec<Instance> {
        instances::load(&self.portable.tool())
    }

    /// A new instance: one version, and a game folder of its own. With no folder it plays beside
    /// this executable; with one it plays wherever that folder is.
    pub fn create_instance(
        &mut self,
        name: &str,
        version: &str,
        game: Option<PathBuf>,
    ) -> Result<String, String> {
        let name = name.trim();
        if name.is_empty() {
            return Err(i18n::t("home.name_required"));
        }
        let Some(found) = self.versions().into_iter().find(|v| v.id == version) else {
            return Err(i18n::t("home.no_version"));
        };
        let game = game.unwrap_or_else(|| self.home());
        std::fs::create_dir_all(&game).map_err(|e| format!("{}: {e}", game.display()))?;
        let mut list = self.instances();
        list.retain(|known| known.name != name);
        list.push(Instance {
            name: name.to_string(),
            version: found.id,
            game,
        });
        instances::save(&self.portable.tool(), &list)?;
        self.portable.set("instance", name)?;
        Ok(i18n::tf("home.instance_created", &[("name", name.to_string())]))
    }

    /// Forgets an instance. Nothing on disk is deleted: the folder stays where it is.
    pub fn remove_instance(&mut self, name: &str) -> Result<String, String> {
        let mut list = self.instances();
        let before = list.len();
        list.retain(|known| known.name != name);
        if list.len() == before {
            return Err(i18n::tf("home.instance_missing", &[("name", name.to_string())]));
        }
        instances::save(&self.portable.tool(), &list)?;
        Ok(i18n::tf("home.instance_removed", &[("name", name.to_string())]))
    }

    /// Starts an instance in its own game folder, as the offline name.
    pub fn play_instance(&mut self, name: &str) -> Result<String, String> {
        let Some(instance) = self.instances().into_iter().find(|known| known.name == name) else {
            return Err(i18n::tf("home.instance_missing", &[("name", name.to_string())]));
        };
        let versions = self.versions();
        let version = versions
            .iter()
            .find(|v| v.id == instance.version && v.game == instance.game)
            .or_else(|| versions.iter().find(|v| v.id == instance.version))
            .cloned()
            .ok_or_else(|| i18n::t("home.no_version"))?;
        let name = self.player_name();
        launch::plan(&instance.game, &version, &name)?.spawn()?;
        self.portable.set("instance", &instance.name)?;
        Ok(i18n::tf(
            "home.launched",
            &[("version", version.id), ("name", name)],
        ))
    }

    /// The command that would start a version, without starting it.
    pub fn plan(&self, id: Option<&str>) -> Result<launch::Plan, String> {
        let versions = self.versions();
        let picked = match id {
            Some(id) => versions.iter().find(|v| v.id == id).cloned(),
            None => self
                .portable
                .get("version")
                .and_then(|id| versions.iter().find(|v| v.id == id).cloned())
                .or_else(|| versions.first().cloned()),
        };
        let picked = picked.ok_or_else(|| i18n::t("home.no_version"))?;
        launch::plan(&picked.game, &picked, &self.player_name())
    }

    /// Starts a version, keeping the choice with the stick. With no name, the one played last.
    pub fn play(&mut self, id: Option<&str>) -> Result<String, String> {
        let plan = self.plan(id)?;
        let name = self.player_name();
        plan.spawn()?;
        self.portable.set("version", &plan.id)?;
        Ok(i18n::tf(
            "home.launched",
            &[("version", plan.id), ("name", name)],
        ))
    }

    pub fn heal(&mut self, world: &World) -> Result<String, String> {
        self.edit(world, "Heal", world::heal)
    }

    pub fn feed(&mut self, world: &World) -> Result<String, String> {
        self.edit(world, "Feed", world::feed)
    }

    pub fn set_level(&mut self, world: &World, level: i32) -> Result<String, String> {
        self.edit(world, "Set level", |root| world::set_level(root, level))
    }

    pub fn add_item(&mut self, world: &World, id: &str, count: i32) -> Result<String, String> {
        let id = id.to_string();
        self.edit(world, "Add item", |root| world::add_item(root, &id, count))
    }

    pub fn remove_slot(&mut self, world: &World, slot: i32) -> Result<String, String> {
        self.edit(world, "Remove item", |root| world::remove_slot(root, slot))
    }

    pub fn set_time(&mut self, world: &World, ticks: i64) -> Result<String, String> {
        self.edit(world, "Set time", |root| world::set_time(root, ticks))
    }

    pub fn set_weather(
        &mut self,
        world: &World,
        weather: Weather,
        ticks: i32,
    ) -> Result<String, String> {
        self.edit(world, "Set weather", |root| world::set_weather(root, weather, ticks))
    }

    /// Points the installed launcher at the folder beside this executable, keeping every copy of
    /// what the settings were.
    pub fn point_launcher(&mut self) -> Result<LauncherChange, String> {
        let launcher = self
            .layout
            .launcher
            .clone()
            .ok_or_else(|| i18n::t("status.no_launcher"))?;
        let change = launcher::point_at(
            &mut self.store,
            &self.portable,
            &launcher,
            &self.portable.minecraft(),
        )?;
        self.layout = launcher::detect(&self.portable);
        Ok(change)
    }

    /// Puts the launcher's settings back.
    pub fn undo_launcher(&mut self) -> Result<PathBuf, String> {
        let launcher = self
            .layout
            .launcher
            .clone()
            .ok_or_else(|| i18n::t("status.no_launcher"))?;
        let file = launcher::undo(&launcher)?;
        self.layout = launcher::detect(&self.portable);
        Ok(file)
    }

    /// The launchers found on this computer.
    pub fn launchers(&self) -> Vec<Found> {
        others::found()
    }

    /// Copies every world of one found launcher into the stick.
    pub fn import(&mut self, id: &str) -> Result<String, String> {
        let found: Vec<Found> = self
            .launchers()
            .into_iter()
            .filter(|f| f.id == id)
            .collect();
        if found.is_empty() {
            return Err(i18n::t("launchers.none"));
        }
        let mut copied = 0;
        for install in &found {
            copied += others::import(install, &self.portable.minecraft())?;
        }
        Ok(i18n::tf(
            "status.imported",
            &[
                ("count", copied.to_string()),
                ("launcher", found[0].name.to_string()),
            ],
        ))
    }

    pub fn backups(&self) -> &[Backup] {
        &self.store.entries
    }

    pub fn restore(&mut self, id: &str) -> Result<String, String> {
        let file = self.store.restore(id)?;
        Ok(file.display().to_string())
    }

    pub fn set_language(&mut self, code: &str) -> Result<String, String> {
        i18n::save(&self.portable, code)?;
        i18n::init(code);
        self.lang = code.to_string();
        Ok(i18n::tf("status.lang_set", &[("lang", code.to_string())]))
    }

    /// The lines `status` prints: where everything is, and what it found.
    pub fn status(&mut self) -> Vec<String> {
        self.rescan();
        let mut lines = vec![
            format!("{} {}", i18n::t("app.name"), VERSION),
            format!("{}: {}", i18n::t("portable.minecraft"), self.home().display()),
            format!("{}: {}", i18n::t("portable.tool"), self.portable.tool().display()),
            format!(
                "{}: {}",
                i18n::t("portable.telemetry"),
                i18n::t("portable.telemetry_off")
            ),
            format!(
                "{}: {}",
                i18n::t("portable.launcher"),
                self.layout
                    .launcher
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "—".into())
            ),
        ];
        let snapshot = self.snapshot();
        lines.push(format!(
            "{}: {}",
            i18n::t("nav.worlds"),
            snapshot.worlds.len()
        ));
        for preview in &snapshot.previews {
            lines.push(format!(
                "  {} — {} {}",
                preview.label,
                preview.health.map(|h| format!("{h:.1}/20")).unwrap_or_else(|| "—".into()),
                if preview.items.is_empty() {
                    String::new()
                } else {
                    format!("· {} {}", preview.items.len(), i18n::t("worlds.items"))
                }
            ));
        }
        let found = self.launchers();
        if found.is_empty() {
            lines.push(i18n::t("launchers.none"));
        } else {
            for install in found {
                lines.push(format!(
                    "{} ({}) — {} {} — {}",
                    install.name,
                    install.kind,
                    install.worlds,
                    i18n::t("launchers.worlds"),
                    install.game.display()
                ));
            }
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("offhorizon-actions-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_save_edited_through_the_session_is_written_and_kept() {
        let dir = scratch("edit");
        let home = dir.join("stick/.minecraft");
        let world_dir = home.join("saves/Test");
        std::fs::create_dir_all(&world_dir).unwrap();
        let mut root = Root {
            name: String::new(),
            tag: crate::nbt::Tag::Compound(vec![(
                "Data".into(),
                crate::nbt::Tag::Compound(vec![(
                    "Player".into(),
                    crate::nbt::Tag::Compound(vec![
                        ("Health".into(), crate::nbt::Tag::Float(4.0)),
                        ("foodLevel".into(), crate::nbt::Tag::Int(3)),
                    ]),
                )]),
            )]),
        };
        world::save(&world_dir.join("level.dat"), &mut root).unwrap();

        let mut session = Session {
            portable: Portable::at(dir.join("stick")),
            store: Store::at(dir.join("stick/.offhorizon/backups")),
            layout: Layout::default(),
            lang: "en".into(),
        };
        i18n::init("en");
        let world = session
            .find_world(Some("Test"))
            .expect("the world is under the portable folder");
        session.heal(&world).unwrap();
        let saved = world::load(&world.level).unwrap();
        assert_eq!(world::stats(&saved).unwrap().health, 20.0);
        assert_eq!(session.store.entries.len(), 1);
    }
}
