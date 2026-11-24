# Plan: stage 3, a mini toolkit for custom-drawn controls

Status: agreed in review (2026-11-23). Pull request 2 (pointer input, button and toggle in `crates/toolkit`) and 3 (layout, used by `app-winit`) are merged; 4 (slider and knob logic) are merged; 5 (the 7-segment number display) is being built. Follows [stage 2](stage2-plan.md), which is finished: the winit program does what the egui program does with windows. The whole route is in [the roadmap](roadmap-winit.svg). Stage 3 ends at **Gate B**.

## Where stage 2 left us
- Several real OS windows from one event loop; detach and dock back by buttons or by dragging; a ghost follows the pointer; correct at 150% display scaling; see-through corners on Windows, with an opaque fallback elsewhere.
- Pointer events keep arriving outside the window while a button is held, so drags across windows work (checked on Windows only).
- Drawing is cheap: the spike measured about 1 ms for a 320x420 window in a release build. In a debug build with optimised dependencies a 360x280 frame took 4 to 6 ms and the 150x90 ghost under 1 ms; no frame was above about 11 ms.
- The pure rules (`dock`) are shared with the egui program and tested without a window: 27 tests there, plus 26 in `app-winit`.
- Not yet checked: a second monitor or mixed scales, macOS, Linux (Wayland cannot place windows or read the global pointer).

## Goal
Answer the Gate B question with a real panel, not an opinion: **is building a panel from custom-drawn controls pleasant, or are we rebuilding egui slowly?** For that, build the small set of controls the panels need, build one realistic panel with them, and compare it with the egui version.

Controls: button, toggle, slider, knob. Layout: rows and columns with gaps. States: hover, pressed, disabled. No text (stage 5): a control shows its value by its shape and, where a number helps, by a small 7-segment display (digits, a dot and a minus, drawn from shapes).

## Where the code lives
A new crate, **`crates/toolkit`**: pure logic only (pointer state, widget state machines, layout arithmetic), **no dependencies**, tested without a window, checked in CI like `dock`. Drawing (tiny-skia) and the window layer stay in `app-winit`.

Why a crate and not more files in `app-winit`: the toolkit logic is the part worth reusing when this is folded into the larger project, and a dependency-free crate can be tested here and in CI without any window. `dock` stays about panels and windows; `toolkit` is about controls inside a window.

## Pull requests, in order
1. **This plan**, with the roadmap marking stage 2 done (docs only).
2. **`crates/toolkit`: pointer input and button/toggle logic.** Hover, pressed, pointer capture (a press keeps the pointer until release, even outside the control), click detection. A button and a toggle as state machines: events in, "clicked" or "changed" out. Added to the workspace and to the CI "no dependencies" check.
3. **Layout.** Rows and columns with gaps, padding, fixed and stretching sizes, in points with the scale applied and every rectangle snapped to whole pixels (the `Rect` and snapping from `app-winit/src/ui.rs` move here). Tests at 100% to 250%.
4. **Slider and knob logic.** Value from pointer position or drag distance, clamping and steps, a finer drag with Shift, the wheel, double-click to reset. Pure and tested.
5. **Number display.** A 7-segment display: a small table says which of the seven bars each digit lights (plus the dot and the minus), and pure code turns a number into the bars' rectangles for a given size. Pure and tested; it needs no font.
6. **Drawing.** A small theme (colours, radii) and drawing for the four controls and the number display in `app-winit`, with hover, pressed and disabled looks, wired to the winit events. Shown in a test window (`--controls`) so they can be tried before the real panel.
7. **A real panel.** The Properties panel gets sliders, a knob, a toggle and buttons, replacing its coloured block, in the main window and in a window of its own. A panel is then no longer a block you can press anywhere to drag, so it gets a **grip strip** at the top that starts the drag-out and drag-back, and the rest of the panel is for its controls. **7b** (its own pull request, so each stays small): the egui program gets the same controls from egui, to compare.
8. **Gate B review** (docs only): the comparison below, and a go or no-go.

Each is its own pull request with a "Check before merging" list and a "Not verified here" section, as before.

## Gate B: what we will look at
- **Size:** lines of code for the Properties panel in each program, and for one extra control added afterwards.
- **Effort:** how long adding a new control took, and what was awkward.
- **Feel:** dragging a slider and a knob, hover and pressed states, at 100% and 150%, in a release build.
- **Cost:** frame time while dragging a slider (release build, `DOCK_TRACE=1`).
- **What egui gave for free and we lack:** scrolling, focus and keyboard use, tooltips, text. Each is listed with a rough cost to add, so the go or no-go is made knowing them.
- **Go** if the panel was pleasant and the missing pieces are affordable; **no-go** if we were mostly rebuilding egui, and then the egui program stays the main program (it is intact), possibly with skinned windows as the earlier experiment showed.

## Risks
- **No text yet.** Labels are missing, so controls are told apart by place and shape; the 7-segment display covers numbers. Letters are not covered until stage 5.
- **The grip strip changes how dragging works** in the winit program (pull request 7). It is small, but it replaces "press anywhere on the block".
- **Pointer capture across windows** was only seen on Windows. The toolkit takes capture as a rule of its own, and the window layer provides it.
- **Scope creep.** A toolkit has no natural end. The four controls and one panel are the limit until Gate B.

## Decisions confirmed in the review (2026-11-23)
1. **Crate:** `crates/toolkit`, pure and dependency-free.
2. **Number display:** yes, a 7-segment display drawn from shapes, in its own pull request (5).
3. **Skins from images:** after Gate B, not in stage 3. Other work comes first.

## What is not in this plan
- Text and text fields (stage 5), including fonts, selection, clipboard and IME.
- Skins from images (PNG nine-slice): after Gate B.
- The dock tree, tab groups and drop zones (stage 4).
- Scrolling, keyboard focus, tooltips and accessibility (listed at Gate B, built only if it goes on).
- macOS and Linux, saving the layout, packaging (stage 6).
