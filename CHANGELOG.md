# Changelog

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
