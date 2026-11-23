//! The same detachable panels on winit + softbuffer + tiny-skia, without egui.
//! One event loop, one OS window per detached panel. All placement rules live in the
//! `dock` crate (`crates/dock`); this file only reacts to the window system and calls them.
//!
//! Panels have no text yet (stage 5): each is a coloured block, the arrow on a button says
//! what it does (right = detach, left = dock back), and the window title names the panel.

mod panels;
mod render;
mod ui;
mod win;

use dock::drag::{DragTracker, DropAction, Release, Source};
use dock::layout::{Layout, PanelId};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{WindowAttributes, WindowId};

use win::Win;

/// Sizes of the windows, in points.
const MAIN_SIZE: [f64; 2] = [360.0, 280.0];
const DETACHED_SIZE: [f64; 2] = [320.0, 420.0];

/// Drag-out: how far outside its window the pointer must be (points), and how far below the
/// new window's top edge the pointer lands (points). Same values as the egui program.
const DRAG_MARGIN: f64 = 8.0;
const TITLE_OFFSET: f64 = 15.0;

const MAIN_TITLE: &str = "Dock prototype (winit)";
const MAIN_TITLE_EMPTY: &str = "Dock prototype (winit): all panels are in their own windows";

/// Which window an event is for.
#[derive(Clone, Copy)]
enum Target {
    Main,
    Detached(PanelId),
}

struct App {
    /// Source of truth for where each panel is.
    layout: Layout,
    main: Option<Win>,
    detached: Vec<(PanelId, Win)>,
    /// Which panel the left button is down on, if any.
    drag: DragTracker,
}

impl App {
    fn new() -> Self {
        Self {
            layout: Layout::new(panels::ALL),
            main: None,
            detached: Vec::new(),
            drag: DragTracker::new(),
        }
    }

    fn target_of(&self, id: WindowId) -> Option<Target> {
        if self.main.as_ref().is_some_and(|w| w.window.id() == id) {
            return Some(Target::Main);
        }
        self.detached
            .iter()
            .find(|(_, w)| w.window.id() == id)
            .map(|(panel, _)| Target::Detached(*panel))
    }

    fn win_mut(&mut self, target: Target) -> Option<&mut Win> {
        match target {
            Target::Main => self.main.as_mut(),
            Target::Detached(panel) => self
                .detached
                .iter_mut()
                .find(|(p, _)| *p == panel)
                .map(|(_, w)| w),
        }
    }

    fn redraw(&mut self, target: Target) {
        match target {
            Target::Main => {
                let docked = self.layout.docked().to_vec();
                let Some(win) = self.main.as_mut() else {
                    return;
                };
                let cursor = win.cursor;
                win.present(|canvas, scale| {
                    let hover = cursor.and_then(|point| {
                        ui::detach_button_at(point, canvas.width() as f32, scale, &docked)
                    });
                    render::draw_main(canvas, scale, &docked, hover);
                });
            }
            Target::Detached(panel) => {
                let Some(win) = self.win_mut(target) else {
                    return;
                };
                let cursor = win.cursor;
                win.present(|canvas, scale| {
                    let (w, h) = (canvas.width() as f32, canvas.height() as f32);
                    let hover = cursor.is_some_and(|p| ui::dock_back_button_at(p, w, h, scale));
                    render::draw_detached(canvas, scale, panel, hover);
                });
            }
        }
    }

    fn request_redraw(&self, target: Target) {
        match target {
            Target::Main => {
                if let Some(win) = &self.main {
                    win.window.request_redraw();
                }
            }
            Target::Detached(panel) => {
                if let Some((_, win)) = self.detached.iter().find(|(p, _)| *p == panel) {
                    win.window.request_redraw();
                }
            }
        }
    }

    /// Move a docked panel into its own window, at `position` (screen pixels) or, with
    /// `None`, where the OS decides.
    fn detach(
        &mut self,
        event_loop: &ActiveEventLoop,
        panel: PanelId,
        position: Option<(f32, f32)>,
    ) {
        if self.layout.detach(panel, position).is_err() {
            return;
        }
        let mut attributes = WindowAttributes::default()
            .with_title(panels::title(panel))
            .with_inner_size(LogicalSize::new(DETACHED_SIZE[0], DETACHED_SIZE[1]));
        if let Some((x, y)) = position {
            attributes =
                attributes.with_position(PhysicalPosition::new(x.round() as i32, y.round() as i32));
        }
        self.detached
            .push((panel, Win::new(event_loop, attributes)));
        self.after_layout_change();
    }

    /// Put a detached panel back in the main window and close its window.
    fn redock(&mut self, panel: PanelId) {
        if self.layout.redock(panel).is_err() {
            return;
        }
        // The window the drag started in is going away.
        self.drag.cancel();
        self.detached.retain(|(p, _)| *p != panel);
        self.after_layout_change();
    }

    fn after_layout_change(&mut self) {
        if let Some(win) = &self.main {
            let title = if self.layout.docked().is_empty() {
                MAIN_TITLE_EMPTY
            } else {
                MAIN_TITLE
            };
            win.window.set_title(title);
            win.window.request_redraw();
        }
    }

    /// The left button went down in `target`: a button acts at once, a block starts a drag.
    fn press(&mut self, event_loop: &ActiveEventLoop, target: Target) {
        match target {
            Target::Main => {
                let Some(win) = &self.main else {
                    return;
                };
                let Some(cursor) = win.cursor else {
                    return;
                };
                let (width, _, scale) = win.metrics();
                let docked = self.layout.docked();
                if let Some(panel) = ui::detach_button_at(cursor, width, scale, docked) {
                    self.detach(event_loop, panel, None);
                } else if let Some(panel) = ui::block_at(cursor, width, scale, docked) {
                    self.drag.press(panel, Source::Main);
                }
            }
            Target::Detached(panel) => {
                let Some(win) = self.win_mut(target) else {
                    return;
                };
                let Some(cursor) = win.cursor else {
                    return;
                };
                let (width, height, scale) = win.metrics();
                if ui::dock_back_button_at(cursor, width, height, scale) {
                    self.redock(panel);
                } else if ui::content_at(cursor, width, height, scale) {
                    self.drag.press(panel, Source::Detached);
                }
            }
        }
    }

    /// The left button was let go in `target`. The pointer keeps being reported outside the
    /// window while the button is held, so `cursor` is where the drag ended.
    fn release(&mut self, event_loop: &ActiveEventLoop, target: Target) {
        let main_window = self
            .main
            .as_ref()
            .and_then(|w| w.screen_origin().map(|o| (o, w.metrics())))
            .map(|(origin, (w, h, _))| (origin, (w, h)));
        let Some(win) = self.win_mut(target) else {
            self.drag.cancel();
            return;
        };
        let Some(pointer) = win.cursor else {
            self.drag.cancel();
            return;
        };
        let (width, height, scale) = win.metrics();
        let release = Release {
            pointer,
            window_size: (width, height),
            margin: DRAG_MARGIN as f32 * scale,
            window_origin: win.screen_origin(),
            main_window,
            new_window_width: DETACHED_SIZE[0] as f32 * scale,
            title_offset: TITLE_OFFSET as f32 * scale,
        };
        match self.drag.release(release) {
            Some(DropAction::Detach { panel, position }) => {
                self.detach(event_loop, panel, position)
            }
            Some(DropAction::Redock { panel }) => self.redock(panel),
            None => {}
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.main.is_some() {
            return;
        }
        event_loop.set_control_flow(ControlFlow::Wait);
        let attributes = WindowAttributes::default()
            .with_title(MAIN_TITLE)
            .with_inner_size(LogicalSize::new(MAIN_SIZE[0], MAIN_SIZE[1]));
        self.main = Some(Win::new(event_loop, attributes));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(target) = self.target_of(id) else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => match target {
                Target::Main => event_loop.exit(),
                // Closing a detached window puts its panel back in the main window.
                Target::Detached(panel) => self.redock(panel),
            },
            WindowEvent::Resized(_) => self.request_redraw(target),
            WindowEvent::RedrawRequested => self.redraw(target),
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(win) = self.win_mut(target) {
                    win.cursor = Some((position.x as f32, position.y as f32));
                }
                self.request_redraw(target);
            }
            // The window moved to a display with another scale, or the Windows setting changed.
            // Sizes are worked out from the scale every frame, so only the pointer position
            // (in the old pixels) is forgotten and the window redrawn.
            WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(win) = self.win_mut(target) {
                    win.cursor = None;
                }
                self.request_redraw(target);
            }
            // While the button is held the pointer keeps being reported outside the window,
            // so the last position is kept: that is where the drag will end.
            WindowEvent::CursorLeft { .. } => {
                if self.drag.dragging().is_none() {
                    if let Some(win) = self.win_mut(target) {
                        win.cursor = None;
                    }
                    self.request_redraw(target);
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => self.press(event_loop, target),
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => self.release(event_loop, target),
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("run");
}
