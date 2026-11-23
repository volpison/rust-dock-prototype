//! One OS window with its softbuffer surface and a tiny-skia canvas.
//! The same calls as the spike (see spikes/winit), which ran on Windows.

use std::num::NonZeroU32;
use std::rc::Rc;

use softbuffer::{Context, Surface};
use std::time::Instant;
use tiny_skia::Pixmap;

use crate::ui::Rect;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

pub struct Win {
    pub window: Rc<Window>,
    surface: Surface<Rc<Window>, Rc<Window>>,
    pixmap: Option<Pixmap>,
    /// Where the pointer is inside this window, in pixels; `None` when it is outside.
    pub cursor: Option<(f32, f32)>,
    /// The whole window must be redrawn and shown (first frame, resize, anything not narrowed).
    full: bool,
    /// The part that changed since the last frame, when only part did.
    damage: Option<Rect>,
    /// Print how long each frame took (set `DOCK_TRACE=1`).
    trace: bool,
}

impl Win {
    pub fn new(event_loop: &ActiveEventLoop, attributes: WindowAttributes) -> Self {
        let window = Rc::new(event_loop.create_window(attributes).expect("create window"));
        let context = Context::new(window.clone()).expect("softbuffer context");
        let surface = Surface::new(&context, window.clone()).expect("softbuffer surface");
        Self {
            window,
            surface,
            pixmap: None,
            cursor: None,
            full: true,
            damage: None,
            trace: std::env::var_os("DOCK_TRACE").is_some(),
        }
    }

    /// The window's size in pixels and its scale factor.
    pub fn metrics(&self) -> (f32, f32, f32) {
        let size = self.window.inner_size();
        (
            size.width as f32,
            size.height as f32,
            self.window.scale_factor() as f32,
        )
    }

    /// The window's top-left corner (inside the frame) on the screen, in pixels. `None` when
    /// the platform does not say (Wayland).
    pub fn screen_origin(&self) -> Option<(f32, f32)> {
        let position = self.window.inner_position().ok()?;
        Some((position.x as f32, position.y as f32))
    }

    /// The next frame must redraw and show the whole window.
    pub fn mark_full(&mut self) {
        self.full = true;
        self.damage = None;
    }

    /// Only `rect` (in pixels) changed. Joined with what is already waiting; ignored when a
    /// whole-window frame is already waiting.
    pub fn mark_damage(&mut self, rect: Rect) {
        if !self.full {
            self.damage = Some(self.damage.map_or(rect, |old| old.union(rect)));
        }
    }

    /// Draws a frame with `draw(canvas, scale_factor)` and shows it.
    /// When only a part changed (`mark_damage`), softbuffer is told which part, so the system
    /// only has to update that part of the screen. The whole pixmap is still copied into the
    /// buffer, because the buffer may hold an older frame.
    pub fn present(&mut self, draw: impl FnOnce(&mut Pixmap, f32)) {
        let started = Instant::now();
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
        let damage = self.damage.take().filter(|_| !self.full && !stale);
        self.full = false;
        let Some(pixmap) = self.pixmap.as_mut() else {
            return;
        };
        draw(pixmap, self.window.scale_factor() as f32);

        let Ok(mut buffer) = self.surface.buffer_mut() else {
            return;
        };
        for (dst, src) in buffer.iter_mut().zip(pixmap.data().chunks_exact(4)) {
            *dst = to_softbuffer(src);
        }
        self.window.pre_present_notify();
        let part = damage.and_then(|rect| rect.to_pixels(size.width, size.height));
        let shown = match part {
            Some((x, y, width, height)) => {
                match (NonZeroU32::new(width), NonZeroU32::new(height)) {
                    (Some(width), Some(height)) => {
                        buffer.present_with_damage(&[softbuffer::Rect {
                            x,
                            y,
                            width,
                            height,
                        }])
                    }
                    _ => buffer.present(),
                }
            }
            None => buffer.present(),
        };
        let _ = shown;
        if self.trace {
            eprintln!(
                "frame {:.2} ms, {}",
                started.elapsed().as_secs_f64() * 1000.0,
                match part {
                    Some((_, _, w, h)) => format!("changed part {w}x{h}"),
                    None => "whole window".to_string(),
                }
            );
        }
    }
}

/// Turns one tiny-skia pixel (premultiplied red, green, blue, alpha bytes) into the `u32`
/// softbuffer wants. This is the one place that knows the pixel format, so a platform
/// that needs a different one only has to change this function.
///
/// softbuffer 0.4.8 documents the format as 0RGB: the top byte is unused. All windows here
/// are opaque, so alpha is dropped. See-through windows will need their own path (pull
/// request 7 of the stage 2 plan); the spike (`spikes/winit`) put alpha in the top byte.
fn to_softbuffer(pixel: &[u8]) -> u32 {
    let (r, g, b) = (pixel[0] as u32, pixel[1] as u32, pixel[2] as u32);
    (r << 16) | (g << 8) | b
}
