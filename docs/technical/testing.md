# Tests

## Running the tests

```
cargo test                          # layout rules (tests/layout.rs); also builds the window layer
cargo run                           # try the window layer by hand
node --test "scripts/*.test.mjs"    # tests of the docs check
node scripts/check-docs.mjs         # links, heading links and the docs index
```

The Node commands need Node 20 or newer. They are run by CI, so you only need them to check the docs before pushing.

## What the tests cover

- **Layout rules** (`src/layout.rs`): `tests/layout.rs`. A panel is docked or detached, never both and never lost; unknown panels and double moves are refused. These are pure, so they are fast and need no window.
- **Window layer** (`src/app.rs`, `src/panels.rs`): not covered by automatic tests. It is checked by building it and by running the program: detach a panel, dock it back, close a detached window, detach every panel.
- **Docs check** (`scripts/check-docs.mjs`): `scripts/check-docs.test.mjs`.

## What CI does

`.github/workflows/ci.yml` runs on every push to `main` and every pull request:

- **docs** (Ubuntu): the docs check tests and the docs check itself.
- **rust** (Windows): `cargo test --locked`, which builds everything and runs the layout tests. Windows is the platform the maintainer uses. macOS and Linux are not tested yet.

CI does not run the program or check formatting (`cargo fmt`) or lints (`cargo clippy`) yet.

## Writing a test

- A new pure rule gets a test in the matching file under `tests/`.
- Compare paths with forward slashes; never assume the machine's separator.
- A bug fix starts with a test that fails for the bug.
