use toolkit::input::{Input, Released, WidgetId};

const A: WidgetId = WidgetId(1);
const B: WidgetId = WidgetId(2);
const POS: (f32, f32) = (10.0, 10.0);

#[test]
fn nothing_is_hovered_or_pressed_at_the_start() {
    let input = Input::new();
    assert!(!input.hovered(A) && !input.pressed(A));
    assert_eq!(input.pointer(), None);
    assert_eq!(input.captured(), None);
}

#[test]
fn the_control_under_the_pointer_is_hovered() {
    let mut input = Input::new();
    assert!(input.pointer_moved(POS, Some(A)));
    assert!(input.hovered(A));
    assert!(!input.hovered(B));
    assert_eq!(input.pointer(), Some(POS));
}

#[test]
fn moving_within_the_same_control_changes_no_look() {
    let mut input = Input::new();
    input.pointer_moved(POS, Some(A));
    assert!(!input.pointer_moved((12.0, 11.0), Some(A)));
    assert_eq!(input.pointer(), Some((12.0, 11.0)));
}

#[test]
fn moving_between_controls_and_off_them_changes_the_look() {
    let mut input = Input::new();
    input.pointer_moved(POS, Some(A));
    assert!(input.pointer_moved(POS, Some(B)));
    assert!(input.hovered(B) && !input.hovered(A));
    assert!(input.pointer_moved(POS, None));
    assert!(!input.hovered(B));
    assert!(!input.pointer_moved(POS, None));
}

#[test]
fn pressing_a_control_captures_it_and_shows_it_pressed() {
    let mut input = Input::new();
    input.pointer_moved(POS, Some(A));
    assert!(input.press(POS, Some(A)));
    assert_eq!(input.captured(), Some(A));
    assert!(input.pressed(A));
}

#[test]
fn pressing_on_nothing_captures_nothing() {
    let mut input = Input::new();
    input.pointer_moved(POS, None);
    assert!(!input.press(POS, None));
    assert_eq!(input.captured(), None);
    assert_eq!(input.release(POS, None), (Released::Nothing, false));
}

#[test]
fn letting_go_over_the_same_control_is_a_click() {
    let mut input = Input::new();
    input.press(POS, Some(A));
    let (released, changed) = input.release(POS, Some(A));
    assert_eq!(released, Released::Click(A));
    assert!(changed, "no longer pressed");
    assert_eq!(input.captured(), None);
    assert!(input.hovered(A) && !input.pressed(A));
}

#[test]
fn letting_go_somewhere_else_is_not_a_click() {
    let mut input = Input::new();
    input.press(POS, Some(A));
    for under in [None, Some(B)] {
        let mut input = input;
        assert_eq!(input.release(POS, under).0, Released::Cancelled(A));
        assert_eq!(input.captured(), None);
    }
}

#[test]
fn dragging_off_the_control_stops_it_looking_pressed_and_back_on_makes_it_pressed_again() {
    let mut input = Input::new();
    input.press(POS, Some(A));
    assert!(input.pointer_moved((50.0, 50.0), None));
    assert!(!input.pressed(A));
    assert_eq!(input.captured(), Some(A), "still holds the button");
    assert!(input.pointer_moved(POS, Some(A)));
    assert!(input.pressed(A));
}

#[test]
fn while_a_button_is_held_other_controls_are_not_hovered() {
    let mut input = Input::new();
    input.press(POS, Some(A));
    input.pointer_moved(POS, Some(B));
    assert!(!input.hovered(B));
    assert!(!input.hovered(A), "the pointer is not over A");
    input.pointer_moved(POS, Some(A));
    assert!(input.hovered(A));
}

#[test]
fn a_second_press_while_one_is_held_is_ignored() {
    let mut input = Input::new();
    input.press(POS, Some(A));
    input.press(POS, Some(B));
    assert_eq!(input.captured(), Some(A));
}

#[test]
fn the_pointer_leaving_the_window_clears_the_hover() {
    let mut input = Input::new();
    input.pointer_moved(POS, Some(A));
    assert!(input.pointer_left());
    assert!(!input.hovered(A));
    assert_eq!(input.pointer(), None);
    assert!(!input.pointer_left(), "already gone");
}

#[test]
fn the_pointer_leaving_while_a_button_is_held_forgets_nothing() {
    let mut input = Input::new();
    input.press(POS, Some(A));
    assert!(!input.pointer_left());
    assert_eq!(input.captured(), Some(A));
    assert_eq!(input.pointer(), Some(POS));
    // The button is let go outside the window: not a click.
    assert_eq!(input.release((400.0, 10.0), None).0, Released::Cancelled(A));
}

#[test]
fn cancel_drops_the_capture_without_a_click() {
    let mut input = Input::new();
    input.press(POS, Some(A));
    assert!(input.cancel());
    assert_eq!(input.captured(), None);
    assert!(!input.cancel(), "nothing left to cancel");
    assert_eq!(input.release(POS, Some(A)).0, Released::Nothing);
}

#[test]
fn a_press_that_changes_nothing_visible_says_so() {
    let mut input = Input::new();
    input.press(POS, Some(A));
    input.release(POS, Some(A));
    // Pressed again on the control that is already hovered: the look does change (pressed).
    assert!(input.press(POS, Some(A)));
}
