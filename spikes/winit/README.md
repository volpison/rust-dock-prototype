# Spike: winit + softbuffer + tiny-skia

Throwaway program that answers the questions of Gate A in [the spike plan](../../docs/technical/winit-spike-plan.md). It is not part of the egui program and does not use the `dock` library.

## Run it
```
cd spikes/winit
cargo run
```
Two windows open:
- **borderless**: no title bar, rounded corners that should show the desktop through, moved by dragging its blue top strip.
- **plain**: a normal window with the same drawing.

The terminal prints what the program sees. Close either window to quit.

## What to try, and what to note
1. **Transparency.** Are the corners of the borderless window really see-through, or black or white?
2. **Resize flicker.** Drag an edge of the plain window around quickly. Any black flashes or stale pixels?
3. **Moving.** Does dragging the blue strip move the borderless window smoothly?
4. **Pointer outside.** In either window, press the left button on the drawing, move out of the window, and release. The terminal should print "pointer event outside the window" lines and "button up ... OUTSIDE" with a screen position.
5. **Speed.** Move the mouse over a window and read the "average ms per redraw" lines. The two `[bench]` lines at the start show drawing time without any window.

Results go in `RESULTS.md` in this folder once the spike has been run.
