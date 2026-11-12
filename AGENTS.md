# AGENTS.md

Instructions for AI agents working in this repository.

## 1. What this is

**Offline Patch** — a reversible profile patcher for Horizon, the Minecraft: Java Edition client
written in Rust. Its purpose, in one line: let a player use an offline profile without losing the
safety of the save.

- **Language / runtime:** Rust 2021, one binary, `eframe` (egui) window, no other service.
- **What it patches:** `<game folder>/horizon-player.txt` and `saves/<world>/level.dat`
  (`Data.Player`), never anything else.
- **License:** Unlicense. The project takes no warranty and no liability.

## 2. Rules that bite

- **Every write goes through a backup.** Call `store.backup(file, what)` before writing, and never
  write a save in place: `world::save` writes a temporary file and renames it.
- **Reversible, or not at all.** A new feature that changes a file must be listed on the Backups
  page and restored by it. No "this one is safe" exceptions.
- **Read leniently, write faithfully.** The NBT layer keeps every tag and its order; a save that is
  only read writes back the same bytes. Don't "clean up" a save.
- **No player data in the repository.** Tests build their own saves under `std::env::temp_dir()`;
  `OFFLINE_PATCH_HOME` points at a scratch folder in tests. Never a real game folder.
- **No account bypass.** This tool patches profiles, not Horizon's sign-in. A feature that forges
  launcher accounts or tokens is out of scope, however it is asked for.
- **Version in one place.** `version.txt` is the only version: `build.rs` reads it into the binary,
  the release workflow tags with it. Never write a version into a second file.
- **No hardcoded user-facing strings in a second language.** The window is English; keep strings in
  one place in the UI code so a translation can be added later.

## 3. Layout

| Path | Role |
|---|---|
| `src/main.rs` | the window, the icon, `VERSION` from `version.txt` |
| `src/app.rs` | pages: Profile, Player, Inventory, Backups; the log |
| `src/theme.rs` | the dark Project ALM look |
| `src/launcher.rs` | finding the launcher's data folder, game folders, `horizon-player.txt` |
| `src/world.rs` | worlds, the player's numbers, heal / feed / items / level |
| `src/nbt.rs` | NBT (gzipped) read and write, compounds in order |
| `src/store.rs` | backups: copy before writing, restore |
| `src/items.rs` | item id suggestions (any id works) |
| `version.txt` | the single version source |
| `.github/workflows/auto-release.yml` | builds Windows and Linux, publishes the release |

## 4. Release

A push to the default branch whose commit message starts with `v` releases, and
`workflow_dispatch` releases on demand; both read `version.txt`. Anything not ending in `-stable`
is a pre-release, so `0.1.0-nightly` publishes as one. Bump `version.txt`, add the section to
`CHANGELOG.md`, commit `v<version>`.

## 5. Tests

```sh
cargo test
```

One test per changed behaviour, on made-up saves under the temporary folder. Never a test that
needs a real Minecraft installation, a network, or a real launcher folder.

## 6. Branches

`offline-patch` is the default branch and the only one that publishes. Work goes there; scratch
branches are never created.
