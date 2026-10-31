# Balatro Workshop 0.2.0

This repository contains the complete Rust project for Linux, Windows, and macOS. Download desktop packages from the repository's [Actions page](https://github.com/nostrus-dominion/balatro-workshop/actions/workflows/build.yml) after a successful **Desktop builds** run, or build from source below.

Version 0.2.0 adds a **Jokers** tab. Open your run's `save.jkr` to see the Jokers you own, then choose Normal, Foil, Holographic, Polychrome, or Negative for each one. Changing to or from Negative adjusts the Joker slot limit automatically. Choose **Save + backup** when finished. The window title must say **Balatro Workshop 0.2.0**.

## Download a desktop build

Open the latest successful **Desktop builds** run and download the artifact for your platform. Unzip the artifact, then extract the platform archive inside it. On Windows, run `balatro-workshop.exe`. On macOS, open `Balatro Workshop.app`. On Linux, open a terminal in the extracted package directory:

```bash
chmod +x balatro-workshop
./balatro-workshop
```

No Rust installation is needed for a packaged executable. The Linux workflow builds on Ubuntu 24.04; compatible newer desktop Linux distributions should provide its required system libraries. KDE should have `xdg-desktop-portal-kde` installed for file dialogs. Open a specific save from the command line if needed:

```bash
./balatro-workshop /path/to/save.jkr
```

## Build from source

Install stable Rust and the platform build prerequisites described in README.md, then run:

```text
cargo run --locked --release
```

The GitHub Actions workflow runs when changes are pushed. You can also use **Actions → Desktop builds → Run workflow**. It builds Windows x86-64, Linux x86-64, macOS Apple Silicon, and macOS Intel packages. These packages are not code-signed with commercial signing credentials.

Version 0.1.2 completed native builds and automated tests for all four targets. See the current workflow results for native build status and `VALIDATION.md` for the checks performed before this repository upload. Compiled executables are kept in downloadable packages rather than tracked in the source repository.

## First edit

Close Balatro, open your `.jkr` save, edit a value, then choose **Save + backup** or **Save copy**. Keep a separate backup of your profile while testing: no real player save or running game was available for compatibility validation.

You can type a complete multi-digit value without saving between digits. Pressing Enter or moving to another field accepts the entry in memory; only Save writes it to disk. Save also includes the text in the field you are still editing.

Use **Play Balatro** to launch through your installed Steam client. If you have edits, choose **Save & play**, **Play saved version** (keeps your edits in the editor), or **Cancel**. Invalid values or a failed save prevent Save & play from launching the game. Apply or discard any Advanced text draft first if you want to save it.

README.md contains full usage, save folder candidates, backup restoration, limits, and build instructions. VALIDATION.md records the actual checks performed. The fixture under `tests/fixtures/` is test data, not a playable game save.
