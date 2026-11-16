//! Horizon's own shader pack, installed when the registry's can't be drawn, and the check an
//! installed shader pack has to pass.
//!
//! Horizon's shader API is 0.2 and a 0.1 pack is refused (`docs/shaders.md`): the registry still
//! lists packs written for 0.1, so installing one used to end in that refusal, with `shaderpacks/`
//! left empty and the pack never turned on. This module ships Horizon's own `Golden Hour` pack
//! (`crates/horizon-shader-api/packs/golden_hour`, MIT) as the `.hshader` the game reads, so an
//! install still leaves a pack the game draws, and it is turned on when it lands
//! ([`packs::set_enabled`]: `pickedShaderPack` in `horizon-graphics.txt`, which the Shader Packs
//! page and the renderer both read).
use std::fs;
use std::io::Write;
use std::path::Path;

use sha2::Digest;

/// The pack's registry id: its settings key.
pub const ID: &str = "golden_hour";
/// The file it is installed as, in `shaderpacks/`.
pub const FILE: &str = "golden_hour.hshader";
/// Its name, as the Shader Packs page shows it.
pub const TITLE: &str = "Golden Hour";

/// The pack's files, straight from the shader API's own sample packs.
const PACK: &[(&str, &[u8])] = &[
    (
        "horizon-shaders.toml",
        include_bytes!("../../crates/horizon-shader-api/packs/golden_hour/horizon-shaders.toml"),
    ),
    (
        "common.wgsl",
        include_bytes!("../../crates/horizon-shader-api/packs/golden_hour/common.wgsl"),
    ),
    (
        "sky.wgsl",
        include_bytes!("../../crates/horizon-shader-api/packs/golden_hour/sky.wgsl"),
    ),
    (
        "post/glow.wgsl",
        include_bytes!("../../crates/horizon-shader-api/packs/golden_hour/post/glow.wgsl"),
    ),
    (
        "post/grade.wgsl",
        include_bytes!("../../crates/horizon-shader-api/packs/golden_hour/post/grade.wgsl"),
    ),
    (
        "README.md",
        include_bytes!("../../crates/horizon-shader-api/packs/golden_hour/README.md"),
    ),
];

/// The built-in pack as the game finds it: where it landed, its manifest and its checksum.
pub struct Builtin {
    pub pack: crate::packs::Pack,
    pub manifest: horizon_shader_api::Manifest,
    /// `SHA-256:<hex>` of the file, as a browser entry records one.
    pub checksum: String,
}

/// The built-in pack zipped: what `.hshader` is.
pub fn bytes() -> Result<Vec<u8>, String> {
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for (name, body) in PACK {
        zip.start_file(*name, options)
            .map_err(|e| format!("{FILE}: {name}: {e}"))?;
        zip.write_all(body)
            .map_err(|e| format!("{FILE}: {name}: {e}"))?;
    }
    let done = zip.finish().map_err(|e| format!("{FILE}: {e}"))?;
    Ok(done.into_inner())
}

/// Whether `error` is a pack refused for the shader API line it was written for, rather than a
/// download that failed: the registry's packs are 0.1 ones and this Horizon serves 0.2.
pub fn refused_for_api(error: &str) -> bool {
    error.contains("shader API")
}

/// Reads the shader pack at `path` the way the game would, refusing one it can't draw: the
/// manifest, then every stage checked. Its manifest when it is a pack this Horizon loads.
pub fn check(path: &Path) -> Result<horizon_shader_api::Manifest, String> {
    let pack =
        horizon_shader_api::Pack::read(path).map_err(|e| format!("{}: {e}", labelled(path)))?;
    match horizon_shader_api::check(&pack) {
        Ok(_) => Ok(pack.manifest),
        Err(problems) => {
            let said: Vec<String> = problems.iter().map(ToString::to_string).collect();
            Err(format!("{}: {}", labelled(path), said.join("; ")))
        }
    }
}

fn labelled(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

/// Puts the built-in pack in `home`'s `shaderpacks/` and turns it on. Written to a dot part first
/// and renamed, so a failure leaves no half pack the game reads; installing it again replaces it
/// rather than adding a second copy. Returns what landed.
pub fn install(home: &Path) -> Result<Builtin, String> {
    let bytes = bytes()?;
    let dir = home.join(crate::packs::Kind::Shader.folder());
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let path = dir.join(FILE);
    // A `.hshader` name (as `is_pack` wants) the pack list leaves out while it is written.
    let part = dir.join(format!(".{FILE}.part.hshader"));
    fs::write(&part, &bytes).map_err(|e| format!("{}: {e}", part.display()))?;
    let manifest = match check(&part) {
        Ok(manifest) => manifest,
        Err(e) => {
            let _ = fs::remove_file(&part);
            return Err(e);
        }
    };
    fs::rename(&part, &path).map_err(|e| {
        let _ = fs::remove_file(&part);
        format!("{}: {e}", path.display())
    })?;
    let pack = crate::packs::find(home, crate::packs::Kind::Shader, FILE)?;
    activate(home, &pack.file)?;
    let checksum = format!(
        "SHA-256:{}",
        sha2::Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    Ok(Builtin {
        pack,
        manifest,
        checksum,
    })
}

/// Turns the pack `file` in `home`'s `shaderpacks/` on, replacing the one that was on: what the
/// game's Shader Packs page does when a player picks another.
pub fn activate(home: &Path, file: &str) -> Result<(), String> {
    let pack = crate::packs::find(home, crate::packs::Kind::Shader, file)?;
    crate::packs::set_enabled(home, &pack, true)
}

#[cfg(test)]
#[path = "shader_pack/tests.rs"]
mod tests;