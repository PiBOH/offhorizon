//! Every action the window and the terminal can take. Both front ends call the same functions, so
//! anything the window does the terminal does, and the other way round.

use crate::instances::{self, Edition, Instance};
use crate::launch;
use crate::launcher::{self, LauncherChange, Layout};
use crate::native::{self, Native};
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

/// A start worked out but not made yet: the Rust client when one is here, Java when that is what
/// plays.
pub enum Plan {
    Rust(native::Plan),
    Java(launch::Plan),
}

impl Plan {
    /// The version this start would run.
    pub fn version(&self) -> &str {
        match self {
            Plan::Rust(plan) => &plan.tag,
            Plan::Java(plan) => &plan.id,
        }
    }

    /// Starts the game and comes back at once: the window keeps running.
    pub fn spawn(&self) -> Result<(), String> {
        match self {
            Plan::Rust(plan) => plan.spawn(),
            Plan::Java(plan) => plan.spawn(),
        }
    }

    /// The lines `play --dry-run` prints: what would run, and nothing started.
    pub fn lines(&self) -> Vec<String> {
        // shortcut: this is a debug dump, so its labels stay English.
        match self {
            Plan::Rust(plan) => vec![
                "edition: rust".to_string(),
                format!("version: {}", plan.tag),
                format!("program: {}", plan.program.display()),
                format!("args: {}", plan.args.join(" ")),
                format!("cwd: {}", plan.cwd.display()),
                format!("home: {}", plan.home.display()),
            ],
            Plan::Java(plan) => vec![
                "edition: java".to_string(),
                format!("version: {}", plan.id),
                format!("java: {}", plan.java.display()),
                format!("main: {}", plan.main_class),
                format!("jvm: {}", plan.jvm.join(" ")),
                format!("args: {}", plan.game_args.join(" ")),
                format!("cwd: {}", plan.cwd.display()),
            ],
        }
    }
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

    /// Every Horizon client (Minecraft: Rust Edition) found beside this stick, the newest first.
    pub fn natives(&self) -> Vec<Native> {
        native::found(&self.portable.base)
    }

    /// The Rust client to play: the one named, the one played last, or the newest found.
    pub fn native(&self, tag: Option<&str>) -> Option<Native> {
        let found = self.natives();
        match tag {
            Some(tag) => found.into_iter().find(|client| client.tag == tag),
            None => {
                let saved = self.portable.get("native");
                saved
                    .as_deref()
                    .and_then(|tag| found.iter().find(|client| client.tag == tag).cloned())
                    .or_else(|| found.first().cloned())
            }
        }
    }

    /// Where the Rust game keeps its logs, its crash reports and its own settings: with the stick.
    pub fn native_home(&self) -> PathBuf {
        self.portable.tool().join("horizon")
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
        edition: Edition,
    ) -> Result<String, String> {
        let name = name.trim();
        if name.is_empty() {
            return Err(i18n::t("home.name_required"));
        }
        let game = game.unwrap_or_else(|| self.home());
        std::fs::create_dir_all(&game).map_err(|e| format!("{}: {e}", game.display()))?;
        // The version has to be here to play: a Rust client beside the stick, or a Java version in
        // the folder the instance plays from, where its jars are.
        let there = match edition {
            Edition::Rust => self.native(Some(version)).is_some(),
            Edition::Java => self
                .versions()
                .into_iter()
                .any(|v| v.id == version && v.game == game),
        };
        if !there {
            return Err(i18n::tf(
                "home.version_not_there",
                &[
                    ("version", version.to_string()),
                    ("folder", game.display().to_string()),
                ],
            ));
        }
        let mut list = self.instances();
        list.retain(|known| known.name != name);
        list.push(Instance {
            name: name.to_string(),
            version: version.to_string(),
            game,
            edition,
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

    /// The command that would start an instance, without starting it.
    pub fn plan_instance(&self, name: &str) -> Result<Plan, String> {
        let Some(instance) = self.instances().into_iter().find(|known| known.name == name) else {
            return Err(i18n::tf("home.instance_missing", &[("name", name.to_string())]));
        };
        match instance.edition {
            Edition::Rust => {
                let tag = instance.version.as_str();
                self.plan_native(Some(tag), Some(instance.game.clone()))
            }
            Edition::Java => {
                let versions = self.versions();
                let version = versions
                    .iter()
                    .find(|v| v.id == instance.version && v.game == instance.game)
                    .or_else(|| versions.iter().find(|v| v.id == instance.version))
                    .cloned()
                    .ok_or_else(|| i18n::t("home.no_version"))?;
                Ok(Plan::Java(launch::plan(
                    &instance.game,
                    &version,
                    &self.player_name(),
                )?))
            }
        }
    }

    /// Starts an instance in its own game folder, as the offline name.
    pub fn play_instance(&mut self, name: &str) -> Result<String, String> {
        let plan = self.plan_instance(name)?;
        let player = self.player_name();
        let version = plan.version().to_string();
        plan.spawn()?;
        self.portable.set("instance", name)?;
        Ok(i18n::tf(
            "home.launched",
            &[("version", version), ("name", player)],
        ))
    }

    /// The command that would start the Rust game, without starting it. `game` is the folder it
    /// plays in: the stick's when a caller has none of its own.
    pub fn plan_native(&self, tag: Option<&str>, game: Option<PathBuf>) -> Result<Plan, String> {
        let client = self.native(tag).ok_or_else(|| i18n::t("home.rust_missing"))?;
        let game = game.unwrap_or_else(|| self.home());
        Ok(Plan::Rust(native::plan(
            &client,
            &self.player_name(),
            &game,
            &self.native_home(),
        )))
    }

    /// Starts the Rust game as the offline name, keeping the choice with the stick.
    pub fn play_native(&mut self, tag: Option<&str>) -> Result<String, String> {
        let plan = self.plan_native(tag, None)?;
        let version = plan.version().to_string();
        plan.spawn()?;
        self.portable.set("native", &version)?;
        Ok(i18n::tf(
            "home.launched",
            &[("version", version), ("name", self.player_name())],
        ))
    }

    /// The command that would start a version in one edition, without starting it.
    pub fn plan_edition(&self, id: Option<&str>, edition: Edition) -> Result<Plan, String> {
        if edition == Edition::Rust {
            return self.plan_native(id, None);
        }
        let versions = self.versions();
        let picked = match id {
            Some(id) => versions.into_iter().find(|v| v.id == id),
            None => self
                .portable
                .get("version")
                .and_then(|id| versions.iter().find(|v| v.id == id).cloned())
                .or_else(|| versions.first().cloned()),
        };
        let picked = picked.ok_or_else(|| i18n::t("home.no_version"))?;
        Ok(Plan::Java(launch::plan(
            &picked.game,
            &picked,
            &self.player_name(),
        )?))
    }

    /// The command that would start a version, without starting it. Rust first: a name the Rust
    /// clients know plays in Rust, and with no name the newest client plays when there is one.
    pub fn plan(&self, id: Option<&str>) -> Result<Plan, String> {
        match id {
            Some(id) if self.native(Some(id)).is_some() => self.plan_native(Some(id), None),
            Some(id) => self.plan_edition(Some(id), Edition::Java),
            None if self.native(None).is_some() => self.plan_native(None, None),
            None => self.plan_edition(None, Edition::Java),
        }
    }

    /// Starts a version, keeping the choice with the stick. With no edition the one that fits plays;
    /// with no version, the one played last.
    pub fn play_edition(
        &mut self,
        id: Option<&str>,
        edition: Option<Edition>,
    ) -> Result<String, String> {
        let plan = match edition {
            Some(edition) => self.plan_edition(id, edition)?,
            None => self.plan(id)?,
        };
        let name = self.player_name();
        let version = plan.version().to_string();
        plan.spawn()?;
        match &plan {
            Plan::Rust(_) => self.portable.set("native", &version)?,
            Plan::Java(_) => self.portable.set("version", &version)?,
        };
        Ok(i18n::tf(
            "home.launched",
            &[("version", version), ("name", name)],
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
        let clients = self.natives();
        lines.push(format!(
            "{}: {}",
            i18n::t("home.rust"),
            clients.len()
        ));
        for client in &clients {
            lines.push(format!(
                "  {} — {} — {}",
                client.tag,
                native::minecraft_of(client),
                client.program.display()
            ));
        }
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
