//! Rust ports of the Java mods already on the machine, installed while a Horizon instance is
//! prepared: the `mods/` of every Java instance is read the way the mod browser reads a jar
//! (`horizon_port_pack::check`), and what ports lands in the game folder's `horizon-mods/<id>/`,
//! on in `horizon-mods.txt`, where Options > Mods and Home's Mods tab show it to turn off or
//! remove. A jar with nothing to port is named in the log with the reason, so whether a Rust port
//! exists is something the player can read rather than guess.
use crate::browser::{installed, java};
use horizon_port_pack::check::{self, Verdict};
use std::fs;
use std::path::{Path, PathBuf};

/// Where imported jars are carried over: beside the browser's own checks, deleted after each jar.
const WORK: &str = "mod-browser/imported-ports";

/// What became of one jar.
enum Ported {
    Installed(installed::Entry),
    /// A port exists and is already there.
    Present(String),
    /// There is nothing to install, and why.
    None(String),
}

/// Installs the port of every Java mod in `java_homes`' `mods/` into `rust_home`, logging each
/// decision. Returns the entries installed.
pub fn prepare(
    java_homes: &[PathBuf],
    rust_home: &Path,
    log: &dyn Fn(String),
) -> Vec<installed::Entry> {
    let mut done = Vec::new();
    for home in java_homes {
        // A Horizon instance's own mods are not Java mods to port.
        if home == rust_home {
            continue;
        }
        let Ok(dir) = fs::read_dir(home.join("mods")) else {
            continue;
        };
        let mut jars: Vec<PathBuf> = dir
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.is_file()
                    && path
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
            })
            .collect();
        jars.sort();
        for jar in jars {
            match port(&jar, rust_home) {
                Ok(Ported::Installed(entry)) => {
                    log(format!(
                        "Rust port of {} installed automatically ({}).",
                        entry.title,
                        entry.verdict.as_deref().unwrap_or("ported")
                    ));
                    done.push(entry);
                }
                Ok(Ported::Present(why)) => log(why),
                Ok(Ported::None(why)) => log(why),
                Err(why) => log(format!("{}: {why}", jar.display())),
            }
        }
    }
    done
}

/// Checks `jar` and, when something of it ports, installs the carry into `rust_home`.
fn port(jar: &Path, rust_home: &Path) -> Result<Ported, String> {
    let stem = jar
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let work = rust_home.join(WORK).join(&stem);
    let _ = fs::remove_dir_all(&work);
    let checked = check::check(jar, &work).inspect_err(|_| {
        let _ = fs::remove_dir_all(&work);
    })?;
    let carry = work.join("carry");
    let outcome = if checked.verdict == Verdict::No {
        Ok(Ported::None(format!(
            "{}: no Rust port ({})",
            checked.name, checked.summary
        )))
    } else if !carry.is_dir() {
        Ok(Ported::None(format!(
            "{}: nothing to port ({})",
            checked.name, checked.summary
        )))
    } else {
        match java::install_ported(
            rust_home,
            &checked,
            &carry,
            &checked.name,
            &checked.version,
        ) {
            Ok(entry) => Ok(Ported::Installed(entry)),
            Err(why) => Ok(Ported::Present(format!("{}: {why}", checked.name))),
        }
    };
    let _ = fs::remove_dir_all(&work);
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_java_instance_without_mods_installs_nothing() {
        let root = std::env::temp_dir().join(format!("horizon-ports-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let java_home = root.join("java");
        let rust_home = root.join("rust");
        fs::create_dir_all(&java_home).unwrap();
        fs::create_dir_all(&rust_home).unwrap();
        let lines = std::cell::RefCell::new(Vec::new());
        let done = prepare(
            &[java_home.clone(), rust_home.clone()],
            &rust_home,
            &|line| lines.borrow_mut().push(line),
        );
        assert!(done.is_empty());
        assert!(lines.borrow().is_empty());
        // Something that is not a jar is not read as a mod.
        fs::create_dir_all(java_home.join("mods")).unwrap();
        fs::write(java_home.join("mods/notes.txt"), "not a mod").unwrap();
        let done = prepare(&[java_home], &rust_home, &|_| {});
        assert!(done.is_empty());
        fs::remove_dir_all(&root).unwrap();
    }
}
