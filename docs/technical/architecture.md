# Architecture

Two layers, kept apart:

- **Layout rules** (`src/layout.rs`, in the `dock` library, pure): which panels are docked and which are detached, and where each detached window sits. Data in, data out, no dependencies. Tests in `tests/layout.rs`.
- **Window layer** (the binary: `src/main.rs`, `src/app.rs`, `src/panels.rs`): egui + eframe + egui_dock. It reads the layout, draws it, and reports what the user did. It holds no rules.

## Files
| File | Reason to change |
|---|---|
| `src/layout.rs` | the rules for where panels live |
| `src/geometry.rs` | screen arithmetic for dropping a tab: outside the window? where on screen? |
| `src/panels.rs` | which panels exist, their names and content |
| `src/app.rs` | how the layout is drawn: dock area, detached windows, menus |
| `src/main.rs` | starting the app |

## Rule the layout keeps
Every panel is in exactly one place: docked or detached. `detach`, `redock` and `set_position` are the only ways to change that, and they reject unknown panels and double moves.

## How the window layer keeps to it
- `Layout` is the source of truth. `egui_dock`'s `DockState` only remembers how the docked panels are arranged (tabs and splits).
- Detach: `Layout::detach`, then the tab is removed from the `DockState`.
- Each detached panel is drawn in its own egui immediate viewport (`show_viewport_immediate`), which is a real OS window. Immediate viewports run inside the main loop, so every window shares the same state without locks.
- Drag-out: the window layer remembers the tab under the pointer when the button goes down. When the button is released, it asks `geometry::is_outside_window` whether the pointer is outside the main window (with a small margin). If so it detaches that panel, placing the new window at the cursor with `geometry::window_origin_under_cursor`. The pointer keeps reporting positions outside the window during the drag.
- Re-dock: `Layout::redock`, then the tab is pushed back into the `DockState`. Closing the detached window does this.
- Tabs cannot be closed (`is_closeable` is false) and cannot become in-window floating panels (`allowed_in_windows` is false), because either would put a panel somewhere the layout does not know about.

## Pinned versions
`eframe` 0.37.0 and `egui_dock` 0.22.0 (which needs `egui` 0.37). Both change their APIs often; check the signatures in `docs/technical/decisions.md` before upgrading.
