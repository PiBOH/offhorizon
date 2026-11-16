# Changelog — engineering record

What changed inside OffHorizon, with the module, the cause and the check. Players' notes live in
[CHANGELOG.md](CHANGELOG.md); this file never goes into a release body. One version section per
release, newest first, and an Unreleased section for work that is not released yet.

## [Unreleased]

## [0.1.5-nightly] - 2026-11-15

### Added
- `installer::setup_name`: the release asset `OffHorizon-Setup-Windows-x64.exe` is taken for the
  installer as well as the old `horizon-launcher-setup` name, so the binary a player downloads
  from the release page opens the setup instead of the launcher. The Windows job of
  `.github/workflows/auto-release.yml` copies the built binary under both names and uploads
  `dist/*`, so every release carries the launcher and the installer.
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
- `installer::register_first_run` acts only on a double-click (`first_run`, no arguments): a
  script that runs the release `.exe --version`, or an MCP tool, used to leave a Start menu entry
  and an Apps entry pointing at whatever folder the executable sat in. The installer's own test
  now also removes the `Software\OffHorizon\Tests` folders it makes, so a test run leaves the
  account as it found it.
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
- `installer::tests::the_release_ships_an_installer_the_launcher_does_not_take_for_one`.
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
