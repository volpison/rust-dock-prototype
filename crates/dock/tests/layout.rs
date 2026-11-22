use dock::layout::{Layout, LayoutError, PanelId};

const A: PanelId = PanelId(1);
const B: PanelId = PanelId(2);
const C: PanelId = PanelId(3);

fn layout() -> Layout {
    Layout::new([A, B, C])
}

#[test]
fn starts_all_docked_and_ignores_duplicates() {
    let l = Layout::new([A, B, A]);
    assert_eq!(l.docked(), &[A, B]);
    assert!(l.detached().is_empty());
}

#[test]
fn detach_moves_panel_out_of_dock() {
    let mut l = layout();
    l.detach(B, Some((10.0, 20.0))).unwrap();
    assert_eq!(l.docked(), &[A, C]);
    assert!(l.is_detached(B));
    assert_eq!(l.detached()[0].position, Some((10.0, 20.0)));
}

#[test]
fn redock_puts_panel_back() {
    let mut l = layout();
    l.detach(B, None).unwrap();
    l.redock(B).unwrap();
    assert!(l.is_docked(B));
    assert!(l.detached().is_empty());
}

#[test]
fn a_panel_is_never_in_two_places() {
    let mut l = layout();
    l.detach(A, None).unwrap();
    assert_eq!(l.detach(A, None), Err(LayoutError::AlreadyDetached(A)));
    assert_eq!(l.redock(B), Err(LayoutError::AlreadyDocked(B)));
}

#[test]
fn unknown_panels_are_rejected() {
    let mut l = layout();
    let x = PanelId(99);
    assert_eq!(l.detach(x, None), Err(LayoutError::UnknownPanel(x)));
    assert_eq!(l.redock(x), Err(LayoutError::UnknownPanel(x)));
    assert_eq!(
        l.set_position(x, (0.0, 0.0)),
        Err(LayoutError::UnknownPanel(x))
    );
}

#[test]
fn set_position_remembers_where_the_window_went() {
    let mut l = layout();
    l.detach(C, None).unwrap();
    l.set_position(C, (5.0, 6.0)).unwrap();
    assert_eq!(l.detached()[0].position, Some((5.0, 6.0)));
}
