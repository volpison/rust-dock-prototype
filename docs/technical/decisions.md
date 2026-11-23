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

## 2026-11-21: Spike versions and how pixels reach the screen
- **What:** `spikes/winit` pins winit 0.30.13, softbuffer 0.4.8 and tiny-skia 0.12.0. tiny-skia draws into a `Pixmap` (premultiplied RGBA); each frame is copied into softbuffer's `u32` buffer as `0xAARRGGBB`. A new window per panel, one event loop, one softbuffer surface per window.
- **Why:** the versions are the ones a fresh `cargo add` resolved on the maintainer's machine, and the code was written from their sources. Premultiplied `0xAARRGGBB` is what the earlier skinned-window experiment used for see-through windows; softbuffer 0.4.8 has no alpha setting, so whether it works on each platform is a thing the spike must check.
- **Left out:** GPU drawing (vello, wgpu), a platform-specific transparent-window path (for example Windows layered windows) until the spike shows it is needed, and text.

## 2026-11-22: Gate A passed on Windows; continue to stage 2
- **What:** the spike met every check on Windows 10 at 100% scaling: see-through corners, no resize flicker, pointer events outside the window during a drag, smooth moving by a custom strip, and a release-build redraw of about 1 ms for a 320x420 panel (plan threshold: 8 ms). Details in [`spikes/winit/RESULTS.md`](../../spikes/winit/RESULTS.md). Next is stage 2 (windows and input, display scaling), starting by moving the pure code into a Cargo workspace.
- **Why:** the questions Gate A asked are answered with real numbers, and the cost of going on is low: a "no" at Gate B still leaves the egui program intact.
- **Left out:** scaling above 100% (untested; stage 2), macOS and Linux (untested; stage 6); debug-build speed (unusable without optimising dependencies, noted in the results); text.
- **Risk noted:** see-through corners work on Windows but are outside softbuffer's documented pixel format (0RGB), so they are not guaranteed on other platforms or versions.

## 2026-11-22: Split into a Cargo workspace before stage 2
- **What:** `crates/dock` (the pure library), `crates/app-egui` (today's program, same program name) and `crates/app-winit` (new). The spike stays in `spikes/winit`, excluded from the workspace, until `app-winit` absorbs it. Plan: [stage 2](stage2-plan.md).
- **Why:** both programs need `layout.rs` and `geometry.rs`, and a package that contains an egui program cannot be a dependency of a program without egui. One workspace keeps one copy of the tested rules and one set of CI checks, and `cargo run` still starts the egui program.
- **Left out:** copying the pure code into a second repository (two copies to keep in step), and a long-lived branch. A new repository is still possible later if the new stack outgrows this one.

## 2026-11-22: How the workspace is arranged
- **What:** a virtual workspace at the root with `members` and `default-members` set to `crates/dock` and `crates/app-egui`, and `exclude = ["spikes/winit"]`. The egui program keeps the package name `rust-dock-prototype`, so the program and `cargo run` are unchanged. Formatting is checked with `cargo fmt --all --check`. CI checks that `dock` has no dependencies (`cargo tree -p dock`).
- **Why:** `default-members` keeps `cargo run` and `cargo test` working from the root. The spike stays out of the workspace because it has its own `Cargo.lock` and pulls in winit; cargo also refuses a crate inside a workspace folder unless it is a member or excluded. Checking `dock` in CI turns "stays dependency-free" from a rule in `CLAUDE.md` into something that fails.
- **Left out:** moving the spike into the workspace (it becomes `app-winit` in a later pull request), and `opt-level` for dependencies in the dev profile, which comes with `app-winit` because this pull request changes no build settings.

## 2026-11-22: The app-winit skeleton: buttons, no text, one pure layout file
- **What:** `crates/app-winit` (package `rust-dock-winit`, a workspace member but not a default one) draws a block per docked panel with a "detach" button, and each detached panel as its own window with a "dock back" button. The arrows on the buttons say which. Where things are lives in `ui.rs`, which is pure and unit-tested; `render.rs` draws from it and `main.rs` clicks against it. The pixel format hand-over is one small function (`to_softbuffer` in `win.rs`). Dependencies are optimised in the dev profile.
- **Why:** it makes the program usable before stage 3 has real controls, and it uses `Layout` the same way the egui program does, so the two behave alike. Sharing one set of numbers between drawing and hit-testing keeps what you see and what you can click in step. Debug builds of tiny-skia are about 25 times slower (see the spike results), so optimising dependencies is needed to try the program with `cargo run`. One function for the pixel format is the first step of the transparency safety in the plan.
- **Left out:** text (stage 5), drag-out and drag-back (pull request 5), scale-factor changes while running (pull request 4), partial redraws (pull request 6), saving positions, and removing `spikes/winit` (its code is copied here, and it stays until the winit program has settled).

## 2026-11-22: Display scaling in app-winit: physical pixels, snapped rectangles
- **What:** everything in `app-winit` is in physical pixels, and sizes written in points are multiplied by the window's scale factor each frame. `ui.rs` snaps every rectangle to whole pixels (edges rounded, width and height taken from the rounded edges). The scale-change event clears the saved pointer position and redraws. Any margin given to `geometry` is multiplied by the scale (used from drag-out on).
- **Why:** the skeleton already read the size and scale every frame, so a scale change while running only needs a redraw. At 125% and 150% unsnapped rectangles fall between pixels and draw blurry, and a click could be tested against edges that were not drawn; snapping fixes both in one pure, unit-tested place. Winit's screen positions are physical pixels, so one unit avoids converting between points and pixels in the drag code. The tests cover the scales 100%, 125%, 150%, 175%, 200% and 250%.
- **Left out:** moving a window between monitors with different scales (cannot be tried with one monitor; the same code path runs, because the event is the same), and logical sizes for new windows beyond the fixed starting sizes, which winit converts.

## 2026-11-22: Drag-out and drag-back in app-winit: pure drag tracker in `dock`
- **What:** `crates/dock/src/drag.rs` holds a `DragTracker`: pressing on a panel's block remembers it, and on release it returns `Detach { panel, position }` (released outside the window by more than a margin), `Redock { panel }` (a detached panel released over the main window) or nothing. `app-winit` only reports pointer positions and window rectangles and carries out the answer. The margin (8 points) and title offset (15 points) are the egui program's values, multiplied by the scale. While a drag is on, the pointer-left event is ignored so the last position, which may lie outside the window, is where the drag ends.
- **Why:** the rule is the same for both programs and needs no window, so it is tested here with plain numbers (14 tests). The spike showed winit keeps reporting the pointer outside the window while a button is held, which is what makes both drags possible.
- **Feedback while dragging:** the dragged block is drawn half way to the background colour and the pointer becomes a grabbing hand (the first test showed that with no feedback the drag felt invisible). A ghost that follows the pointer is a separate pull request.
- **Left out:** a drag preview that follows the pointer (next pull request); dropping onto a particular spot in the main window (stage 4 drop zones); a Wayland drag-back (windows cannot be located there, so only the buttons work, and a detached window is placed by the system); redocking when the main window is minimised or covered.

## 2026-11-23: A ghost window follows the pointer during a drag
- **What:** while a panel is dragged and the pointer is outside the window it started in (by more than the drag margin), `app-winit` shows a 150 x 90 point borderless, always-on-top window in the panel's colour, centred on the pointer. It is created on the first move outside, moved on each move, and removed when the pointer is back inside or the button is let go. It ignores the pointer (`set_cursor_hittest(false)`), is not active, and on Windows is kept off the taskbar. It is drawn like the other windows (softbuffer + tiny-skia).
- **Why:** the first test of drag-out showed that dimming the block was not enough: nothing moved with the pointer, so the drag felt invisible. A real window is the only way to draw outside our own windows in a pure-Rust program. Ignoring the pointer keeps pointer events and the release with the window the drag started in, which is what the drop logic relies on.
- **Left out:** a ghost on Wayland (needs the window position, which is not available there), a see-through or shadowed ghost (stage 2 pull request 7 decides how transparency is done), and a ghost over our own windows' drop targets (stage 4 drop zones).

## 2026-11-23: Redraw only what changed in app-winit
- **What:** moving the pointer redraws a window only when it changes which button is highlighted (before and after are compared with the same hit-test the drawing uses). In that case the window is told which part changed: the old and the new button, joined. `Win` keeps a "whole window" flag (first frame, resize, scale change, layout change, drag start and end) and an optional changed rectangle; `present` hands the rectangle (grown by one pixel, cut to the window) to softbuffer's `present_with_damage`, and the whole window otherwise. `DOCK_TRACE=1` prints each frame's time and size. The rectangle arithmetic is in `ui.rs` and tested.
- **Why:** before this, every pointer move redrew and showed the whole window, even with nothing changing. Skipping those moves is the big saving; the damage rectangle makes the rest cheaper on systems that only update what they are told. The whole pixmap is still copied into softbuffer's buffer each time, because a buffer can hold an older frame (see its buffer age), so a partial copy could leave stale pixels; the copy is a small cost next to showing.
- **Known risk:** the system can ask for a repaint (a window uncovered, for example) and winit joins that with our own request. If that happens in the same frame as a button change, only the changed part would be shown. It is a rare coincidence, and the next full frame fixes it; if it shows up, `Win` can be told to show the whole window on every OS-requested repaint.
- **Left out:** drawing only the changed part into the pixmap (tiny-skia clipping; the spike measured a whole-window draw at about 1 ms, so it is not the cost), damage for the drag dimming and the ghost (their frames are rare and small), and buffer-age-based partial copies.

## 2026-11-23: Transparency: allowed only where it has been seen to work, with an opaque fallback
- **What:** `crates/app-winit/src/pixel.rs` (pure, 5 tests) holds the pixel hand-over (moved out of `win.rs`) and the rule for see-through windows. A window asks for it with `with_transparent(true)`; `Win::new` grants it only on Windows, and never when `DOCK_OPAQUE` is set. Otherwise the window is opaque, and drawing code asks `Win::see_through()` to choose between transparent pixels and a colour. The drag ghost is the first user: rounded corners when see-through, a plain rectangle when not. `DOCK_TRACE=1` trace lines also show the window size.
- **Why:** softbuffer 0.4.8 documents 0RGB and has no alpha setting. The spike got real see-through corners on Windows 10 by putting alpha in the top byte, but nothing promises it on other platforms or library versions. Making the opaque path the default everywhere else means an untested platform shows a square ghost instead of a black or garbled one, and `DOCK_OPAQUE=1` lets the fallback be tried on Windows.
- **Decided about a real layered-window path (Windows `WS_EX_LAYERED` or similar):** not needed now. Nothing we draw depends on see-through to be usable, the ghost is the only see-through window, and it works on Windows as it is. Revisit if a feature needs see-through on another platform, or if a new softbuffer or winit changes the behaviour; the code that would change is `pixel.rs` and the creation in `Win::new`.
- **Left out:** see-through main or panel windows (stage 3 and later), macOS and Linux see-through (untested; opaque there), a startup self-check that reads a pixel back (no portable way to ask the compositor).

## 2026-11-23: Stage 2 is finished; stage 3 is a mini toolkit in its own pure crate, ending at Gate B
- **What:** stage 2 (windows and input) is complete: the winit program detaches and docks panels by buttons and by dragging, shows a ghost while dragging, follows display scaling, redraws only what changed, and keeps see-through windows to Windows with an opaque fallback. Stage 3 is proposed in [its plan](stage3-plan.md): a dependency-free `crates/toolkit` (pointer state, button, toggle, slider, knob, layout), drawing in `app-winit`, one realistic panel built from it, and a Gate B review that compares it with the egui version. The roadmap marks stage 2 done and stage 3 next.
- **Why:** Gate B is about whether building panels from custom-drawn controls is pleasant, and the only honest test is to build one. Keeping the logic in a pure crate means it is tested without a window, reused when this is folded into the larger project, and checked in CI like `dock`. The egui program stays intact, so a "no" at Gate B costs nothing already built.
- **Left out:** text (stage 5), docking (stage 4), scrolling, keyboard focus, tooltips and accessibility (they are listed at Gate B and built only if the project goes on).
- **Confirmed in the review:** the pure `crates/toolkit`; a 7-segment number display drawn from shapes (its own pull request); skins from images after Gate B, not in stage 3.
