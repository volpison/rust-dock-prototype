use toolkit::value::{Dragger, Range};

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-4
}

#[test]
fn a_value_is_held_between_the_ends() {
    let r = Range::new(10.0, 20.0);
    assert_eq!(r.clamp(5.0), 10.0);
    assert_eq!(r.clamp(25.0), 20.0);
    assert_eq!(r.clamp(15.0), 15.0);
    assert_eq!(r.clamp(f32::NAN), 10.0);
}

#[test]
fn a_range_with_no_width_is_always_its_minimum() {
    let r = Range::new(5.0, 5.0);
    assert_eq!(r.clamp(9.0), 5.0);
    assert_eq!(r.to_unit(5.0), 0.0);
    assert_eq!(r.from_unit(0.7), 5.0);
}

#[test]
fn steps_are_counted_from_the_minimum() {
    let r = Range::new(1.0, 9.0).with_step(2.0);
    assert_eq!(r.snap(2.2), 3.0);
    assert_eq!(r.snap(1.9), 1.0);
    assert_eq!(r.snap(100.0), 9.0);
    assert_eq!(r.snap(-3.0), 1.0);
}

#[test]
fn the_maximum_is_reachable_even_when_it_is_not_a_step() {
    let r = Range::new(0.0, 10.0).with_step(4.0);
    assert_eq!(r.snap(9.9), 8.0);
    assert_eq!(r.snap(10.0), 10.0);
}

#[test]
fn units_run_from_zero_at_the_minimum_to_one_at_the_maximum() {
    let r = Range::new(10.0, 20.0);
    assert_eq!(r.to_unit(10.0), 0.0);
    assert_eq!(r.to_unit(15.0), 0.5);
    assert_eq!(r.to_unit(20.0), 1.0);
    assert_eq!(r.to_unit(99.0), 1.0);
    assert_eq!(r.from_unit(0.25), 12.5);
    assert_eq!(r.from_unit(2.0), 20.0);
    assert_eq!(r.from_unit(-1.0), 10.0);
    assert_eq!(r.from_unit(f32::NAN), 10.0);
}

#[test]
fn from_unit_moves_to_the_nearest_step() {
    let r = Range::new(0.0, 10.0).with_step(1.0);
    assert_eq!(r.from_unit(0.54), 5.0);
    assert_eq!(r.from_unit(0.56), 6.0);
}

#[test]
fn the_default_is_held_to_the_range_and_its_steps() {
    assert_eq!(Range::new(0.0, 10.0).default, 0.0);
    assert_eq!(
        Range::new(0.0, 10.0)
            .with_step(2.0)
            .with_default(5.1)
            .default,
        6.0
    );
    assert_eq!(Range::new(0.0, 10.0).with_default(50.0).default, 10.0);
}

#[test]
fn a_wheel_notch_is_a_step_or_a_hundredth_of_the_range() {
    assert_eq!(Range::new(0.0, 10.0).with_step(5.0).notch(), 5.0);
    assert_eq!(Range::new(0.0, 200.0).notch(), 2.0);
}

#[test]
fn pressing_does_not_move_the_value() {
    let mut d = Dragger::start(100.0, 0.5);
    assert_eq!(d.unit_at(100.0, 200.0, false), 0.5);
}

#[test]
fn the_value_follows_the_pointer_and_stops_at_the_ends() {
    let mut d = Dragger::start(100.0, 0.5);
    assert_eq!(d.unit_at(150.0, 200.0, false), 0.75);
    assert_eq!(d.unit_at(50.0, 200.0, false), 0.25);
    assert_eq!(d.unit_at(1000.0, 200.0, false), 1.0);
    assert_eq!(d.unit_at(-1000.0, 200.0, false), 0.0);
    // Coming back from beyond the end follows the pointer, not where it was lost.
    assert_eq!(d.unit_at(100.0, 200.0, false), 0.5);
}

#[test]
fn a_fine_drag_is_ten_times_slower_and_does_not_jump_when_it_starts_or_ends() {
    let mut d = Dragger::start(0.0, 0.5);
    assert!(close(d.unit_at(40.0, 200.0, false), 0.7));
    // Shift goes down at the same place: no jump.
    assert!(close(d.unit_at(40.0, 200.0, true), 0.7));
    assert!(close(d.unit_at(140.0, 200.0, true), 0.75));
    // Shift comes up at the same place: no jump either.
    assert!(close(d.unit_at(140.0, 200.0, false), 0.75));
    assert!(close(d.unit_at(180.0, 200.0, false), 0.95));
}

#[test]
fn no_distance_to_travel_means_no_movement() {
    for px in [0.0, -5.0, f32::NAN] {
        let mut d = Dragger::start(10.0, 0.3);
        assert_eq!(d.unit_at(500.0, px, false), 0.3);
    }
}
