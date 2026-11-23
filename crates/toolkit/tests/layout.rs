use toolkit::geometry::Rect;
use toolkit::layout::{Cross, Insets, Item, Size, Stack};

/// Scales a person really meets: 100%, 125%, 150%, 175%, 200%, 250%.
const SCALES: [f32; 6] = [1.0, 1.25, 1.5, 1.75, 2.0, 2.5];

fn area(w: f32, h: f32) -> Rect {
    Rect::new(0.0, 0.0, w, h)
}

fn is_whole(r: Rect) -> bool {
    [r.x, r.y, r.w, r.h].iter().all(|v| *v == v.round())
}

#[test]
fn a_column_of_fixed_items_is_stacked_with_gaps() {
    let items = [Item::stretch(Size::Fixed(20.0)); 3];
    let rects = Stack::column()
        .gap(5.0)
        .layout(area(100.0, 200.0), 1.0, &items);
    assert_eq!(rects[0], Rect::new(0.0, 0.0, 100.0, 20.0));
    assert_eq!(rects[1], Rect::new(0.0, 25.0, 100.0, 20.0));
    assert_eq!(rects[2], Rect::new(0.0, 50.0, 100.0, 20.0));
}

#[test]
fn a_row_runs_left_to_right() {
    let items = [
        Item::stretch(Size::Fixed(30.0)),
        Item::stretch(Size::Fixed(40.0)),
    ];
    let rects = Stack::row()
        .gap(10.0)
        .layout(area(200.0, 50.0), 1.0, &items);
    assert_eq!(rects[0], Rect::new(0.0, 0.0, 30.0, 50.0));
    assert_eq!(rects[1], Rect::new(40.0, 0.0, 40.0, 50.0));
}

#[test]
fn insets_are_taken_off_every_side() {
    let items = [Item::stretch(Size::Fill(1.0))];
    let stack = Stack::column().insets(Insets {
        left: 1.0,
        top: 2.0,
        right: 3.0,
        bottom: 4.0,
    });
    let rects = stack.layout(area(100.0, 100.0), 1.0, &items);
    assert_eq!(rects[0], Rect::new(1.0, 2.0, 96.0, 94.0));
}

#[test]
fn fill_items_share_what_is_left_by_weight() {
    let items = [
        Item::stretch(Size::Fixed(40.0)),
        Item::stretch(Size::Fill(1.0)),
        Item::stretch(Size::Fill(3.0)),
    ];
    let rects = Stack::row()
        .gap(10.0)
        .layout(area(200.0, 10.0), 1.0, &items);
    // 200 - 40 - 2 gaps of 10 = 140 to share: 35 and 105.
    assert_eq!(rects[1].w, 35.0);
    assert_eq!(rects[2].w, 105.0);
    assert_eq!(rects[2].x, 40.0 + 10.0 + 35.0 + 10.0);
}

#[test]
fn items_that_do_not_fit_overflow_and_fill_gets_nothing() {
    let items = [
        Item::stretch(Size::Fixed(80.0)),
        Item::stretch(Size::Fixed(80.0)),
        Item::stretch(Size::Fill(1.0)),
    ];
    let rects = Stack::row().layout(area(100.0, 10.0), 1.0, &items);
    assert_eq!(rects[0].w, 80.0);
    assert_eq!(rects[1].w, 80.0);
    assert_eq!(rects[2].w, 0.0);
}

#[test]
fn across_the_stack_an_item_can_stretch_start_or_centre() {
    let items = [
        Item::new(Size::Fixed(10.0), Cross::Stretch),
        Item::new(Size::Fixed(10.0), Cross::Start(30.0)),
        Item::new(Size::Fixed(10.0), Cross::Center(30.0)),
    ];
    let rects = Stack::column().layout(area(100.0, 100.0), 1.0, &items);
    assert_eq!((rects[0].x, rects[0].w), (0.0, 100.0));
    assert_eq!((rects[1].x, rects[1].w), (0.0, 30.0));
    assert_eq!((rects[2].x, rects[2].w), (35.0, 30.0));
}

#[test]
fn the_area_need_not_start_at_the_origin() {
    let items = [Item::stretch(Size::Fixed(10.0))];
    let rects = Stack::column().layout(Rect::new(50.0, 60.0, 100.0, 100.0), 1.0, &items);
    assert_eq!((rects[0].x, rects[0].y), (50.0, 60.0));
}

#[test]
fn points_are_multiplied_by_the_scale() {
    let items = [
        Item::stretch(Size::Fixed(20.0)),
        Item::stretch(Size::Fixed(20.0)),
    ];
    let stack = Stack::column().gap(10.0).insets(Insets::all(12.0));
    let one = stack.layout(area(360.0, 300.0), 1.0, &items);
    let two = stack.layout(area(720.0, 600.0), 2.0, &items);
    for (a, b) in one.iter().zip(two.iter()) {
        assert_eq!(
            (b.x, b.y, b.w, b.h),
            (a.x * 2.0, a.y * 2.0, a.w * 2.0, a.h * 2.0)
        );
    }
}

#[test]
fn at_every_common_scale_all_rectangles_sit_on_whole_pixels() {
    let items = [
        Item::new(Size::Fixed(33.0), Cross::Center(17.0)),
        Item::stretch(Size::Fill(1.0)),
        Item::new(Size::Fixed(21.0), Cross::Start(9.0)),
    ];
    for scale in SCALES {
        for width in [361.0, 457.0, 800.0] {
            let stack = Stack::row().gap(7.0).insets(Insets::all(12.0));
            for rect in stack.layout(area(width, 301.0), scale, &items) {
                assert!(is_whole(rect), "at {scale}: {rect:?}");
            }
        }
    }
}

#[test]
fn at_every_common_scale_neighbours_do_not_overlap() {
    let items = [Item::stretch(Size::Fixed(72.0)); 4];
    for scale in SCALES {
        let rects = Stack::column().gap(10.0).insets(Insets::all(12.0)).layout(
            area(480.0 * scale, 900.0 * scale),
            scale,
            &items,
        );
        for pair in rects.windows(2) {
            assert!(pair[1].y >= pair[0].y + pair[0].h, "overlap at {scale}");
        }
    }
}

#[test]
fn a_tiny_or_empty_area_never_gives_negative_sizes() {
    let items = [
        Item::stretch(Size::Fill(1.0)),
        Item::new(Size::Fixed(10.0), Cross::Center(50.0)),
    ];
    let stack = Stack::column().gap(5.0).insets(Insets::all(12.0));
    for (w, h) in [(10.0, 10.0), (0.0, 0.0), (-5.0, 3.0)] {
        for rect in stack.layout(area(w, h), 1.5, &items) {
            assert!(rect.w >= 0.0 && rect.h >= 0.0, "{w}x{h}: {rect:?}");
        }
    }
}

#[test]
fn no_items_means_no_rectangles() {
    assert!(Stack::row().layout(area(100.0, 100.0), 1.0, &[]).is_empty());
}

#[test]
fn zero_weights_give_nothing() {
    let items = [
        Item::stretch(Size::Fill(0.0)),
        Item::stretch(Size::Fill(0.0)),
    ];
    let rects = Stack::row().layout(area(100.0, 10.0), 1.0, &items);
    assert_eq!((rects[0].w, rects[1].w), (0.0, 0.0));
}
