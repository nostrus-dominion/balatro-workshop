# Attribution

Balatro Workshop is a new Rust desktop implementation inspired by:

- **balatro-save-loader**, by WilsontheWolf: https://github.com/WilsontheWolf/balatro-save-loader
- Source reference inspected: commit `2202fb942c5ba0832f51cf618181e42a8182d0af`.
- Original web editor: https://balatro.shorty.systems/
- Upstream license: Mozilla Public License 2.0 (MPL-2.0).

The save container format and quick-edit field paths were derived from the upstream project's `src/helpers/loading.js`, `static/js/saveUI.js`, `profileUI.js`, and `settingsUI.js`. The parser, serializer, native interface, discovery, backup handling, and tests are newly implemented. This project is also distributed under MPL-2.0; the complete license is included in LICENSE.

No Balatro game code, artwork, music, real player saves, or calculator compatibility assets are bundled. The test save was generated from the synthetic Lua fixture with Python's zlib using raw DEFLATE (`wbits=-15`).

The dependencies listed in Cargo.toml and Cargo.lock retain their respective licenses and notices. Source packages include the exact lockfile so these dependencies can be retrieved. Redistributors should retain the source and applicable notices when distributing binaries.

Documentation references: https://docs.rs/eframe/0.31.1/eframe/ and https://docs.rs/rfd/0.15.4/rfd/.
