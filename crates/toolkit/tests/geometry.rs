use toolkit::geometry::Rect;

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::new(x, y, w, h)
}

#[test]
fn a_point_inside_is_contained_and_the_edges_count() {
    let r = rect(10.0, 20.0, 30.0, 40.0);
    assert!(r.contains((25.0, 40.0)));
    assert!(r.contains((10.0, 20.0)));
    assert!(r.contains((40.0, 60.0)));
}

#[test]
fn a_point_outside_is_not_contained() {
    let r = rect(10.0, 20.0, 30.0, 40.0);
    for point in [(9.9, 30.0), (40.1, 30.0), (20.0, 19.9), (20.0, 60.1)] {
        assert!(!r.contains(point), "{point:?}");
    }
}

#[test]
fn snapping_moves_each_edge_to_a_whole_pixel() {
    let snapped = rect(0.4, 1.6, 10.2, 4.0).snapped();
    assert_eq!((snapped.x, snapped.y), (0.0, 2.0));
    assert_eq!(snapped.x + snapped.w, (0.4f32 + 10.2).round());
    assert_eq!(snapped.y + snapped.h, (1.6f32 + 4.0).round());
}

#[test]
fn rectangles_that_touched_before_snapping_still_touch() {
    let a = rect(0.0, 0.0, 10.4, 5.0);
    let b = rect(a.x + a.w, 0.0, 7.3, 5.0);
    let (a, b) = (a.snapped(), b.snapped());
    assert_eq!(a.x + a.w, b.x);
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
