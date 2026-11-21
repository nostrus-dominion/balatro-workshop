# Validation

## 0.3.1

- `cargo fmt --check` and `cargo clippy --locked --all-targets -- -D warnings` passed on Linux x86-64.
- `cargo test --locked` passed all 55 tests. The core-only suite passed all 38 tests.
- Regression tests cover edition changes, unchanged cards outside the selection, and all-or-nothing results when a selection includes unsupported or missing cards.

The automated save-and-restore test uses a synthetic run. It edits a playing card to Lucky, Polychrome, and Red Seal, applies a batch edition, switches an owned Joker to Negative, saves and reopens the run, then restores the byte-for-byte original backup. The in-game check below is pending a playable save and an installed copy of Balatro.

### In-game save and restore check

1. In a spare base-game profile, create a disposable run, buy a Joker, and reach a hand or shop where you can quit and resume. Record the game version, platform, and original run state. Close Balatro and copy the whole test profile somewhere separate.
2. Open that profile's `save.jkr` in Workshop. Set one card to Lucky, Polychrome, and Red Seal. Change the edition of two other selected cards together. Change the owned Joker to Negative. Leave a comparable card unselected. Use **Save + backup** and keep the backup path.
3. Launch Balatro, continue the run, and verify the edited cards and Joker, the extra Joker slot, and the unselected card. Play at least one hand and verify the edited playing-card edition scores when its card scores. Record the outcome, including any load error or Steam Cloud prompt.
4. Close Balatro, keep a copy of the edited save, and replace `save.jkr` with the Workshop backup. Confirm the restored bytes match the original test-profile copy. Resume again and verify the original cards, Joker edition, and slot count.

## 0.2.0 — 2026-10-31

- `cargo fmt --check` and `cargo clippy --locked --all-targets -- -D warnings`: passed on Linux x86-64.
- `cargo test --locked`: **47 tests passed** (16 UI/launcher, 24 existing core, 7 Joker integration tests).
- `cargo test --locked --no-default-features`: **31 integration tests passed**.
- A synthetic six-Joker fixture covers duplicate names, all five editions, owned-vs-shop/booster discovery, and unrelated saved data.
- All 25 edition transitions are checked, including complete bonus fields, replacing the previous edition, returning to Normal, and changing the saved slot limit exactly once when entering or leaving Negative.
- Invalid or missing slot limits, conflicting edition flags, unknown edition fields, and recognized modded slot configurations are rejected without partial mutation. Sparse numeric keys and distinct string keys are preserved.
- Full application-loop tests click the Jokers tab and dropdowns, change Foil to Polychrome to Negative, inspect Advanced text, save/reopen, verify the exact original backup, revert, switch files, and block edits while raw text is unapplied.
- Native Linux debug application exercised under Xvfb/X11 with software OpenGL: visually inspected the six-Joker list and dropdown; changed Foil to Polychrome to Negative; confirmed no disk write before Save; saved; independently decompressed the result and verified two Negative Jokers, slot limit 7, the other Joker's Eternal flag, and a byte-exact original backup.

No dependencies changed. The fixture is synthetic and is not a playable run. Real player saves, in-game loading, Steamodded/custom edition compatibility, and native Wayland interaction remain unverified. The edition dropdown intentionally supports the base-game schema and leaves recognized custom schemas intact. See the GitHub Actions run for this commit for native release-build, test, and packaging results on Linux, Windows, and both macOS architectures.

## 0.1.2 — 2026-10-26

- `cargo fmt --check`: passed.
- `cargo clippy --locked --all-targets -- -D warnings`: passed.
- `cargo test --locked`: **13 UI/launcher tests and 24 core integration tests passed**. UI tests now run the same `Workshop::show` application loop as the desktop window, including keyboard shortcuts, dirty state, dialogs, and launch handling.
- `cargo build --locked --release`: passed on Linux x86-64 (Ubuntu 24.04, Rust 1.99.1).
- `cargo check --locked --target x86_64-pc-windows-msvc`, `aarch64-apple-darwin`, and `x86_64-apple-darwin`: passed for the updated source, including each platform's Steam launch command.
- Input tests cover typing six digits without committing or saving between digits; Save including the active field; Enter and focus-loss commits; incomplete negatives/decimals/exponents; cursor editing; switching to Advanced text; clearing drafts on Revert and file changes; and invalid entries preventing Save & play.
- Native release executable tested in Xvfb with X11 and software OpenGL: typed `987654` one digit every 250 ms with no intervening save, and confirmed the disk save remained unchanged until Save & play. Then verified the complete value and a byte-exact backup existed before the launch handler was called.
- Native Play saved version check confirmed the on-disk value stayed `125` during launch and that the unsaved `987654` remained available to save afterward.
- The launch tests used a local stand-in for `xdg-open`, recorded its exact argument (`steam://rungameid/2379780`), and inspected the save at the moment it was invoked. No real Steam installation or game was available. Missing handlers and nonzero exits are covered by launcher tests.
- Play button, unsaved-change dialog, and resulting editor state were visually inspected.

The reported need to save between digits was not independently reproduced in 0.1.1 under X11 or the full application-loop tests. Version 0.1.2 replaces per-keystroke document mutation with persistent text drafts and independent field IDs, but confirmation on the affected KDE/Wayland desktop remains outstanding. A native Wayland test was attempted; the sandbox could not create the compositor socket, so that test did not run. No dependencies changed.

Windows/macOS native execution, real game launch, real player saves, and signing remain unverified. Cross-target compile checks are not native runtime tests.

## 0.1.1 — 2026-10-26

- `cargo fmt --check`: passed.
- `cargo clippy --locked --all-targets -- -D warnings`: passed.
- `cargo test --locked`: **4 UI regression tests and 24 core integration tests passed**.
- `cargo build --locked --release`: passed on Linux x86-64 (Ubuntu 24.04, Rust 1.99.1).
- Regression reproduced before the fix: typing `4`, changing the visible Quick edit rows, then typing `56` left the field at `4`. With stable field identities, the same test produces `456`.
- UI tests also cover separate keypresses across redraws, replacing selected text, clearing a number, typing `-12.5e-3` through incomplete intermediate states, cursor movement, backspace, text fields, and save/reopen of a multi-digit value.
- Native Linux release executable checked in Xvfb with X11 and software OpenGL: typed `987654` one digit every 250 ms, saved with Ctrl+S, independently decompressed the save with Python zlib, and verified the backup matched the original bytes. Cursor/backspace/insertion produced `987604`; clearing and entering `-12.5e-3` worked; editing Ante to `3456` preserved the Money field. The resulting UI was visually inspected.

The original Linux binary accepted ordinary multi-digit typing in the Xvfb test environment. This update fixes a demonstrated focus-loss path during layout changes; the user's exact desktop-only typing failure has not been independently reproduced. Native Wayland behavior and real player saves remain unverified. No dependencies changed. Windows/macOS binaries were not rebuilt or executed for this patch.

## 0.1.0 — 2026-10-26 (previous release)

### Completed

- `cargo fmt --check`: passed.
- `cargo clippy --locked --all-targets -- -D warnings`: passed.
- `cargo test --locked`: **24 integration tests passed**.
- `cargo test --locked --no-default-features`: **24 integration tests passed**.
- `cargo build --locked --release`: passed on Linux x86-64.
- `cargo check --locked --target x86_64-pc-windows-msvc`: passed.
- `cargo check --locked --target aarch64-apple-darwin`: passed.
- `cargo check --locked --target x86_64-apple-darwin`: passed.
- Linux release binary launched in Xvfb using software OpenGL; the Quick edit, Data tree, and Advanced text views rendered and were visually inspected.
- End-to-end UI check: opened a copy of the synthetic `.jkr` fixture, changed Money from 125 to 777 using the text field, saved with Ctrl+S, decompressed the resulting file independently using Python zlib, verified the changed value, and compared the automatic backup byte-for-byte with the original. Passed.
- The window exited cleanly after the UI check.

Build host: Ubuntu 24.04, Rust 1.99.1. Dependencies are recorded in Cargo.lock. The release profile disables LTO and uses one code-generation unit.

### Not verified in this environment

- Windows/macOS linking, packaging, native window behavior, and execution. Cross-target `cargo check` type-checks the target-specific code; it does not produce or run those executables. The provided GitHub Actions workflow supplies the native build environments but has not been run remotely.
- Native OS file picker interaction. Linux QA loaded a file through the command-line argument; file dialogs use rfd and require a working desktop portal on Linux.
- Compatibility with a real player save, a running Balatro installation, every game version, or arbitrary mods. Tests use a synthetic fixture based on the upstream editor's save format and field paths.
- Code signing/notarization, automated restore, mobile apps, and calculator URL export.
