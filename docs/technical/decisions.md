# Decisions

## 2026-11-20: Pure Rust with egui, eframe and egui_dock
- **What:** build the window layer on egui + eframe, with egui_dock for docking inside the main window and egui viewports for detached windows.
- **Why:** it is pure Rust, gives real OS windows, and is the shortest path to the goal.
- **Left out:** Tauri (web UI), GTK and Qt bindings (not pure Rust, as the project requires). iced was considered; docking there is mostly do-it-yourself.

## 2026-11-20: Layout rules first, with no dependencies
- **What:** `src/layout.rs` holds the docked/detached rules and depends on nothing.
- **Why:** it can be tested on its own, and it moves into the larger project unchanged whatever UI it ends up with.
- **Left out:** putting the rules inside the egui code.

## 2026-11-20: Pin eframe 0.37.0 and egui_dock 0.22.0, written against their sources
- **What:** the window layer uses `eframe::App::ui` (not `update`), viewport closures that receive a `Ui`, `DockArea::show_inside`, and `TabViewer::context_menu` with a `NodePath`.
- **Why:** these differ from older versions, and the first sketch in the project's history used the old forms. The signatures were read from the crate sources on the developer's machine.
- **Left out:** supporting several egui versions at once.

## 2026-11-20: Tabs cannot be closed or floated inside the main window
- **What:** `is_closeable` and `allowed_in_windows` return false.
- **Why:** a closed tab would lose its panel, and a floating in-window panel would be in a third place the layout does not track.
- **Left out:** letting egui_dock manage floating windows itself.

## 2026-11-20: Detach from a tab menu first, drag-out later
- **What:** detaching uses the tab's context menu. Dragging a tab out of the window is on the roadmap.
- **Why:** winit gives little pointer information once a drag leaves the window, so drag-out needs its own design.
- **Left out:** drag-out in this version.

## 2026-11-20: CI on Windows, with the docs check copied from the project template
- **What:** `.github/workflows/ci.yml` has a docs job (Ubuntu) running `scripts/check-docs.mjs` and its tests, and a Rust job (Windows) running `cargo test --locked`. The script, its tests and the pull request template come unchanged from the `project-template` repository.
- **Why:** the conventions say CI is the referee and tests run on the platform users really have. The project is built and used on Windows.
- **Left out:** macOS and Linux jobs (not tried yet; the window libraries may need system packages there), `cargo fmt` and `cargo clippy` (the code is not formatted yet; formatting goes in its own pull request), the version script and release workflow (no release yet).

## 2026-11-21: Check formatting in CI
- **What:** the Windows job runs `cargo fmt --check` before the tests. The code was formatted in its own pull request first, so the check starts green.
- **Why:** one agreed layout keeps diffs about changes, not spacing.
- **Left out:** `cargo clippy`. Lints can only be checked once the whole program is compiled, and the first run may show warnings to review; it goes in its own pull request.

## 2026-11-21: Drag-out detaches on release; live tear-off later
- **What:** dragging a tab out of the window will detach the panel when the button is released outside the window, placed at the cursor. Details in [the plan](drag-out-plan.md).
- **Why:** it needs only the pointer position, which winit usually keeps sending during a drag. A window that follows the cursor needs the OS to hand over a drag begun in another window, which is likely to fail on Windows.
- **Left out:** live tear-off, and dropping a detached window onto a chosen spot in the dock.

## 2026-11-21: The dragged tab is the tab under the pointer when the button goes down
- **What:** `Viewer::on_tab_button` records the hovered tab each frame; the app keeps the one hovered at the moment the primary button is pressed, and uses it when the button is released outside the window.
- **Why:** an experiment on Windows showed that `egui_dock` never reports a tab drag through `on_tab_button` (`drag_started` stayed false), while the pointer position and the release do arrive outside the window.
- **Left out:** asking `egui_dock` which tab is dragged (it does not say), and a live window that follows the cursor.

## 2026-11-21: Drag a detached panel back by a strip inside its window
- **What:** each detached window has a draggable strip. On release, if the pointer is over the main window, the panel is re-docked. The whole main window is the target; the panel goes to the focused area.
- **Why:** the OS moves a window by its title bar and gives the app little pointer information meanwhile, while a drag started inside the window keeps reporting positions outside it (seen in the Windows experiment). The same screen-position arithmetic as drag-out then decides the drop.
- **Left out:** dropping onto a particular split or tab group, a highlight while hovering over the main window, and moving the window along with the pointer during the drag (moving it by the title bar stays as before). The strip text is not selectable, so dragging it does not select the words.

## 2026-11-21: Move to winit + softbuffer + tiny-skia in stages, with two go/no-go gates
- **What:** the route is in [the roadmap](roadmap-winit.svg): a spike, windows and input, a mini toolkit, docking, text, polish. After the spike (Gate A) and after the mini toolkit (Gate B) we decide whether to go on or stay on egui.
- **Why:** a custom stack means writing a small toolkit, and a gate after each risky stage keeps the cost of a "no" low. The panels are mostly custom-drawn controls, so skipping a general toolkit is plausible; text fields are the expensive part and come last, and only if viable.
- **Left out:** switching all at once, and starting with text fields. The egui program stays as it is until a gate says otherwise.

## 2026-11-21: The spike lives in this repository, as a separate crate
- **What:** `spikes/winit/`, with its own `Cargo.toml` and `Cargo.lock`, on `main` through normal pull requests. Plan: [the spike plan](winit-spike-plan.md).
- **Why:** the spike is small and may end at Gate A, so it should be cheap to delete. A separate crate keeps winit, softbuffer and tiny-skia out of the egui build and its CI. A long-lived branch would drift from `main` (dependencies, CI, docs) and fight the one-topic-per-PR routine; a new repository would copy the pure code too early.
- **Left out:** a new repository and a long-lived branch for now. Reusing the `dock` library in the spike: depending on the root package would pull in eframe, so the library moves into a Cargo workspace first (stage 2). If the new stack outgrows this repository, it can be split off then.
