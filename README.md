# rust-dock-prototype

A prototype of Photoshop-style panels that can be pulled out of the main window into **real, separate OS windows** (so you can put one on a second monitor) and put back again. Pure Rust: no Tauri, no GTK, no Qt.

It is one of the "Prototypes" that will later be folded into a larger project.

**Status:** early. The panel layout rules exist and are tested; the window layer (egui + eframe + egui_dock) is next.

- What is planned, and what this is not: [ROADMAP.md](ROADMAP.md)
- How it is built: [docs/technical/architecture.md](docs/technical/architecture.md)
- Why things were chosen: [docs/technical/decisions.md](docs/technical/decisions.md)
- All docs: [docs/index.md](docs/index.md)

## Try it

```
cargo test
```
