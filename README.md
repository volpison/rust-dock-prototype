# rust-dock-prototype

Photoshop-style panels that can be pulled out of the main window into **real, separate OS windows** (so you can put one on a second monitor) and put back again. Pure Rust: no Tauri, no GTK, no Qt.

It is one of the "Prototypes" that will later be folded into a larger project.

**Status:** it works on Windows. Not yet tried on macOS or Linux.

## What you can do
- Drag tabs around the main window to reorder them or split the view.
- Right-click a tab and choose **Detach to window** to move that panel into its own window.
- Click **Dock back** in a panel's window, or close that window, to return the panel.

## Run it
You need [Rust](https://www.rust-lang.org/tools/install).

```
cargo run
```

To run the tests: `cargo test`.

## More
- What is planned, and what this is not: [ROADMAP.md](ROADMAP.md)
- How to use it: [User guide](docs/user-guide/README.md)
- How it is built: [docs/technical/architecture.md](docs/technical/architecture.md)
- Why things were chosen: [docs/technical/decisions.md](docs/technical/decisions.md)
- All docs: [docs/index.md](docs/index.md)
- What changed: [CHANGELOG.md](CHANGELOG.md)

Licence: [MIT](LICENSE).
