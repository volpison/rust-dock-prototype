# Project rules for Claude

House rules: the `project-guidelines` skill (full text in https://github.com/volpison/project-template).

## This project
- Pure Rust only. No Tauri, no GTK, no Qt, no web UI.
- Real separate OS windows for detached panels (egui viewports), not floating areas inside one window.
- The `dock` crate (`crates/dock`: `src/lib.rs`, `src/layout.rs`, `src/geometry.rs`) stays dependency-free. The window layer is the egui program (`crates/app-egui`: `src/main.rs`, `src/app.rs`, `src/panels.rs`).
- The `toolkit` crate (`crates/toolkit`: `src/input.rs`, `src/widgets.rs`) is pure logic for custom-drawn controls and stays dependency-free too: no drawing, windows, disk or network.
- Keep `crates/dock/src/layout.rs` pure: no UI, disk or network. The UI layer calls it; it contains no rules.
- Never write to `main`. Work on `claude/<topic>`, one topic per PR.
- Before pushing: run `cargo fmt --all --check`, `cargo test`, `node --test "scripts/*.test.mjs"` and `node scripts/check-docs.mjs`, and read your own diff. Say what was and was not built or run.
- Record each decision in `docs/technical/decisions.md`.
- No e-mail addresses, tokens or local paths in files, history or docs.
