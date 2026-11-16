//! What makes this fork OffHorizon: its name and the defaults its first start applies.
//!
//! The game's No AI switch is shared with the launcher (`horizon_platform::no_ai`), so turning
//! it on here also keeps every AI surface off in the game. An explicit `noAi` line, whichever
//! way the player set it, is never overwritten.

use std::path::Path;

/// The name players see, in the title bar, the installer and the opening.
pub const NAME: &str = "OffHorizon";

/// Performance mode is on from the first start: the glass scene is the expensive part of the
/// launcher, and OffHorizon should stay usable on older computers before anything is configured.
pub const PERFORMANCE_DEFAULT: bool = true;

/// The dedicated GPU is asked for from the first start (`crate::gpu`): a laptop with two cards
/// plays on the fast one without the player finding Settings > Display > Graphics.
pub const DEDICATED_GPU: bool = true;

/// Applies the defaults for this computer's game folder.
pub fn apply() {
    apply_at(&crate::backend::game_home());
}

/// Whether `home` has an explicit `noAi` line, whichever way the player set it.
fn has_no_ai_line(home: &Path) -> bool {
    std::fs::read_to_string(home.join(horizon_platform::no_ai::FILE)).is_ok_and(|text| {
        text.lines().any(|line| {
            line.split_once(':')
                .is_some_and(|(key, _)| key.trim() == horizon_platform::no_ai::KEY)
        })
    })
}

/// Whether AI is off for `home`: off until the player writes an explicit choice, then exactly
/// what that choice says. This is the launcher's default too, so `mcp` and `agent-config` are
/// refused from a fresh install before the window ever opens.
pub fn ai_is_off(home: &Path) -> bool {
    !has_no_ai_line(home) || horizon_platform::no_ai::is_on(home)
}

/// Applies the defaults under `home`, keeping every choice the player already made.
pub fn apply_at(home: &Path) {
    if !has_no_ai_line(home) {
        // A folder that cannot be written still starts; the switch simply stays at its default.
        let _ = horizon_platform::no_ai::set(home, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("offhorizon-defaults-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_first_start_turns_ai_off_and_keeps_the_other_extras() {
        let home = scratch("first");
        std::fs::write(home.join(horizon_platform::no_ai::FILE), "zoom:true\n").unwrap();
        apply_at(&home);
        assert!(horizon_platform::no_ai::is_on(&home));
        let text = std::fs::read_to_string(home.join(horizon_platform::no_ai::FILE)).unwrap();
        assert!(text.contains("zoom:true"));
        assert_eq!(text.matches("noAi:").count(), 1);
        apply_at(&home);
        let again = std::fs::read_to_string(home.join(horizon_platform::no_ai::FILE)).unwrap();
        assert_eq!(again.matches("noAi:").count(), 1);
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn ai_is_off_until_a_player_writes_a_choice() {
        let home = scratch("ai-default");
        assert!(ai_is_off(&home));
        std::fs::write(home.join(horizon_platform::no_ai::FILE), "noAi:false\n").unwrap();
        assert!(!ai_is_off(&home));
        std::fs::write(home.join(horizon_platform::no_ai::FILE), "noAi:true\n").unwrap();
        assert!(ai_is_off(&home));
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn performance_mode_is_on_until_a_player_turns_it_off() {
        assert!(crate::backend::Settings::parse("{}").performance());
        assert!(!crate::backend::Settings::parse(r#"{"performance":false}"#).performance());
    }

    #[test]
    fn a_players_explicit_choice_is_never_overwritten() {
        let home = scratch("chosen");
        std::fs::write(home.join(horizon_platform::no_ai::FILE), "noAi:false\n").unwrap();
        apply_at(&home);
        assert!(!horizon_platform::no_ai::is_on(&home));
        let _ = std::fs::remove_dir_all(&home);
    }
}
