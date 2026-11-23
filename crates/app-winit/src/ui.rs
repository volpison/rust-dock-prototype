//! Where things are inside a window, in physical pixels. Pure arithmetic: no winit and no
//! drawing, so it is tested on its own. The drawing (`render.rs`) and the click handling
//! (`main.rs`) both ask this file, so what you see is what you can click.
//!
//! Sizes below are in points (what a person would call "pixels at 100% scaling");
//! every function multiplies them by the window's scale factor. At scales like 125% or 150%
//! that gives fractions of a pixel, so every rectangle is snapped to whole pixels: edges
//! stay crisp when drawn, and a click is tested against exactly what was drawn.

use dock::layout::PanelId;
use toolkit::layout::{Cross, Insets, Item, Size, Stack};

pub use toolkit::geometry::Rect;

const MARGIN: f32 = 12.0;
const ROW_HEIGHT: f32 = 72.0;
const ROW_GAP: f32 = 10.0;
const BUTTON_WIDTH: f32 = 96.0;
const BUTTON_HEIGHT: f32 = 44.0;

/// One docked panel in the main window: its block and its "detach" button.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Row {
    pub panel: PanelId,
    pub area: Rect,
    pub button: Rect,
}

/// The docked panels, one row each, from the top. `width` is the window's width in pixels.
pub fn main_rows(width: f32, scale: f32, docked: &[PanelId]) -> Vec<Row> {
    let count = docked.len();
    let height =
        (2.0 * MARGIN + count as f32 * ROW_HEIGHT + count.saturating_sub(1) as f32 * ROW_GAP)
            * scale;
    let areas = Stack::column()
        .gap(ROW_GAP)
        .insets(Insets::all(MARGIN))
        .layout(
            Rect::new(0.0, 0.0, width, height),
            scale,
            &vec![Item::stretch(Size::Fixed(ROW_HEIGHT)); count],
        );
    // In each row the button is at the right end, MARGIN from the edge, centred vertically.
    let inside_row = Stack::row().insets(Insets {
        right: MARGIN,
        ..Insets::default()
    });
    let row_items = [
        Item::stretch(Size::Fill(1.0)),
        Item::new(Size::Fixed(BUTTON_WIDTH), Cross::Center(BUTTON_HEIGHT)),
    ];
    docked
        .iter()
        .zip(areas)
        .map(|(panel, area)| Row {
            panel: *panel,
            area,
            button: inside_row.layout(area, scale, &row_items)[1],
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
    let rects = Stack::column()
        .gap(MARGIN)
        .insets(Insets::all(MARGIN))
        .layout(
            Rect::new(0.0, 0.0, width, height),
            scale,
            &[
                Item::new(Size::Fixed(BUTTON_HEIGHT), Cross::Start(BUTTON_WIDTH)),
                Item::stretch(Size::Fill(1.0)),
            ],
        );
    DetachedView {
        button: rects[0],
        content: rects[1],
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
