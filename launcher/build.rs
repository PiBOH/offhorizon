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
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("gnu")
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon(&ico);
        if let Err(e) = resource.compile() {
            panic!("embedding {ico}: {e}");
        }
    }
}
