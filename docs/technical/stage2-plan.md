# Plan: stage 2, windows and input on winit

Status: pull requests 1 (this plan), 2 (workspace split), 3 (the `app-winit` skeleton) and 4 (display scaling) are merged; 5 (drag-out and drag-back) is being built. Follows the [spike](winit-spike-plan.md), whose [results](../../spikes/winit/RESULTS.md) passed Gate A on Windows. The whole route is in [the roadmap](roadmap-winit.svg).

## Goal
Make the new stack do what the egui program does with windows: several real OS windows from one event loop, panels detached and put back, drag-out, drag-back, correct on scaled displays, and cheap to redraw. Panels stay placeholder drawings. Controls are stage 3, text is stage 5.

The egui program stays as it is until Gate B says otherwise. Both programs use the same pure rules, `layout.rs` and `geometry.rs`.

## First, a Cargo workspace
The spike could not use the `dock` library, because depending on the root package would pull in eframe. The fix is to split the project into crates:

```
Cargo.toml          workspace file only
crates/dock/        the pure library: layout.rs, geometry.rs, their tests (no dependencies)
crates/app-egui/    today's egui program (the program is still called rust-dock-prototype)
crates/app-winit/   new: the stage 2 program, starting from the spike's code
spikes/winit/       stays as it is, excluded from the workspace, deleted once app-winit has absorbed it
```

- `cargo run` at the root keeps running the egui program (the workspace's default member), so nothing changes for the maintainer.
- `dock` keeps no dependencies. A check in CI keeps it that way (`cargo tree -p dock` shows no dependencies).
- Files move with `git mv`, in a pull request of their own with no behaviour change, so history follows the files.
- `CLAUDE.md`, the README and the architecture and testing pages mention `src/...` paths; they move with it.

## Pull requests, in order
1. **This plan** (docs only).
2. **Workspace split.** Moves only. Same tests, same behaviour; `cargo test` and `cargo run` give the same results.
3. **`app-winit` skeleton.** One event loop, one window per panel (the main window and detached ones), a window manager keyed by `PanelId`, the layout from `dock`. A minimal drawn "Detach" and "Dock back" rectangle per panel, with a tiny hit-test, so the program can be used before stage 3 has real controls.
4. **Display scaling.** Pointer and window sizes in physical pixels, scale-factor changes while the program runs, and `geometry` fed with the right units. Checked by changing Windows' display scale (125%, 150%) on the maintainer's one monitor. The skeleton already read the size and scale every frame, so this came down to two things: snap every rectangle to whole pixels (crisp edges at 125% and 150%, and clicks tested against what was drawn), and handle the scale-change event. Units for `geometry`: physical pixels everywhere in `app-winit`, with any margin multiplied by the scale (that comes with drag-out, pull request 5).
5. **Drag-out and drag-back.** The same flow as the egui program, on winit's pointer events, which the spike showed keep arriving outside the window while a button is held. The drag state (pressed on a panel, released outside or over a window) is pure logic, so it is in `dock` (`drag.rs`) with tests.
6. **Redraw only what changed.** Redraw just the window that needs it, and give softbuffer the changed area (`present_with_damage`). Measured against the spike's numbers.
7. **Transparency safety.** See the risk below.

Each is its own pull request with a "Check before merging" list and a "Not verified here" section.

## Risks
- **Transparency is outside softbuffer's documented format.** The spike's see-through corners worked on Windows, but softbuffer documents the pixel as 0RGB, so this is not guaranteed on other platforms or versions. Plan: put the pixel hand-over in one small function, so a platform path (for example a Windows layered window) can replace it, and add a startup check or a visible fallback to an opaque window. Decide in pull request 7 whether a real layered-window path is needed.
- **Scaling.** egui hid most of this. Mixing physical pixels and points is the likely source of bugs; pull request 4 exists for this, and `geometry` keeps one unit.
- **Cross-window drags.** Only the pointer position outside the window was checked, not a drop onto another window. Pull request 5 does that.
- **Wayland.** Apps cannot place windows or read the global cursor position there. Linux is stage 6; the layout already treats a window position as optional, and the window layer will report what the platform can do.
- **Debug-build speed.** tiny-skia is about 25 times slower without optimisation. The workspace sets `opt-level = 3` for dependencies in the dev profile (added with `app-winit`; the repository's own crates stay unoptimised).

## Decisions confirmed in the review (2026-11-22)
1. The crate names and folders above (`crates/dock`, `crates/app-egui`, `crates/app-winit`).
2. The egui program keeps its program name, so `cargo run` is unchanged.
3. Scaling is checked by changing the Windows display scale on the maintainer's one monitor. As a baseline, both existing programs (the egui program and the spike) were run at 150% and looked right; pull request 4 repeats this for `app-winit`.

## What is not in this plan
- Controls, hit-testing beyond two rectangles, layout of widgets (stage 3).
- The dock tree, tab groups and drop zones (stage 4).
- Text (stage 5).
- macOS and Linux, saving the layout, packaging (stage 6).
