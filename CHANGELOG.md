# Changelog

All notable changes are listed here. Format: [Keep a Changelog](https://keepachangelog.com/), versions follow [SemVer](https://semver.org/).

## [Unreleased]

### Added
- Winit version: while a panel is dragged outside its window, a small ghost window in the panel's colour follows the pointer (not on Wayland, where the program cannot read the window position).
- Winit version: drag a panel's block (it dims while dragged) out of the main window and let go outside it to detach it there; drag a detached panel's block over the main window and let go to dock it again. Where the platform gives no window position (Wayland) a panel dragged out opens where the system puts it, and dragging back is not available.
- Drag a detached panel's strip over the main window and let go to dock it again.
- Drag a panel's tab out of the window and let go outside it to detach the panel there.
- A message in the main window when every panel has been detached, instead of a blank window.
- A window: right-click a panel tab and choose "Detach to window" to move it into its own window. Use "Dock back" or close that window to return it.
- Panel layout rules: a panel is either docked or detached, and can move between the two.
