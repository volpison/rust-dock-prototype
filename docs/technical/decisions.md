# Decisions

## 2026-11-20: Pure Rust with egui, eframe and egui_dock
- **What:** build the window layer on egui + eframe, with egui_dock for docking inside the main window and egui viewports for detached windows.
- **Why:** it is pure Rust, gives real OS windows, and is the shortest path to the goal.
- **Left out:** Tauri (web UI), GTK and Qt bindings (not pure Rust, as the project requires). iced was considered; docking there is mostly do-it-yourself.

## 2026-11-20: Layout rules first, with no dependencies
- **What:** `src/layout.rs` holds the docked/detached rules and depends on nothing.
- **Why:** it can be tested on its own, and it moves into the larger project unchanged whatever UI it ends up with.
- **Left out:** putting the rules inside the egui code.
