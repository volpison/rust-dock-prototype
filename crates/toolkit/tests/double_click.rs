use toolkit::input::{DoubleClick, WidgetId};

const A: WidgetId = WidgetId(1);
const B: WidgetId = WidgetId(2);
const POS: (f32, f32) = (10.0, 10.0);

fn detector() -> DoubleClick {
    DoubleClick::new(400, 4.0)
}

#[test]
fn one_click_is_not_a_double_click() {
    assert!(!detector().click(A, 1000, POS));
}

#[test]
fn two_quick_clicks_on_the_same_control_are_a_double_click() {
    let mut d = detector();
    assert!(!d.click(A, 1000, POS));
    assert!(d.click(A, 1300, (12.0, 11.0)));
}

#[test]
fn two_slow_clicks_are_not() {
    let mut d = detector();
    d.click(A, 1000, POS);
    assert!(!d.click(A, 1401, POS));
}

#[test]
fn the_time_limit_itself_still_counts() {
    let mut d = detector();
    d.click(A, 1000, POS);
    assert!(d.click(A, 1400, POS));
}

#[test]
fn clicks_on_different_controls_or_far_apart_are_not() {
    let mut d = detector();
    d.click(A, 1000, POS);
    assert!(!d.click(B, 1100, POS));
    let mut d = detector();
    d.click(A, 1000, POS);
    assert!(!d.click(A, 1100, (30.0, 10.0)));
}

#[test]
fn a_third_quick_click_starts_a_new_pair() {
    let mut d = detector();
    d.click(A, 1000, POS);
    assert!(d.click(A, 1100, POS));
    assert!(!d.click(A, 1200, POS));
    assert!(d.click(A, 1300, POS));
}

#[test]
fn a_clock_that_goes_backwards_is_not_a_double_click() {
    let mut d = detector();
    d.click(A, 1000, POS);
    assert!(!d.click(A, 900, POS));
}
