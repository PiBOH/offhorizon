//! Every file this tool changes is copied before it is written, and the copy is listed here so
//! that any change can be taken back. A patch that can't be undone is not one this tool makes.

use crate::portable::Portable;
use std::path::{Path, PathBuf};

/// One file as it was before the tool touched it.
#[derive(Clone, Debug)]
pub struct Backup {
    pub id: String,
    pub when: u64,
    pub original: PathBuf,
    pub copy: PathBuf,
    pub bytes: u64,
    pub tool: String,
}

/// The folder holding the copies, and what is in it.
#[derive(Clone, Debug)]
pub struct Store {
    pub root: PathBuf,
    pub entries: Vec<Backup>,
}

impl Store {
    /// The backups beside the executable (`<exe>/.offhorizon/backups`).
    pub fn open() -> Self {
        Self::at(Portable::find().backups())
    }

    /// The same, in a folder of the caller's choosing (the tests keep theirs in the temp folder).
    pub fn at(root: PathBuf) -> Self {
        let mut store = Self {
            root,
            entries: Vec::new(),
        };
        store.reload();
        store
    }

    /// Reads the copies on disk back into `entries`.
    pub fn reload(&mut self) {
        self.entries.clear();
        let Ok(list) = std::fs::read_dir(&self.root) else {
            return;
        };
        for entry in list.flatten() {
            let copy = entry.path().join("original");
            let manifest = entry.path().join("manifest.json");
            let (Ok(text), Ok(meta)) = (
                std::fs::read_to_string(&manifest),
                std::fs::metadata(&copy),
            ) else {
                continue;
            };
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
                continue;
            };
            let original = value
                .get("path")
                .and_then(|p| p.as_str())
                .map(PathBuf::from);
            let Some(original) = original else { continue };
            self.entries.push(Backup {
                id: entry.file_name().to_string_lossy().into_owned(),
                when: value.get("when").and_then(|w| w.as_u64()).unwrap_or(0),
                original,
                copy,
                bytes: meta.len(),
                tool: value
                    .get("tool")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string(),
            });
        }
        self.entries.sort_by(|a, b| b.when.cmp(&a.when));
    }

    /// Copies `file` aside, answering where. Every write of the tool goes through here first.
    pub fn backup(&mut self, file: &Path, tool: &str) -> Result<PathBuf, String> {
        let bytes = std::fs::read(file).map_err(|e| format!("{}: {e}", file.display()))?;
        let when = now();
        let mut id = when.to_string();
        let mut n = 0;
        while self.root.join(&id).exists() {
            n += 1;
            id = format!("{when}-{n}");
        }
        let dir = self.root.join(&id);
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let copy = dir.join("original");
        std::fs::write(&copy, &bytes).map_err(|e| format!("{}: {e}", copy.display()))?;
        let manifest = serde_json::json!({
            "path": file.display().to_string(),
            "when": when,
            "tool": tool,
        });
        std::fs::write(dir.join("manifest.json"), format!("{manifest}\n"))
            .map_err(|e| e.to_string())?;
        self.reload();
        Ok(copy)
    }

    /// Keeps a second copy of a file the tool is about to rewrite, outside the backup folder.
    /// The launcher's own folder and the temporary folder each get one, so a change can be undone
    /// even when the stick is gone.
    pub fn copy_to(&self, bytes: &[u8], dir: &Path, name: &str) -> Result<PathBuf, String> {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let file = dir.join(name);
        std::fs::write(&file, bytes).map_err(|e| format!("{}: {e}", file.display()))?;
        Ok(file)
    }

    /// Puts a copy back over the file it came from, keeping the copy.
    pub fn restore(&mut self, id: &str) -> Result<PathBuf, String> {
        let entry = self
            .entries
            .iter()
            .find(|b| b.id == id)
            .cloned()
            .ok_or_else(|| "That backup is gone.".to_string())?;
        let bytes = std::fs::read(&entry.copy).map_err(|e| format!("{}: {e}", entry.copy.display()))?;
        if let Some(dir) = entry.original.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        std::fs::write(&entry.original, bytes)
            .map_err(|e| format!("{}: {e}", entry.original.display()))?;
        Ok(entry.original)
    }
}

/// Milliseconds since the epoch, used for the backup ids and the manifests.
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("offhorizon-store-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_backup_holds_the_bytes_the_file_had() {
        let dir = scratch("backup");
        let file = dir.join("level.dat");
        std::fs::write(&file, b"before").unwrap();
        let mut store = Store::at(dir.join("backups"));
        store.backup(&file, "heal").unwrap();
        std::fs::write(&file, b"after").unwrap();
        assert_eq!(std::fs::read(&file).unwrap(), b"after");
        assert_eq!(std::fs::read(&store.entries[0].copy).unwrap(), b"before");
    }

    #[test]
    fn restoring_puts_the_old_bytes_back() {
        let dir = scratch("restore");
        let file = dir.join("horizon-player.txt");
        std::fs::write(&file, "name:Steve\n").unwrap();
        let mut store = Store::at(dir.join("backups"));
        store.backup(&file, "offline name").unwrap();
        std::fs::write(&file, "name:Alex\n").unwrap();
        let id = store.entries[0].id.clone();
        store.restore(&id).unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "name:Steve\n");
    }
}
