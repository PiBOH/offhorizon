# Changelog — engineering record

What changed inside OffHorizon, with the module, the cause and the check. Players' notes live in
[CHANGELOG.md](CHANGELOG.md); this file never goes into a release body. One version section per
release, newest first, and an Unreleased section for work that is not released yet.

## [Unreleased]

### Added
- `tools/site_download_links.py`: rewrites every `releases/download/<tag>/…`, `releases/tag/<tag>`
  and the same tag in the site's prose to the given tag, across the root `*.html` and
  `pages/*.html` of a site folder. It exits 1 when no page references a tag, so a page rewrite
  cannot leave the release job silently doing nothing. Tested on a copy of `gh-pages`:
  `v0.1.5-nightly` → `v0.2.0-nightly` touched all 8 pages (including the terms page's version
  without the `v`), a second run changed nothing.
- `.github/workflows/auto-release.yml`: the `release` job checks `gh-pages` out into `site/` after
  publishing and runs the script with `needs.version.outputs.tag`, then commits and pushes the
  retagged pages. A `release: published` workflow cannot do this: GitHub does not start workflows
  from events its own `GITHUB_TOKEN` caused, so the step lives in the release job that creates the
  release. The buttons carry a tag rather than `/releases/latest/download/…` because a nightly is
  a pre-release and `/latest` leaves pre-releases out (that URL answers 404).

## [0.2.2-nightly] - 2026-11-17

### Added
- `other_launchers::bring_known_players` (and `bring_known_players_in`, which takes the accounts
  file and the skin folder so a test drives it): at startup `Launcher::new` reads each known
  launcher's folder — the official `.minecraft`, Prism, MultiMC, SKLauncher, ATLauncher, the
  Modrinth App, GDLauncher, OffHorizon — and every account it recorded becomes an offline account
  with that nickname, plus the skin the launcher kept. The cause of “I can't see my profile”: the
  GUI never called `bring_player` (only `import` on the command line did), so the launcher kept
  the synthetic `offline_name()` account and the real nickname never appeared. The first profile
  brought in is selected unless a Microsoft account was picked; an account of the same name is
  never shadowed. Check: `the_player_another_launcher_recorded_comes_across_at_startup`.
- `instance_icons::MARK` is `packaging/icons/hicolor/256x256/apps/offhorizon.png` again (it said
  `horizon.png`), so an instance's fallback mark is OffHorizon's own logo.
- `packaging/windows/offhorizon.iss`: Inno Setup script for an optional
  `OffHorizon-Setup-Windows-x64.exe`. `auto-release.yml`'s Windows job installs Inno Setup with
  Chocolatey, compiles the script into `dist/` and uploads it beside the portable exe; the
  uninstaller removes `%APPDATA%\offhorizon` and the install folder, never `horizon-launcher`.

### Fixed
- `mojang::libraries` (the version file's library list, shared by `download` and the new
  `download_libraries`) and `backend::preparer`: before the preparer runs, OffHorizon's own game
  folder gets every library of the pinned manifest from Mojang's public Maven files. A folder
  copied from another launcher, or a download that stopped, was left without `jtracy-1.14.38.jar`
  and `horizon-prepare` failed with the misleading “Horizon does not download the game” hint. The
  client jar and the assets are still only downloaded for an owner.
- `Launcher::scene_size`: a frame whose window size differs from the last one paints the sky tiles
  instead of the glass scene, so a resize no longer shows the window's white clear (the flash).
- `launcher/lang/*.json` and `crates/horizon-client/assets/horizon/lang/*.json`: a repair pass put
  the protected names back (`Discord`, `Rust`, `Java`, `Vanilla`), replaced building nouns used
  for a program's build with `build`, and rebranded `Horizon` to `OffHorizon`. Check:
  `the_names_that_are_never_translated_stay_english` in `i18n/tests.rs`.
- The site in `.local/site_final` is the playhorizon.dev capture with Next's runtime removed (its
  hydration rewrote the brand back), `/_next/image` rewritten to the captured pictures, `_next`
  CSS kept at the paths the markup asks for, and every download, repository and site link pointed
  at OffHorizon.

## [0.2.1-nightly] - 2026-11-17

### Added
- `launcher/src/bin/offhorizon-uninstaller.rs`: a portable uninstaller, built by
  `.github/workflows/auto-release.yml` for Linux and Windows and attached beside the launcher (so
  `SHA256SUMS` covers it). Every folder goes through `ours()`, an allowlist of the names OffHorizon
  and Horizon use (`horizon`, `horizon-launcher`, `offhorizon`, `OffHorizon`, `Horizon Launcher`,
  `OffHorizon Launcher`); a folder it does not know is reported, never deleted, so `.minecraft`,
  Prism, MultiMC or SKlauncher data cannot be removed. `--remove-data` and `--remove-worlds` are
  opt-in, `--dry-run` prints the plan.
- `other_launchers::horizon_instances`, `player_profile`, `account_root` and
  `relink_player_files`, plus the `sources::known_folders` rows for `data/horizon` and
  `data/horizon-launcher`: OffHorizon and Horizon game folders are importable, the profile is read
  from the launcher's root (`launcher_accounts.json`, `accounts.json`, `profiles/`) rather than
  the game folder, and `saves/<world>/{playerdata,stats,advancements}` gain a copy under the
  offline account's UUID (Java's `OfflinePlayer:<name>`), which is what the world loads by.
  `account::offline_uuid` is public for that fallback when the other launcher recorded no UUID.
- `other_launchers::sklauncher_instances` reads every folder SKlauncher used at once — 4.0's
  `instances/`, and 3.2's `profiles/` or `skprofiles/` — instead of stopping at the first that
  exists, so 3.2 profiles without an `instance.json` are no longer hidden by a newer `instances/`
  folder. Deduplicated by game folder; `sklauncher_3_profiles_come_across_beside_the_new_instances_folder`
  holds a 3.2 profile with `saves/`, `playerdata/` and `servers.dat` next to a 4.0 instance and
  checks both are found and that `copy_game` copies the world.
- `other_launchers::bring_player` (with `bring_player_with` for the test) makes the imported
  profile an offline account whenever it is missing — a signed-in Microsoft account no longer hid
  it — selects it, copies the other launcher's `skins/<name>.png` to `data/skins/<uuid>.png` and
  links `playerdata`, `stats` and `advancements` to its UUID. `heads::offline_skin_in` shows that
  face for offline accounts instead of the UUID's default. Test:
  `an_imported_profile_becomes_the_offline_account_with_its_skin`.
- `Settings::glass_tint` defaults to 0 (flat plates);
  `glass_and_blur_default_to_flat_plates_and_stay_in_range` holds it.
- `installer::first_start_with` makes the shortcuts only on the start that answers the question,
  so a deleted one stays deleted; `installer::create_desktop_shortcut` is the new Windows button
  in Settings → About, tested by `the_settings_button_makes_the_desktop_shortcut_alone`.
- `mojang::resilient` retries a dropped connection three times and asks Maven Central for a
  library, so one TLS reset (`peer closed connection without sending TLS close_notify`) no longer
  ends a 26.3 preparation at a single jar.
- `mojang::download` fetches the game and its libraries with `fetch_each(..., false)`: a file that
  fails no longer stops the others, so one attempt finishes as much as it can and the next asks
  only for what is missing. Test: `a_download_resumes_where_it_stopped`.

### Changed
- Dedicated GPU removed end to end: `launcher/src/gpu.rs` (and `lib.rs`'s module), `Settings::dedicated_gpu`
  and its `offhorizon::DEDICATED_GPU` default, `Launch::dedicated_gpu` and `launch::switch_env`
  (`DRI_PRIME`, `__NV_PRIME_RENDER_OFFLOAD`, `__VK_LAYER_NV_optimus`, `__GLX_VENDOR_LIBRARY_NAME`),
  `Found::{two_gpus,nvidia}`, the Launch panel's switch, the Appearance row, and
  `instance launch --dedicated-gpu`. Old `settings.json`/instance files that still carry
  `dedicated_gpu` load unchanged: serde ignores the unknown key. `gfx.rs`'s device request no
  longer asks for `PowerPreference::HighPerformance`.

### Fixed
- `windows_runtime::redirect_in` returns a command with neither `HORIZON_HOME` nor a working
  folder unchanged instead of `expect`ing one, and `backend::client_as` refuses a missing or
  non-executable game program with a message before `spawn`.

## [0.1.5-nightly] - 2026-11-15

### Added
- A portable release again: the Windows job of `.github/workflows/auto-release.yml` uploads one
  `OffHorizon-Launcher-Windows-x64.exe`. `installer::setup_name` no longer takes
  `OffHorizon-Setup-Windows-x64` for an installer (only the legacy `horizon-launcher-setup`), so a
  release asset cannot turn into a setup by its name; the install/uninstall code of 0.1.5-nightly
  stays for the player who installed that build, and `installer::tests`
  `the_portable_release_is_no_setup_and_the_old_one_still_is` holds the line.
- `installer::shortcut_target`: `create_shortcuts` and `shortcuts_present` work on the copy in
  `%APPDATA%\offhorizon` rather than on the file the player launched, so an entry keeps opening
  the newest launcher after an update overwrites that copy; with no copy yet they fall back to the
  running file. `installer::sweep_old_copies` deletes `OffHorizon*` files other than
  `OffHorizon.exe` in that folder once they are a week old (`KEEP_OLD`), called after a successful
  copy; `the_launcher_keeps_a_copy_and_only_the_shortcuts_it_is_told_to_make` sets a stale file
  eight days back with `File::set_modified` and asserts only `OffHorizon.exe` survives.
- `installer::{home, home_exe, keep_a_copy}`: a portable launcher keeps itself in
  `%APPDATA%\offhorizon\OffHorizon.exe`. The copy is written beside the target first
  (`OffHorizon.exe.new`) and renamed over it, so a failed copy never leaves a broken launcher and a
  copy that is running (a player started it from there) keeps the old file until the next start;
  `HORIZON_LAUNCHER_PORTABLE_DIR` moves the folder, for tests.
- `installer::{first_start, first_start_with}`: the first start copies the launcher and asks once,
  in a `MessageBoxW` dialog (`Win32_UI_WindowsAndMessaging` is a `windows` feature now), then
  stores the answer in `Settings::shortcuts` (`Some(false)` is a no that stays a no, `Some(true)`
  keeps the shortcuts in step on later starts). `first_start_with` takes the question, the file and
  the argument count, so the whole path is testable without a dialog;
  `installer::tests::the_first_start_asks_once_and_keeps_the_answer` reads the copy from disk, the
  answer from `settings.json` (`HORIZON_LAUNCHER_HOME` under `.local/`), the shortcut with
  `points_to`, and checks that a script's command line asks nothing. The tests that share the
  process environment take a `PLACES` mutex after two of them raced over `HORIZON_LAUNCHER_SETUP_MENU`.
- `installer::{create_shortcuts, shortcuts_for, remove_shortcuts, shortcuts_present}`: the Start
  menu and desktop links, the same two the old setup wrote, without the registry entry: a portable
  launcher is not an installed program. `Settings::shortcuts()` (unset = no) and the `Shortcuts`
  switch in Settings → About (`Built.shortcuts`, Windows only, the sibling of the Linux
  applications-menu switch) change the answer afterwards, and `tr` keys
  "Add OffHorizon to the Start menu and the desktop?", "Shortcuts",
  "OffHorizon is in your Start menu and on your desktop." and its negation carry it in every
  language, with Italian in `launcher/lang/it.json` and the English fallback elsewhere.
- `installer::install_from(source, options, progress)`: what `install` copies is a parameter now,
  so a test installs a small file rather than the whole test binary.
- `installer::keys`, `parent_of`, `testing` and the `HORIZON_LAUNCHER_INSTALL_KEY` override, plus
  `HORIZON_LAUNCHER_SETUP_MENU` for `start_menu_link`: a test writes its Start menu shortcut and
  its two HKCU keys under a folder and a key base of its own, and `uninstall` closes no running
  launcher then (`testing`), so nothing of a player's or a developer's is touched or killed.
- `installer::tests::install_puts_the_shortcut_and_the_apps_entry_there_and_uninstall_takes_them_away`:
  installs a fake setup into a temp folder with spaces, then reads the shortcut's target from the
  file (`points_to`), the Apps entry's `DisplayName`/`Publisher`/`InstallLocation`/`DisplayIcon`/
  `UninstallString`/`QuietUninstallString` from HKCU, and repeats the install (an update) to see
  the folder still holds one file; `uninstall` must return no leftovers and leave no file, no
  shortcut and no key.
- `installer::first_run` still needs a double-click (`first_start` returns early otherwise): a
  script that runs the release `.exe --version`, or an MCP tool, copies nothing and asks nothing.
  The installer's own test also removes the `Software\OffHorizon\Tests` folders it makes, so a test
  run leaves the account as it found it.
- `installer::delete_soon`: the copy the uninstaller runs from can only be renamed out of the
  install folder, so a small `.cmd` written beside it waits for the process to end and deletes the
  copy, then itself. The line goes through a script because `cmd /C` splits a whole command line
  handed to it as one argument wherever the path has spaces; `delete_at_restart` stays as the
  backstop. An uninstall leaves no file beside the install folder and none in %TEMP% (checked on
  a `D:` install with %TEMP% on `C:`).
- `launcher/build.rs` embeds the icon and the version resource for a GNU toolchain too
  (`gnu_resource`, `resource_script`, `numeric_version`): `winresource` drives `windres` with an
  unquoted `-I<CARGO_MANIFEST_DIR>`, which gcc splits on a checkout whose path has spaces — the
  reason the resource used to be MSVC-only. The script lives in `OUT_DIR`, so no spaced path
  reaches the compiler. The installed `.exe` here now carries all seven frames of
  `packaging/icons/offhorizon.ico` and `FileDescription`/`ProductName`/`CompanyName` = OffHorizon.
  Checked by extracting the icon from the installed copy and by finding the ico's frames in it.
- `installer::tests::the_portable_release_is_no_setup_and_the_old_one_still_is`.
- `gpu::apply` (called at start, Windows only): writes the per-program DirectX preference
  (`HKCU\Software\Microsoft\DirectX\UserGpuPreferences`, `GpuPreference=2;`) for the launcher and
  every `horizon-client` under the game home (`gpu::programs`, four folders deep), so the pair
  runs on the discrete card. `Settings::dedicated_gpu` (unset = `offhorizon::DEDICATED_GPU`, on)
  and the `dedicated_gpu_row` switch in Appearance turn it off, which deletes the value; on other
  systems `set` does nothing, the game already asks wgpu for `HighPerformance`.
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
- `shader_pack` (new): `bytes` zips `crates/horizon-shader-api/packs/golden_hour` (through
  `include_bytes!`) into the `golden_hour.hshader` the game reads; `check` runs
  `horizon_shader_api::Pack::read` then `horizon_shader_api::check`, so a manifest for another
  shader API line is refused with its own reason; `install` writes a `.<file>.part.hshader` and
  renames it, then `packs::set_enabled(.., true)` → `pickedShaderPack:horizon:golden_hour` in
  `horizon-graphics.txt`, which `packs` and the Shader Packs page both read. `activate` is that
  same pick for a pack that just installed.
- `browser::horizon::install_version` passes the shader check as `install_version_guarded`'s
  guard and turns a just-installed `Kind::Shader` on. The registry's 13 shaders are 0.1 packs and
  `horizon_mods::index` refuses them while reading their facts, so before this every shader
  install died before `shaderpacks/` was written.
- `browser::installed::builtin_shader` and the Horizon branch of `browser::installed::install`: an
  install refused for its API line (`shader_pack::refused_for_api`) installs the built-in pack and
  lists it (`SourceId::Local`, `shaderpacks/golden_hour.hshader`), so the Shaders page, `mods list`
  and a restart all show it installed; the refusal itself goes to `progress`.
- Tests: `shader_pack::tests` (the pack is one `check` accepts, install turns it on and keeps the
  other `horizon-graphics.txt` lines, a second install does not duplicate it, a 0.1 pack is
  refused); `browser::horizon::tests::installs_a_mod_and_a_shader_into_a_folder_with_spaces`
  (mod `.hmod` + `horizon-mods.txt`, shader in `shaderpacks/` + `pickedShaderPack`, both in
  `mod-browser.json` on a temp folder with spaces);
  `browser::installed::tests::a_shader_for_the_old_api_leaves_the_builtin_one_installed_and_on`;
  `installs_a_shader_pack_into_shaderpacks` now expects the pack turned on.
- `glass::tabbed` draws a solid plate in performance mode instead of painting a piece of the scene:
  the tabbed panel is the card that stayed translucent with performance mode on.
- `backend::download` retries a transport failure twice (`transient_download_error`: «No
  connection», «Download interrupted»), with 400 ms/800 ms backoff, and then tries
  `mirror(url)` — the same path under `repo1.maven.org/maven2` for a `libraries.minecraft.net`
  URL — twice more. `write_checked` still decides, so a mirror cannot stand in for a wrong file;
  the error the player sees names the last transport failure, not «does not download the game».
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
