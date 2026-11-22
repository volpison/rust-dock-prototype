use dock::geometry::{is_outside_window, is_over_window, to_screen, window_origin_under_cursor};

const SIZE: (f32, f32) = (800.0, 600.0);

#[test]
fn points_inside_the_window_are_not_outside() {
    assert!(!is_outside_window((0.0, 0.0), SIZE, 0.0));
    assert!(!is_outside_window((400.0, 300.0), SIZE, 0.0));
    assert!(!is_outside_window((800.0, 600.0), SIZE, 0.0));
}

#[test]
fn points_beyond_any_edge_are_outside() {
    assert!(is_outside_window((-1.0, 300.0), SIZE, 0.0));
    assert!(is_outside_window((400.0, -1.0), SIZE, 0.0));
    assert!(is_outside_window((801.0, 300.0), SIZE, 0.0));
    assert!(is_outside_window((400.0, 601.0), SIZE, 0.0));
    assert!(is_outside_window((-833.0, 440.0), SIZE, 0.0));
}

#[test]
fn a_margin_forgives_a_small_overshoot() {
    assert!(!is_outside_window((-5.0, 300.0), SIZE, 8.0));
    assert!(is_outside_window((-9.0, 300.0), SIZE, 8.0));
    assert!(!is_outside_window((805.0, 605.0), SIZE, 8.0));
    assert!(is_outside_window((809.0, 300.0), SIZE, 8.0));
}

#[test]
fn to_screen_adds_the_window_origin() {
    // From the experiment: window top-left (1034, 333), pointer (-833, 440) in the window.
    assert_eq!(to_screen((1034.0, 333.0), (-833.0, 440.0)), (201.0, 773.0));
    assert_eq!(to_screen((0.0, 0.0), (5.0, 6.0)), (5.0, 6.0));
}

#[test]
fn new_window_is_centred_under_the_cursor() {
    assert_eq!(
        window_origin_under_cursor((500.0, 400.0), 320.0, 15.0),
        (340.0, 385.0)
    );
}

#[test]
fn a_screen_point_is_over_a_window_when_inside_its_rectangle() {
    // Main window from the experiment: top-left (1034, 333), 800 x 600.
    let origin = (1034.0, 333.0);
    assert!(is_over_window((1100.0, 400.0), origin, SIZE));
    assert!(is_over_window((1034.0, 333.0), origin, SIZE));
    assert!(is_over_window((1834.0, 933.0), origin, SIZE));
}

#[test]
fn a_screen_point_is_not_over_a_window_when_beside_it() {
    let origin = (1034.0, 333.0);
    assert!(!is_over_window((201.0, 773.0), origin, SIZE));
    assert!(!is_over_window((1033.0, 500.0), origin, SIZE));
    assert!(!is_over_window((1500.0, 332.0), origin, SIZE));
    assert!(!is_over_window((1835.0, 500.0), origin, SIZE));
    assert!(!is_over_window((1500.0, 934.0), origin, SIZE));
}
