# Changelog

All notable changes to OffHorizon, in the order a player meets them. This file is what the release
body quotes: it says what changed for the people who play, not how it was built. The engineering
record — modules, causes, tests — is [CHANGELOG.dev.md](CHANGELOG.dev.md).

The shape follows [PiBOH/vivi-music-de](https://github.com/PiBOH/vivi-music-de): an Unreleased
section, one section per version, and Added / Changed / Fixed groups.

## [Unreleased]

### ✨ Added
- **The site's download buttons follow each release on their own**: publishing a build moves the
  links on `piboh.github.io/offhorizon` to that build's files, so the Download button always
  downloads the newest launcher instead of waiting for the pages to be edited by hand.

## [0.2.1-nightly] - 2026-11-17

### ✨ Added
- **A portable uninstaller**: every release carries `OffHorizon-Uninstaller` for Linux and
  Windows. It removes only OffHorizon's and Horizon's own folders, shortcuts and registry keys;
  your worlds stay unless you ask for them, and your other launchers (`.minecraft`, Prism,
  MultiMC, SKlauncher…) are never touched.
- **Worlds from SKlauncher 3.2 come across too**: a profile that keeps no `instance.json` is
  found — even beside a newer `instances/` folder — with its worlds and server list.
- **The profile imported from another launcher is the offline account**: it keeps the name and
  the skin that launcher kept, even when a Microsoft account is already signed in.
- **Glass starts flat**: the panels take no white (0 %) until the slider moves, and 0 % is what
  Reset brings back.
- **Shortcuts are asked once**: the question is on the first start only, and Settings → About
  has a **Create a desktop shortcut** button that puts one back.
- **A dropped download is tried again**: a library that Mojang's host closes half-way now comes
  from Maven Central too, so 26.3 no longer stops at a missing `jtracy` jar.
- **A download picks up where it stopped**: the files already complete are kept, and one library
  that fails no longer leaves the rest unfetched.
- **Importing from another launcher finds OffHorizon and Horizon too**, and the player comes with
  the instance: their name, the worlds (inventory, ender chest, position, experience and
  advancements are linked to the offline account), and the server list.

### 🔧 Changed
- **The graphics card is the system's choice again**: OffHorizon no longer asks Windows for the
  high-performance GPU and no longer sets `DRI_PRIME` or NVIDIA's offload variables, so the
  driver picks the card that runs the game. A settings file from an older version keeps every
  other choice.

### 🐛 Fixed
- **Play reports a missing game program instead of failing at spawn**, and a Windows game command
  without a home folder no longer panics.

## [0.1.5-nightly] - 2026-11-15

### ✨ Added
- **One portable `.exe`, as in 0.1.4**: download it, run it from wherever you keep it, and
  OffHorizon needs no installer and no administrator rights. On its first start it keeps a copy of
  itself in `%APPDATA%\offhorizon` and asks once whether to add its shortcuts to the Start menu
  and the desktop; the same switch lives in Settings → About, and the `.exe`, the shortcut and the
  window all carry the OffHorizon icon.
- **The shortcuts always open the newest OffHorizon**: they launch the copy in
  `%APPDATA%\offhorizon`, which every start refreshes, so Start menu, desktop and taskbar follow
  an update on their own. A copy an update left behind is removed after seven days.
- **A tooltip shows what a clipped label hides**: an instance's name and state, a server's, the
  account chip and the Play button's progress all read in full when the pointer rests on them.
- **Dedicated GPU on by default**: on a laptop with two graphics cards the launcher now asks
  Windows for the high-performance one, for itself and for the game, and the switch to turn that
  off is in Appearance.
- **A library a network refuses to serve still arrives**: when `libraries.minecraft.net` keeps
  dropping the connection, the same file is fetched from Maven Central and checked against the
  same hash, so preparing stops failing with “Missing …wjgl-spvc…jar”.
- **Importing an instance from another launcher brings the official Rust server** into its
  Multiplayer list, and the name the other launcher played as (an offline account, yours to rename
  in OffHorizon). Key bindings, options and worlds already travelled with the instance.

### 🔧 Changed
- **Technical names stay as they are in every language**: Rust, Vanilla, Java and Launcher are never
  translated, and On/Off are left in English instead of becoming “su” and “giù”.
- **The game window says OffHorizon**, in the title bar and in the taskbar, instead of Horizon.
- **Version 0.1.5-nightly.**

### 🐛 Fixed
- **A shader pack now installs, stays installed and is on**: the packs the registry lists are
  written for the shader API line the game no longer draws, so every install ended in
  “needs shader API 0.1” with an empty `shaderpacks/` and nothing to pick in game. A pack is now
  read the way the game reads it before it lands, a pack that can't be drawn says why, and
  Horizon's own **Golden Hour** pack is installed and turned on in its place.
- **Performance mode now reaches the tabbed panel too**: it used to keep sampling the sky, so the
  cards stayed see-through while every other plate was solid.
- **A reset connection while a Mojang library downloads is tried again** (three times, with a short
  pause), instead of ending preparation with “No connection: … os error 10054”.
- **SKlauncher's own game folders are searched** for Minecraft 26.3 (`sklauncher/minecraft`,
  `instances`, `profiles`) instead of only its program folder, which holds no game files.

## [0.1.4-nightly] - 2026-11-15

### 🐛 Fixed
- **The launcher no longer offers the version it already runs as an update**: `v0.1.4-nightly`
  and `launcher-v0.1.4-nightly` compare as the same launcher version.
- **A missing Minecraft 26.3 client is downloaded from Mojang's public files**, without a
  Microsoft account or ownership check. SKlauncher can keep a Fabric profile without the
  vanilla jar; that no longer blocks preparation.
- **Settings fit a narrow window**: below the wide breakpoint the section list becomes a
  wrapping grid instead of a tall full-width column that pushes the card off-screen.
- **Visible names say OffHorizon everywhere**: the default instance, the settings tiles and
  the error text no longer fall back to “Horizon”.
- **The crack in the mark is light red**, in the PNG and every frame of the Windows icon.

### 🔧 Changed
- **Italian translations were revised**: offline-account text, Microsoft-account wording,
  technical terms such as jar and mixins, and strings that had stayed in English.
- **The README download button points to the OffHorizon site**, and the upstream sync keeps
  `README.md` and `AGENTS.md` from overwriting it.
- **The `gh-pages` site is rebuilt as OffHorizon**, with direct release downloads and no
  analytics or telemetry.

## [0.1.3-nightly] - 2026-11-15

### ✨ Added
- **SKlauncher instances can be imported** from its `.sklauncher/instances` and `skprofiles`
  folders, with their Minecraft version and loader when SKlauncher records them.
- **The OffHorizon mark carries a single crack**, in the launcher and the window icon.
- **The changelog is part of the repository**, so the release body comes from the version's
  section instead of a generated commit list.
