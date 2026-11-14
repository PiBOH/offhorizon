# offhorizon: a map for agents

offhorizon is PiBOH's fork of [Horizon](https://github.com/horizon-rust/horizon), Minecraft: Java
Edition 26.3 rebuilt in Rust. It adds one thing upstream does not: **you can play with an offline
account, without a Microsoft sign-in**. Everything else follows upstream, and vanilla is the
specification wherever it has the feature.

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

## Version and releases

`version.txt` in the root is the **single source of truth** for the program version. It holds one
line, for example `0.1.0-nightly`.

- `launcher/build.rs` reads it into `HORIZON_LAUNCHER_VERSION`; the game's build script reads it
  into `HORIZON_VERSION`. Do not add a second version number anywhere.
- A push whose **first line starts with `v`** runs `.github/workflows/auto-release.yml`. The
  workflow reads `version.txt`, tags `v<version>`, builds the launcher for Linux x86_64, macOS
  arm64 and Windows x64, and publishes a GitHub release. A version with a suffix
  (`-nightly`, `-beta`, `-alpha`, `-rc`) is a pre-release.
- No version-bump commits: change `version.txt` in the same commit as the code.
- A changelog is optional. If `CHANGELOG.md` has a `## [<version>]` section it goes in the release
  body; otherwise the body lists the commits since the previous tag.

## Licence

The repo is released under the [Unlicense](LICENSE): no warranty, no liability. Code derived from
Horizon keeps its MIT notice in [LICENSE-MIT](LICENSE-MIT). Do not change either without PiBOH's
word.

## Build and test

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
  `HORIZON_HOME`, `HORIZON_LAUNCHER_HOME`, `XDG_DATA_HOME` and `XDG_CONFIG_HOME` folders, plus
  `HORIZON_LAUNCHER_NO_SELF_UPDATE=1` and `HORIZON_NO_AUDIO=1`.
- **Mark what an agent posts.** PR bodies start with `> 🤖 Opened by <login>'s AI agent (<tool,
  model>), not by <login> personally.`; comments start with `🤖 Agent:`.
- **Attribution.** Commits end with a `Co-Authored-By:` line for the agent that wrote them.
- **One test per changed behaviour.** No test for a refactor, a move or a constant. Test what a
  player can observe, not words in source.
- **Compile cheaply.** `cargo check` after an edit; clippy, fmt and the tests once, at the end.
  Never park in polling loops: start long commands in the background or wait once.

## Horizon-only parts

Horizon's extras, cosmetics, shop and telemetry stay behind their flags and are off by default
where they change the vanilla look ([docs/design-system.md](docs/design-system.md)). The fork does
not add telemetry of its own.
