//! A 7-segment number display, like a calculator's. No font: a number becomes a handful of
//! rectangles (the lit bars, and the unlit ones to draw dimly behind them), so it can be drawn
//! at any size and in any colour. Pure arithmetic.
//!
//! ```text
//!    A
//!   ---
//! F|   |B
//!   -G-
//! E|   |C
//!   ---
//!    D
//! ```
//! Characters: the digits 0 to 9, `-` (a minus), `.` (a dot) and ` ` (a blank cell, drawn as
//! all-unlit bars). Anything else is drawn as a blank cell.

use crate::geometry::Rect;

pub const A: u8 = 1 << 0;
pub const B: u8 = 1 << 1;
pub const C: u8 = 1 << 2;
pub const D: u8 = 1 << 3;
pub const E: u8 = 1 << 4;
pub const F: u8 = 1 << 5;
pub const G: u8 = 1 << 6;

/// Which of the seven bars are lit for `c`. Bit 0 is A and bit 6 is G (see the constants).
/// `.` has no bars (it is a dot of its own), and unknown characters are blank.
pub fn segments(c: char) -> u8 {
    match c {
        '0' => A | B | C | D | E | F,
        '1' => B | C,
        '2' => A | B | D | E | G,
        '3' => A | B | C | D | G,
        '4' => B | C | F | G,
        '5' => A | C | D | F | G,
        '6' => A | C | D | E | F | G,
        '7' => A | B | C,
        '8' => A | B | C | D | E | F | G,
        '9' => A | B | C | D | F | G,
        '-' => G,
        _ => 0,
    }
}

/// `value` as the text a display shows: `decimals` places, no "-0", and `---` for a number
/// that is not a number (NaN or infinite).
pub fn format_value(value: f32, decimals: usize) -> String {
    if !value.is_finite() {
        return "---".to_string();
    }
    let text = format!("{value:.decimals$}");
    // A small negative that rounds to zero prints as "-0.0"; show it as zero.
    match text.strip_prefix('-') {
        Some(rest) if rest.chars().all(|c| c == '0' || c == '.') => rest.to_string(),
        _ => text,
    }
}

/// Where the text sits in the area, along the line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
}

/// The size of a display, from the height of its digits in points; the other sizes follow
/// from it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Height of a digit, in points.
    pub height: f32,
}

impl Style {
    pub fn new(height: f32) -> Self {
        Self { height }
    }

    /// Everything below is in points.
    fn digit_width(&self) -> f32 {
        self.height * 0.55
    }

    fn thickness(&self) -> f32 {
        self.height * 0.14
    }

    /// Between neighbouring cells.
    fn spacing(&self) -> f32 {
        self.height * 0.14
    }

    /// Shaved off each bar so neighbouring bars do not touch.
    fn gap(&self) -> f32 {
        self.height * 0.03
    }

    fn cell_width(&self, c: char) -> f32 {
        if c == '.' {
            self.thickness()
        } else {
            self.digit_width()
        }
    }

    /// The width of `text` in pixels at `scale`, rounded up to a whole pixel, so a layout can
    /// give the display exactly that much room.
    pub fn width(&self, text: &str, scale: f32) -> f32 {
        let cells = text.chars().count();
        if cells == 0 {
            return 0.0;
        }
        let widths: f32 = text.chars().map(|c| self.cell_width(c)).sum();
        ((widths + self.spacing() * (cells - 1) as f32) * scale).ceil()
    }
}

/// The rectangles to draw.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Bars {
    /// The bars and dots that are on.
    pub lit: Vec<Rect>,
    /// The bars that are off: draw them dimly behind the lit ones, as a real display does.
    pub unlit: Vec<Rect>,
}

/// The bars for `text` inside `area` (pixels) at `scale`, with the digits centred up and down
/// and the text placed along the area by `align`. If the text is wider than the area it
/// overflows on the side opposite to the alignment; nothing is cut.
pub fn layout(text: &str, area: Rect, scale: f32, align: Align, style: Style) -> Bars {
    let (h, w, t) = (
        style.height * scale,
        style.digit_width() * scale,
        style.thickness() * scale,
    );
    let (spacing, inset) = (style.spacing() * scale, style.gap() * scale / 2.0);
    let total = style.width(text, scale);
    let mut x = match align {
        Align::Start => area.x,
        Align::Center => area.x + (area.w - total) / 2.0,
        Align::End => area.x + area.w - total,
    };
    let y = area.y + (area.h - h) / 2.0;

    let mut bars = Bars::default();
    for c in text.chars() {
        if c == '.' {
            // A square dot on the baseline.
            bars.lit.push(bar(Rect::new(x, y + h - t, t, t), 0.0));
            x += t + spacing;
            continue;
        }
        let lit = segments(c);
        for (bit, rect) in digit_bars(x, y, w, h, t) {
            let target = if lit & bit != 0 {
                &mut bars.lit
            } else {
                &mut bars.unlit
            };
            target.push(bar(rect, inset));
        }
        x += w + spacing;
    }
    bars
}

/// The seven bars of a digit cell at (`x`, `y`), `w` wide and `h` tall, `t` thick.
fn digit_bars(x: f32, y: f32, w: f32, h: f32, t: f32) -> [(u8, Rect); 7] {
    let side = ((h - 3.0 * t) / 2.0).max(0.0);
    let middle = y + (h - t) / 2.0;
    let across = (w - 2.0 * t).max(0.0);
    [
        (A, Rect::new(x + t, y, across, t)),
        (B, Rect::new(x + w - t, y + t, t, side)),
        (C, Rect::new(x + w - t, middle + t, t, side)),
        (D, Rect::new(x + t, y + h - t, across, t)),
        (E, Rect::new(x, middle + t, t, side)),
        (F, Rect::new(x, y + t, t, side)),
        (G, Rect::new(x + t, middle, across, t)),
    ]
}

/// A bar shrunk by `inset` on every side, on whole pixels and at least one pixel each way.
fn bar(rect: Rect, inset: f32) -> Rect {
    let shrunk = Rect::new(
        rect.x + inset,
        rect.y + inset,
        (rect.w - 2.0 * inset).max(0.0),
        (rect.h - 2.0 * inset).max(0.0),
    );
    let mut snapped = shrunk.snapped();
    if rect.w > 0.0 && snapped.w < 1.0 {
        snapped.w = 1.0;
    }
    if rect.h > 0.0 && snapped.h < 1.0 {
        snapped.h = 1.0;
    }
    snapped
}
