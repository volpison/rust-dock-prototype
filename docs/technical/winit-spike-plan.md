# Plan: a spike on winit + softbuffer + tiny-skia

Status: the spike program is written (`spikes/winit/`); it has not been run yet, and no results are in. The whole route is in [the roadmap](roadmap-winit.svg); this page covers stage 1 only.

## Why
The larger project may want panels that are mostly custom-drawn controls, on our own windowing and drawing stack instead of egui. Before committing to that, a small program should answer one question: **is drawing our own windows with winit + softbuffer + tiny-skia smooth enough?** If not, we stay on egui and skin the windows there. This is Gate A on the roadmap.

## Where it lives
In this repository, on `main`, through normal `claude/<topic>` pull requests. It is a separate crate in `spikes/winit/` with its own `Cargo.toml` and `Cargo.lock`, and it is **not** a dependency of the egui program.

- The egui program, its CI build and its `Cargo.lock` stay untouched. winit, softbuffer and tiny-skia never enter the main build.
- The spike does **not** use the `dock` library yet. Depending on the root package would pull eframe and egui_dock into the spike, so reuse waits for stage 2, when the pure code moves into its own crate (a Cargo workspace: `dock`, the egui app, the winit app).
- If Gate A says "stay on egui", the folder is deleted and nothing else changes.

Left out: a new repository, and a long-lived branch next to `main` (see [decisions](decisions.md)).

## What the spike builds
A small program, no toolkit, no text, no docking:

1. One window with a softbuffer surface; tiny-skia fills a rounded rectangle and a few shapes; the window can be resized.
2. Two windows from one event loop, each with its own surface.
3. A borderless window with transparent corners, moved by dragging a custom title area (`drag_window`).
4. A drag that starts in one window and ends over the other: do the pointer events keep arriving outside the window while the button is held? (The egui experiment on Windows says yes; this checks winit directly.)
5. A timer for a full redraw of a typical panel scene.

Versions, read from the downloaded crate sources: winit 0.30.13, softbuffer 0.4.8, tiny-skia 0.12.0. The signatures were read from those sources before the code was written, as was done for egui (the first egui sketch used the wrong APIs). One finding: softbuffer 0.4.8 has no setting for transparency (its macOS backend skips the alpha channel), so see-through corners are the riskiest check.

## Gate A: what counts as "smooth enough"
The thresholds are a first guess and can change before the spike starts.
- Resizing shows no visible flicker or black flash.
- Transparent corners really are transparent on Windows.
- Pointer events keep arriving outside the window during a drag.
- A full redraw of a typical panel (about 320 x 420 points) at 100% scaling takes under 8 ms, and a 1920 x 1080 clear is measured and written down.
- Moving a borderless window by its custom title area feels normal.

## Risks
- **Transparency.** Whether softbuffer gives true per-pixel alpha on each platform is unknown until tried.
- **Resize flicker.** Software drawing can show stale pixels while resizing.
- **Display scaling.** Needs a second monitor or a scaling change to check; not available here. Stage 2 handles it, so the spike is tested at 100% only and says so.
- **Wayland.** Apps cannot place windows or read the global cursor position.

## Steps
1. **This pull request.** The roadmap, this plan and the decisions. Docs only.
2. **Pull request: the spike**, items 1 to 5 above, with a short results note in this folder. It also adds a CI job that builds the spike on Windows.
3. **Gate A.** Decide with the results: continue to stage 2, or stay on egui.

## What is not in this plan
- Text, text fields and fonts (stage 5).
- Controls, layout and hit-testing (stage 3).
- Docking and the layout rules on the new stack (stage 4).
- macOS and Linux (stage 6).
