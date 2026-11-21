# Roadmap

## Done
- Panel layout rules (`src/layout.rs`): each panel is docked or detached, never both, never lost.
- Window layer (`src/app.rs`, `src/panels.rs`): egui + eframe + egui_dock. Right-click a tab, "Detach to window", and the panel becomes a real OS window; "Dock back" or closing that window puts it back. Dragging a tab out of the window and letting go outside it does the same, placing the new window at the cursor. *Compiles and runs on Windows: docking, splitting, detaching and docking back confirmed.*

## Next
1. Drag a detached panel back onto the main window (see [the plan](docs/technical/drag-out-plan.md), step 3).
2. Remember window positions between runs (needs a versioned save format, see the guidelines).

## What it is not
- Not a finished app. It is a prototype meant to be moved into a larger project.
- Not Tauri or a web UI, and not GTK or Qt bindings. Pure Rust only.
- Not a general docking library for others to use (yet).
- No promise that windows can be placed exactly on Wayland, which does not allow it.
