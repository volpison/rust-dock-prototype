use toolkit::input::WidgetId;
use toolkit::knob::Knob;
use toolkit::value::Range;

const ID: WidgetId = WidgetId(2);

fn close(a: Option<f32>, b: f32) -> bool {
    a.is_some_and(|a| (a - b).abs() < 1e-3)
}

fn knob(value: f32) -> Knob {
    Knob::new(ID, Range::new(0.0, 100.0).with_default(50.0), value)
}

#[test]
fn dragging_up_turns_the_knob_up_and_down_turns_it_down() {
    let mut k = knob(50.0);
    k.press(200.0);
    assert!(k.is_dragging());
    assert!(close(k.drag(180.0, 100.0, false), 70.0));
    assert!(close(k.drag(230.0, 100.0, false), 20.0));
}

#[test]
fn pressing_does_not_move_the_value() {
    let mut k = knob(50.0);
    k.press(200.0);
    assert_eq!(k.drag(200.0, 100.0, false), None);
}

#[test]
fn the_knob_stops_at_its_ends() {
    let mut k = knob(50.0);
    k.press(200.0);
    assert_eq!(k.drag(0.0, 100.0, false), Some(100.0));
    assert_eq!(k.drag(-50.0, 100.0, false), None);
    assert_eq!(k.drag(900.0, 100.0, false), Some(0.0));
}

#[test]
fn more_pixels_per_range_means_slower_turning() {
    let mut k = knob(50.0);
    k.press(200.0);
    assert!(close(k.drag(190.0, 200.0, false), 55.0));
}

#[test]
fn a_fine_drag_is_ten_times_slower_without_a_jump() {
    let mut k = knob(50.0);
    k.press(200.0);
    assert!(close(k.drag(190.0, 100.0, false), 60.0));
    assert_eq!(k.drag(190.0, 100.0, true), None);
    assert!(close(k.drag(170.0, 100.0, true), 62.0));
}

#[test]
fn dragging_without_a_press_or_after_the_release_does_nothing() {
    let mut k = knob(50.0);
    assert_eq!(k.drag(10.0, 100.0, false), None);
    k.press(200.0);
    k.release();
    assert!(!k.is_dragging());
    assert_eq!(k.drag(10.0, 100.0, false), None);
}

#[test]
fn the_pointer_sweeps_270_degrees() {
    assert_eq!(knob(0.0).angle_degrees(), -135.0);
    assert_eq!(knob(50.0).angle_degrees(), 0.0);
    assert_eq!(knob(100.0).angle_degrees(), 135.0);
}

#[test]
fn a_knob_with_steps_snaps_while_turning() {
    let mut k = Knob::new(ID, Range::new(0.0, 10.0).with_step(1.0), 5.0);
    k.press(100.0);
    assert_eq!(k.drag(96.0, 100.0, false), None);
    assert_eq!(k.drag(94.0, 100.0, false), Some(6.0));
}

#[test]
fn the_wheel_and_reset_work_like_a_slider() {
    let mut k = knob(60.0);
    assert!(close(k.wheel(1.0, false), 61.0));
    assert!(close(k.wheel(1.0, true), 61.1));
    assert_eq!(k.reset(), Some(50.0));
    assert_eq!(k.reset(), None);
}

#[test]
fn setting_from_outside_holds_the_value_and_reports_changes_only() {
    let mut k = knob(50.0);
    assert_eq!(k.set(-5.0), Some(0.0));
    assert_eq!(k.set(0.0), None);
}
