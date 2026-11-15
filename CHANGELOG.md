# Changelog

All notable changes to OffHorizon are recorded here. It follows the same shape as
[PiBOH/vivi-music-de](https://github.com/PiBOH/vivi-music-de): an Unreleased section, one
section per version, and Added / Changed / Fixed groups.

## [Unreleased]

### ✨ Added
- **SKlauncher instances can be imported** from its `.sklauncher/instances` and `skprofiles`
  folders, with their Minecraft version and loader when SKlauncher records them.
- **The OffHorizon mark carries a single crack**, in the launcher and the window icon.
- **The changelog is part of the repository**, so the release body comes from the version's
  section instead of a generated commit list.

### 🔧 Changed
- **The opening says OFFHORIZON**, and the launcher's own mark and title use the same name.
- **Performance mode is on from the first start.**
- **AI features are off from the first start**: No AI is written before any page is drawn, and
  an explicit choice by the player is kept.

### 🐛 Fixed
- **The launcher no longer presents a Microsoft account as required**: offline accounts play
  both the Rust game and Java instances, and the Microsoft card only adds optional online play.

## [0.1.2-nightly] - 2026-11-15

### ✨ Added
- **Offline accounts play Minecraft Rust and Minecraft Java** without a Microsoft sign-in.
- **The launcher updates from `PiBOH/offhorizon`**, while the Rust game still installs from
  `horizon-rust/horizon`.
- **Native release assets**: `OffHorizon-Launcher-Linux-x86_64` and
  `OffHorizon-Launcher-Windows-x64.exe`, with `SHA256SUMS` and a source archive.
- **`version.txt` is the one version number**, embedded by the build and used for the release
  tag, the launcher version and the update check.
- **The upstream sync runs every 30 minutes** and reapplies the offline-account, no-telemetry
  and branding patches.
- **The launcher is called OffHorizon**, not offhorizon.

### 🔧 Changed
- **The release workflow publishes raw native executables**, not launcher zip files.
- **macOS x64 is opt-in** (`OFFHORIZON_MACOS=true`), so a queued runner cannot hold releases.

### 🐛 Fixed
- **An offline account is created at first start** when the launcher has no account.
- **Java instances download their version from Mojang's public files**, with or without a
  Microsoft account.
- **The release checksums and version check match the fork's workflows.**
