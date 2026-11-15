#!/usr/bin/env python3
"""Reapply offhorizon's promises after merging upstream, then check they still hold.

The sync workflow merges `horizon-rust/horizon` with our side winning textual conflicts.
This script does the part a merge cannot: it rebrands language values that upstream added,
fills new game language keys from `en_us.json`, and refuses to push if the fork's guarantees
(offline accounts, no telemetry collection, OffHorizon branding) disappeared.

Only language *values* are rewritten. Keys, code, crate names, URLs and the word Rust are
left alone.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
GAME_LANG = ROOT / "crates" / "horizon-client" / "assets" / "horizon" / "lang"
LAUNCHER_LANG = ROOT / "launcher" / "lang"

# "OffHorizon" has no word boundary between "Off" and "Horizon", so it is untouched.
# Lowercase `horizon-rust` and `playhorizon.dev` do not match either.
HORIZON = re.compile(r"\bHorizon\b(?!-rust)")


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def write_if_changed(path: Path, text: str) -> bool:
    if read(path) == text:
        return False
    path.write_text(text, encoding="utf-8", newline="\n")
    return True


def rebrand(value: str) -> str:
    return HORIZON.sub("OffHorizon", value)


def rebrand_table(path: Path) -> bool:
    data = json.loads(read(path))
    if not isinstance(data, dict):
        raise ValueError(f"{path}: expected an object")
    changed = False
    for key, value in list(data.items()):
        if not isinstance(value, str):
            raise ValueError(f"{path}: {key!r} is not a string")
        new = rebrand(value)
        if new != value:
            data[key] = new
            changed = True
    if changed:
        write_if_changed(
            path,
            json.dumps(data, ensure_ascii=False, indent=2) + "\n",
        )
    return changed


def complete_game_languages() -> tuple[int, int]:
    """Rebrand every game table and fill keys upstream added from en_us."""
    en_path = GAME_LANG / "en_us.json"
    english = json.loads(read(en_path))
    if not isinstance(english, dict):
        raise ValueError(f"{en_path}: expected an object")
    english = {key: rebrand(value) for key, value in english.items()}
    missing_total = 0
    changed_files = 0
    for path in sorted(GAME_LANG.glob("*.json")):
        table = json.loads(read(path))
        missing = [key for key in english if key not in table]
        missing_total += len(missing)
        # Upstream's key set is the source: keep its order, drop retired keys, add new ones
        # with the English text as the unreviewed fallback.
        completed = {
            key: rebrand(table.get(key, value))
            for key, value in english.items()
        }
        changed_files += write_if_changed(
            path,
            json.dumps(completed, ensure_ascii=False, indent=2) + "\n",
        )
    return missing_total, changed_files


def rebrand_launcher_languages() -> int:
    return sum(rebrand_table(path) for path in sorted(LAUNCHER_LANG.glob("*.json")))


def check() -> list[str]:
    errors: list[str] = []

    endpoint = read(ROOT / "crates" / "horizon-telemetry" / "src" / "lib.rs")
    if not re.search(
        r"pub fn endpoint\(\) -> Option<String> \{\s*None\s*\}",
        endpoint,
    ):
        errors.append("horizon_telemetry::endpoint() is no longer hard-wired to None")

    launcher = ROOT / "launcher" / "src"
    if (launcher / "collection.rs").exists():
        errors.append("launcher/src/collection.rs came back")
    main = read(launcher / "main.rs")
    if re.search(r"\bmod collection\b|\bcollection::", main):
        errors.append("launcher/src/main.rs registers the data-collection module")

    account = read(launcher / "account.rs")
    if "add_offline" not in account or "offline" not in account:
        errors.append("launcher/src/account.rs lost offline accounts")
    backend = read(launcher / "backend.rs")
    if '"--name"' not in backend:
        errors.append("the Rust game is not started with the offline --name")
    if "download_version_public" not in backend:
        errors.append("Java instances lost their account-free Mojang download")
    offhorizon = read(launcher / "offhorizon.rs")
    if "ai_is_off" not in offhorizon:
        errors.append("the launcher's AI features are no longer off by default")
    main = read(launcher / "main.rs")
    if "ensure_offline_account" not in main:
        errors.append("the launcher no longer creates an offline account when it has none")
    if "OFFHORIZON" not in main or "offhorizon::apply" not in main:
        errors.append("the launcher's opening or first-run defaults are no longer OffHorizon's")
    other = read(launcher / "other_launchers.rs")
    if "sklauncher_instances" not in other or "SKlauncher" not in other:
        errors.append("launcher/src/other_launchers.rs lost SKlauncher imports")
    settings = read(launcher / "backend.rs")
    if "PERFORMANCE_DEFAULT" not in settings:
        errors.append("Performance mode is no longer on by default")
    if not (ROOT / "CHANGELOG.md").is_file():
        errors.append("CHANGELOG.md is missing")
    if not (ROOT / "packaging/icons/offhorizon.ico").is_file():
        errors.append("the OffHorizon crack icon is missing")
    readme = read(ROOT / "README.md")
    if "https://piboh.github.io/offhorizon/" not in readme:
        errors.append("README.md lost the OffHorizon download link")

    installer = read(launcher / "installer.rs")
    if 'APP_NAME: &str = "OffHorizon Launcher"' not in installer:
        errors.append("the installer no longer calls the product OffHorizon Launcher")
    i18n = read(launcher / "i18n.rs")
    if "fn branded(" not in i18n:
        errors.append("the launcher no longer rewrites visible Horizon to OffHorizon")
    titlebar = read(launcher / "titlebar.rs")
    if "OffHorizon" not in titlebar:
        errors.append("the launcher title is not OffHorizon")
    window = read(ROOT / "crates" / "horizon-client" / "src" / "game" / "window_handler.rs")
    if '"OffHorizon"' not in window:
        errors.append("the game window title is not OffHorizon")

    english = json.loads(read(GAME_LANG / "en_us.json"))
    for path in sorted(GAME_LANG.glob("*.json")):
        table = json.loads(read(path))
        missing = [key for key in english if key not in table]
        if missing:
            errors.append(f"{path.relative_to(ROOT)} lacks {len(missing)} keys")
        for key, value in table.items():
            if isinstance(value, str) and HORIZON.search(value):
                errors.append(f"{path.relative_to(ROOT)} still says Horizon in {key!r}")
                break
    for path in sorted(LAUNCHER_LANG.glob("*.json")):
        table = json.loads(read(path))
        for key, value in table.items():
            if isinstance(value, str) and HORIZON.search(value):
                errors.append(f"{path.relative_to(ROOT)} still says Horizon in {key!r}")
                break
    return errors


def main() -> int:
    missing, changed = complete_game_languages()
    changed += rebrand_launcher_languages()
    print(f"Game language keys filled from English: {missing}")
    print(f"Language files rewritten: {changed}")

    errors = check()
    if errors:
        print("\nOffHorizon promises are broken after the upstream merge:", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1
    print("OffHorizon check: offline accounts, no collection, branding and translations OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
