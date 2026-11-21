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
