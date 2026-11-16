//! The dedicated GPU, on Windows: Settings > Display > Graphics keeps one preference per program
//! in `HKCU\Software\Microsoft\DirectX\UserGpuPreferences`, and a program set to "High performance"
//! gets the discrete card instead of the one the screen is plugged into. OffHorizon writes that
//! preference for the game and for itself, on by default ([`crate::offhorizon::DEDICATED_GPU`]),
//! so a laptop with two graphics cards plays on the fast one without the player finding that page.
//!
//! On other systems the driver decides, and the game already asks wgpu for the high-performance
//! adapter (`crates/horizon-client-render/src/gfx.rs`), so there is nothing to write.
use std::path::{Path, PathBuf};

/// Where Windows keeps the per-program preference.
#[cfg(windows)]
const KEY: &str = r"HKCU\Software\Microsoft\DirectX\UserGpuPreferences";

/// How deep the game's folder is searched for its executable.
const DEPTH: usize = 4;

/// The programs the preference is written for: the launcher, and the game's executable wherever
/// `home` keeps the prepared versions.
pub fn programs(home: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::env::current_exe().ok().into_iter().collect();
    out.extend(clients(home, DEPTH));
    out.sort();
    out.dedup();
    out
}

/// The `horizon-client` executables under `home`, at most `depth` folders down.
fn clients(home: &Path, depth: usize) -> Vec<PathBuf> {
    if depth == 0 {
        return Vec::new();
    }
    let Ok(dir) = std::fs::read_dir(home) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(clients(&path, depth - 1));
        } else if path.file_name().is_some_and(|n| n == crate::backend::CLIENT) {
            out.push(path);
        }
    }
    out
}

/// Asks for the high-performance GPU for every program, or takes the preference away; says what
/// it did, one line per program, for the launcher's log.
pub fn apply(on: bool) -> Vec<String> {
    programs(&crate::backend::game_home())
        .into_iter()
        .filter_map(|exe| set(&exe, on).err().map(|why| format!("{}: {why}", exe.display())))
        .collect()
}

/// Writes (or removes) the preference of one program.
#[cfg(windows)]
pub fn set(exe: &Path, on: bool) -> Result<(), String> {
    use std::process::Command;
    let path = exe.to_string_lossy();
    let mut command = Command::new("reg");
    if on {
        // 2 is "High performance"; 0 would be "Let Windows decide".
        command.args(["add", KEY, "/v", &path, "/t", "REG_SZ", "/d", "GpuPreference=2;", "/f"]);
    } else {
        command.args(["delete", KEY, "/v", &path, "/f"]);
    }
    let out = command
        .output()
        .map_err(|e| format!("running reg: {e}"))?;
    if out.status.success() {
        return Ok(());
    }
    // Removing a preference that was never written is not a failure.
    let said = String::from_utf8_lossy(&out.stderr).to_lowercase();
    if !on && (said.contains("unable to find") || said.contains("impossibile trovare")) {
        return Ok(());
    }
    Err(String::from_utf8_lossy(&out.stderr).trim().to_owned())
}

/// Nothing to write: the driver and wgpu's own preference decide.
#[cfg(not(windows))]
pub fn set(_exe: &Path, _on: bool) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_games_executable_is_found_where_the_versions_keep_it() {
        let root = std::env::temp_dir().join(format!("horizon-gpu-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let version = root.join("26.3-nightly");
        std::fs::create_dir_all(&version).unwrap();
        std::fs::write(version.join(crate::backend::CLIENT), b"").unwrap();
        // A folder deeper than the search goes is left alone.
        let deep = root.join("a/b/c/d/e");
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::write(deep.join(crate::backend::CLIENT), b"").unwrap();
        let found = clients(&root, DEPTH);
        assert_eq!(found, vec![version.join(crate::backend::CLIENT)]);
        std::fs::remove_dir_all(&root).unwrap();
    }
}
