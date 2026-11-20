# Project rules for Claude

House rules: the `project-guidelines` skill (full text in https://github.com/volpison/project-template).

## This project
- Pure Rust only. No Tauri, no GTK, no Qt, no web UI.
- Real separate OS windows for detached panels (egui viewports), not floating areas inside one window.
- Keep `src/layout.rs` pure: no UI, disk or network. The UI layer calls it; it contains no rules.
- Never write to `main`. Work on `claude/<topic>`, one topic per PR.
- Run `cargo test` before pushing. Say what was and was not built or run.
- Record each decision in `docs/technical/decisions.md`.
- No e-mail addresses, tokens or local paths in files, history or docs.
