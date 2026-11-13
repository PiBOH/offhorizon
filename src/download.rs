//! Downloading the Horizon client: the Rust game itself, so a stick with no client gets one
//! without a file copied by hand.
//!
//! The client comes from the Horizon releases on GitHub — the newest one that has this machine's
//! build (`horizon-windows-x64.zip`, `horizon-linux-x86_64.tar.gz`, the macOS zips). The archive
//! holds one folder with `horizon-client`, `horizon-prepare` and the game's pinned data; it lands
//! as `versions/<version>/`, the layout this tool already reads. Its resources are then prepared
//! by the archive's `horizon-prepare`, from whatever Minecraft is here — or from the copy this
//! tool fetches from Mojang when there is none.
//!
//! No new dependency: `curl` fetches and `tar` unpacks (bsdtar reads both formats on Windows and
//! macOS, GNU tar reads the `.tar.gz` Linux gets).

use crate::i18n;
use crate::minecraft;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The releases of the game, newest first.
const RELEASES: &str = "https://api.github.com/repos/horizon-rust/horizon/releases?per_page=20";

/// What part of a whole install the client archive takes: the rest is Minecraft and the
/// preparation that reads it.
pub const CLIENT_SHARE: f32 = 0.15;

/// The archive this machine runs.
pub fn asset() -> Result<&'static str, String> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => Ok("horizon-windows-x64.zip"),
        ("windows", "aarch64") => Ok("horizon-windows-arm64.zip"),
        ("linux", "x86_64") => Ok("horizon-linux-x86_64.tar.gz"),
        ("macos", "aarch64") => Ok("horizon-macos-arm64.zip"),
        ("macos", "x86_64") => Ok("horizon-macos-x64.zip"),
        (os, arch) => Err(format!("{os} {arch}")),
    }
}

/// One release that has this machine's build.
#[derive(Clone, Debug)]
pub struct Release {
    /// The version it installs as: `nightly-<date>`, or the release's tag.
    pub version: String,
    pub asset: String,
    pub url: String,
    /// The archive's size, for the progress the window shows.
    pub size: u64,
}

/// The newest build for this machine.
pub fn release() -> Result<Release, String> {
    let asset = asset()?;
    let text = curl(&["-L", "--fail", "--silent", "--show-error", RELEASES])?;
    let list = serde_json::from_str::<Value>(&text).map_err(|e| format!("{e}"))?;
    let Some(list) = list.as_array() else {
        return Err(text.chars().take(200).collect());
    };
    for release in list {
        if release.get("draft").and_then(Value::as_bool) == Some(true) {
            continue;
        }
        let tag = release.get("tag_name").and_then(Value::as_str).unwrap_or("");
        let name = release.get("name").and_then(Value::as_str).unwrap_or(tag);
        let Some(file) = release
            .get("assets")
            .and_then(Value::as_array)
            .and_then(|assets| {
                assets
                    .iter()
                    .find(|entry| entry.get("name").and_then(Value::as_str) == Some(asset))
            })
        else {
            continue;
        };
        let Some(url) = file.get("browser_download_url").and_then(Value::as_str) else {
            continue;
        };
        return Ok(Release {
            version: version_of(name, tag),
            asset: asset.to_string(),
            url: url.to_string(),
            size: file.get("size").and_then(Value::as_u64).unwrap_or(0),
        });
    }
    Err(format!("{asset}"))
}

/// How a release is called once installed: a Nightly by its date (`nightly-2026-11-13`), anything
/// else by its tag.
fn version_of(name: &str, tag: &str) -> String {
    for word in name.split_whitespace() {
        let digits = word.chars().filter(|c| c.is_ascii_digit()).count();
        let dashes = word.chars().filter(|c| *c == '-').count();
        if digits == 8 && dashes == 2 && word.len() == 10 {
            return format!("nightly-{word}");
        }
    }
    tag.to_string()
}

/// Downloads the newest client and unpacks it as `versions/<version>`, answering the version.
/// `said` takes the log lines, `progress` the fraction of the whole install the archive covers.
/// Nothing is left behind when the download or the unpack fails.
pub fn install(
    versions: &Path,
    said: &mut dyn FnMut(String),
    progress: &mut dyn FnMut(f32),
) -> Result<String, String> {
    let release = release()?;
    said(i18n::t("home.rust_downloading"));
    said(format!("{} · {}", release.version, release.asset));
    let partial = versions.join(format!(".{}.partial", release.version));
    let _ = std::fs::remove_dir_all(&partial);
    let unpacked = partial.join("unpacked");
    std::fs::create_dir_all(&unpacked).map_err(|e| format!("{}: {e}", unpacked.display()))?;
    let archive = partial.join("download");
    progress(0.0);
    fetch_file(&release.url, &archive, release.size, &mut |fraction| {
        progress(CLIENT_SHARE * fraction)
    })?;
    let (from, into) = (text(&archive), text(&unpacked));
    run("tar", &["-xf", from.as_str(), "-C", into.as_str()])?;
    let top = unpacked_folder(&unpacked)?;
    let target = versions.join(&release.version);
    let _ = std::fs::remove_dir_all(&target);
    std::fs::rename(&top, &target).map_err(|e| format!("{}: {e}", target.display()))?;
    std::fs::write(target.join(".installed"), "")
        .map_err(|e| format!("{}: {e}", target.display()))?;
    let _ = std::fs::remove_dir_all(&partial);
    progress(CLIENT_SHARE);
    Ok(release.version)
}

/// Prepares a client's resources from one Minecraft install, as the official launcher does. The
/// archive ships the preparer, so nothing else is needed.
pub fn prepare(
    version: &Path,
    source: &minecraft::Source,
    server_jar: Option<&Path>,
    said: &mut dyn FnMut(String),
) -> Result<(), String> {
    let program = version.join(if cfg!(windows) {
        "horizon-prepare.exe"
    } else {
        "horizon-prepare"
    });
    if !program.is_file() {
        return Err(format!("{}", program.display()));
    }
    if !source.dir.is_dir() {
        return Err(i18n::t("home.rust_no_minecraft"));
    }
    let mut command = std::process::Command::new(&program);
    command
        .current_dir(version)
        .arg("--prepare")
        .arg("--root")
        .arg(version)
        .arg(source.flag())
        .arg(&source.dir);
    if let Some(server) = server_jar {
        command.arg("--server-jar").arg(server);
    }
    let output = command
        .output()
        .map_err(|e| format!("{}: {e}", program.display()))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = text.lines().filter(|line| !line.trim().is_empty()).collect();
    for line in lines.iter().skip(lines.len().saturating_sub(3)) {
        said(line.trim().to_string());
    }
    if output.status.success() {
        return Ok(());
    }
    let why = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if why.is_empty() {
        return Err(lines
            .last()
            .map(|line| line.trim().to_string())
            .unwrap_or_default());
    }
    Err(why)
}

/// The folder holding the client inside an unpacked archive: the archives hold one top folder, and
/// a bare build may not.
fn unpacked_folder(dir: &Path) -> Result<PathBuf, String> {
    let client = if cfg!(windows) {
        "horizon-client.exe"
    } else {
        "horizon-client"
    };
    if dir.join(client).is_file() {
        return Ok(dir.to_path_buf());
    }
    let mut folders: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    folders.sort();
    if let Some(found) = folders.iter().find(|path| path.join(client).is_file()) {
        return Ok(found.clone());
    }
    Err(format!("{client}"))
}

fn text(path: &Path) -> String {
    path.display().to_string()
}

fn run(program: &str, args: &[&str]) -> Result<(), String> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("{program}: {e}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
}

/// Fetches one file, watching how much of it is on disk: the window shows a percentage, not a
/// spinner that says nothing.
fn fetch_file(
    url: &str,
    to: &Path,
    size: u64,
    progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
    let mut child = std::process::Command::new("curl")
        .args([
            "-L",
            "--fail",
            "--silent",
            "--show-error",
            "--retry",
            "2",
            "-o",
        ])
        .arg(to)
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("curl: {e}"))?;
    loop {
        std::thread::sleep(Duration::from_millis(200));
        if size > 0 {
            let done = std::fs::metadata(to).map(|meta| meta.len()).unwrap_or(0);
            progress((done as f32 / size as f32).min(1.0));
        }
        match child.try_wait() {
            Ok(Some(status)) if status.success() => {
                progress(1.0);
                return Ok(());
            }
            Ok(Some(_)) => return Err(format!("curl: {url}")),
            Ok(None) => {}
            Err(e) => return Err(format!("curl: {e}")),
        }
    }
}

fn curl(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("curl")
        .args(args)
        .output()
        .map_err(|e| format!("curl: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_nightly_installs_under_its_date_and_a_release_under_its_tag() {
        assert_eq!(
            version_of("Nightly 2026-11-13 · 9f42064", "nightly"),
            "nightly-2026-11-13"
        );
        assert_eq!(version_of("Horizon 0.2.63", "v0.2.63"), "v0.2.63");
        assert_eq!(version_of("", "nightly"), "nightly");
    }

    #[test]
    fn this_platform_has_a_build_to_download() {
        let asset = asset().expect("the runner is a supported machine");
        assert!(asset.starts_with("horizon-"));
        assert!(asset.ends_with(".zip") || asset.ends_with(".tar.gz"));
    }

    #[test]
    fn the_unpacked_folder_is_the_one_with_the_game() {
        let base =
            std::env::temp_dir().join(format!("offhorizon-download-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let inner = base.join("horizon-windows-x64");
        std::fs::create_dir_all(&inner).unwrap();
        let client = if cfg!(windows) {
            "horizon-client.exe"
        } else {
            "horizon-client"
        };
        std::fs::write(inner.join(client), b"").unwrap();
        assert_eq!(unpacked_folder(&base).unwrap(), inner);
    }
}
