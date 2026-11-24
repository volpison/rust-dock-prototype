use toolkit::input::WidgetId;
use toolkit::slider::{Slider, Track};
use toolkit::value::Range;

const ID: WidgetId = WidgetId(1);
/// The thumb's centre travels from 10 to 110: value 0 is at 10, value 100 at 110.
const TRACK: Track = Track {
    start: 10.0,
    len: 100.0,
};
const THUMB: f32 = 10.0;

fn close(a: Option<f32>, b: f32) -> bool {
    a.is_some_and(|a| (a - b).abs() < 1e-3)
}

fn slider(value: f32) -> Slider {
    Slider::new(ID, Range::new(0.0, 100.0).with_default(25.0), value)
}

#[test]
fn a_new_slider_holds_its_value_to_the_range_and_steps() {
    assert_eq!(slider(150.0).value(), 100.0);
    let s = Slider::new(ID, Range::new(0.0, 10.0).with_step(1.0), 3.4);
    assert_eq!(s.value(), 3.0);
    assert_eq!(slider(50.0).unit(), 0.5);
}

#[test]
fn pressing_on_the_thumb_moves_nothing_and_dragging_keeps_the_grip() {
    let mut s = slider(50.0); // thumb centre at 60
    assert_eq!(s.press(62.0, TRACK, THUMB), None);
    assert!(s.is_dragging());
    assert!(close(s.drag(72.0, TRACK, false), 60.0));
    assert!(close(s.drag(52.0, TRACK, false), 40.0));
}

#[test]
fn pressing_on_the_track_jumps_the_thumb_under_the_pointer_and_keeps_dragging() {
    let mut s = slider(50.0);
    assert!(close(s.press(30.0, TRACK, THUMB), 20.0));
    assert!(close(s.drag(40.0, TRACK, false), 30.0));
}

#[test]
fn pressing_beyond_the_ends_goes_to_the_ends() {
    let mut s = slider(50.0);
    assert_eq!(s.press(500.0, TRACK, THUMB), Some(100.0));
    s.release();
    assert_eq!(s.press(0.0, TRACK, THUMB), Some(0.0));
}

#[test]
fn dragging_without_a_press_or_after_the_release_does_nothing() {
    let mut s = slider(50.0);
    assert_eq!(s.drag(80.0, TRACK, false), None);
    s.press(62.0, TRACK, THUMB);
    s.release();
    assert!(!s.is_dragging());
    assert_eq!(s.drag(80.0, TRACK, false), None);
}

#[test]
fn dragging_far_past_the_end_stops_at_it() {
    let mut s = slider(50.0);
    s.press(60.0, TRACK, THUMB);
    assert_eq!(s.drag(900.0, TRACK, false), Some(100.0));
    assert_eq!(s.drag(901.0, TRACK, false), None);
}

#[test]
fn a_slider_with_steps_only_reports_when_a_step_is_crossed() {
    let mut s = Slider::new(ID, Range::new(0.0, 10.0).with_step(1.0), 5.0);
    let track = Track {
        start: 0.0,
        len: 100.0,
    };
    s.press(50.0, track, THUMB);
    assert_eq!(s.drag(54.0, track, false), None);
    assert_eq!(s.drag(56.0, track, false), Some(6.0));
    assert_eq!(s.drag(57.0, track, false), None);
}

#[test]
fn a_fine_drag_is_ten_times_slower_without_a_jump() {
    let mut s = slider(50.0);
    s.press(60.0, TRACK, THUMB);
    assert!(close(s.drag(80.0, TRACK, false), 70.0));
    assert_eq!(
        s.drag(80.0, TRACK, true),
        None,
        "no jump when Shift goes down"
    );
    assert!(close(s.drag(90.0, TRACK, true), 71.0));
}

#[test]
fn the_wheel_moves_by_a_hundredth_of_the_range_and_a_tenth_of_that_when_fine() {
    let mut s = slider(50.0);
    assert!(close(s.wheel(1.0, false), 51.0));
    assert!(close(s.wheel(-2.0, false), 49.0));
    assert!(close(s.wheel(1.0, true), 49.1));
}

#[test]
fn with_steps_a_wheel_notch_is_one_step_even_when_fine() {
    let mut s = Slider::new(ID, Range::new(0.0, 10.0).with_step(2.0), 4.0);
    assert_eq!(s.wheel(1.0, true), Some(6.0));
    assert_eq!(s.wheel(-1.0, false), Some(4.0));
}

#[test]
fn the_wheel_stops_at_the_ends() {
    let mut s = slider(50.0);
    assert_eq!(s.wheel(1000.0, false), Some(100.0));
    assert_eq!(s.wheel(1.0, false), None);
}

#[test]
fn reset_goes_back_to_the_default_and_says_nothing_when_already_there() {
    let mut s = slider(50.0);
    assert_eq!(s.reset(), Some(25.0));
    assert_eq!(s.reset(), None);
}

#[test]
fn setting_from_outside_holds_the_value_and_reports_changes_only() {
    let mut s = slider(50.0);
    assert_eq!(s.set(150.0), Some(100.0));
    assert_eq!(s.set(100.0), None);
}

#[test]
fn a_track_with_no_length_does_not_break() {
    let mut s = slider(50.0);
    let none = Track {
        start: 10.0,
        len: 0.0,
    };
    s.press(500.0, none, 0.0);
    assert_eq!(s.drag(900.0, none, false), None);
    assert_eq!(s.value(), 50.0);
}
