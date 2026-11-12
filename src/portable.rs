//! Everything this tool writes sits beside the executable: nothing in a system folder, nothing in
//! the player's home. On a USB stick the game folder, the backups and the settings travel with it.

use std::path::{Path, PathBuf};

/// The folder the executable runs from, and the folders derived from it.
#[derive(Clone, Debug)]
pub struct Portable {
    /// The folder holding the executable (or the one the caller named, in tests).
    pub base: PathBuf,
}

impl Portable {
    /// The folder the running executable sits in.
    pub fn find() -> Self {
        let base = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(Path::to_path_buf))
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        Self::at(base)
    }

    /// The same, in a folder of the caller's choosing (the tests keep theirs in the temp folder).
    pub fn at(base: PathBuf) -> Self {
        Self { base }
    }

    /// The tool's own folder: `.offhorizon` beside the executable. `OFFHORIZON_HOME` moves it.
    pub fn tool(&self) -> PathBuf {
        match std::env::var_os("OFFHORIZON_HOME").filter(|v| !v.is_empty()) {
            Some(home) => PathBuf::from(home),
            None => self.base.join(".offhorizon"),
        }
    }

    /// The game folder this tool plays from: `.minecraft` beside the executable.
    pub fn minecraft(&self) -> PathBuf {
        self.base.join(".minecraft")
    }

    /// The launcher data folder, when the portable stick carries one of its own.
    pub fn launcher(&self) -> PathBuf {
        self.base.join("launcher")
    }

    /// The copy of every launcher file this tool rewrote, kept with the stick.
    pub fn launcher_backups(&self) -> PathBuf {
        self.tool().join("launcher-backups")
    }

    /// The backup folder of the game files.
    pub fn backups(&self) -> PathBuf {
        self.tool().join("backups")
    }

    /// Where the language choice and the rest of the portable settings live.
    pub fn settings_file(&self) -> PathBuf {
        self.tool().join("settings.json")
    }

    /// A second copy of every launcher backup, outside the stick's launcher folder.
    pub fn temp_copy(&self) -> PathBuf {
        std::env::temp_dir().join("offhorizon")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_folder_stays_beside_the_executable() {
        let base = std::env::temp_dir().join(format!("offhorizon-portable-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let portable = Portable::at(base.clone());
        assert_eq!(portable.tool(), base.join(".offhorizon"));
        assert_eq!(portable.minecraft(), base.join(".minecraft"));
        assert_eq!(portable.backups(), base.join(".offhorizon").join("backups"));
    }
}
