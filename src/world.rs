//! The worlds and the player in them: reading a save, curing it, feeding it, and putting items in
//! and out of the inventory. Nothing here writes a file; `save` is the one call that does, and the
//! window copies the file first.

use crate::nbt::{self, Root, Tag};
use std::path::{Path, PathBuf};

/// A world on disk: its folder name and its `level.dat`.
#[derive(Clone, Debug)]
pub struct World {
    pub name: String,
    pub level: PathBuf,
}

/// One stack in the inventory, as the window shows it.
#[derive(Clone, Debug)]
pub struct Item {
    pub slot: i32,
    pub id: String,
    pub count: i32,
}

/// What the player is, at the moment the save was last written.
#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub health: f32,
    pub absorption: f32,
    pub food: i32,
    pub saturation: f32,
    pub exhaustion: f32,
    pub air: i32,
    pub xp_level: i32,
    pub xp_total: i32,
    pub items: Vec<Item>,
}

/// The worlds under a game folder's `saves/`.
pub fn worlds(home: &Path) -> Vec<World> {
    let mut found = Vec::new();
    let Ok(list) = std::fs::read_dir(home.join("saves")) else {
        return found;
    };
    for entry in list.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let level = dir.join("level.dat");
        if !level.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        found.push(World { name, level });
    }
    found.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    found
}

/// Reads a world's `level.dat`.
pub fn load(level: &Path) -> Result<Root, String> {
    let bytes = std::fs::read(level).map_err(|e| format!("{}: {e}", level.display()))?;
    nbt::read_gzip(&bytes).map_err(|e| format!("{}: {e}", level.display()))
}

/// Writes a world's `level.dat` back, through a temporary file so a crash leaves the old one whole.
pub fn save(level: &Path, root: &Root) -> Result<(), String> {
    let bytes = nbt::write_gzip(root).map_err(|e| e.to_string())?;
    let tmp = level.with_extension("dat.offhorizon.tmp");
    std::fs::write(&tmp, bytes).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, level).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("{}: {e}", level.display())
    })
}

/// The player the save holds. A world that has never been opened has none yet.
fn player(root: &Root) -> Result<&Tag, String> {
    let player = root
        .get("Data")
        .and_then(|d| d.get("Player"))
        .ok_or_else(|| {
            "This save has no player yet: open the world once, save and quit, then try again."
                .to_string()
        })?;
    if player.get("Health").is_none() && player.get("foodLevel").is_none() {
        return Err("The save's player data is empty.".into());
    }
    Ok(player)
}

fn player_mut(root: &mut Root) -> Result<&mut Tag, String> {
    root.path_mut(&["Data", "Player"]).ok_or_else(|| {
        "This save has no player yet: open the world once, save and quit, then try again."
            .to_string()
    })
}

/// The player's numbers and inventory.
pub fn stats(root: &Root) -> Result<Stats, String> {
    let player = player(root)?;
    let mut stats = Stats {
        health: player.get("Health").and_then(Tag::as_f32).unwrap_or(20.0),
        absorption: player
            .get("AbsorptionAmount")
            .and_then(Tag::as_f32)
            .unwrap_or(0.0),
        food: player.get("foodLevel").and_then(Tag::as_i32).unwrap_or(20),
        saturation: player
            .get("foodSaturationLevel")
            .and_then(Tag::as_f32)
            .unwrap_or(5.0),
        exhaustion: player
            .get("foodExhaustionLevel")
            .and_then(Tag::as_f32)
            .unwrap_or(0.0),
        air: player.get("Air").and_then(Tag::as_i32).unwrap_or(300),
        xp_level: player.get("XpLevel").and_then(Tag::as_i32).unwrap_or(0),
        xp_total: player.get("XpTotal").and_then(Tag::as_i32).unwrap_or(0),
        items: Vec::new(),
    };
    if let Some(list) = player.get("Inventory").and_then(Tag::list) {
        for item in list {
            let Some(id) = item.get("id").and_then(Tag::as_str) else {
                continue;
            };
            stats.items.push(Item {
                slot: item.get("Slot").and_then(Tag::as_i32).unwrap_or(0),
                id: id.to_string(),
                count: item.get("Count").and_then(Tag::as_i32).unwrap_or(1),
            });
        }
    }
    stats.items.sort_by_key(|i| i.slot);
    Ok(stats)
}

/// Full health, no fire, air back: what a respawn gives, on the save the player already has.
pub fn heal(root: &mut Root) -> Result<String, String> {
    let player = player_mut(root)?;
    player.set("Health", Tag::Float(20.0));
    player.set("AbsorptionAmount", Tag::Float(0.0));
    player.set("Air", Tag::Short(300));
    player.set("Fire", Tag::Short(-1));
    player.set("HurtTime", Tag::Short(0));
    player.set("DeathTime", Tag::Short(0));
    Ok("Health 20/20, air 300, fire out".into())
}

/// A full hunger bar, with the saturation a fresh meal gives.
pub fn feed(root: &mut Root) -> Result<String, String> {
    let player = player_mut(root)?;
    player.set("foodLevel", Tag::Int(20));
    player.set("foodSaturationLevel", Tag::Float(5.0));
    player.set("foodExhaustionLevel", Tag::Float(0.0));
    Ok("Hunger 20/20, saturation 5.0".into())
}

/// Where a new stack of `id` goes: an existing stack that isn't full, else the first free slot.
fn place(player: &mut Tag, id: &str, count: i32) -> Result<(i32, i32, bool), String> {
    if !player.get("Inventory").is_some_and(|i| matches!(i, Tag::List(_))) {
        player.set("Inventory", Tag::List(Vec::new()));
    }
    let mut used: Vec<i32> = Vec::new();
    if let Some(list) = player.get("Inventory").and_then(Tag::list) {
        for item in list {
            let slot = item.get("Slot").and_then(Tag::as_i32).unwrap_or(0);
            used.push(slot);
            let same = item.get("id").and_then(Tag::as_str) == Some(id);
            let current = item.get("Count").and_then(Tag::as_i32).unwrap_or(0);
            let components = item.get("components").is_none();
            if same && components && current < 64 {
                return Ok((slot, current, true));
            }
        }
    }
    // The hotbar and the main inventory: 0-8 and 9-35. Armour and the offhand are left alone.
    let free = (0..36).find(|slot| !used.contains(slot));
    match free {
        Some(slot) => {
            let mut stack = Tag::Compound(vec![
                ("Slot".into(), Tag::Byte(slot as i8)),
                ("id".into(), Tag::String(id.to_string())),
                ("Count".into(), Tag::Byte(count.clamp(1, 64) as i8)),
            ]);
            stack.set_if_missing("Count", Tag::Byte(count.clamp(1, 64) as i8));
            if let Some(list) = player.get_mut("Inventory").and_then(Tag::list_mut) {
                list.push(stack);
            }
            Ok((slot, 0, false))
        }
        None => Err("The inventory is full: take something out first.".into()),
    }
}

/// Puts `count` of `id` in the inventory, over a stack that isn't full when there is one.
pub fn add_item(root: &mut Root, id: &str, count: i32) -> Result<String, String> {
    let id = id.trim();
    if id.is_empty() {
        return Err("Type an item id, for example minecraft:bread.".into());
    }
    if count <= 0 {
        return Err("The count must be at least 1.".into());
    }
    let player = player_mut(root)?;
    let (slot, had, stacked) = place(player, id, count)?;
    if stacked {
        let left = count;
        if let Some(list) = player.get_mut("Inventory").and_then(Tag::list_mut) {
            for item in list.iter_mut() {
                if item.get("Slot").and_then(Tag::as_i32) == Some(slot)
                    && item.get("id").and_then(Tag::as_str) == Some(id)
                {
                    let total = (had + left).min(64);
                    item.set("Count", Tag::Byte(total as i8));
                    break;
                }
            }
        }
        return Ok(format!("{id} x{count} added to slot {slot}"));
    }
    Ok(format!("{id} x{count} added to slot {slot}"))
}

/// Takes the stack in `slot` out.
pub fn remove_slot(root: &mut Root, slot: i32) -> Result<String, String> {
    let player = player_mut(root)?;
    let mut taken = None;
    if let Some(list) = player.get_mut("Inventory").and_then(Tag::list_mut) {
        if let Some(at) = list
            .iter()
            .position(|item| item.get("Slot").and_then(Tag::as_i32) == Some(slot))
        {
            let removed = list.remove(at);
            taken = removed
                .get("id")
                .and_then(Tag::as_str)
                .map(|id| id.to_string());
        }
    }
    match taken {
        Some(id) => Ok(format!("{id} removed from slot {slot}")),
        None => Err(format!("Slot {slot} is empty.")),
    }
}

/// Sets the experience level, keeping the total points in step with it.
pub fn set_level(root: &mut Root, level: i32) -> Result<String, String> {
    let level = level.clamp(0, 30_000);
    let player = player_mut(root)?;
    let total: i32 = (0..level)
        .map(|l| if l < 16 { 2 * l + 7 } else if l < 31 { 5 * l - 38 } else { 9 * l - 158 })
        .sum();
    player.set("XpLevel", Tag::Int(level));
    player.set("XpTotal", Tag::Int(total));
    player.set("XpP", Tag::Float(0.0));
    Ok(format!("Level {level}, {total} points"))
}

/// Ticks into a Minecraft day: dawn, noon, dusk and midnight.
pub const TICKS_DAY: i64 = 1_000;
pub const TICKS_NOON: i64 = 6_000;
pub const TICKS_NIGHT: i64 = 13_000;
pub const TICKS_MIDNIGHT: i64 = 18_000;

/// The weather the save should hold from now on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weather {
    Clear,
    Rain,
    Thunder,
}

impl Weather {
    /// The name the CLI and the window both use.
    pub fn parse(word: &str) -> Option<Self> {
        match word.trim().to_lowercase().as_str() {
            "clear" | "sun" | "sunny" | "sereno" => Some(Weather::Clear),
            "rain" | "raining" | "pioggia" => Some(Weather::Rain),
            "thunder" | "storm" | "temporale" => Some(Weather::Thunder),
            _ => None,
        }
    }
}

/// A world as the Worlds page shows it, before the player opens it: what the save holds and a word
/// on whether it was read at all.
#[derive(Clone, Debug, Default)]
pub struct Preview {
    /// The folder name under `saves/`.
    pub name: String,
    /// The name the game shows (`Data.LevelName`), falling back to the folder.
    pub label: String,
    pub level: PathBuf,
    pub bytes: u64,
    pub modified: u64,
    pub last_played: Option<i64>,
    pub health: Option<f32>,
    pub food: Option<i32>,
    pub xp_level: Option<i32>,
    pub items: Vec<Item>,
    pub error: Option<String>,
}

impl Preview {
    /// The health as a bar reads it, `0..=20`.
    pub fn health_fraction(&self) -> f32 {
        (self.health.unwrap_or(0.0) / 20.0).clamp(0.0, 1.0)
    }
}

/// The size of a world folder, every file under it.
pub fn folder_size(dir: &Path) -> u64 {
    let Ok(list) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut total = 0;
    for entry in list.flatten() {
        let path = entry.path();
        match entry.metadata() {
            Ok(meta) if meta.is_dir() => total += folder_size(&path),
            Ok(meta) => total += meta.len(),
            Err(_) => {}
        }
    }
    total
}

/// What the Worlds page shows for a world it has not opened.
pub fn preview(world: &World) -> Preview {
    let mut preview = Preview {
        name: world.name.clone(),
        label: world.name.clone(),
        level: world.level.clone(),
        bytes: world
            .level
            .parent()
            .map(folder_size)
            .unwrap_or_default(),
        modified: std::fs::metadata(&world.level)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        ..Preview::default()
    };
    match load(&world.level) {
        Ok(root) => {
            if let Some(label) = root
                .get("Data")
                .and_then(|d| d.get("LevelName"))
                .and_then(Tag::as_str)
                .filter(|l| !l.trim().is_empty())
            {
                preview.label = label.to_string();
            }
            preview.last_played = root
                .get("Data")
                .and_then(|d| d.get("LastPlayed"))
                .and_then(Tag::as_i32)
                .map(|v| v as i64);
            match stats(&root) {
                Ok(stats) => {
                    preview.health = Some(stats.health);
                    preview.food = Some(stats.food);
                    preview.xp_level = Some(stats.xp_level);
                    preview.items = stats.items;
                }
                Err(e) => preview.error = Some(e),
            }
        }
        Err(e) => preview.error = Some(e),
    }
    preview
}

/// The ticks in the world's own day (`Data.Time`).
pub fn time_of_day(root: &Root) -> Option<i64> {
    root.get("Data")
        .and_then(|d| d.get("Time"))
        .and_then(|t| t.as_i32().map(i64::from).or_else(|| match t {
            Tag::Long(v) => Some(*v),
            _ => None,
        }))
}

/// Whether it rains and whether it thunders, right now.
pub fn weather_of(root: &Root) -> (bool, bool) {
    let data = root.get("Data");
    let flag = |name: &str| {
        data.and_then(|d| d.get(name))
            .and_then(Tag::as_i32)
            .is_some_and(|v| v != 0)
    };
    (flag("raining"), flag("thundering"))
}

/// Sets the time of day, keeping the world's day count: `Data.Time` is the tick in the day and
/// `Data.DayTime` the total the game counts, so noon of a later day is still that day's noon.
pub fn set_time(root: &mut Root, ticks: i64) -> Result<String, String> {
    let ticks = ticks.rem_euclid(24_000);
    let data = root
        .path_mut(&["Data"])
        .ok_or_else(|| "This save has no Data tag.".to_string())?;
    let total = data
        .get("DayTime")
        .and_then(|t| t.as_i32().map(i64::from).or_else(|| match t {
            Tag::Long(v) => Some(*v),
            _ => None,
        }))
        .unwrap_or(0);
    let day = total.div_euclid(24_000);
    data.set("Time", Tag::Long(ticks));
    data.set("DayTime", Tag::Long(day * 24_000 + ticks));
    Ok(format!("time set to {ticks} ticks"))
}

/// Sets the weather and how long it lasts, in ticks.
pub fn set_weather(root: &mut Root, weather: Weather, ticks: i32) -> Result<String, String> {
    let ticks = ticks.max(1);
    let data = root
        .path_mut(&["Data"])
        .ok_or_else(|| "This save has no Data tag.".to_string())?;
    let (rain, thunder) = match weather {
        Weather::Clear => (false, false),
        Weather::Rain => (true, false),
        Weather::Thunder => (true, true),
    };
    data.set("raining", Tag::Byte(i8::from(rain)));
    data.set("thundering", Tag::Byte(i8::from(thunder)));
    data.set("rainTime", Tag::Int(if rain { ticks } else { 0 }));
    data.set("thunderTime", Tag::Int(if thunder { ticks } else { 0 }));
    data.set("clearWeatherTime", Tag::Int(if rain { 0 } else { ticks }));
    let said = match weather {
        Weather::Clear => format!("clear for {ticks} ticks"),
        Weather::Rain => format!("rain for {ticks} ticks"),
        Weather::Thunder => format!("thunder for {ticks} ticks"),
    };
    Ok(said)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world(names: &[(&str, Tag)]) -> Root {
        let player = Tag::Compound(
            names
                .iter()
                .map(|(name, value)| ((*name).to_string(), value.clone()))
                .collect(),
        );
        Root {
            name: String::new(),
            tag: Tag::Compound(vec![(
                "Data".into(),
                Tag::Compound(vec![("Player".into(), player)]),
            )]),
        }
    }

    fn bare() -> Root {
        world(&[
            ("Health".into(), Tag::Float(3.0)),
            ("foodLevel".into(), Tag::Int(2)),
            ("Air".into(), Tag::Short(40)),
        ])
    }

    #[test]
    fn healing_fills_health_and_puts_the_fire_out() {
        let mut root = bare();
        heal(&mut root).unwrap();
        let stats = stats(&root).unwrap();
        assert_eq!(stats.health, 20.0);
        assert_eq!(stats.air, 300);
        assert_eq!(
            root.path_mut(&["Data", "Player", "Fire"]).unwrap().as_i32(),
            Some(-1)
        );
    }

    #[test]
    fn feeding_fills_the_hunger_bar() {
        let mut root = bare();
        feed(&mut root).unwrap();
        let stats = stats(&root).unwrap();
        assert_eq!(stats.food, 20);
        assert_eq!(stats.saturation, 5.0);
    }

    #[test]
    fn adding_an_item_puts_it_in_the_first_free_slot() {
        let mut root = bare();
        assert!(add_item(&mut root, "minecraft:bread", 3).is_ok());
        assert!(add_item(&mut root, "minecraft:diamond", 1).is_ok());
        let stats = stats(&root).unwrap();
        assert_eq!(stats.items.len(), 2);
        assert_eq!(stats.items[0].slot, 0);
        assert_eq!(stats.items[0].id, "minecraft:bread");
        assert_eq!(stats.items[0].count, 3);
        assert_eq!(stats.items[1].slot, 1);
    }

    #[test]
    fn adding_the_same_item_stacks_onto_the_one_there() {
        let mut root = bare();
        add_item(&mut root, "minecraft:bread", 3).unwrap();
        add_item(&mut root, "minecraft:bread", 4).unwrap();
        let stats = stats(&root).unwrap();
        assert_eq!(stats.items.len(), 1);
        assert_eq!(stats.items[0].count, 7);
    }

    #[test]
    fn removing_a_stack_leaves_the_others() {
        let mut root = bare();
        add_item(&mut root, "minecraft:bread", 3).unwrap();
        add_item(&mut root, "minecraft:apple", 2).unwrap();
        remove_slot(&mut root, 0).unwrap();
        let stats = stats(&root).unwrap();
        assert_eq!(stats.items.len(), 1);
        assert_eq!(stats.items[0].id, "minecraft:apple");
        assert!(remove_slot(&mut root, 0).is_err());
    }

    #[test]
    fn a_level_sets_the_points_a_player_would_have() {
        let mut root = bare();
        set_level(&mut root, 30).unwrap();
        let stats = stats(&root).unwrap();
        assert_eq!(stats.xp_level, 30);
        assert_eq!(stats.xp_total, 1395);
    }

    #[test]
    fn the_time_is_set_inside_the_day_the_world_is_on() {
        let mut root = world(&[("Health".into(), Tag::Float(20.0))]);
        root.path_mut(&["Data"])
            .unwrap()
            .set("DayTime", Tag::Long(3 * 24_000 + 100));
        set_time(&mut root, TICKS_NIGHT).unwrap();
        assert_eq!(time_of_day(&root), Some(TICKS_NIGHT));
        let total = match root.get("Data").unwrap().get("DayTime").unwrap() {
            Tag::Long(v) => *v,
            other => panic!("DayTime is {other:?}"),
        };
        assert_eq!(total, 3 * 24_000 + TICKS_NIGHT);
    }

    #[test]
    fn a_time_outside_the_day_wraps_into_it() {
        let mut root = world(&[("Health".into(), Tag::Float(20.0))]);
        set_time(&mut root, 24_000 + TICKS_NOON).unwrap();
        assert_eq!(time_of_day(&root), Some(TICKS_NOON));
    }

    #[test]
    fn weather_is_written_the_way_the_game_reads_it() {
        let mut root = world(&[("Health".into(), Tag::Float(20.0))]);
        set_weather(&mut root, Weather::Thunder, 6_000).unwrap();
        assert_eq!(weather_of(&root), (true, true));
        set_weather(&mut root, Weather::Clear, 12_000).unwrap();
        assert_eq!(weather_of(&root), (false, false));
        assert_eq!(
            root.get("Data").unwrap().get("clearWeatherTime").unwrap().as_i32(),
            Some(12_000)
        );
    }

    #[test]
    fn a_preview_reads_the_player_without_opening_the_world() {
        let dir = std::env::temp_dir().join(format!("offhorizon-preview-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut root = world(&[("Health".into(), Tag::Float(20.0))]);
        add_item(&mut root, "minecraft:bread", 2).unwrap();
        let level = dir.join("level.dat");
        save(&level, &root).unwrap();
        let preview = preview(&World {
            name: "Test".into(),
            level,
        });
        assert_eq!(preview.health, Some(20.0));
        assert_eq!(preview.items.len(), 1);
        assert!(preview.bytes > 0);
        assert!(preview.error.is_none());
    }

    #[test]
    fn a_save_with_no_player_says_so_instead_of_panicking() {
        let root = Root {
            name: String::new(),
            tag: Tag::Compound(vec![("Data".into(), Tag::Compound(vec![]))]),
        };
        assert!(stats(&root).is_err());
    }
}
