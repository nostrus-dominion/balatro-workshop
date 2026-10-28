# Changelog

## 0.1.2 — 2026-10-26

- Reworked Quick edit and Data tree text fields to retain an independent draft while typing. Complete values are accepted on Enter, focus loss, or Save; no intermediate disk save is needed.
- Validate all pending entries before saving, include the active field, preserve incomplete entries for correction, and clear old drafts on Revert or file changes.
- Use field identities independent of UI layout and saved/unsaved state.
- Added **Play Balatro** using the installed Steam client's launch handler on Linux, Windows, and macOS.
- Offer Save & play, Play saved version, or Cancel when edits are pending. Save failures prevent launch; choosing the saved version preserves edits.
- Run launch requests outside the UI thread and report handler failures.
- Show the version in the window title and exercise the full application loop in UI tests.

## 0.1.1 — 2026-10-26

- Give Quick edit and Data tree text fields stable identities so layout and visible-row changes do not discard keyboard focus or cursor state mid-entry.
- Keep the filter and Advanced text editor identities stable across redraws.
- Add UI regression coverage for character-by-character numeric input, incomplete negative/decimal/exponent values, text editing, layout changes, and saving the complete entered number.
- Display the application version from Cargo metadata.

## 0.1.0 — 2026-10-26

- Added native Rust/egui save editing for desktop platforms.
- Added run, profile, career statistics, and settings controls for existing fields.
- Added a typed table tree and an advanced data-only Lua editor with validation.
- Added bounded raw-DEFLATE decoding and a parser that does not execute Lua.
- Preserved numeric keys, sparse tables, unknown fields, string bytes, and numeric literals.
- Added automatic backups, disk-change detection, same-directory atomic replacement, and copy collision protection.
- Added native file dialogs, drag-and-drop, save-folder discovery, keyboard shortcuts, and unsaved-change prompts.
- Added synthetic compatibility fixtures, integration tests, and a multi-platform build workflow.

## Planned

- Validate against real, anonymized saves from supported game versions and common mods.
- Add a backup browser with preview and restore.
- Add undo/redo history and a before/after change list.
- Add dedicated joker/card editing after verifying game invariants.
- Evaluate calculator export compatibility separately.
- Add signed Windows packages and notarized macOS releases when signing credentials are available.
