# Tests

## Running the tests

```
cargo fmt --all --check             # formatting of every crate (run `cargo fmt --all` to fix it)
cargo test                          # the tests of crates/dock; also builds the window layer
cargo run                           # try the egui program by hand
cargo test -p rust-dock-winit       # the winit program's tests (it is not a default member)
cargo run -p rust-dock-winit        # try the winit program by hand
node --test "scripts/*.test.mjs"    # tests of the docs check
node scripts/check-docs.mjs         # links, heading links and the docs index
```

The Node commands need Node 20 or newer. They are run by CI, so you only need them to check the docs before pushing.

## What the tests cover

- **Toolkit** (`crates/toolkit`): `crates/toolkit/tests/input.rs` and `widgets.rs`. Hover follows the control under the pointer; a button held down is captured until it is let go, even outside the control or window; letting go over the same control is a click, elsewhere is not; other controls are not hovered while one is held; a second press is ignored; every call says whether a look changed, so the window redraws only then; buttons and toggles ignore clicks when disabled. `value.rs`, `slider.rs`, `knob.rs` and `double_click.rs` tests: ranges hold values to their ends and steps (the maximum stays reachable); a drag follows the pointer without a jump on press, and without a jump when Shift is pressed or released halfway; Shift makes it ten times slower; a press on the thumb grabs it where it is and a press on the track jumps it under the pointer; the ends, steps, the wheel (a step, or a hundredth of the range, finer with Shift), reset to the default, a track with no length, and double-clicks (same control, near, quick, a third click starts a new pair, a clock going backwards is not one). `digits.rs` tests: the bars each digit lights (and their counts), blanks and unknown characters, number formatting (places, no "-0", dashes for NaN), lit and unlit bars per cell, whole pixels and at least one pixel at 100% to 250% for three digit heights, no overlapping bars, cells left to right, the width a layout can reserve, start, centre and end alignment, centring up and down, overflow instead of cutting, the dot on the baseline, and tiny sizes (a 1 point digit): the reserved width covers what is drawn and nothing overlaps. `geometry.rs` tests: `Rect` edges count as inside, snapping keeps touching rectangles touching, union, changed-area pixels. `layout.rs` tests: rows and columns with gaps and insets, fixed and filling items by weight, overflow, centring across, the area not at the origin, points times scale, and at 100% to 250% every rectangle on whole pixels, no overlaps and never a negative size. All pure, no window.
- **Layout rules** (`crates/dock/src/layout.rs`): `crates/dock/tests/layout.rs`. A panel is docked or detached, never both and never lost; unknown panels and double moves are refused. These are pure, so they are fast and need no window.
- **Drag logic** (`crates/dock/src/drag.rs`): `crates/dock/tests/drag.rs`. Letting go inside, just past the edge and well outside; where the new window is placed; a detached panel is docked back only when let go over the main window; nothing happens when a window position is unknown.
- **Screen geometry** (`crates/dock/src/geometry.rs`): `crates/dock/tests/geometry.rs`. Outside-the-window test with and without a margin, "is this screen point over that window?", window-to-screen conversion (with numbers from a real run), and where a dropped window is placed.
- **Winit layout and hit-testing** (`crates/app-winit/src/ui.rs`): unit tests in the file, which also check that the toolkit's layout reproduces the rows and buttons the program had before it. Rows and buttons stack and scale as expected, a click finds the right panel's button, a tiny window never gives negative sizes, and at 100%, 125%, 150%, 175%, 200% and 250% every rectangle sits on whole pixels, rows never overlap, and a click is tested against exactly the drawn button. Pressing on a block (not its button) finds the panel to drag. The changed-area rectangles (union, whole pixels cut to the window) and the button rectangles used for them are tested too. The pixel hand-over (`crates/app-winit/src/pixel.rs`) has its own unit tests: opaque windows drop the alpha, see-through ones keep it in the top byte, and see-through is only allowed on Windows and never with `DOCK_OPAQUE`. To try the opaque fallback on Windows, set `DOCK_OPAQUE=1` before running: the drag ghost then has square corners. To see what a run costs, start the program with `DOCK_TRACE=1` (for example `set DOCK_TRACE=1` then `cargo run --release -p rust-dock-winit`): each frame prints its time and whether the whole window or only a part was shown. Drawing and windows (`render.rs`, `win.rs`, `main.rs`) are checked by running the program.
- **Window layer** (`crates/app-egui/src/app.rs`, `crates/app-egui/src/panels.rs`): not covered by automatic tests. It is checked by building it and by running the program: detach a panel from the menu, drag a tab out and let go outside the window, dock it back by the button, by the strip dragged over the main window, and by closing it, close a detached window, detach every panel.
- **Docs check** (`scripts/check-docs.mjs`): `scripts/check-docs.test.mjs`.

## What CI does

`.github/workflows/ci.yml` runs on every push to `main` and every pull request:

- **docs** (Ubuntu): the docs check tests and the docs check itself.
- **rust** (Windows): `cargo fmt --all --check`, checks that `crates/dock` and `crates/toolkit` have no dependencies, then `cargo test --locked`, which builds the default members and runs the layout tests, then `cargo test --locked -p rust-dock-winit` for the winit program. The same two checks (format, `cargo build --locked`) run on the winit spike in `spikes/winit`, which is a separate crate. Windows is the platform the maintainer uses. macOS and Linux are not tested yet.

CI does not run the program or check lints (`cargo clippy`) yet.

## Writing a test

- A new pure rule gets a test in the matching file under `crates/dock/tests/`.
- Compare paths with forward slashes; never assume the machine's separator.
- A bug fix starts with a test that fails for the bug.
