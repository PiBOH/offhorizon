# Changelog

## [0.2.2-nightly]

Minecraft: Rust Edition plays from here now, and it plays first.

- **Rust first**: OffHorizon finds the Horizon client (`horizon-client`) beside the stick — in
  `versions/<tag>/`, where the official launcher keeps it, or dropped next to the executable — and
  starts it with `--resources`, `--game-dir` and `--name`, its logs and settings in
  `.offhorizon/horizon`. Java stays: `--rust` and `--java` pick the edition, and with neither the
  Rust client plays when there is one (`versions`, `play`).
- **Instances carry their edition**: a new instance picks Rust or Java and its Play starts that
  game; an instance made earlier plays Java, as it did.
- **One home**: Play starts the Rust client when there is one, the edition switch and the client
  list say which, and the sidebar shows each instance's edition.

## [0.2.1-nightly]

The window is a launcher now.

- **Home**: it opens on the versions found on this computer and on a big *Play*, which builds the
  command from the version's JSON and starts the game as the offline name (`versions`, `play`).
- **Instances**: a version in a folder of its own. At creation you choose the folder: beside this
  executable (portable, the default) or any install already here (`instance new|list|remove`).
- **The official look**: the window is now the official Horizon launcher's shell — the pages as
  tabs in the top bar, the instances in the sidebar, the glass plates, corners and type steps from
  `crates/horizon-theme`, no background picture and no telemetry. OffHorizon keeps its accent.
- **Release notes**: the release body is the changelog section of the released version, no longer
  the whole file.

## [0.2.0-nightly]

OffHorizon, portable, and the same from the terminal as from the window.

- **Launcher home**: the window opens on the versions found here and on a big *Play*, which builds
  the command from the version's JSON and starts the game as the offline name (`versions`, `play`).
- **Instances**: a version in a folder of its own, made where you choose the folder — beside this
  executable (portable, the default) or any install already on the computer (`instance new|list|remove`).

- **Renamed** to OffHorizon: binary `offhorizon`, branch `offhorizon`, assets
  `offhorizon-windows-x64.exe` and `offhorizon-linux-x64`.
- **Portable**: the game folder is `./.minecraft` and everything else is `./.offhorizon`, both beside
  the executable. Nothing is written to the system any more; it runs from a USB stick.
- **Launcher, reversible**: *Point the launcher here* writes the portable folder into the installed
  launcher's `settings.json`, keeping `settings.json.offhorizon.bak`, a dated copy in the launcher's
  `offhorizon-backups/`, one in the temporary folder and one in the backup list; *Undo* restores it.
- **Worlds page**: every save with a preview before opening it — health bar, hunger, level,
  inventory and size.
- **Time and weather**: `Time`, `DayTime`, `raining`, `thundering`, rain and thunder timers, with a
  backup. Day, noon, night and midnight one click away.
- **Other launchers**: finds 16 launchers (Horizon, SKLauncher, Prism, MultiMC, CurseForge, Modrinth,
  ATLauncher, GDLauncher, Technic, FTB, Lunar, Badlion, XMCL, TLauncher, PolyMC, Minecraft) and
  imports their worlds onto the stick.
- **Terminal parity**: every button is a command (`status`, `worlds`, `name`, `heal`, `feed`,
  `level`, `item add|remove`, `time`, `weather`, `launcher point|undo`, `launchers`, `import`,
  `backups`, `restore`, `lang`), all through one `actions` module.
- **Eighteen languages** in XML tables under `lang/`: en, it, de, fr, es, pt, nl, pl, cs, sv, ru,
  uk, tr, ar, hi, zh, ja, ko.
- **New mark**: the red fracture tile (`brand/offhorizon.svg`), drawn in code for the window icon and
  the header, so no image file is needed.
- **No telemetry**: no socket, no report, no update check.

## [0.1.0-nightly]

First nightly. A window with four pages and a log:

- **Profile** — finds Horizon's launcher folder and the game folders on this computer, shows the
  offline player name the game plays under and writes a new one to `horizon-player.txt`.
- **Player** — reads the player out of a world's `level.dat`: health, hunger, saturation, air,
  experience level and points. **Heal** and **Feed** put them back to full.
- **Inventory** — lists the stacks in the save's inventory and takes any of them out; **Add item**
  puts an item id (modded ids included) and a count in the first free slot, stacking onto one that
  isn't full.
- **Backups** — every file the tool writes is copied first, and any copy can be put back.

An NBT reader and writer for gzipped saves that keeps every tag it doesn't understand, in order,
so a save that is only read is written back unchanged.
