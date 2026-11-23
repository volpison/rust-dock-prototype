//! Rectangles in a window, in pixels.

/// A rectangle: top-left corner, width and height.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// True when `point` (relative to the window's top-left corner) is inside the rectangle.
    /// The edges count as inside, so a click is tested against exactly what was drawn.
    pub fn contains(&self, point: (f32, f32)) -> bool {
        point.0 >= self.x
            && point.0 <= self.x + self.w
            && point.1 >= self.y
            && point.1 <= self.y + self.h
    }

    /// The smallest rectangle that holds both.
    pub fn union(self, other: Rect) -> Rect {
        let (x0, y0) = (self.x.min(other.x), self.y.min(other.y));
        let x1 = (self.x + self.w).max(other.x + other.w);
        let y1 = (self.y + self.h).max(other.y + other.h);
        Rect {
            x: x0,
            y: y0,
            w: x1 - x0,
            h: y1 - y0,
        }
    }

    /// The rectangle as whole pixels `(x, y, width, height)` inside a window of
    /// `width` x `height` pixels, grown by one pixel on each side (anti-aliased edges) and
    /// cut to the window. `None` when nothing of it is inside the window.
    pub fn to_pixels(self, width: u32, height: u32) -> Option<(u32, u32, u32, u32)> {
        let x0 = (self.x.floor() - 1.0).max(0.0);
        let y0 = (self.y.floor() - 1.0).max(0.0);
        let x1 = ((self.x + self.w).ceil() + 1.0).min(width as f32);
        let y1 = ((self.y + self.h).ceil() + 1.0).min(height as f32);
        if x1 <= x0 || y1 <= y0 {
            return None;
        }
        Some((x0 as u32, y0 as u32, (x1 - x0) as u32, (y1 - y0) as u32))
    }

    /// The same rectangle with its edges moved to whole pixels. The width and height come
    /// from the rounded edges, so two rectangles that touched before still touch.
    pub fn snapped(self) -> Rect {
        let (x0, y0) = (self.x.round(), self.y.round());
        let (x1, y1) = ((self.x + self.w).round(), (self.y + self.h).round());
        Rect {
            x: x0,
            y: y0,
            w: x1 - x0,
            h: y1 - y0,
        }
    }
}
