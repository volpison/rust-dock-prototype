# Architecture

Two layers, kept apart:

- **Layout rules** (`crates/dock/src/layout.rs`, in the `dock` crate, pure): which panels are docked and which are detached, and where each detached window sits. Data in, data out, no dependencies. Tests in `crates/dock/tests/layout.rs`.
- **Window layer** (the egui program, `crates/app-egui`: `src/main.rs`, `src/app.rs`, `src/panels.rs`): egui + eframe + egui_dock. It reads the layout, draws it, and reports what the user did. It holds no rules.

## Crates
A Cargo workspace: `crates/dock` (the pure library, no dependencies), `crates/app-egui` (the egui program, which depends on `dock`), and `crates/app-winit` (the same panels on winit + softbuffer + tiny-skia, started in [stage 2](stage2-plan.md); run with `cargo run -p rust-dock-winit`). `spikes/winit` is a separate crate outside the workspace, with its own `Cargo.lock`. More in [the stage 2 plan](stage2-plan.md).

## Files
| File | Reason to change |
|---|---|
| `crates/dock/src/layout.rs` | the rules for where panels live |
| `crates/dock/src/geometry.rs` | screen arithmetic for dropping a tab: outside the window? where on screen? |
| `crates/dock/src/drag.rs` | what letting go of a dragged panel does: detach where, dock back or nothing |
| `crates/app-egui/src/panels.rs` | which panels exist, their names and content |
| `crates/app-egui/src/app.rs` | how the layout is drawn: dock area, detached windows, menus |
| `crates/app-egui/src/main.rs` | starting the app |
| `crates/app-winit/src/ui.rs` | where things are in a window (pure: rows, buttons, hit-testing) |
| `crates/app-winit/src/render.rs` | drawing them with tiny-skia |
| `crates/app-winit/src/win.rs` | one OS window: softbuffer surface, canvas, pixel hand-over |
| `crates/app-winit/src/panels.rs` | the panels' ids, names and colours |
| `crates/app-winit/src/main.rs` | the event loop: windows, clicks, drags, detach and dock back |

## Rule the layout keeps
Every panel is in exactly one place: docked or detached. `detach`, `redock` and `set_position` are the only ways to change that, and they reject unknown panels and double moves.

## How the window layer keeps to it
- `Layout` is the source of truth. `egui_dock`'s `DockState` only remembers how the docked panels are arranged (tabs and splits).
- Detach: `Layout::detach`, then the tab is removed from the `DockState`.
- Each detached panel is drawn in its own egui immediate viewport (`show_viewport_immediate`), which is a real OS window. Immediate viewports run inside the main loop, so every window shares the same state without locks.
- Drag-out: the window layer remembers the tab under the pointer when the button goes down. When the button is released, it asks `geometry::is_outside_window` whether the pointer is outside the main window (with a small margin). If so it detaches that panel, placing the new window at the cursor with `geometry::window_origin_under_cursor`. The pointer keeps reporting positions outside the window during the drag.
- Drag back: each detached window has a strip that can be dragged. When it is released, the window layer converts the pointer to a screen position (using the detached window's own position) and asks `geometry::is_over_window` whether it is over the main window. If so the panel is re-docked.
- Re-dock: `Layout::redock`, then the tab is pushed back into the `DockState`. Closing the detached window does this.
- Tabs cannot be closed (`is_closeable` is false) and cannot become in-window floating panels (`allowed_in_windows` is false), because either would put a panel somewhere the layout does not know about.

## The winit program
Same rules, no egui. One event loop; the main window shows one coloured block per docked panel with a "detach" button (arrow pointing right), and each detached panel is its own OS window with a "dock back" button (arrow pointing left). `Layout` is still the source of truth: a click calls `Layout::detach` or `Layout::redock` and the windows follow. Closing a detached window re-docks its panel; closing the main window quits. There is no text (stage 5), so panels are told apart by colour and by the window title. Windows are placed by the OS; drag-out and drag-back, scaling changes and partial redraws come in the next pull requests of the stage 2 plan. `ui.rs` is pure so what is drawn and what is clickable come from the same numbers, and it is unit-tested. Every rectangle is snapped to whole pixels, so edges stay crisp at scales like 125% and 150%. Sizes are worked out from the window's size and scale factor on every frame, so the window follows a scale change while it runs; when the scale changes, the saved pointer position (in the old pixels) is forgotten.

## Pinned versions
`eframe` 0.37.0 and `egui_dock` 0.22.0 (which needs `egui` 0.37) for the egui program; `winit` 0.30.13, `softbuffer` 0.4.8 and `tiny-skia` 0.12.0 for the winit program. Both change their APIs often; check the signatures in `docs/technical/decisions.md` before upgrading.
