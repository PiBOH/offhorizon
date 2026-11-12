# AGENTS.md

Instructions for AI agents working in this repository.

## 1. What this is

**OffHorizon** — the portable, telemetry-free companion for Horizon, the Minecraft: Java Edition
client written in Rust.

- **Language / runtime:** Rust 2021, one binary, `eframe` (egui) window, no service, no network.
- **What it patches:** `horizon-player.txt`, `saves/<world>/level.dat` (`Data.Player`, `Data.Time`,
  `Data.DayTime`, the weather tags) and, only when asked, the installed launcher's `settings.json`.
- **License:** Unlicense. No warranty, no liability.

## 2. Rules that bite

- **Portable, or not at all.** Every folder the tool writes is beside the executable:
  `./.minecraft` (the game), `./.offhorizon` (backups, settings, launcher copies). Never `%APPDATA%`,
  never `~/.local/share`, never a folder elsewhere on the system. `OFFHORIZON_HOME` moves the tool
  folder for tests only.
- **The launcher is the one file outside the stick.** Changing it is allowed only through
  `launcher::point_at`, which writes `settings.json.offhorizon.bak` beside the file, a dated copy in
  `<launcher>/offhorizon-backups/`, another in the temporary folder and one in the tool's backup
  list, and `launcher::undo` puts every byte back.
- **Every write goes through a backup.** Call `store.backup(file, what)` before writing, and never
  write a save in place: `world::save` writes a temporary file and renames it. The tool's own
  `settings.json` (the language) is the one exception: it holds nothing a player could lose.
- **Reversible, or not at all.** A feature that changes a file must be listed on the Backups page
  and restored by it.
- **Read leniently, write faithfully.** The NBT layer keeps every tag and its order; a save that is
  only read writes back the same bytes.
- **GUI and CLI never drift.** Both front ends call `src/actions.rs`, and nothing else. A new action
  goes there first, then gets a button and a command. A button with no command is a bug.
- **No telemetry, ever.** The tool opens no socket and reports nothing. Don't add analytics, update
  checks or crash uploads.
- **No account bypass.** The launcher's sign-in is not forged or imitated: no fake Microsoft
  account, no token written into its secret store, no patched ownership flag. A feature that fakes
  an account is out of scope, however it is asked for. What this tool does instead is legal and
  enough: it writes the offline name the game plays under, points the launcher at the portable
  folder, and imports worlds from other launchers.
- **Version in one place.** `version.txt` is the only version: `build.rs` reads it into the binary,
  the release workflow tags with it.
- **Every visible string comes from `lang/`.** `i18n::t("key")`, `i18n::tf("key", args)`; never a
  literal in the UI or in the terminal. English is the fallback table.
- **No player data in the repository.** Tests build their own saves under `std::env::temp_dir()`.

## 3. Translations

The window and the terminal read XML tables under `lang/`, one file per language, all of them with
the same keys:

| Code | Language | Code | Language |
|---|---|---|---|
| `en` | English | `pl` | Polski |
| `it` | Italiano | `cs` | Čeština |
| `de` | Deutsch | `sv` | Svenska |
| `fr` | Français | `ru` | Русский |
| `es` | Español | `uk` | Українська |
| `pt` | Português | `tr` | Türkçe |
| `nl` | Nederlands | `ar` | العربية |
| `hi` | हिन्दी | `zh` | 中文 |
| `ja` | 日本語 | `ko` | 한국어 |

These are the languages the other PiBOH repositories ship, and none is left out. A new string is
added to **all** eighteen files; `i18n::tests::every_language_carries_every_key_the_english_table_has`
fails otherwise. Format:

```xml
<string key="nav.worlds">Worlds</string>
```

`OFFHORIZON_LANG` overrides the choice; otherwise the tool folder's `settings.json`, then the system
locale, then English.

## 4. Layout

| Path | Role |
|---|---|
| `src/main.rs` | window without arguments, terminal with them |
| `src/actions.rs` | every action, shared by both front ends |
| `src/app.rs` | the window: Profile, Worlds, Player, Inventory, Time and weather, Launchers, Backups |
| `src/cli.rs` | one command per action |
| `src/portable.rs` | the folders beside the executable |
| `src/launcher.rs` | launcher folders, `horizon-player.txt`, `point_at` / `undo` |
| `src/world.rs` | worlds, previews, player, time, weather |
| `src/others.rs` | the 16 launchers it looks for, and importing their worlds |
| `src/nbt.rs` | NBT (gzipped) read and write, compounds in order |
| `src/store.rs` | backups: copy before writing, restore |
| `src/i18n.rs` | the XML tables |
| `src/logo.rs`, `brand/offhorizon.svg` | the mark: drawn in code, no image file |
| `lang/*.xml` | the translations |
| `version.txt` | the single version source |
| `.github/workflows/auto-release.yml` | Windows and Linux builds, the release |

## 5. Release

A push to `offhorizon` whose commit message starts with `v` releases, and `workflow_dispatch`
releases on demand; both read `version.txt`. Anything not ending in `-stable` is a pre-release.
Bump `version.txt`, add the section to `CHANGELOG.md`, commit `v<version>`.

## 6. Tests

```sh
cargo test
```

One test per changed behaviour, on made-up saves under the temporary folder. Never a test that needs
a real Minecraft installation, a network or a real launcher folder.

## 7. Branches

`offhorizon` is the default branch and the only one that publishes. Work goes there; scratch
branches are never created.
