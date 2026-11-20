# Roadmap

## Done
- Panel layout rules (`src/layout.rs`): each panel is docked or detached, never both, never lost.

## Next
1. Window layer: `egui` + `eframe` + `egui_dock`, with detached panels shown through egui viewports (real OS windows). Detach from a tab menu; closing the window re-docks the panel.
2. Drag a tab out of the window to detach it under the cursor.
3. Drag a detached panel back onto the dock.
4. Remember window positions between runs (needs a versioned save format, see the guidelines).

## What it is not
- Not a finished app. It is a prototype meant to be moved into a larger project.
- Not Tauri or a web UI, and not GTK or Qt bindings. Pure Rust only.
- Not a general docking library for others to use (yet).
- No promise that windows can be placed exactly on Wayland, which does not allow it.
