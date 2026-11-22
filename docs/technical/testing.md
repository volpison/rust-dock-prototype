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

- **Layout rules** (`crates/dock/src/layout.rs`): `crates/dock/tests/layout.rs`. A panel is docked or detached, never both and never lost; unknown panels and double moves are refused. These are pure, so they are fast and need no window.
- **Screen geometry** (`crates/dock/src/geometry.rs`): `crates/dock/tests/geometry.rs`. Outside-the-window test with and without a margin, "is this screen point over that window?", window-to-screen conversion (with numbers from a real run), and where a dropped window is placed.
- **Winit layout and hit-testing** (`crates/app-winit/src/ui.rs`): unit tests in the file. Rows and buttons stack and scale as expected, a click finds the right panel's button, a tiny window never gives negative sizes. Drawing and windows (`render.rs`, `win.rs`, `main.rs`) are checked by running the program.
- **Window layer** (`crates/app-egui/src/app.rs`, `crates/app-egui/src/panels.rs`): not covered by automatic tests. It is checked by building it and by running the program: detach a panel from the menu, drag a tab out and let go outside the window, dock it back by the button, by the strip dragged over the main window, and by closing it, close a detached window, detach every panel.
- **Docs check** (`scripts/check-docs.mjs`): `scripts/check-docs.test.mjs`.

## What CI does

`.github/workflows/ci.yml` runs on every push to `main` and every pull request:

- **docs** (Ubuntu): the docs check tests and the docs check itself.
- **rust** (Windows): `cargo fmt --all --check`, a check that `crates/dock` has no dependencies, then `cargo test --locked`, which builds the default members and runs the layout tests, then `cargo test --locked -p rust-dock-winit` for the winit program. The same two checks (format, `cargo build --locked`) run on the winit spike in `spikes/winit`, which is a separate crate. Windows is the platform the maintainer uses. macOS and Linux are not tested yet.

CI does not run the program or check lints (`cargo clippy`) yet.

## Writing a test

- A new pure rule gets a test in the matching file under `crates/dock/tests/`.
- Compare paths with forward slashes; never assume the machine's separator.
- A bug fix starts with a test that fails for the bug.
