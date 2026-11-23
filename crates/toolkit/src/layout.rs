//! Rows and columns. Sizes are written in points (what a person calls "pixels at 100%
//! scaling"); the layout multiplies them by the window's scale factor and snaps every
//! rectangle to whole pixels, so edges are crisp at 125% and 150% and a click is tested
//! against exactly what was drawn. Pure arithmetic: the area is in pixels, the answer too.

use crate::geometry::Rect;

/// Which way a stack runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// Items side by side, left to right.
    Horizontal,
    /// Items one below the other, top to bottom.
    Vertical,
}

/// Space around the inside of a stack, in points.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Insets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Insets {
    /// The same space on every side.
    pub fn all(points: f32) -> Self {
        Self {
            left: points,
            top: points,
            right: points,
            bottom: points,
        }
    }
}

/// How much of the stack's direction an item takes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    /// This many points.
    Fixed(f32),
    /// A share of what the fixed items and gaps leave over, in proportion to the weights.
    Fill(f32),
}

/// How an item sits across the stack's direction (its height in a row, its width in a column).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cross {
    /// As big as the stack's inside.
    Stretch,
    /// This many points, at the start (left or top).
    Start(f32),
    /// This many points, in the middle.
    Center(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Item {
    pub main: Size,
    pub cross: Cross,
}

impl Item {
    pub fn new(main: Size, cross: Cross) -> Self {
        Self { main, cross }
    }

    /// `main` along the stack, stretched across it.
    pub fn stretch(main: Size) -> Self {
        Self::new(main, Cross::Stretch)
    }
}

/// A row or a column of items, with a gap between them and space around them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stack {
    pub axis: Axis,
    /// Between neighbouring items, in points.
    pub gap: f32,
    pub insets: Insets,
}

impl Stack {
    pub fn row() -> Self {
        Self::new(Axis::Horizontal)
    }

    pub fn column() -> Self {
        Self::new(Axis::Vertical)
    }

    fn new(axis: Axis) -> Self {
        Self {
            axis,
            gap: 0.0,
            insets: Insets::default(),
        }
    }

    pub fn gap(mut self, points: f32) -> Self {
        self.gap = points;
        self
    }

    pub fn insets(mut self, insets: Insets) -> Self {
        self.insets = insets;
        self
    }

    /// The rectangle of each item, in pixels, inside `area` (in pixels), for a window of
    /// scale factor `scale`.
    ///
    /// - Fixed items and gaps keep their size. If they do not fit, they overflow and `Fill`
    ///   items get nothing: nothing is squeezed.
    /// - `Fill` items share what is left in proportion to their weights.
    /// - Nothing has a negative size, whatever the area.
    pub fn layout(&self, area: Rect, scale: f32, items: &[Item]) -> Vec<Rect> {
        let inner = self.inner(area, scale);
        let (main_len, cross_len) = match self.axis {
            Axis::Horizontal => (inner.w, inner.h),
            Axis::Vertical => (inner.h, inner.w),
        };
        let gap = self.gap * scale;
        let gaps = gap * items.len().saturating_sub(1) as f32;
        let fixed: f32 = items
            .iter()
            .map(|item| match item.main {
                Size::Fixed(points) => points.max(0.0) * scale,
                Size::Fill(_) => 0.0,
            })
            .sum();
        let weights: f32 = items
            .iter()
            .map(|item| match item.main {
                Size::Fill(weight) => weight.max(0.0),
                Size::Fixed(_) => 0.0,
            })
            .sum();
        let left_over = (main_len - fixed - gaps).max(0.0);

        let (mut at, cross_start) = match self.axis {
            Axis::Horizontal => (inner.x, inner.y),
            Axis::Vertical => (inner.y, inner.x),
        };
        items
            .iter()
            .map(|item| {
                let main = match item.main {
                    Size::Fixed(points) => points.max(0.0) * scale,
                    Size::Fill(weight) if weights > 0.0 => left_over * weight.max(0.0) / weights,
                    Size::Fill(_) => 0.0,
                };
                let (cross_at, cross) = match item.cross {
                    Cross::Stretch => (cross_start, cross_len),
                    Cross::Start(points) => (cross_start, points.max(0.0) * scale),
                    Cross::Center(points) => {
                        let size = points.max(0.0) * scale;
                        (cross_start + (cross_len - size) / 2.0, size)
                    }
                };
                let rect = match self.axis {
                    Axis::Horizontal => Rect::new(at, cross_at, main, cross),
                    Axis::Vertical => Rect::new(cross_at, at, cross, main),
                };
                at += main + gap;
                rect.snapped()
            })
            .collect()
    }

    /// `area` with the insets taken off, never smaller than nothing.
    fn inner(&self, area: Rect, scale: f32) -> Rect {
        let Insets {
            left,
            top,
            right,
            bottom,
        } = self.insets;
        Rect {
            x: area.x + left * scale,
            y: area.y + top * scale,
            w: (area.w - (left + right) * scale).max(0.0),
            h: (area.h - (top + bottom) * scale).max(0.0),
        }
    }
}
