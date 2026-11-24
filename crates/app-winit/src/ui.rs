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

use crate::controls;
use crate::panels;

const MARGIN: f32 = 12.0;
const ROW_HEIGHT: f32 = 72.0;
const ROW_GAP: f32 = 10.0;
const BUTTON_WIDTH: f32 = 96.0;
const BUTTON_HEIGHT: f32 = 44.0;
/// A panel with controls: the strip at its top that is pressed to drag it, in a window of its
/// own (the main window's strip also holds the "detach" button, so it is taller).
const GRIP_HEIGHT: f32 = 24.0;
const HEADER_HEIGHT: f32 = 56.0;

/// One docked panel in the main window: its block and its "detach" button.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Row {
    pub panel: PanelId,
    pub area: Rect,
    pub button: Rect,
    /// Where pressing starts a drag: the whole block, or the strip on top of a panel with
    /// controls (the button sits in it).
    pub grip: Rect,
    /// The room for the controls, for a panel that has them.
    pub body: Option<Rect>,
}

/// How tall a docked panel's row is, in points.
fn row_height(panel: PanelId) -> f32 {
    if panels::has_controls(panel) {
        HEADER_HEIGHT + controls::HEIGHT
    } else {
        ROW_HEIGHT
    }
}

/// The docked panels, one row each, from the top. `width` is the window's width in pixels.
pub fn main_rows(width: f32, scale: f32, docked: &[PanelId]) -> Vec<Row> {
    let count = docked.len();
    let heights: Vec<f32> = docked.iter().map(|p| row_height(*p)).collect();
    let height =
        (2.0 * MARGIN + heights.iter().sum::<f32>() + count.saturating_sub(1) as f32 * ROW_GAP)
            * scale;
    let items: Vec<Item> = heights
        .iter()
        .map(|h| Item::stretch(Size::Fixed(*h)))
        .collect();
    let areas = Stack::column()
        .gap(ROW_GAP)
        .insets(Insets::all(MARGIN))
        .layout(Rect::new(0.0, 0.0, width, height), scale, &items);
    // In each row the button is at the right end of its top strip, MARGIN from the edge,
    // centred vertically in that strip.
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
        .map(|(panel, area)| {
            let (grip, body) = if panels::has_controls(*panel) {
                let parts = Stack::column().layout(
                    area,
                    scale,
                    &[
                        Item::stretch(Size::Fixed(HEADER_HEIGHT)),
                        Item::stretch(Size::Fill(1.0)),
                    ],
                );
                (parts[0], Some(parts[1]))
            } else {
                (area, None)
            };
            Row {
                panel: *panel,
                area,
                button: inside_row.layout(grip, scale, &row_items)[1],
                grip,
                body,
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

/// In a detached window of a panel with controls: the strip at the top of its block that is
/// pressed to drag it, and the room for the controls below it.
pub fn detached_grip_body(width: f32, height: f32, scale: f32) -> (Rect, Rect) {
    let content = detached_view(width, height, scale).content;
    let parts = Stack::column().layout(
        content,
        scale,
        &[
            Item::stretch(Size::Fixed(GRIP_HEIGHT)),
            Item::stretch(Size::Fill(1.0)),
        ],
    );
    (parts[0], parts[1])
}

/// Where the controls of `panel` are in the main window, if it is docked and has any.
pub fn body_of(width: f32, scale: f32, docked: &[PanelId], panel: PanelId) -> Option<Rect> {
    main_rows(width, scale, docked)
        .into_iter()
        .find(|row| row.panel == panel)
        .and_then(|row| row.body)
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
        .find(|row| row.grip.contains(point) && !row.button.contains(point))
        .map(|row| row.panel)
}

/// True when `point` is where pressing starts a drag in a detached window: the whole block, or
/// only the strip on top of it for a panel with controls.
pub fn content_at(point: (f32, f32), width: f32, height: f32, scale: f32, panel: PanelId) -> bool {
    if panels::has_controls(panel) {
        detached_grip_body(width, height, scale).0.contains(point)
    } else {
        detached_view(width, height, scale).content.contains(point)
    }
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
        assert!(content_at((100.0, 200.0), 320.0, 420.0, 1.0, A));
        assert!(!content_at((20.0, 20.0), 320.0, 420.0, 1.0, A));
        assert!(!content_at((2.0, 200.0), 320.0, 420.0, 1.0, A));
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

    const P: PanelId = panels::PROPERTIES;

    #[test]
    fn a_panel_with_controls_has_a_taller_row_with_a_strip_and_a_body() {
        let rows = main_rows(360.0, 1.0, &[A, P]);
        assert_eq!(rows[0].area.h, 72.0);
        assert_eq!(rows[0].grip, rows[0].area);
        assert_eq!(rows[0].body, None);
        let row = rows[1];
        assert_eq!(row.area.h, HEADER_HEIGHT + controls::HEIGHT);
        assert_eq!(row.grip.h, HEADER_HEIGHT);
        let body = row.body.unwrap();
        assert_eq!(body.y, row.grip.y + row.grip.h);
        assert_eq!(body.y + body.h, row.area.y + row.area.h);
        assert_eq!(rows[1].area.y, rows[0].area.y + rows[0].area.h + ROW_GAP);
    }

    #[test]
    fn the_detach_button_of_a_panel_with_controls_is_in_its_strip() {
        let row = main_rows(360.0, 1.0, &[P])[0];
        assert!(row.button.y >= row.grip.y);
        assert!(row.button.y + row.button.h <= row.grip.y + row.grip.h);
        assert_eq!(row.button.x + row.button.w + 12.0, row.area.x + row.area.w);
    }

    #[test]
    fn pressing_a_panels_strip_drags_it_but_its_controls_do_not() {
        let row = main_rows(360.0, 1.0, &[P])[0];
        let on_strip = (row.grip.x + 5.0, row.grip.y + 5.0);
        let on_body = (row.body.unwrap().x + 20.0, row.body.unwrap().y + 20.0);
        assert_eq!(block_at(on_strip, 360.0, 1.0, &[P]), Some(P));
        assert_eq!(block_at(on_body, 360.0, 1.0, &[P]), None);
        assert_eq!(
            body_of(360.0, 1.0, &[P], P),
            Some(row.body.unwrap()),
            "the controls are found by panel"
        );
        assert_eq!(body_of(360.0, 1.0, &[A], A), None);
        assert_eq!(body_of(360.0, 1.0, &[A], P), None);
    }

    #[test]
    fn in_a_detached_window_only_the_strip_of_a_panel_with_controls_drags_it() {
        let (grip, body) = detached_grip_body(320.0, 420.0, 1.0);
        let content = detached_view(320.0, 420.0, 1.0).content;
        assert_eq!(grip.y, content.y);
        assert_eq!(body.y, grip.y + grip.h);
        assert!(content_at(
            (grip.x + 5.0, grip.y + 5.0),
            320.0,
            420.0,
            1.0,
            P
        ));
        assert!(!content_at(
            (body.x + 5.0, body.y + 5.0),
            320.0,
            420.0,
            1.0,
            P
        ));
        // A panel without controls drags from anywhere on its block.
        assert!(content_at(
            (body.x + 5.0, body.y + 5.0),
            320.0,
            420.0,
            1.0,
            A
        ));
    }

    #[test]
    fn the_controls_fit_in_the_detached_window_at_every_scale() {
        for scale in [1.0_f32, 1.25, 1.5, 1.75, 2.0, 2.5] {
            let (_, body) = detached_grip_body(320.0 * scale, 420.0 * scale, scale);
            assert!(
                body.h + 1.0 >= controls::HEIGHT * scale,
                "at {scale}: {body:?}"
            );
            assert!(body.w + 1.0 >= 296.0 * scale, "at {scale}: {body:?}");
        }
    }

    #[test]
    fn every_rectangle_of_a_panel_with_controls_is_on_whole_pixels() {
        for scale in [1.0_f32, 1.25, 1.5, 1.75, 2.0, 2.5] {
            for row in main_rows(360.0 * scale, scale, &[A, P]) {
                let mut rects = vec![row.area, row.button, row.grip];
                rects.extend(row.body);
                for r in rects {
                    assert!(
                        [r.x, r.y, r.w, r.h].iter().all(|v| *v == v.round()),
                        "at {scale}: {r:?}"
                    );
                }
            }
        }
    }
}
