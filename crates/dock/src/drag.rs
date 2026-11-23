//! What dragging a panel does when the mouse button is let go. Pure logic: the window layer
//! tells it where the pointer was, it says what to do. No UI, disk or network.
//!
//! - A panel dragged out of the main window and let go outside it is **detached**, with its
//!   new window placed under the pointer when the screen position is known.
//! - A detached panel dragged out of its own window and let go over the main window is
//!   **re-docked**.
//!
//! All positions are in the same unit (pixels or points: only one must be used), and the
//! pointer keeps being reported outside the window while a button is held, which is what
//! makes both drags possible.

use crate::geometry::{is_outside_window, is_over_window, to_screen, window_origin_under_cursor};
use crate::layout::PanelId;

/// The kind of window a drag started in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Main,
    Detached,
}

/// Where the pointer was when the button was let go, and what is known about the windows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Release {
    /// The pointer, relative to the window the drag started in.
    pub pointer: (f32, f32),
    /// The size of that window.
    pub window_size: (f32, f32),
    /// How far outside the window the pointer must be to count as having left it, so a
    /// small overshoot of the edge does not count.
    pub margin: f32,
    /// The top-left corner of that window on the screen, when the platform tells us
    /// (it does not on Wayland).
    pub window_origin: Option<(f32, f32)>,
    /// The main window's top-left corner and size on the screen, when known.
    pub main_window: Option<((f32, f32), (f32, f32))>,
    /// The width of the window a detached panel gets, and how far below its top edge the
    /// pointer should sit, used to place that window under the pointer.
    pub new_window_width: f32,
    pub title_offset: f32,
}

/// What the window layer should do.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DropAction {
    /// Detach the panel. `position` is where its window's top-left corner should go, or
    /// `None` to let the system decide.
    Detach {
        panel: PanelId,
        position: Option<(f32, f32)>,
    },
    /// Put the panel back in the main window.
    Redock { panel: PanelId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Drag {
    panel: PanelId,
    source: Source,
}

/// Remembers which panel the button went down on.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DragTracker {
    current: Option<Drag>,
}

impl DragTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// The button went down on `panel`, in a window of kind `source`.
    pub fn press(&mut self, panel: PanelId, source: Source) {
        self.current = Some(Drag { panel, source });
    }

    /// Forget the drag (for example because its panel's window went away).
    pub fn cancel(&mut self) {
        self.current = None;
    }

    /// The panel being dragged, if the button is down on one.
    pub fn dragging(&self) -> Option<PanelId> {
        self.current.map(|d| d.panel)
    }

    /// The button was let go. Ends the drag and says what to do, if anything.
    pub fn release(&mut self, release: Release) -> Option<DropAction> {
        let drag = self.current.take()?;
        if !is_outside_window(release.pointer, release.window_size, release.margin) {
            return None;
        }
        match drag.source {
            Source::Main => {
                let position = release.window_origin.map(|origin| {
                    window_origin_under_cursor(
                        to_screen(origin, release.pointer),
                        release.new_window_width,
                        release.title_offset,
                    )
                });
                Some(DropAction::Detach {
                    panel: drag.panel,
                    position,
                })
            }
            Source::Detached => {
                let origin = release.window_origin?;
                let (main_origin, main_size) = release.main_window?;
                let screen = to_screen(origin, release.pointer);
                is_over_window(screen, main_origin, main_size)
                    .then_some(DropAction::Redock { panel: drag.panel })
            }
        }
    }
}
