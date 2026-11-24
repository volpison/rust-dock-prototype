//! The same detachable panels on winit + softbuffer + tiny-skia, without egui.
//! One event loop, one OS window per detached panel. All placement rules live in the
//! `dock` crate (`crates/dock`); this file only reacts to the window system and calls them.
//!
//! Panels have no text yet (stage 5): each is a coloured block, the arrow on a button says
//! what it does (right = detach, left = dock back), and the window title names the panel.

mod controls;
mod controls_render;
mod panels;
mod pixel;
mod render;
mod ui;
mod win;

use dock::drag::{DragTracker, DropAction, Release, Source};
use dock::geometry::{is_outside_window, to_screen, window_origin_under_cursor};
use dock::layout::{Layout, PanelId};
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::ModifiersState;
#[cfg(windows)]
use winit::platform::windows::WindowAttributesExtWindows;
use winit::window::{CursorIcon, WindowAttributes, WindowId, WindowLevel};

use ui::Rect;
use win::Win;

/// Sizes of the windows, in points.
const MAIN_SIZE: [f64; 2] = [360.0, 280.0];
const DETACHED_SIZE: [f64; 2] = [320.0, 420.0];

/// Drag-out: how far outside its window the pointer must be (points), and how far below the
/// new window's top edge the pointer lands (points). Same values as the egui program.
const DRAG_MARGIN: f64 = 8.0;
const TITLE_OFFSET: f64 = 15.0;

/// The ghost that follows the pointer while a panel is dragged outside its window (points).
const GHOST_SIZE: [f64; 2] = [150.0, 90.0];

const MAIN_TITLE: &str = "Dock prototype (winit)";
const MAIN_TITLE_EMPTY: &str = "Dock prototype (winit): all panels are in their own windows";

/// Which window an event is for.
#[derive(Clone, Copy)]
enum Target {
    Main,
    Detached(PanelId),
    /// The small window that follows the pointer during a drag. It takes no input.
    Ghost,
}

struct App {
    /// Source of truth for where each panel is.
    layout: Layout,
    main: Option<Win>,
    detached: Vec<(PanelId, Win)>,
    /// Which panel the left button is down on, if any.
    drag: DragTracker,
    /// Follows the pointer while a panel is dragged outside its window.
    ghost: Option<(PanelId, Win)>,
    /// The test window for the toolkit's controls (`--controls`); not part of the dock.
    controls: Option<ControlsWindow>,
    started: Instant,
}

struct ControlsWindow {
    win: Win,
    panel: controls::Panel,
}

impl ControlsWindow {
    fn plan(&self) -> controls::Plan {
        let (w, h, scale) = self.win.metrics();
        controls::plan(w, h, scale)
    }

    fn redraw(&mut self) {
        let plan = self.plan();
        let panel = &self.panel;
        self.win
            .present(|canvas, _| controls_render::draw(canvas, panel, &plan));
    }

    fn changed(&mut self, changed: bool) {
        if changed {
            self.win.mark_full();
            self.win.window.request_redraw();
        }
    }

    fn event(&mut self, event: &WindowEvent, now_ms: u64) {
        match event {
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => self.changed(true),
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::CursorMoved { position, .. } => {
                let pos = (position.x as f32, position.y as f32);
                self.win.cursor = Some(pos);
                let plan = self.plan();
                let changed = self.panel.pointer_moved(&plan, pos);
                self.changed(changed);
            }
            WindowEvent::CursorLeft { .. } => {
                // While a button is held the last position is kept (the drag goes on).
                let changed = self.panel.pointer_left();
                self.changed(changed);
            }
            WindowEvent::ModifiersChanged(m) => {
                let fine = m.state().contains(ModifiersState::SHIFT);
                self.panel.set_fine(fine);
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                let Some(pos) = self.win.cursor else {
                    return;
                };
                let plan = self.plan();
                let changed = match state {
                    ElementState::Pressed => self.panel.press(&plan, pos, now_ms),
                    ElementState::Released => self.panel.release(&plan, pos),
                };
                self.changed(changed);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let Some(pos) = self.win.cursor else {
                    return;
                };
                let plan = self.plan();
                let changed = match delta {
                    MouseScrollDelta::LineDelta(_, y) => self.panel.wheel(&plan, pos, *y),
                    MouseScrollDelta::PixelDelta(p) => {
                        self.panel.wheel_pixels(&plan, pos, p.y as f32)
                    }
                };
                self.changed(changed);
            }
            _ => {}
        }
    }
}

impl App {
    fn new() -> Self {
        Self {
            layout: Layout::new(panels::ALL),
            main: None,
            detached: Vec::new(),
            drag: DragTracker::new(),
            ghost: None,
            controls: None,
            started: Instant::now(),
        }
    }

    fn target_of(&self, id: WindowId) -> Option<Target> {
        if self.main.as_ref().is_some_and(|w| w.window.id() == id) {
            return Some(Target::Main);
        }
        if self
            .ghost
            .as_ref()
            .is_some_and(|(_, w)| w.window.id() == id)
        {
            return Some(Target::Ghost);
        }
        self.detached
            .iter()
            .find(|(_, w)| w.window.id() == id)
            .map(|(panel, _)| Target::Detached(*panel))
    }

    fn win_mut(&mut self, target: Target) -> Option<&mut Win> {
        match target {
            Target::Main => self.main.as_mut(),
            Target::Ghost => self.ghost.as_mut().map(|(_, w)| w),
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
                let dragging = self.drag.dragging();
                let Some(win) = self.main.as_mut() else {
                    return;
                };
                let cursor = win.cursor;
                win.present(|canvas, scale| {
                    let hover = cursor.and_then(|point| {
                        ui::detach_button_at(point, canvas.width() as f32, scale, &docked)
                    });
                    render::draw_main(canvas, scale, &docked, hover, dragging);
                });
            }
            Target::Detached(panel) => {
                let dragging = self.drag.dragging() == Some(panel);
                let Some(win) = self.win_mut(target) else {
                    return;
                };
                let cursor = win.cursor;
                win.present(|canvas, scale| {
                    let (w, h) = (canvas.width() as f32, canvas.height() as f32);
                    let hover = cursor.is_some_and(|p| ui::dock_back_button_at(p, w, h, scale));
                    render::draw_detached(canvas, scale, panel, hover, dragging);
                });
            }
            Target::Ghost => {
                let Some((panel, win)) = self.ghost.as_mut() else {
                    return;
                };
                let panel = *panel;
                let see_through = win.see_through();
                win.present(|canvas, scale| render::draw_ghost(canvas, scale, panel, see_through));
            }
        }
    }

    /// Redraw the whole of a window.
    fn request_redraw(&mut self, target: Target) {
        if let Some(win) = self.win_mut(target) {
            win.mark_full();
            win.window.request_redraw();
        }
    }

    /// Redraw only `rect` of a window (pixels).
    fn request_redraw_rect(&mut self, target: Target, rect: Rect) {
        if let Some(win) = self.win_mut(target) {
            win.mark_damage(rect);
            win.window.request_redraw();
        }
    }

    /// The panel whose button the pointer is over in `target`: a docked panel's "detach"
    /// button in the main window, or the window's own panel for its "dock back" button.
    fn hover_of(&self, target: Target) -> Option<PanelId> {
        match target {
            Target::Main => {
                let win = self.main.as_ref()?;
                let (width, _, scale) = win.metrics();
                ui::detach_button_at(win.cursor?, width, scale, self.layout.docked())
            }
            Target::Detached(panel) => {
                let (_, win) = self.detached.iter().find(|(p, _)| *p == panel)?;
                let (width, height, scale) = win.metrics();
                ui::dock_back_button_at(win.cursor?, width, height, scale).then_some(panel)
            }
            Target::Ghost => None,
        }
    }

    /// Where the button for `hover` (a result of `hover_of`) is in `target`.
    fn hover_rect(&self, target: Target, hover: Option<PanelId>) -> Option<Rect> {
        let panel = hover?;
        match target {
            Target::Main => {
                let (width, _, scale) = self.main.as_ref()?.metrics();
                ui::detach_button_rect(width, scale, self.layout.docked(), panel)
            }
            Target::Detached(p) => {
                let (_, win) = self.detached.iter().find(|(q, _)| *q == p)?;
                let (width, height, scale) = win.metrics();
                Some(ui::dock_back_button_rect(width, height, scale))
            }
            Target::Ghost => None,
        }
    }

    /// Set the pointer position of `target` (`None` = outside) and redraw only if that
    /// changes which button is highlighted, and then only the buttons involved.
    fn move_pointer(&mut self, target: Target, cursor: Option<(f32, f32)>) {
        let before = self.hover_of(target);
        if let Some(win) = self.win_mut(target) {
            win.cursor = cursor;
        }
        let after = self.hover_of(target);
        if before == after {
            return;
        }
        let changed = [
            self.hover_rect(target, before),
            self.hover_rect(target, after),
        ]
        .into_iter()
        .flatten()
        .reduce(Rect::union);
        match changed {
            Some(rect) => self.request_redraw_rect(target, rect),
            None => self.request_redraw(target),
        }
    }

    /// Keep the ghost under the pointer while a panel is dragged outside its window, and
    /// remove it when the pointer is back inside or there is no drag. It needs the window's
    /// screen position, so there is no ghost where the platform does not give one (Wayland).
    fn update_ghost(&mut self, event_loop: &ActiveEventLoop, target: Target) {
        let Some(panel) = self.drag.dragging() else {
            self.ghost = None;
            return;
        };
        let Some(win) = self.win_mut(target) else {
            return;
        };
        let (Some(pointer), Some(origin)) = (win.cursor, win.screen_origin()) else {
            return;
        };
        let (width, height, scale) = win.metrics();
        if !is_outside_window(pointer, (width, height), DRAG_MARGIN as f32 * scale) {
            self.ghost = None;
            return;
        }
        let size = (GHOST_SIZE[0] as f32 * scale, GHOST_SIZE[1] as f32 * scale);
        // The pointer sits in the middle of the ghost.
        let (x, y) = window_origin_under_cursor(to_screen(origin, pointer), size.0, size.1 / 2.0);
        let position = PhysicalPosition::new(x.round() as i32, y.round() as i32);
        if let Some((_, ghost)) = &self.ghost {
            ghost.window.set_outer_position(position);
            return;
        }
        let attributes = WindowAttributes::default()
            .with_title("")
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(false)
            .with_active(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_inner_size(PhysicalSize::new(
                size.0.round() as u32,
                size.1.round() as u32,
            ))
            .with_position(position);
        #[cfg(windows)]
        let attributes = attributes.with_skip_taskbar(true);
        let ghost = Win::new(event_loop, attributes);
        // The ghost must never take the pointer: the drag belongs to the window it started in.
        let _ = ghost.window.set_cursor_hittest(false);
        ghost.window.request_redraw();
        self.ghost = Some((panel, ghost));
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
        self.ghost = None;
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
        }
        self.request_redraw(Target::Main);
    }

    /// The left button went down in `target`: a button acts at once, a block starts a drag.
    fn press(&mut self, event_loop: &ActiveEventLoop, target: Target) {
        self.press_inner(event_loop, target);
        // A drag has started: dim the block and show a grabbing pointer.
        if self.drag.dragging().is_some() {
            self.set_cursor_icon(target, CursorIcon::Grabbing);
            self.request_redraw(target);
        }
    }

    fn set_cursor_icon(&mut self, target: Target, icon: CursorIcon) {
        if let Some(win) = self.win_mut(target) {
            win.window.set_cursor(icon);
        }
    }

    fn press_inner(&mut self, event_loop: &ActiveEventLoop, target: Target) {
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
            Target::Ghost => {}
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
        self.ghost = None;
        self.finish_drag(event_loop, target);
        // Back to the normal pointer and block (the window may be gone by now).
        self.set_cursor_icon(target, CursorIcon::Default);
        self.request_redraw(target);
    }

    fn finish_drag(&mut self, event_loop: &ActiveEventLoop, target: Target) {
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
        if std::env::args().any(|a| a == "--controls") {
            let attributes = WindowAttributes::default()
                .with_title("Controls (toolkit test)")
                .with_inner_size(LogicalSize::new(320.0, 300.0));
            self.controls = Some(ControlsWindow {
                win: Win::new(event_loop, attributes),
                panel: controls::Panel::new(),
            });
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if self
            .controls
            .as_ref()
            .is_some_and(|c| c.win.window.id() == id)
        {
            if matches!(event, WindowEvent::CloseRequested) {
                self.controls = None;
                return;
            }
            let now_ms = self.started.elapsed().as_millis() as u64;
            if let Some(c) = self.controls.as_mut() {
                c.event(&event, now_ms);
            }
            return;
        }
        let Some(target) = self.target_of(id) else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => match target {
                Target::Main => event_loop.exit(),
                Target::Ghost => {}
                // Closing a detached window puts its panel back in the main window.
                Target::Detached(panel) => self.redock(panel),
            },
            WindowEvent::Resized(_) => self.request_redraw(target),
            WindowEvent::RedrawRequested => self.redraw(target),
            WindowEvent::CursorMoved { position, .. } => {
                self.move_pointer(target, Some((position.x as f32, position.y as f32)));
                self.update_ghost(event_loop, target);
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
                    self.move_pointer(target, None);
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
