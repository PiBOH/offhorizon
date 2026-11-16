//! Embeds OffHorizon's icon (`packaging/icons/offhorizon.ico`) in the Windows executables, as icon
//! resource 1: Explorer and the task bar show it, and the window takes it as its icon.
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        // Resolve imports before main from System32, never a nearby injector DLL (#13108).
        println!("cargo:rustc-link-arg-bins=/DEPENDENTLOADFLAG:0x800");
    }
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    // version.txt in the repo root is the one version number (AGENTS.md).
    let version_file = std::path::Path::new(&dir).join("../version.txt");
    println!("cargo:rerun-if-changed={}", version_file.display());
    let version = std::fs::read_to_string(&version_file)
        .ok()
        .and_then(|text| {
            text.lines()
                .map(str::trim)
                .find(|line| !line.is_empty() && !line.starts_with('#'))
                .map(str::to_owned)
        })
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_owned());
    println!("cargo:rustc-env=HORIZON_LAUNCHER_VERSION={version}");
    // windres cannot carry a path with spaces (this checkout's folder has them). Copy the icon
    // into OUT_DIR, which Cargo keeps under the target folder, and hand windres that path.
    let ico_src = std::path::Path::new(&dir).join("../packaging/icons/offhorizon.ico");
    let ico_src = ico_src.canonicalize().unwrap_or(ico_src);
    println!("cargo:rerun-if-changed={}", ico_src.display());
    let ico = std::path::PathBuf::from(
        std::env::var_os("OUT_DIR").expect("build scripts always get OUT_DIR"),
    )
    .join("offhorizon.ico");
    std::fs::copy(&ico_src, &ico)
        .unwrap_or_else(|e| panic!("copying {} to {}: {e}", ico_src.display(), ico.display()));
    let ico = ico.to_string_lossy().replace('\\', "/");
    // Every Windows build gets it, the GNU one too: Explorer, the task bar and Settings > Apps
    // read this resource, and a launcher built here has to show OffHorizon like the one the
    // release ships. `winresource` cannot drive windres from a checkout whose path has spaces
    // (it hands windres an unquoted `-I<manifest dir>`, which gcc then splits), so the GNU build
    // compiles the same script itself ([`gnu_resource`]).
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        match std::env::var("CARGO_CFG_TARGET_ENV").as_deref() {
            Ok("gnu") => gnu_resource(&ico, &version),
            _ => {
                let mut resource = winresource::WindowsResource::new();
                resource.set_icon(&ico);
                for (name, value) in resource_values(&version) {
                    resource.set(name, value);
                }
                if let Err(e) = resource.compile() {
                    panic!("embedding {ico}: {e}");
                }
            }
        }
    }
}

/// What the executable says about itself: the icon resource is 1, the version resource says
/// OffHorizon, not the name of the project it is a fork of.
#[cfg(windows)]
fn resource_values(version: &str) -> Vec<(&'static str, &str)> {
    vec![
        ("FileDescription", "OffHorizon"),
        ("ProductName", "OffHorizon"),
        ("CompanyName", "OffHorizon"),
        ("InternalName", "horizon-launcher"),
        ("OriginalFilename", "OffHorizon.exe"),
        ("FileVersion", version),
        ("ProductVersion", version),
    ]
}

/// `0.1.5-nightly` as the four numbers a version resource wants: 0, 1, 5, 0.
#[cfg(windows)]
fn numeric_version(version: &str) -> [u16; 4] {
    let mut parts = version
        .split(['.', '-', '+'])
        .map(|part| part.parse::<u16>().unwrap_or(0));
    [
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    ]
}

/// `offhorizon.rc`: the icon and the version strings, for windres.
#[cfg(windows)]
fn resource_script(ico: &str, version: &str) -> String {
    let [major, minor, patch, build] = numeric_version(version);
    let number = format!("{major},{minor},{patch},{build}");
    let mut values = String::new();
    for (name, value) in resource_values(version) {
        values.push_str(&format!("      VALUE \"{name}\", \"{value}\"\n"));
    }
    format!(
        "1 ICON \"{ico}\"\n\
         1 VERSIONINFO\n\
         \x20FILEVERSION {number}\n\
         \x20PRODUCTVERSION {number}\n\
         \x20FILEFLAGSMASK 0x3fL\n\
         \x20FILEFLAGS 0x0L\n\
         \x20FILEOS 0x40004L\n\
         \x20FILETYPE 0x1L\n\
         \x20FILESUBTYPE 0x0L\n\
         BEGIN\n\
         \x20 BLOCK \"StringFileInfo\"\n\
         \x20 BEGIN\n\
         \x20   BLOCK \"040904b0\"\n\
         \x20   BEGIN\n\
         {values}\
         \x20   END\n\
         \x20 END\n\
         \x20 BLOCK \"VarFileInfo\"\n\
         \x20 BEGIN\n\
         \x20   VALUE \"Translation\", 0x409, 1200\n\
         \x20 END\n\
         END\n"
    )
}

/// The resource for a GNU toolchain: `windres` compiles [`resource_script`] into an object rustc
/// links into every binary, all under `OUT_DIR` so no path of a checkout with spaces reaches it.
#[cfg(windows)]
fn gnu_resource(ico: &str, version: &str) {
    use std::path::PathBuf;
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("build scripts always get OUT_DIR"));
    let script = out.join("offhorizon.rc");
    std::fs::write(&script, resource_script(ico, version)).expect("writing the resource script");
    let object = out.join("offhorizon-resource.o");
    let windres = std::process::Command::new("windres")
        .args(["-O", "coff", "--input"])
        .arg(&script)
        .arg("--output")
        .arg(&object)
        .output();
    match windres {
        Ok(done) if done.status.success() => {
            println!("cargo:rustc-link-arg-bins={}", object.display());
        }
        Ok(done) => panic!(
            "windres {}: {}",
            script.display(),
            String::from_utf8_lossy(&done.stderr)
        ),
        Err(e) => panic!("windres is what a GNU toolchain embeds the icon with: {e}"),
    }
}
