# offhorizon: a map for agents

offhorizon is PiBOH's unofficial fork of [Horizon](https://github.com/horizon-rust/horizon),
Minecraft: Java Edition 26.3 rebuilt in Rust. It adds one thing upstream does not: **you can play
with an offline account, without a Microsoft sign-in**. Everything else follows upstream, and
vanilla is the specification wherever it has the feature.

**offhorizon is not approved by or associated with horizon-rust, Mojang or Microsoft.** It is a
fork, not an official Horizon build, and not an official Minecraft product. Say so wherever the
project describes itself, including release notes.

The fork stays on GitHub as `PiBOH/offhorizon`; the default branch is `offhorizon`, which is the
only branch releases are cut from. Upstream (`horizon-rust/horizon`) is the source of the game
builds; this fork builds the launcher.

## What offline play means

- An **offline account** is a local name in `accounts.json` (`offline: true`), with Java's
  `OfflinePlayer:<name>` UUID. It is selected like a Microsoft account and starts the Rust game or
  a Java instance with that name. Online servers still need a Microsoft account.
- Offline play starts what the player already has: a prepared resource folder for the Rust game, or
  an installed Java version. **Mojang's client jar and assets are still downloaded only for an
  account the services said owns the game.** Never add a way around `download_for_owner` or
  `download_version_for_owner`; the fork is not a piracy tool, only a launcher that does not demand
  a sign-in to start local play.
- Never commit Mojang's files: game jars, assets, decompiled source, prepared resources
  (`.local/`). `tools/check_public_tree.py` rejects them.

## Telemetry and data collection

offhorizon **collects no telemetry and sends none**. The launcher has no "Data collection" page, no
consent screen and no `data` command; the game keeps no crash, performance or benchmark item for
upload; the anonymous "I'm playing" ping is not registered. `horizon_telemetry::endpoint()` is
hard-wired to `None`, so even an old outbox or an environment variable cannot send anything.

`docs/telemetry.md` and the telemetry crates stay in the tree as upstream reference and for the
ingest server. **Do not delete that documentation**, and do not add a new collection path, consent
question or ping. Local benchmark and crash files that the player writes for themselves are not
telemetry; keep them local.

## Version and releases

`version.txt` in the root is the **single source of truth** for the program version. It holds one
line, for example `0.1.2-nightly`.

- `launcher/build.rs` reads it into `HORIZON_LAUNCHER_VERSION`; the game's build script reads it
  into `HORIZON_VERSION`. Do not add a second version number anywhere in code or workflow output.
- Cargo cannot read a package version from another file, so `tools/sync_version.py` copies the same
  value into `launcher/Cargo.toml` and `launcher/Cargo.lock`. The release workflow runs it before
  every build; a local build should run it after editing `version.txt`.
- A push whose **first line starts with `v`** runs `.github/workflows/auto-release.yml`. The
  workflow reads `version.txt`, tags `v<version>`, builds the launcher for Linux x86_64 and Windows
  x64 (and tries macOS x64 without blocking the release), and publishes a GitHub release. A version
  with a suffix (`-nightly`, `-beta`, `-alpha`, `-rc`) is a pre-release.
- Assets are the finished native files, never `.zip`/`.tar.gz` wrappers: the Windows `.exe`, the
  Linux executable, and the macOS executable. `SHA256SUMS` accompanies them for the launcher's
  unsigned self-update.
- The launcher updates itself from `PiBOH/offhorizon`; Minecraft/Rust builds still come from
  `horizon-rust/horizon`. Keep both repositories separate in `launcher/src/backend.rs`.
- An empty account list gets an offline account automatically. Microsoft sign-in is optional; Java
  instances download Mojang's public version files without an account. Online servers still need a
  Microsoft account that owns the game.
- `.github/workflows/sync-upstream.yml` runs every 30 minutes. It merges with `-X ours`, restores
  the offline/telemetry files from the pre-merge commit, and only pushes if
  `tools/offhorizon_sync.py` passes its checks.
- No version-bump commits: change `version.txt` in the same commit as the code.
- A changelog is optional. If `CHANGELOG.md` has a `## [<version>]` section it goes in the release
  body; otherwise the body lists the commits since the previous tag.
- The release body must keep the line that offhorizon is not approved by or associated with
  horizon-rust, Mojang or Microsoft.

## Pull requests

**Do not open a pull request for offhorizon itself.** OffHorizon's code, workflows, licence and
branding go straight to the `offhorizon` branch, pushed by PiBOH or an agent he asked. There is
no OffHorizon PR to review.

The one exception is translations for **upstream Horizon**: branch `traduzioni-upstream` is based
on `horizon-rust/horizon` main and opens a PR there containing **only**
`crates/horizon-client/assets/horizon/lang/*.json` (and `launcher/lang/*.json` only if upstream
needs them). Its values say Horizon, never OffHorizon. Never mix code, branding, workflows,
licence or version changes into it. OffHorizon's own translation tables are already complete on
`offhorizon`; do not open a translation PR on the fork.

## Upstream sync

`.github/workflows/sync-upstream.yml` runs every day (and on demand). It fetches
`horizon-rust/horizon` main, merges it with `-X ours` so our side wins a conflicting hunk, then
runs `tools/offhorizon_sync.py`. That script rebrands language values upstream added, fills new
game keys from `en_us.json` with the English fallback, and refuses to push if offline accounts,
the hard-wired `None` telemetry endpoint, the absent collection module or the OffHorizon name
disappeared. A failed merge or check opens (or comments on) an issue titled "Upstream sync
failed" and pushes nothing. The sync never opens an OffHorizon PR.

## Branding and languages

The product is **OffHorizon** in the game and the launcher. The launcher rewrites the visible
"Horizon" to "OffHorizon" in every language; the game's strings do it in their own files. The
word **Rust** is a language name and stays `Rust` in every language, never translated.

Release assets and installers are named `offhorizon-launcher-…` (not `horizon-launcher-…`). The
Cargo package and binary names stay as they are; only what a player downloads and sees changes.

OffHorizon's language tables are complete on `offhorizon`: every key in `en_us.json` exists in
every game language, and every key the launcher asks for exists in every launcher language. The
separate upstream PR (`traduzioni-upstream`) carries Horizon-branded game tables only, with every
`en_us.json` key present in every language; it never carries OffHorizon values. Missing entries
were filled with the English text as a fallback; they are **unreviewed** and must not be marked
reviewed (`tools/lang_review/`). Improve a translation only by writing it in that language.
`tools/offhorizon_sync.py` keeps new upstream game keys filled on future syncs; new launcher keys
still need a translation commit.

## Licence

The repo is released under the [Unlicense](LICENSE): no warranty, no liability. There is no
`LICENSE-MIT` file. The code derives from Horizon, whose contributors license it under MIT; that
attribution stays in the README and `THIRD_PARTY_NOTICES.md`. Do not add another licence file or
change the terms without PiBOH's word.

## Build and test

**`.local/` is the only scratch folder.** Every temporary file, downloaded toolchain, unpacked
archive and Cargo target goes under `.local/` (for example `.local/target`, `.local/tools`), never
in the repo root, the system temporary folder or a home directory. `.local/` is git-ignored and
`tools/check_public_tree.py` treats it as private. If a path with spaces breaks a linker, map a
drive letter to the checkout and keep the toolchain under that drive's `.local/`, then remove the
mapping when done.

```sh
cd launcher
cargo check --locked                 # after an edit
cargo test --release --locked        # before pushing
cargo build --release --locked       # what the release workflow builds

cargo check -p horizon-client        # the game (root workspace)
python3 tools/pr_ready.py            # the upstream self-check, when the change reaches the game
```

The launcher is its own Cargo workspace; the game is the root workspace. Linux needs
`libxkbcommon-x11-dev libwayland-dev libx11-xcb-dev libfontconfig-dev libvulkan-dev libasound2-dev`.
Use `CARGO_TARGET_DIR=.local/target` when the default target would land outside `.local/`.

## Where things live

| Area | Files |
|---|---|
| Accounts, Microsoft sign-in, offline accounts | `launcher/src/account.rs`, `account/session.rs`, `account/tests.rs` |
| Starting the Rust game, the owner gate | `launcher/src/backend.rs` (`client`, `client_as`), `launcher/src/account.rs` (`session_for_game`) |
| Starting a Java instance | `launcher/src/java_launch.rs`, `launcher/src/backend.rs` (`java_game`) |
| Account UI | `launcher/src/glass_account.rs`, `launcher/src/main.rs` (`microsoft_rows`), `launcher/src/frame.rs` |
| CLI and MCP | `launcher/src/cli.rs`, `launcher/src/mcp.rs` |
| Install, update, versions | `launcher/src/backend.rs`, `launcher/src/versions.rs` |
| Release | `version.txt`, `.github/workflows/auto-release.yml` |

The game's own map is upstream's: [docs/architecture.md](docs/architecture.md), and the crates are
described in the upstream AGENTS.md history. Keep the layer rule: the engine crates must not
depend on Horizon-only crates (`tools/check_layers.py`).

## Rules that bite

- **No hardcoded user-facing strings.** In the launcher, wrap visible text with `tr("...")` or
  `trf("... {arg} ...", &[("arg", &val)])` from `crate::i18n`; in the game use translation keys.
- **Controllers reach what registers.** Draw controls with a registering helper (`ui::button`,
  `settings_shell::foot_button`); a screen that registers only part of itself ends its crosshair.
- **Keep runs away from real player data.** Start games and the launcher with fresh temporary
  `HORIZON_HOME`, `HORIZON_LAUNCHER_HOME`, `XDG_DATA_HOME` and `XDG_CONFIG_HOME` folders under
  `.local/`, plus `HORIZON_LAUNCHER_NO_SELF_UPDATE=1` and `HORIZON_NO_AUDIO=1`.
- **Mark what an agent posts.** PR bodies start with `> 🤖 Opened by <login>'s AI agent (<tool,
  model>), not by <login> personally.`; comments start with `🤖 Agent:`.
- **Attribution.** Commits end with a `Co-Authored-By:` line for the agent that wrote them.
- **One test per changed behaviour.** No test for a refactor, a move or a constant. Test what a
  player can observe, not words in source.
- **Compile cheaply.** `cargo check` after an edit; clippy, fmt and the tests once, at the end.
  Never park in polling loops: start long commands in the background or wait once.

## Horizon-only parts

Horizon's extras, cosmetics and shop stay behind their flags and are off by default where they
change the vanilla look ([docs/design-system.md](docs/design-system.md)). Telemetry is documented
upstream but disabled here; see the section above.
