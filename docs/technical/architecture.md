# Architecture

Two layers, kept apart:

- **Layout rules** (`src/layout.rs`, pure): which panels are docked and which are detached, and where each detached window sits. Data in, data out. Tests live in `tests/layout.rs`.
- **Window layer** (not written yet): egui + eframe + egui_dock. It reads the layout, draws it, and reports what the user did ("detach B", "window closed"). It holds no rules.

## Rule the layout keeps
Every panel is in exactly one place: docked or detached. `detach`, `redock` and `set_position` are the only ways to change that, and they reject unknown panels and double moves.

## Planned window layer
- The main window shows the docked panels with `egui_dock`.
- Each detached panel gets its own viewport (`show_viewport_immediate`), so it is a real OS window that can go to another monitor.
- Closing a detached window re-docks its panel.
