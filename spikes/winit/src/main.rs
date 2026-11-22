//! Spike: two windows from one event loop, drawn with tiny-skia and shown with softbuffer.
//! It answers the questions of Gate A in docs/technical/winit-spike-plan.md and prints
//! what it sees to the terminal. It is throwaway code, not part of the dock library.
//!
//! - "borderless": no title bar, transparent corners, moved by dragging its top strip.
//! - "plain": a normal window, to check resizing and the pointer while a button is held.

use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::{Duration, Instant};

use softbuffer::{Context, Surface};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Transform};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

/// Height of the borderless window's drag strip, in points.
const STRIP: f32 = 32.0;
/// How many redraws are averaged for each timing line.
const FRAMES_PER_REPORT: u32 = 60;

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Borderless,
    Plain,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Borderless => "borderless",
            Kind::Plain => "plain",
        }
    }
}

struct Panel {
    kind: Kind,
    window: Rc<Window>,
    surface: Surface<Rc<Window>, Rc<Window>>,
    pixmap: Option<Pixmap>,
    cursor: Option<PhysicalPosition<f64>>,
    /// The left button went down inside the client area and has not been released.
    button_down: bool,
    /// Pointer events seen outside the window while the button was held.
    outside_events: u32,
    frames: u32,
    spent: Duration,
}

impl Panel {
    fn new(event_loop: &ActiveEventLoop, kind: Kind, position: (i32, i32)) -> Self {
        let mut attrs = WindowAttributes::default()
            .with_title(format!("spike: {}", kind.name()))
            .with_inner_size(LogicalSize::new(320.0, 420.0))
            .with_position(PhysicalPosition::new(position.0, position.1));
        if kind == Kind::Borderless {
            attrs = attrs.with_decorations(false).with_transparent(true);
        }
        let window = Rc::new(event_loop.create_window(attrs).expect("create window"));
        let context = Context::new(window.clone()).expect("softbuffer context");
        let surface = Surface::new(&context, window.clone()).expect("softbuffer surface");
        Self {
            kind,
            window,
            surface,
            pixmap: None,
            cursor: None,
            button_down: false,
            outside_events: 0,
            frames: 0,
            spent: Duration::ZERO,
        }
    }

    fn is_outside(&self, pos: PhysicalPosition<f64>) -> bool {
        let size = self.window.inner_size();
        pos.x < 0.0 || pos.y < 0.0 || pos.x >= size.width as f64 || pos.y >= size.height as f64
    }

    fn redraw(&mut self) {
        let start = Instant::now();
        let size = self.window.inner_size();
        let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else {
            return; // minimised
        };
        if self.surface.resize(w, h).is_err() {
            return;
        }

        let stale = match &self.pixmap {
            Some(p) => p.width() != w.get() || p.height() != h.get(),
            None => true,
        };
        if stale {
            self.pixmap = Pixmap::new(w.get(), h.get());
        }
        let Some(pixmap) = self.pixmap.as_mut() else {
            return;
        };
        let scale = self.window.scale_factor() as f32;
        draw_scene(pixmap, self.kind, scale, self.cursor);

        let Ok(mut buffer) = self.surface.buffer_mut() else {
            return;
        };
        // tiny-skia: premultiplied RGBA bytes. softbuffer: one u32 per pixel, 0xAARRGGBB
        // (alpha is only meaningful where the platform keeps it; that is one of the questions).
        for (dst, src) in buffer.iter_mut().zip(pixmap.data().chunks_exact(4)) {
            let (r, g, b, a) = (src[0] as u32, src[1] as u32, src[2] as u32, src[3] as u32);
            *dst = (a << 24) | (r << 16) | (g << 8) | b;
        }
        self.window.pre_present_notify();
        if buffer.present().is_err() {
            return;
        }

        self.spent += start.elapsed();
        self.frames += 1;
        if self.frames == FRAMES_PER_REPORT {
            let avg = self.spent.as_secs_f64() * 1000.0 / self.frames as f64;
            eprintln!(
                "[{}] {}x{} px: average {:.2} ms per redraw (draw + copy + present) over {} redraws",
                self.kind.name(),
                w,
                h,
                avg,
                self.frames
            );
            self.frames = 0;
            self.spent = Duration::ZERO;
        }
    }

    fn cursor_moved(&mut self, pos: PhysicalPosition<f64>) {
        self.cursor = Some(pos);
        if self.button_down && self.is_outside(pos) {
            self.outside_events += 1;
            if self.outside_events <= 5 || self.outside_events % 50 == 0 {
                eprintln!(
                    "[{}] pointer event outside the window while the button is held (#{}): x={:.0} y={:.0}",
                    self.kind.name(),
                    self.outside_events,
                    pos.x,
                    pos.y
                );
            }
        }
        self.window.request_redraw();
    }

    fn mouse_button(&mut self, state: ElementState) {
        match state {
            ElementState::Pressed => {
                let Some(pos) = self.cursor else { return };
                let strip = (STRIP * self.window.scale_factor() as f32) as f64;
                if self.kind == Kind::Borderless && pos.y < strip {
                    // The OS takes over and moves the window; we get no release afterwards.
                    if let Err(e) = self.window.drag_window() {
                        eprintln!("[borderless] drag_window failed: {e}");
                    }
                } else {
                    self.button_down = true;
                    self.outside_events = 0;
                    eprintln!(
                        "[{}] button down at x={:.0} y={:.0}",
                        self.kind.name(),
                        pos.x,
                        pos.y
                    );
                }
            }
            ElementState::Released => {
                if !self.button_down {
                    return;
                }
                self.button_down = false;
                let Some(pos) = self.cursor else { return };
                let outside = self.is_outside(pos);
                let screen = self
                    .window
                    .inner_position()
                    .map(|p| format!("{:.0}, {:.0}", p.x as f64 + pos.x, p.y as f64 + pos.y))
                    .unwrap_or_else(|_| "unknown".into());
                eprintln!(
                    "[{}] button up at x={:.0} y={:.0}: {} the window; screen position {}; {} outside events seen",
                    self.kind.name(),
                    pos.x,
                    pos.y,
                    if outside { "OUTSIDE" } else { "inside" },
                    screen,
                    self.outside_events
                );
            }
        }
    }
}

/// A filled rounded rectangle (x, y, width, height and radius in pixels).
fn rounded_rect(pm: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, r: f32, rgba: [u8; 4]) {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]);
        paint.anti_alias = true;
        pm.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

/// A small panel-like scene: background, title strip, buttons, a slider that follows the cursor.
fn draw_scene(pm: &mut Pixmap, kind: Kind, scale: f32, cursor: Option<PhysicalPosition<f64>>) {
    let (w, h) = (pm.width() as f32, pm.height() as f32);
    let s = scale;
    match kind {
        Kind::Borderless => {
            // Fully transparent first, so the rounded corners show the desktop through.
            pm.fill(Color::TRANSPARENT);
            rounded_rect(pm, 0.0, 0.0, w, h, 16.0 * s, [38, 42, 52, 255]);
            rounded_rect(pm, 0.0, 0.0, w, STRIP * s, 16.0 * s, [64, 110, 190, 255]);
        }
        Kind::Plain => {
            pm.fill(Color::from_rgba8(38, 42, 52, 255));
            rounded_rect(pm, 0.0, 0.0, w, STRIP * s, 0.0, [64, 110, 190, 255]);
        }
    }

    // A grid of buttons gives the renderer something to do.
    let (cols, rows) = (3, 6);
    let cell_w = (w - 24.0 * s) / cols as f32;
    let cell_h = 40.0 * s;
    for row in 0..rows {
        for col in 0..cols {
            let x = 12.0 * s + col as f32 * cell_w;
            let y = (STRIP + 16.0) * s + row as f32 * (cell_h + 8.0 * s);
            let shade = 70 + ((row * cols + col) * 7) as u8;
            rounded_rect(
                pm,
                x,
                y,
                cell_w - 8.0 * s,
                cell_h,
                8.0 * s,
                [shade, shade + 20, shade + 40, 255],
            );
        }
    }

    // A slider whose knob follows the pointer, so moving the mouse redraws something visible.
    let track_y = h - 36.0 * s;
    rounded_rect(
        pm,
        12.0 * s,
        track_y,
        w - 24.0 * s,
        8.0 * s,
        4.0 * s,
        [20, 22, 28, 255],
    );
    let knob_x = cursor
        .map(|c| (c.x as f32).clamp(12.0 * s, w - 12.0 * s))
        .unwrap_or(w / 2.0);
    rounded_rect(
        pm,
        knob_x - 8.0 * s,
        track_y - 6.0 * s,
        16.0 * s,
        20.0 * s,
        6.0 * s,
        [240, 200, 80, 255],
    );
}

struct App {
    panels: Vec<Panel>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.panels.is_empty() {
            return;
        }
        event_loop.set_control_flow(ControlFlow::Wait);
        self.panels
            .push(Panel::new(event_loop, Kind::Borderless, (120, 120)));
        self.panels
            .push(Panel::new(event_loop, Kind::Plain, (520, 120)));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(panel) = self.panels.iter_mut().find(|p| p.window.id() == id) else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) => panel.window.request_redraw(),
            WindowEvent::RedrawRequested => panel.redraw(),
            WindowEvent::CursorMoved { position, .. } => panel.cursor_moved(position),
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => panel.mouse_button(state),
            _ => {}
        }
    }
}

/// How long drawing the scene takes with no window involved: a typical panel and a full-HD clear.
fn bench_drawing() {
    let mut panel = Pixmap::new(320, 420).expect("pixmap");
    let start = Instant::now();
    let runs = 200;
    for _ in 0..runs {
        draw_scene(&mut panel, Kind::Plain, 1.0, None);
    }
    eprintln!(
        "[bench] panel scene 320x420: {:.2} ms per draw (no window, no copy)",
        start.elapsed().as_secs_f64() * 1000.0 / runs as f64
    );

    let mut full = Pixmap::new(1920, 1080).expect("pixmap");
    let start = Instant::now();
    for _ in 0..runs {
        full.fill(Color::from_rgba8(38, 42, 52, 255));
    }
    eprintln!(
        "[bench] clear 1920x1080: {:.2} ms per clear",
        start.elapsed().as_secs_f64() * 1000.0 / runs as f64
    );
}

fn main() {
    bench_drawing();
    eprintln!("Drag the top strip of the borderless window to move it.");
    eprintln!("In either window: press the left button, move out of the window, release. Watch the lines below.");
    eprintln!(
        "Resize the plain window by its edges and watch for flicker. Close a window to quit."
    );
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App { panels: Vec::new() };
    event_loop.run_app(&mut app).expect("run");
}
