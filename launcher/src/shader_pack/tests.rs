use super::*;
use std::io::Write;
use std::path::PathBuf;

/// A game folder whose name has spaces in it, as a Windows one often does.
fn home(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "horizon shader pack {name} {}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn graphics(home: &Path) -> String {
    fs::read_to_string(home.join("horizon-graphics.txt")).unwrap_or_default()
}

/// A `.hshader` for another shader API line, as the registry's 0.1 packs are.
fn old_api_pack(path: &Path) {
    let mut zip = zip::ZipWriter::new(fs::File::create(path).unwrap());
    let options = zip::write::SimpleFileOptions::default();
    zip.start_file("horizon-shaders.toml", options).unwrap();
    write!(
        zip,
        "schema = 1\nid = \"glow\"\nname = \"Soft Glow\"\nversion = \"1.0.0\"\napi = \"0.1\"\n\
         description = \"A post pass.\"\n\n[[post]]\nfile = \"post.wgsl\"\nname = \"Colour\"\n"
    )
    .unwrap();
    zip.start_file("post.wgsl", options).unwrap();
    zip.write_all(b"@fragment fn colour() -> vec4<f32> { vec4(1.0) }").unwrap();
    zip.finish().unwrap();
}

#[test]
fn the_built_in_pack_is_one_the_game_reads() {
    let home = home("reads");
    let path = home.join(FILE);
    fs::write(&path, bytes().unwrap()).unwrap();
    let manifest = check(&path).expect("the game reads the pack the launcher ships");
    assert_eq!(manifest.id, ID);
    assert_eq!(manifest.name, TITLE);
    assert_eq!(manifest.version, "2.0.0");
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn installing_puts_it_in_shaderpacks_and_turns_it_on() {
    let home = home("installs");
    fs::write(
        home.join("horizon-graphics.txt"),
        "preset:balanced\nlodDistance:8\npickedShaderPack:\n",
    )
    .unwrap();
    let builtin = install(&home).unwrap();
    assert_eq!(builtin.pack.file, FILE);
    assert_eq!(builtin.manifest.id, ID);
    assert!(builtin.checksum.starts_with("SHA-256:"));
    assert!(home.join("shaderpacks").join(FILE).is_file());
    let text = graphics(&home);
    assert!(text.contains("pickedShaderPack:horizon:golden_hour"), "{text}");
    // The player's other settings are left as they were.
    assert!(text.contains("preset:balanced"), "{text}");
    assert!(text.contains("lodDistance:8"), "{text}");
    let listed = crate::packs::find(&home, crate::packs::Kind::Shader, FILE).unwrap();
    assert!(listed.enabled, "the pack page shows it on");
    // Nothing else was left behind, and the pack is not written twice.
    let installed = install(&home).unwrap();
    assert_eq!(installed.pack.file, FILE);
    let shaderpacks: Vec<String> = fs::read_dir(home.join("shaderpacks"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(shaderpacks, [FILE]);
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn a_pack_for_the_old_shader_api_is_refused_with_its_reason() {
    let home = home("old api");
    let path = home.join("glow-1.0.0.hshader");
    old_api_pack(&path);
    let said = check(&path).unwrap_err();
    assert_eq!(
        said,
        "glow-1.0.0.hshader: horizon-shaders.toml: needs shader API 0.1; this Horizon serves 0.2"
    );
    assert!(refused_for_api(&said));
    assert!(!refused_for_api("Connection Failed: tls connection init failed"));
    fs::remove_dir_all(home).unwrap();
}
