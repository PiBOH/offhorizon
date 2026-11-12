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
    let tmp = level.with_extension("dat.offline-patch.tmp");
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
    fn a_save_with_no_player_says_so_instead_of_panicking() {
        let root = Root {
            name: String::new(),
            tag: Tag::Compound(vec![("Data".into(), Tag::Compound(vec![]))]),
        };
        assert!(stats(&root).is_err());
    }
}
