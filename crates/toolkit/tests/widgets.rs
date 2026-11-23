use toolkit::input::{Input, Released, WidgetId};
use toolkit::widgets::{Button, Look, Toggle};

const ID: WidgetId = WidgetId(7);
const OTHER: WidgetId = WidgetId(8);
const POS: (f32, f32) = (5.0, 5.0);

#[test]
fn a_button_looks_normal_hover_and_pressed() {
    let button = Button::new(ID);
    let mut input = Input::new();
    assert_eq!(button.look(&input, true), Look::Normal);
    input.pointer_moved(POS, Some(ID));
    assert_eq!(button.look(&input, true), Look::Hover);
    input.press(POS, Some(ID));
    assert_eq!(button.look(&input, true), Look::Pressed);
    input.pointer_moved(POS, None);
    assert_eq!(button.look(&input, true), Look::Normal);
}

#[test]
fn a_disabled_control_always_looks_disabled() {
    let button = Button::new(ID);
    let mut input = Input::new();
    input.pointer_moved(POS, Some(ID));
    input.press(POS, Some(ID));
    assert_eq!(button.look(&input, false), Look::Disabled);
}

#[test]
fn a_button_is_clicked_only_by_a_click_on_itself() {
    let button = Button::new(ID);
    assert!(button.clicked(Released::Click(ID), true));
    assert!(!button.clicked(Released::Click(OTHER), true));
    assert!(!button.clicked(Released::Cancelled(ID), true));
    assert!(!button.clicked(Released::Nothing, true));
}

#[test]
fn a_disabled_button_is_never_clicked() {
    assert!(!Button::new(ID).clicked(Released::Click(ID), false));
}

#[test]
fn a_whole_click_on_a_button_goes_through_the_input() {
    let button = Button::new(ID);
    let mut input = Input::new();
    input.pointer_moved(POS, Some(ID));
    input.press(POS, Some(ID));
    let (released, _) = input.release(POS, Some(ID));
    assert!(button.clicked(released, true));
}

#[test]
fn a_toggle_flips_on_each_click_and_reports_the_new_state() {
    let mut toggle = Toggle::new(ID, false);
    assert_eq!(toggle.handle(Released::Click(ID), true), Some(true));
    assert!(toggle.on);
    assert_eq!(toggle.handle(Released::Click(ID), true), Some(false));
    assert!(!toggle.on);
}

#[test]
fn a_toggle_ignores_everything_that_is_not_a_click_on_it() {
    let mut toggle = Toggle::new(ID, true);
    assert_eq!(toggle.handle(Released::Click(OTHER), true), None);
    assert_eq!(toggle.handle(Released::Cancelled(ID), true), None);
    assert_eq!(toggle.handle(Released::Nothing, true), None);
    assert!(toggle.on);
}

#[test]
fn a_disabled_toggle_does_not_flip() {
    let mut toggle = Toggle::new(ID, false);
    assert_eq!(toggle.handle(Released::Click(ID), false), None);
    assert!(!toggle.on);
}

#[test]
fn a_toggle_looks_like_a_button_does() {
    let toggle = Toggle::new(ID, true);
    let mut input = Input::new();
    input.pointer_moved(POS, Some(ID));
    assert_eq!(toggle.look(&input, true), Look::Hover);
    assert_eq!(toggle.look(&input, false), Look::Disabled);
}
