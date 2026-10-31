# Balatro Workshop

A native, offline Balatro save editor for **Linux, Windows, and macOS**, written in Rust with egui. Inspired by [WilsontheWolf's balatro-save-loader](https://github.com/WilsontheWolf/balatro-save-loader). No browser, local server, account, or game installation is needed to open an exported save.

## What it does

- Opens raw-DEFLATE `.jkr` saves and backups, including run, profile, settings, and collection data.
- Edits money, ante, round, hand size, joker/consumable slots, hands, discards, reroll cost, probabilities, profile stats, and settings when those fields exist.
- Lists the owned Jokers in a run by name and position. Change their edition to Normal, Foil, Holographic, Polychrome, or Negative using a dropdown; Negative slot changes are handled automatically.
- Provides an expandable data tree and an advanced **data-only Lua table** editor. Numeric keys, sparse tables, byte strings, unknown fields, and decimal numeric text are preserved.
- Finds candidate save folders on Windows, macOS, native Linux, and Steam/Proton, including Flatpak Steam and external Steam libraries. Browse any custom folder when automatic discovery misses it.
- Creates a uniquely named, byte-for-byte backup before each source overwrite, validates output, and uses a same-directory temporary file to replace the save.
- Refuses to overwrite a source changed since it was opened. Save copy always requires a new filename.
- Supports drag-and-drop, Ctrl/Cmd+O, Ctrl/Cmd+S, and Ctrl/Cmd+Shift+S.
- Keeps complete text entries in memory while typing, including temporarily incomplete numbers. Accepts them on Enter, focus loss, or Save without writing between digits.
- Launches Balatro from **Play Balatro** using the installed Steam client, with a choice to save pending changes first.

## Run a packaged build

Unpack the platform archive. On Windows, launch `balatro-workshop.exe`. On macOS, open `Balatro Workshop.app`. On Linux:

```bash
./balatro-workshop
# Or open a specific file:
./balatro-workshop /path/to/Balatro/1/save.jkr
```

Linux requires a graphical X11 or Wayland session, an OpenGL driver (software rendering also works), and an XDG desktop portal with a file chooser, such as `xdg-desktop-portal-kde` on KDE. X11 also requires `libxkbcommon-x11-0` on Debian/Ubuntu. Packaged Linux binaries built on Ubuntu 24.04 require compatible system libraries; build from source for older distributions.

The included GitHub Actions workflow builds Linux x86-64, Windows x86-64, and separate macOS Apple Silicon and Intel packages. It uploads downloadable workflow artifacts; it does not publish releases automatically. Windows builds are unsigned. macOS packages use ad-hoc signing, without Developer ID signing or notarization.

## Build from source

Install the current stable [Rust toolchain](https://rustup.rs/). The locked dependency set requires Rust 1.89 or newer. Use `--locked` to retain the tested dependency versions.

**Debian/Ubuntu build dependencies:**

```bash
sudo apt install build-essential pkg-config libx11-dev libxi-dev libxcursor-dev \
  libxrandr-dev libxinerama-dev libxkbcommon-dev libxkbcommon-x11-dev \
  libwayland-dev libgl1-mesa-dev
```

**Windows:** install the Visual Studio Build Tools with “Desktop development with C++” and the Windows SDK; use Rust's MSVC toolchain. **macOS:** install Xcode Command Line Tools (`xcode-select --install`).

From this project directory:

```bash
cargo run --locked --release
# Or build without launching:
cargo build --locked --release
```

The executable is in `target/release/`. Python 3.11+ is needed only for optional packaging:

```bash
python3 scripts/package.py
```

Use `python` instead of `python3` on Windows if that is your installed command. Packages and SHA-256 checksums appear in `dist/`. The Linux x86-64 artifact is named `linux_x86_64`; it contains `linux_x86_64.tar.gz` and its checksum.

## Editing a save

1. Close Balatro. Make sure you have a separate backup of the complete profile folder before your first edit.
2. Open `save.jkr` for the active run, `profile.jkr` for profile stats, `settings.jkr` for preferences, or `meta.jkr` for collection data.
3. Use **Jokers** for edition changes, change common values in **Quick edit**, edit values in **Data tree**, or edit the table in **Advanced text** and click **Apply text**.
4. Choose **Save + backup** to overwrite the opened file, or **Save copy** to create a separate file. Save copy switches the editor to the new file.
5. Start Balatro and check the result. Steam Cloud may replace local edits during synchronization; resolve cloud conflicts deliberately.

Quick controls change the named stored values, not every related piece of game state. For example, “Hands per round” changes `GAME.round_resets.hands`, not the current round's remaining hands. The data tree exposes other stored values. Slot changes may need a new round. Structurally valid values can still be unsuitable for the game's current state.

**Revert** discards unsaved edits; it does not undo a previous disk save. **Discard text edits** discards only unapplied text changes. Changing files or closing the window prompts before discarding unsaved changes.

### Change a Joker's edition

Open the active run's `save.jkr` and choose **Jokers**. The editor reads `cardAreas.jokers.cards`, displays owned Jokers in saved position order, and shows their names, internal IDs, and current editions. Shop, booster, playing-card, and collection entries are excluded. Duplicate Jokers remain separate rows.

Use the **Edition** dropdown on the desired row:

| Edition | Bonus |
| --- | --- |
| Normal | No edition bonus |
| Foil | +50 Chips |
| Holographic | +10 Mult |
| Polychrome | ×1.5 Mult |
| Negative | +1 Joker slot |

Changing an edition replaces its old edition bonus. Converting a non-Negative Joker to Negative raises the saved Joker slot limit by one; converting it back lowers that limit by one. Repeatedly selecting the same edition does not add slots. Other card fields, including abilities, stickers, and unknown data, are preserved. Edits stay in memory until **Save + backup**, **Save copy**, or **Save & play**; **Revert** restores the last loaded or saved state.

This control supports the base game's edition and slot layout. Custom edition fields and recognized modded slot layouts are displayed with an explanation and left intact; use Advanced text for those saves. An invalid slot limit prevents a Negative transition without applying a partial change. Profile and collection files show an explanation instead of an empty run editor. Apply or discard any Advanced text draft before using the Jokers controls.

### Launch Balatro

Click **Play Balatro** in the sidebar. With unsaved changes, choose **Save & play**, **Play saved version**, or **Cancel**. Save & play validates and saves first, with the usual backup; a failed save prevents launch. Play saved version keeps your unsaved work in the editor. Apply or discard Advanced text drafts before using Save & play.

Steam must be installed and registered to handle `steam://` links. The launcher sends `steam://rungameid/2379780` through `xdg-open` on Linux, `open` on macOS, or the Windows URL handler. This uses Steam's existing game installation and launch settings, including Proton where configured. The status confirms a launch request was sent, not that the game finished starting. See Valve's [Steam browser protocol](https://developer.valvesoftware.com/wiki/Steam_browser_protocol).

### Restore a backup

Close the game and the editor. In your file manager, rename the current save to keep it, then copy the desired `save.jkr.backup-…` file back to `save.jkr`. Backups contain the original compressed bytes; do not decompress them. They can also be opened directly in the editor. Backups are kept beside the source and are not automatically pruned.

### Candidate folders

| Environment | Location checked |
| --- | --- |
| Windows | `%APPDATA%\Balatro` |
| macOS | `~/Library/Application Support/Balatro` |
| Native Linux / LÖVE | `$XDG_DATA_HOME/love/Balatro`, `~/.local/share/love/Balatro`, `~/.local/share/Balatro` |
| Steam / Proton | `<Steam library>/steamapps/compatdata/2379780/pfx/drive_c/users/steamuser/AppData/Roaming/Balatro` |

These are discovery candidates, not a guarantee for every installation. Profile subfolders are usually numbered. Browsing scans up to three subdirectory levels; symlink subdirectories are not traversed. No folders are created during discovery.

## Compatibility and limits

This is a first implementation, not a game-version compatibility certification. The fixture is synthetic, based on the upstream field layout; **it is not a playable save**. Testing on your actual saves is still needed. Keep an untouched backup when trying it.

The parser accepts a `return` table containing quoted or long strings, finite decimal numbers, booleans, nested tables, numeric/string/boolean keys, comments, and positional entries. It never evaluates Lua. Functions, expressions, `nil` entries, hex number literals, non-finite numbers, duplicate keys, and mod-specific constructor calls are rejected with an error. It does not silently delete unknown data or try to execute mod code. Modded saves using only supported data values can be edited, but mod compatibility is not guaranteed.

Limits: 16 MiB compressed, 32 MiB expanded text, 128 levels of nesting, and 500,000 parsed nodes. Canonical formatting can expand a file beyond the text limit, in which case writing is refused. Numeric spelling is preserved; the game's Lua runtime still determines numeric precision. Empty and sparse tables remain tables rather than being converted to JSON arrays.

The advanced editor uses Lua table text instead of JSON to preserve distinct keys such as `[1]` and `["1"]`. Non-ASCII string bytes are displayed as valid Lua decimal escapes there. The tree and quick controls show valid UTF-8 strings normally.

The upstream calculator URL export and browser-specific UI are not included in this version. This is a desktop app; mobile platforms and console save transfer are outside this release.

The editor performs two source-content checks during Save, but does not lock the game out of writing the file. Keep the game closed to avoid a concurrent write after the final check. Files remain local; the editor has no telemetry or direct network requests. Play hands off to your installed Steam client, which manages its own network activity.

## Validation

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
# Codec and filesystem tests without desktop dependencies:
cargo test --locked --no-default-features
```

Tests cover independent zlib-produced `.jkr` input, round trips, sparse/mixed keys, escaped/binary strings, numeric precision, malformed and executable input, decompression limits, exact backups, conflicts, copy collision protection, revert, discovery, and Unix file permissions. See `VALIDATION.md` for this delivery's actual results.

## Project layout

| File | Purpose |
| --- | --- |
| `src/main.rs` | Native desktop interface and editing flow |
| `src/jokers.rs` | Owned-Joker discovery, edition bonuses, and atomic Negative slot updates |
| `src/edits.rs` | Persistent field drafts, validation, and committing complete entries |
| `src/launch.rs` | Asynchronous Steam launch request and error handling |
| `src/ui_tests.rs` | Full application-loop input and launch-flow regression tests |
| `src/lua.rs` | Restricted Lua parser and serializer |
| `src/codec.rs` | Raw DEFLATE compression and bounded decoding |
| `src/document.rs` | Document state, backups, conflict checks, atomic replacement |
| `src/discovery.rs` | Save directory and Steam library discovery |
| `src/fields.rs` | Friendly labels for known save fields |
| `tests/` | Codec and filesystem integration tests; synthetic fixture |
| `.github/workflows/build.yml` | Platform builds and downloadable artifacts |

Licensed under MPL-2.0. See `NOTICE.md` for upstream credit. This project is unofficial and is not affiliated with LocalThunk or Playstack.
