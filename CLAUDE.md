# Project rules for Claude

House rules: the `project-guidelines` skill (full text in https://github.com/volpison/project-template).

## This project
- Pure Rust only. No Tauri, no GTK, no Qt, no web UI.
- Real separate OS windows for detached panels (egui viewports), not floating areas inside one window.
- The `dock` library (`src/lib.rs`, `src/layout.rs`) stays dependency-free. The window layer is the binary (`src/main.rs`, `src/app.rs`, `src/panels.rs`).
- Keep `src/layout.rs` pure: no UI, disk or network. The UI layer calls it; it contains no rules.
- Never write to `main`. Work on `claude/<topic>`, one topic per PR.
- Before pushing: run `cargo test`, `node --test "scripts/*.test.mjs"` and `node scripts/check-docs.mjs`, and read your own diff. Say what was and was not built or run.
- Record each decision in `docs/technical/decisions.md`.
- No e-mail addresses, tokens or local paths in files, history or docs.
