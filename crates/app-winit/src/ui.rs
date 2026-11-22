//! Where things are inside a window, in physical pixels. Pure arithmetic: no winit and no
//! drawing, so it is tested on its own. The drawing (`render.rs`) and the click handling
//! (`main.rs`) both ask this file, so what you see is what you can click.
//!
//! Sizes below are in points (what a person would call "pixels at 100% scaling");
//! every function multiplies them by the window's scale factor.

use dock::geometry::is_over_window;
use dock::layout::PanelId;

const MARGIN: f32 = 12.0;
const ROW_HEIGHT: f32 = 72.0;
const ROW_GAP: f32 = 10.0;
const BUTTON_WIDTH: f32 = 96.0;
const BUTTON_HEIGHT: f32 = 44.0;

/// A rectangle: top-left corner, width and height.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    /// True when `point` (relative to the window's top-left corner) is inside the rectangle.
    pub fn contains(&self, point: (f32, f32)) -> bool {
        is_over_window(point, (self.x, self.y), (self.w, self.h))
    }
}

/// One docked panel in the main window: its block and its "detach" button.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Row {
    pub panel: PanelId,
    pub area: Rect,
    pub button: Rect,
}

/// The docked panels, one row each, from the top. `width` is the window's width in pixels.
pub fn main_rows(width: f32, scale: f32, docked: &[PanelId]) -> Vec<Row> {
    let (margin, row_h, gap) = (MARGIN * scale, ROW_HEIGHT * scale, ROW_GAP * scale);
    let (button_w, button_h) = (BUTTON_WIDTH * scale, BUTTON_HEIGHT * scale);
    docked
        .iter()
        .enumerate()
        .map(|(i, panel)| {
            let area = Rect {
                x: margin,
                y: margin + i as f32 * (row_h + gap),
                w: (width - 2.0 * margin).max(0.0),
                h: row_h,
            };
            let button = Rect {
                x: area.x + area.w - button_w - margin,
                y: area.y + (row_h - button_h) / 2.0,
                w: button_w,
                h: button_h,
            };
            Row {
                panel: *panel,
                area,
                button,
            }
        })
        .collect()
}

/// A detached window: a "dock back" button at the top and the panel's block below it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetachedView {
    pub button: Rect,
    pub content: Rect,
}

pub fn detached_view(width: f32, height: f32, scale: f32) -> DetachedView {
    let (margin, button_w, button_h) =
        (MARGIN * scale, BUTTON_WIDTH * scale, BUTTON_HEIGHT * scale);
    let button = Rect {
        x: margin,
        y: margin,
        w: button_w,
        h: button_h,
    };
    let top = 2.0 * margin + button_h;
    let content = Rect {
        x: margin,
        y: top,
        w: (width - 2.0 * margin).max(0.0),
        h: (height - top - margin).max(0.0),
    };
    DetachedView { button, content }
}

/// The panel whose "detach" button is under `point`, if any.
pub fn detach_button_at(
    point: (f32, f32),
    width: f32,
    scale: f32,
    docked: &[PanelId],
) -> Option<PanelId> {
    main_rows(width, scale, docked)
        .into_iter()
        .find(|row| row.button.contains(point))
        .map(|row| row.panel)
}

/// True when `point` is over a detached window's "dock back" button.
pub fn dock_back_button_at(point: (f32, f32), width: f32, height: f32, scale: f32) -> bool {
    detached_view(width, height, scale).button.contains(point)
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: PanelId = PanelId(1);
    const B: PanelId = PanelId(2);

    #[test]
    fn rows_are_stacked_from_the_top_with_a_gap() {
        let rows = main_rows(360.0, 1.0, &[A, B]);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].area.y, 12.0);
        assert_eq!(rows[1].area.y, 12.0 + 72.0 + 10.0);
        assert_eq!(rows[0].area.w, 360.0 - 24.0);
        assert_eq!(rows[0].panel, A);
        assert_eq!(rows[1].panel, B);
    }

    #[test]
    fn no_docked_panels_means_no_rows() {
        assert!(main_rows(360.0, 1.0, &[]).is_empty());
    }

    #[test]
    fn the_button_sits_at_the_right_end_of_its_row_and_is_centred_vertically() {
        let row = main_rows(360.0, 1.0, &[A])[0];
        assert_eq!(row.button.x + row.button.w + 12.0, row.area.x + row.area.w);
        let above = row.button.y - row.area.y;
        let below = (row.area.y + row.area.h) - (row.button.y + row.button.h);
        assert_eq!(above, below);
    }

    #[test]
    fn everything_scales_with_the_scale_factor() {
        let one = main_rows(360.0, 1.0, &[A, B]);
        let two = main_rows(720.0, 2.0, &[A, B]);
        for (a, b) in one.iter().zip(two.iter()) {
            assert_eq!(b.area.x, a.area.x * 2.0);
            assert_eq!(b.area.y, a.area.y * 2.0);
            assert_eq!(b.button.w, a.button.w * 2.0);
            assert_eq!(b.button.h, a.button.h * 2.0);
        }
    }

    #[test]
    fn clicking_a_detach_button_finds_its_panel() {
        let rows = main_rows(360.0, 1.0, &[A, B]);
        let on_second = (rows[1].button.x + 5.0, rows[1].button.y + 5.0);
        assert_eq!(detach_button_at(on_second, 360.0, 1.0, &[A, B]), Some(B));
    }

    #[test]
    fn clicking_the_panel_block_outside_the_button_does_nothing() {
        let rows = main_rows(360.0, 1.0, &[A, B]);
        let on_block = (rows[0].area.x + 5.0, rows[0].area.y + 5.0);
        assert_eq!(detach_button_at(on_block, 360.0, 1.0, &[A, B]), None);
        assert_eq!(detach_button_at((0.0, 0.0), 360.0, 1.0, &[A, B]), None);
    }

    #[test]
    fn a_button_is_found_at_a_scaled_position_too() {
        let rows = main_rows(540.0, 1.5, &[A]);
        let inside = (rows[0].button.x + 10.0, rows[0].button.y + 10.0);
        assert_eq!(detach_button_at(inside, 540.0, 1.5, &[A]), Some(A));
    }

    #[test]
    fn the_dock_back_button_is_top_left_and_the_content_is_below_it() {
        let view = detached_view(320.0, 420.0, 1.0);
        assert!(view.button.y + view.button.h < view.content.y);
        assert_eq!(view.content.x, 12.0);
        assert_eq!(view.content.w, 320.0 - 24.0);
        assert_eq!(view.content.y + view.content.h, 420.0 - 12.0);
    }

    #[test]
    fn dock_back_is_hit_only_on_its_button() {
        assert!(dock_back_button_at((20.0, 20.0), 320.0, 420.0, 1.0));
        assert!(!dock_back_button_at((200.0, 20.0), 320.0, 420.0, 1.0));
        assert!(!dock_back_button_at((20.0, 300.0), 320.0, 420.0, 1.0));
    }

    #[test]
    fn a_tiny_window_does_not_produce_negative_sizes() {
        let view = detached_view(10.0, 10.0, 1.0);
        assert!(view.content.w >= 0.0 && view.content.h >= 0.0);
        assert!(main_rows(10.0, 1.0, &[A])[0].area.w >= 0.0);
    }
}
