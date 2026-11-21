# Plan: drag a tab out of the window to detach it

Status: step 1 (the experiment) is done and step 2 (drag a tab out to detach) is built. Steps 3 and 4 are not.

## What the experiment found (Windows, 100% scaling, one monitor)
- While the mouse button is held, pointer positions keep arriving after the cursor leaves the window (down to 833 points left of it), and the release is reported outside the window.
- egui sends a "pointer gone" event as the cursor crosses the edge, but the drag continues.
- `egui_dock` does not tell the app which tab is being dragged (`on_tab_button`'s `drag_started` never fired), so the dragged tab is taken to be the tab under the pointer when the button went down.
- Not tested: the right and bottom edges, a second monitor, other scaling, macOS, Linux.

## Goal
Drag a tab out of the main window. When it is released outside the window, the panel opens as its own window under the cursor. Today a panel is detached from the tab's context menu ("Detach to window").

## Approach
Detach **on release**, not live.

Live tear-off (the new window follows the cursor while the button is still down) is the Photoshop feel, but the OS would have to hand over a drag that began in another window. That is likely to fail on Windows, so it is left for later.

1. While a tab is dragged, watch its drag state. `egui_dock`'s `TabViewer::on_tab_button` gives the tab's `Response`.
2. Turn the pointer position into a screen position: the main window's top-left corner plus the pointer position inside the window.
3. On release, if that screen position is outside the main window, call `Layout::detach(id, Some(position))`. The detached window already accepts a position.
4. The test "is this point outside the window?" is pure logic. It goes in its own small file with tests, not in the UI layer.

## Risks
- **Pointer outside the window.** On Windows, winit normally keeps sending mouse positions while a button is held, even outside the window. egui may still treat leaving the window as "pointer gone" and drop the drag. Unknown until tried.
- **egui_dock's own drag.** It may still try to move the dragged tab when it is released outside its area, and may need a small workaround.
- **Monitors with different scaling.** egui works in points and the OS in pixels, so a window could land in the wrong place. Needs a second monitor to check.
- **Wayland.** Apps cannot read the global cursor position or place windows. The tab menu stays as the fallback there.

## Steps
1. **Experiment (not merged).** A throwaway branch logs the pointer position and drag state while a tab is dragged out of the window. Run on Windows; the output decides the next steps.
2. **Pull request: drag a tab out to detach.** The pure test, the wiring, docs and tests.
3. **Pull request: drag a detached panel back onto the main window.** Uses the same screen-position conversion, with a grab handle in the panel window.
4. **Pull request: remember window positions between runs.** Needs a saved-file format with a version number (`schema_version`), as the project guidelines require.

## What is not in this plan
- Live tear-off, where the window follows the cursor during the drag.
- Docking by dropping a detached window onto a specific spot (a split or tab group) in the main window. Step 3 returns the panel to the main window's focused area only.
