use dock::drag::{DragTracker, DropAction, Release, Source};
use dock::layout::PanelId;

const PANEL: PanelId = PanelId(7);

/// A 300 x 200 window whose top-left corner is at (100, 100) on the screen, a main window
/// at (0, 0) of size 800 x 600, and a 10 point margin.
fn release_at(pointer: (f32, f32)) -> Release {
    Release {
        pointer,
        window_size: (300.0, 200.0),
        margin: 10.0,
        window_origin: Some((100.0, 100.0)),
        main_window: Some(((0.0, 0.0), (800.0, 600.0))),
        new_window_width: 200.0,
        title_offset: 15.0,
    }
}

fn pressed(source: Source) -> DragTracker {
    let mut tracker = DragTracker::new();
    tracker.press(PANEL, source);
    tracker
}

#[test]
fn nothing_happens_when_the_button_is_let_go_without_a_press() {
    let mut tracker = DragTracker::new();
    assert_eq!(tracker.release(release_at((-100.0, 50.0))), None);
}

#[test]
fn a_press_is_remembered_until_the_release() {
    let mut tracker = pressed(Source::Main);
    assert_eq!(tracker.dragging(), Some(PANEL));
    tracker.release(release_at((150.0, 100.0)));
    assert_eq!(tracker.dragging(), None);
}

#[test]
fn cancel_forgets_the_drag() {
    let mut tracker = pressed(Source::Main);
    tracker.cancel();
    assert_eq!(tracker.dragging(), None);
    assert_eq!(tracker.release(release_at((-100.0, 50.0))), None);
}

#[test]
fn a_second_press_replaces_the_first() {
    let mut tracker = pressed(Source::Main);
    tracker.press(PanelId(8), Source::Detached);
    assert_eq!(tracker.dragging(), Some(PanelId(8)));
}

#[test]
fn letting_go_inside_the_window_does_nothing_for_either_kind_of_window() {
    for source in [Source::Main, Source::Detached] {
        let mut tracker = pressed(source);
        assert_eq!(tracker.release(release_at((150.0, 100.0))), None);
    }
}

#[test]
fn a_small_overshoot_of_the_edge_does_not_count_as_leaving() {
    let mut tracker = pressed(Source::Main);
    assert_eq!(tracker.release(release_at((-8.0, 100.0))), None);
}

#[test]
fn letting_go_well_outside_the_main_window_detaches_the_panel_under_the_pointer() {
    let mut tracker = pressed(Source::Main);
    // The pointer is 50 left of the window, so on the screen at (50, 300).
    // The new window is 200 wide: its top-left corner is 100 left of the pointer
    // and 15 above it.
    let action = tracker.release(release_at((-50.0, 200.0)));
    assert_eq!(
        action,
        Some(DropAction::Detach {
            panel: PANEL,
            position: Some((-50.0, 285.0)),
        })
    );
}

#[test]
fn every_side_counts_as_outside() {
    for pointer in [
        (-50.0, 100.0),
        (400.0, 100.0),
        (150.0, -50.0),
        (150.0, 300.0),
    ] {
        let mut tracker = pressed(Source::Main);
        assert!(
            matches!(
                tracker.release(release_at(pointer)),
                Some(DropAction::Detach { .. })
            ),
            "pointer {pointer:?}"
        );
    }
}

#[test]
fn without_a_screen_position_the_panel_is_still_detached_but_the_system_places_it() {
    let mut tracker = pressed(Source::Main);
    let mut release = release_at((-50.0, 200.0));
    release.window_origin = None;
    assert_eq!(
        tracker.release(release),
        Some(DropAction::Detach {
            panel: PANEL,
            position: None,
        })
    );
}

#[test]
fn a_detached_panel_let_go_outside_its_window_over_the_main_window_is_redocked() {
    let mut tracker = pressed(Source::Detached);
    // On the screen the pointer is at (100 - 50, 100 + 100) = (50, 200): over the main window.
    assert_eq!(
        tracker.release(release_at((-50.0, 100.0))),
        Some(DropAction::Redock { panel: PANEL })
    );
}

#[test]
fn a_detached_panel_let_go_outside_its_window_but_not_over_the_main_window_stays_detached() {
    let mut tracker = pressed(Source::Detached);
    // On the screen the pointer is at (1000, 150): beyond the main window.
    assert_eq!(tracker.release(release_at((900.0, 50.0))), None);
}

#[test]
fn a_detached_window_lying_over_the_main_window_is_not_redocked_by_a_click_inside_it() {
    let mut tracker = pressed(Source::Detached);
    // The pointer is inside the detached window, which itself is over the main window.
    assert_eq!(tracker.release(release_at((150.0, 100.0))), None);
}

#[test]
fn redocking_needs_to_know_where_both_windows_are() {
    let mut no_origin = release_at((-50.0, 100.0));
    no_origin.window_origin = None;
    let mut no_main = release_at((-50.0, 100.0));
    no_main.main_window = None;
    for release in [no_origin, no_main] {
        let mut tracker = pressed(Source::Detached);
        assert_eq!(tracker.release(release), None);
    }
}

#[test]
fn the_main_window_edge_counts_as_over_it() {
    let mut tracker = pressed(Source::Detached);
    // Screen position (800, 200): exactly on the main window's right edge.
    assert_eq!(
        tracker.release(release_at((700.0, 100.0))),
        Some(DropAction::Redock { panel: PANEL })
    );
}
