# User guide

## Pull a panel out into its own window
1. Right-click the panel's tab (for example "Layers").
2. Choose **Detach to window**. The panel opens in its own window, which you can drag to another monitor.

You can also **drag the tab out of the window** and let go outside it. The panel opens in its own window where you dropped it. (Let go a little way past the edge: a tiny overshoot counts as still inside.)

## Put a panel back
Any of these returns the panel to the main window:
- Click **Dock back** in the panel's window.
- Close that window.
- Press on the strip at the top of the panel's window ("Drag here, then let go over the main window to dock"), drag the pointer over the main window and let go. The panel's window does not move while you drag the strip; use its title bar to move the window.

## The winit version (experimental)
Run it with `cargo run -p rust-dock-winit`. The main window shows one coloured block per panel. Click the button with the right-pointing arrow to move that panel into its own window. In that window, click the button with the left-pointing arrow, or close the window, to put it back. The windows follow the display scaling, including a change while the program runs. You can also press on a panel's block, drag it out of the main window and let go outside to detach it where you let go; in a panel window, drag its block over the main window and let go to dock it back. On Wayland the system places the new window and dragging back is not available, so use the buttons. There is no text: panels are told apart by colour and by the window's title.

## Good to know
- If every panel is detached, the egui program's main window shows a short message (the winit version only changes the window's title). Use **Dock back**, or the left-pointing arrow in the winit version, in any panel window to bring panels back.
- Panels cannot be closed or lost: they are always either in the main window or in their own window.
- You can still drag tabs around inside the main window to reorder them or split the view.
- On Linux with Wayland, the system decides where a new window appears.
