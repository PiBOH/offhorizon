# Changelog — engineering record

What changed inside OffHorizon, with the module, the cause and the check. Players' notes live in
[CHANGELOG.md](CHANGELOG.md); this file never goes into a release body. One version section per
release, newest first, and an Unreleased section for work that is not released yet.

## [Unreleased]

## [0.1.5-nightly] - 2026-11-15

### Added
- `auto_ports::prepare` (called from `main`'s prepare thread): every Java instance's `mods/*.jar`
  goes through `horizon_port_pack::check`, and what ports is installed into the game folder's
  `horizon-mods/<id>/` through `browser::java::install_ported`, on in `horizon-mods.txt`. Each jar
  logs its outcome (`Rust port of Ruby installed automatically (Works)` / `Ruby: no Rust port
  (…)`), so whether a Rust port exists is visible. A mod already there is kept, and the entry it
  writes is `SourceId::Local` («Imported»), so Mods lists it and Remove works.
- `browser::SourceId::Local`: a Java mod ported from a jar already on the machine, with no project
  page to name.
- `site/`: the Horizon website from `.local/website`, rebranded OffHorizon (titles, meta, og,
  copy, links to `piboh.github.io/offhorizon`), telemetry claims replaced with «OffHorizon
  collects no telemetry», and Next's `/_next/image` proxy rewritten to `media/` (the mirror has no
  optimiser). `_next/static/{chunks,media}` keep the paths the HTML already uses, and `.nojekyll`
  keeps Pages from dropping the underscore folder. It is the gh-pages content: `git checkout
  gh-pages` and copy `site/` over the branch root, or `git subtree push --prefix=site origin
  gh-pages`.
- `packaging/icons/hicolor/*/apps/offhorizon.png`, `scalable/apps/offhorizon.svg` and
  `packaging/icons/offhorizon.icns`, generated from `packaging/icons/offhorizon.ico` (Pillow), so
  the Linux and macOS packages carry the fork's mark under its own name.
- `glass_ui::tip` (with the `Tip` render element): a tooltip view built only when the tooltip
  shows, attached with `StatefulInteractiveElement::tooltip`. `glass_ui::row_text` gives its row
  an id and attaches it when the name is over 22 characters or the state over 28 (instance and
  server rows); the account chip in `frame.rs` and the Play button's state in `glass_home.rs`
  always do. Text that fits allocates nothing.
- `installer::register_first_run` (called from `main` before the window): a release build started
  from a folder with no HKCU install writes the Start menu shortcut and the Uninstall/Apps entry
  for the file it runs from. `HORIZON_LAUNCHER_NO_SHORTCUT` opts out; debug builds never register.
- `other_launchers::import` now adds the official Rust server (`server_list::add_official` with
  `Links::default`) and, when another launcher recorded a usable name (`player_name`: Modrinth's
  `accounts.json`, `launcher_accounts.json`, SKlauncher `profiles/*.json`), creates the matching
  offline account (`account::Accounts::add_offline`, Java's `OfflinePlayer` UUID) if the account
  list is empty.

### Changed
- `build.rs` writes the Windows version resource too: `FileDescription`, `ProductName`,
  `CompanyName` = OffHorizon with `version.txt`'s version, so Explorer, the task bar and Settings
  > Apps read OffHorizon beside the icon (MSVC targets only; a GNU build has no resource).
- `desktop::launcher_entry`, `packaging/linux/*.desktop`, `packaging/linux/appimage.sh`,
  `packaging/macos/Info.plist` and `packaging/macos/package.sh` name OffHorizon and the
  `offhorizon` icon: Start menu entry, AppImage and `OffHorizon.app`/`.dmg`.
- `links.json` and `Links::default` point the website button at `https://piboh.github.io/offhorizon`;
  the registry, ratings, mirror and agent-docs URLs stay on `playhorizon.dev` (they are the
  service, not the brand).
- `i18n::keep_technical`: a translated value that used a word for the metal (it «ruggine», fr
  «rouille», de «Rost», …) gets `Rust` back; the cache key is now (language, source string).
- `launcher/lang/it.json`: `Rust Edition` → «Edizione Rust», `On`/`Off` stay English.
- `horizon-client` window titles (`loading_screen`, `no_resources`, `split/windows`) say OffHorizon.
- `sync-upstream.yml` restores `launcher/`, `AGENTS.md`, `README.md`, `CHANGELOG.md` and
  `CHANGELOG.dev.md` from the pre-merge commit, so the launcher and its download links stay ours.

### Fixed
- `glass::tabbed` draws a solid plate in performance mode instead of painting a piece of the scene:
  the tabbed panel is the card that stayed translucent with performance mode on.
- `backend::download` retries a transport failure twice (`transient_download_error`: «No
  connection», «Download interrupted»), with 400 ms/800 ms backoff; a wrong checksum still fails.
- `sources::known_folders` searches SKlauncher's game folders (`sklauncher/minecraft`,
  `sklauncher/instances`, `sklauncher/profiles`) as well as `.sklauncher`.

## [0.1.4-nightly] - 2026-11-15

### Fixed
- `backend::release_order` compares the fork's `vX.Y-nightly` and `launcher-vX.Y-nightly` tags as
  one semver version, so the launcher stops offering itself.
- `download_version_public` fetches a missing 26.3 client from Mojang's public version files.
- `glass_settings` lays its section list out as a wrapping grid below the wide breakpoint.
- `i18n::branded` rewrites the product name in every table, including lowercase and tiles.
- The mark's crack is light red in `icon.png` and every frame of `packaging/icons/offhorizon.ico`.

### Changed
- Italian table revised; `README.md` download button points at `https://piboh.github.io/offhorizon/`.
- `gh-pages` carries the OffHorizon site, with no analytics and no telemetry.

## [0.1.3-nightly] - 2026-11-15

### Added
- `other_launchers::sklauncher_instances` reads `.sklauncher/instances` and `skprofiles`.
- `CHANGELOG.md` feeds the release body (`auto-release.yml`).
