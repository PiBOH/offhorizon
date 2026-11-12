# OffHorizon

The portable, telemetry-free companion for **Horizon** — Minecraft: Java Edition rewritten in Rust.
It reads and edits your saves and your launcher, keeps a copy of everything it touches, and sends
nothing anywhere.

![mark](brand/offhorizon.svg)

## What it does

- **Profile** — the offline player name (`horizon-player.txt`) and the portable game folder beside
  the executable.
- **Worlds** — every save with a preview before you open it: health, hunger, level, inventory and
  size.
- **Player** — heal, feed, experience level.
- **Inventory** — see the stacks, take one out, add one (modded ids too).
- **Time and weather** — `Time`, `DayTime`, `raining`, `thundering` and their timers, with a backup
  each time.
- **Launchers** — finds the 16 launchers players use (Horizon, SKLauncher, Prism, MultiMC, CurseForge,
  Modrinth, ATLauncher, GDLauncher, Technic, FTB, Lunar, Badlion, XMCL, TLauncher, PolyMC, the
  Minecraft Launcher) and imports their worlds onto the stick.
- **Backups** — every file it wrote, and one button to put each back.

The same list from the terminal:

```sh
offhorizon status
offhorizon worlds
offhorizon time night --world "My World"
offhorizon weather thunder 6000
offhorizon launcher point          # the installed launcher plays from this stick
offhorizon launcher undo           # settings.json back as it was
offhorizon launchers
offhorizon import sklauncher
offhorizon backups
offhorizon restore 1731...
```

Without arguments the window opens; every button there is one of these commands.

## Portable

Everything lives beside the executable — on a USB stick, the stick carries the whole setup:

| Path | What |
|---|---|
| `./.minecraft` | the game folder: worlds, options, `horizon-player.txt` |
| `./.offhorizon/backups` | a copy of every file before it was written |
| `./.offhorizon/settings.json` | the language |
| `./.offhorizon/launcher-backups` | copies of launcher files |

The one file outside the stick it may touch is the installed launcher's `settings.json`, and only
when you press **Point the launcher here**: it leaves `settings.json.offhorizon.bak` beside it, a
dated copy in `<launcher>/offhorizon-backups/`, one in the temporary folder and one in its own
backup list. **Undo** restores it.

## Languages

Eighteen XML tables in `lang/`, the same set as the other PiBOH projects: en, it, de, fr, es, pt, nl,
pl, cs, sv, ru, uk, tr, ar, hi, zh, ja, ko. The list and the format are in [AGENTS.md](AGENTS.md).

## Accounts

It does not forge a Microsoft account or write into Horizon's sign-in store, and it never will: the
game plays under the offline name this tool writes, and the launcher plays from the folder this tool
points it at. Its own games never need an account.

## Build

```sh
cargo build --release
cargo test
```

`version.txt` is the only version source.

## License

Unlicense — public domain, no warranty of any kind. See [LICENSE](LICENSE).
