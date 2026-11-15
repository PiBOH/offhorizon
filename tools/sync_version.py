#!/usr/bin/env python3
"""Keep version.txt as the one version number for the launcher.

Cargo cannot read a package version from another file, so the release workflow runs this
before building. It writes the first non-empty line of version.txt into launcher/Cargo.toml
and into horizon-launcher's entry in launcher/Cargo.lock. The build script also embeds the
same text as HORIZON_LAUNCHER_VERSION, so the binary and its manifests agree even when the
script is not run.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
VERSION_FILE = ROOT / "version.txt"
CARGO_TOML = ROOT / "launcher" / "Cargo.toml"
CARGO_LOCK = ROOT / "launcher" / "Cargo.lock"


def version() -> str:
    for line in VERSION_FILE.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if line and not line.startswith("#"):
            return line.removeprefix("v")
    raise SystemExit("version.txt is empty")


def rewrite(path: Path, pattern: re.Pattern[str], replacement: str) -> bool:
    text = path.read_text(encoding="utf-8")
    updated, count = pattern.subn(replacement, text, count=1)
    if count != 1:
        raise SystemExit(f"{path}: version field not found")
    if updated == text:
        return False
    path.write_text(updated, encoding="utf-8", newline="\n")
    return True


def main() -> int:
    v = version()
    if not re.fullmatch(r"[0-9A-Za-z.+-]+", v):
        raise SystemExit(f"version.txt has a version Cargo cannot use: {v!r}")
    changed = rewrite(
        CARGO_TOML,
        re.compile(r'(?m)^version = "[^"]*"'),
        f'version = "{v}"',
    )
    lock_pattern = re.compile(
        r'(^\[\[package\]\]\nname = "horizon-launcher"\nversion = ")[^"]*(")',
        re.MULTILINE,
    )
    changed |= rewrite(CARGO_LOCK, lock_pattern, rf'\g<1>{v}\g<2>')
    print(f"launcher version {v}: {'updated' if changed else 'already in sync'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
