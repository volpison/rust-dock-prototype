# Spike results

Run on 2026-11-22. Windows 10, one monitor, 100% display scaling. winit 0.30.13, softbuffer 0.4.8, tiny-skia 0.12.0. The checks of the [plan](../../docs/technical/winit-spike-plan.md), in its order:

| Check | Result |
|---|---|
| Transparent corners on Windows | **Works.** A screenshot of the borderless window shows its rounded corners with the terminal behind them. Premultiplied `0xAARRGGBB` through softbuffer is kept by the system, although softbuffer 0.4.8 has no alpha setting. |
| Resizing without flicker | **Works** to the eye: no black flashes or stale pixels while dragging an edge of the plain window, including maximising. |
| Pointer events outside the window while a button is held | **Works.** About 25 drags in both windows, out of every side and corner: events kept arriving (negative coordinates and past the far edge), and the release was reported outside, with a correct screen position. Same behaviour as the egui experiment. |
| Moving a borderless window by its custom strip (`drag_window`) | **Smooth.** The OS takes over the move: no pointer events and no release arrive in our window during it. |
| Redraw time, release build | **Well under 8 ms.** See below. |

## Speed
| Measure | Release | Debug (`cargo run`) |
|---|---|---|
| Panel scene 320x420, drawing only | 0.34 ms | 8.69 ms |
| Clear 1920x1080 | 1.19 ms | 19.80 ms |
| Redraw of a 320x420 window (draw, copy, present) | 0.85 to 1.1 ms | about 14.5 to 17.5 ms |
| Redraw while resizing (120x728 up to 949x617) | 0.9 to 2.7 ms | not measured |
| Redraw of a 1897x1008 window | 4.5 ms | not measured |

The whole frame is redrawn every time (no partial updates yet), so these are worst cases for this scene.

**Debug builds are too slow to judge or to work in.** tiny-skia without optimisation takes 25 times longer. If the project moves to this stack, optimise the dependencies in debug builds (a `[profile.dev.package."*"]` setting with `opt-level = 3`).

## Not tested
- A second monitor, display scaling above 100%, and monitors with different scaling.
- macOS, and Linux (X11 and Wayland). softbuffer's alpha handling in particular differs by platform.
- Dropping a window onto another window (only the pointer position outside was checked).
- Text, input fields and anything beyond rounded rectangles.

## Reading for Gate A
On Windows at 100% scaling the stack meets every threshold in the plan. The open risks are the untested platforms and display scaling, which stage 2 (windows and input) covers. The decision to go on is the maintainer's.
