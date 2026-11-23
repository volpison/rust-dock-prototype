# Roadmap

## Done
- Panel layout rules (`crates/dock/src/layout.rs`): each panel is docked or detached, never both, never lost.
- Window layer (`crates/app-egui/src/app.rs`, `crates/app-egui/src/panels.rs`): egui + eframe + egui_dock. Right-click a tab, "Detach to window", and the panel becomes a real OS window; "Dock back" or closing that window puts it back. Dragging a tab out of the window and letting go outside it does the same, placing the new window at the cursor. A detached panel can be dragged back over the main window by its strip. *Compiles and runs on Windows: docking, splitting, detaching and docking back confirmed.*

## Next
1. Stage 2 of the move to winit + softbuffer + tiny-skia (windows and input): [plan](docs/technical/stage2-plan.md), starting with a Cargo workspace split. The spike is done and Gate A passed on Windows: [results](spikes/winit/RESULTS.md). Route: [roadmap](docs/technical/roadmap-winit.svg). The `app-winit` skeleton (windows, detach and dock back by buttons) is merged; display scaling is merged; drag-out and drag-back (with a ghost) are merged; redrawing only what changed and transparency safety are merged: stage 2 is finished. Stage 3 (a mini toolkit, ending at Gate B) is [proposed](docs/technical/stage3-plan.md).
2. Remember window positions between runs (needs a versioned save format, see the guidelines).

## What it is not
- Not a finished app. It is a prototype meant to be moved into a larger project.
- Not Tauri or a web UI, and not GTK or Qt bindings. Pure Rust only.
- Not a general docking library for others to use (yet).
- No promise that windows can be placed exactly on Wayland, which does not allow it.
