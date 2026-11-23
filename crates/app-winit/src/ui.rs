//! Where things are inside a window, in physical pixels. Pure arithmetic: no winit and no
//! drawing, so it is tested on its own. The drawing (`render.rs`) and the click handling
//! (`main.rs`) both ask this file, so what you see is what you can click.
//!
//! Sizes below are in points (what a person would call "pixels at 100% scaling");
//! every function multiplies them by the window's scale factor. At scales like 125% or 150%
//! that gives fractions of a pixel, so every rectangle is snapped to whole pixels: edges
//! stay crisp when drawn, and a click is tested against exactly what was drawn.

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
                area: area.snapped(),
                button: button.snapped(),
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
    DetachedView {
        button: button.snapped(),
        content: content.snapped(),
    }
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

/// Where the "detach" button of `panel` is in the main window, if the panel is docked.
pub fn detach_button_rect(
    width: f32,
    scale: f32,
    docked: &[PanelId],
    panel: PanelId,
) -> Option<Rect> {
    main_rows(width, scale, docked)
        .into_iter()
        .find(|row| row.panel == panel)
        .map(|row| row.button)
}

/// Where a detached window's "dock back" button is.
pub fn dock_back_button_rect(width: f32, height: f32, scale: f32) -> Rect {
    detached_view(width, height, scale).button
}

/// True when `point` is over a detached window's "dock back" button.
pub fn dock_back_button_at(point: (f32, f32), width: f32, height: f32, scale: f32) -> bool {
    detached_view(width, height, scale).button.contains(point)
}

/// The panel whose block is under `point`, outside its button: pressing there starts a drag.
pub fn block_at(point: (f32, f32), width: f32, scale: f32, docked: &[PanelId]) -> Option<PanelId> {
    main_rows(width, scale, docked)
        .into_iter()
        .find(|row| row.area.contains(point) && !row.button.contains(point))
        .map(|row| row.panel)
}

/// True when `point` is over a detached window's block: pressing there starts a drag.
pub fn content_at(point: (f32, f32), width: f32, height: f32, scale: f32) -> bool {
    detached_view(width, height, scale).content.contains(point)
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

    /// Scales a person really meets: 100%, 125%, 150%, 175%, 200%, 250%.
    const SCALES: [f32; 6] = [1.0, 1.25, 1.5, 1.75, 2.0, 2.5];

    fn is_whole(rect: Rect) -> bool {
        [rect.x, rect.y, rect.w, rect.h]
            .iter()
            .all(|v| *v == v.round())
    }

    #[test]
    fn snapping_moves_each_edge_to_a_whole_pixel() {
        let rect = Rect {
            x: 0.4,
            y: 1.6,
            w: 10.2,
            h: 4.0,
        };
        let snapped = rect.snapped();
        assert_eq!((snapped.x, snapped.y), (0.0, 2.0));
        assert_eq!(snapped.x + snapped.w, (0.4f32 + 10.2).round());
        assert_eq!(snapped.y + snapped.h, (1.6f32 + 4.0).round());
    }

    #[test]
    fn at_every_common_scale_all_rectangles_sit_on_whole_pixels() {
        for scale in SCALES {
            // Odd widths on purpose.
            for width in [361.0, 457.0, 800.0] {
                for row in main_rows(width, scale, &[A, B, PanelId(3)]) {
                    assert!(is_whole(row.area), "area at {scale}: {:?}", row.area);
                    assert!(is_whole(row.button), "button at {scale}: {:?}", row.button);
                }
                let view = detached_view(width, 421.0, scale);
                assert!(is_whole(view.button), "button at {scale}");
                assert!(is_whole(view.content), "content at {scale}");
            }
        }
    }

    #[test]
    fn at_every_common_scale_rows_do_not_overlap_and_buttons_stay_inside_them() {
        for scale in SCALES {
            let rows = main_rows(480.0 * scale, scale, &[A, B, PanelId(3)]);
            for pair in rows.windows(2) {
                let (above, below) = (pair[0].area, pair[1].area);
                assert!(below.y >= above.y + above.h, "overlap at {scale}");
            }
            for row in &rows {
                let (a, b) = (row.area, row.button);
                assert!(b.x >= a.x && b.x + b.w <= a.x + a.w, "button x at {scale}");
                assert!(b.y >= a.y && b.y + b.h <= a.y + a.h, "button y at {scale}");
            }
        }
    }

    #[test]
    fn a_click_is_tested_against_exactly_the_snapped_button() {
        for scale in SCALES {
            let width = 480.0 * scale;
            let button = main_rows(width, scale, &[A])[0].button;
            let inside = (button.x, button.y);
            let last_inside = (button.x + button.w, button.y + button.h);
            let outside = (button.x - 1.0, button.y);
            assert_eq!(detach_button_at(inside, width, scale, &[A]), Some(A));
            assert_eq!(detach_button_at(last_inside, width, scale, &[A]), Some(A));
            assert_eq!(detach_button_at(outside, width, scale, &[A]), None);
        }
    }

    #[test]
    fn a_tiny_window_does_not_produce_negative_sizes() {
        let view = detached_view(10.0, 10.0, 1.0);
        assert!(view.content.w >= 0.0 && view.content.h >= 0.0);
        assert!(main_rows(10.0, 1.0, &[A])[0].area.w >= 0.0);
    }

    #[test]
    fn pressing_a_block_outside_its_button_finds_its_panel() {
        let rows = main_rows(360.0, 1.0, &[A, B]);
        let on_block = (rows[1].area.x + 5.0, rows[1].area.y + 5.0);
        assert_eq!(block_at(on_block, 360.0, 1.0, &[A, B]), Some(B));
    }

    #[test]
    fn the_button_and_the_gaps_are_not_blocks() {
        let rows = main_rows(360.0, 1.0, &[A, B]);
        let on_button = (rows[0].button.x + 5.0, rows[0].button.y + 5.0);
        assert_eq!(block_at(on_button, 360.0, 1.0, &[A, B]), None);
        let in_gap = (rows[0].area.x + 5.0, rows[0].area.y + rows[0].area.h + 5.0);
        assert_eq!(block_at(in_gap, 360.0, 1.0, &[A, B]), None);
    }

    #[test]
    fn the_detached_block_is_below_the_dock_back_button() {
        assert!(content_at((100.0, 200.0), 320.0, 420.0, 1.0));
        assert!(!content_at((20.0, 20.0), 320.0, 420.0, 1.0));
        assert!(!content_at((2.0, 200.0), 320.0, 420.0, 1.0));
    }

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x, y, w, h }
    }

    #[test]
    fn the_union_holds_both_rectangles() {
        let u = rect(10.0, 10.0, 20.0, 20.0).union(rect(50.0, 5.0, 10.0, 10.0));
        assert_eq!(u, rect(10.0, 5.0, 50.0, 25.0));
        let a = rect(1.0, 2.0, 3.0, 4.0);
        assert_eq!(a.union(a), a);
    }

    #[test]
    fn damage_pixels_grow_by_one_and_stay_inside_the_window() {
        assert_eq!(
            rect(10.0, 10.0, 20.0, 20.0).to_pixels(100, 100),
            Some((9, 9, 22, 22))
        );
        assert_eq!(
            rect(0.0, 0.0, 20.0, 20.0).to_pixels(100, 100),
            Some((0, 0, 21, 21))
        );
        assert_eq!(
            rect(90.0, 90.0, 20.0, 20.0).to_pixels(100, 100),
            Some((89, 89, 11, 11))
        );
    }

    #[test]
    fn damage_outside_the_window_is_nothing() {
        assert_eq!(rect(200.0, 10.0, 20.0, 20.0).to_pixels(100, 100), None);
        assert_eq!(rect(10.0, 10.0, 0.0, 0.0).to_pixels(0, 0), None);
    }

    #[test]
    fn the_button_rectangles_are_the_ones_hit_testing_uses() {
        let rows = main_rows(360.0, 1.0, &[A, B]);
        assert_eq!(
            detach_button_rect(360.0, 1.0, &[A, B], B),
            Some(rows[1].button)
        );
        assert_eq!(detach_button_rect(360.0, 1.0, &[A], B), None);
        assert_eq!(
            dock_back_button_rect(320.0, 420.0, 1.0),
            detached_view(320.0, 420.0, 1.0).button
        );
    }
}
