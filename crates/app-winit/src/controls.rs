//! A window of controls to try the toolkit with (`cargo run -p rust-dock-winit -- --controls`):
//! a toggle, a slider, two knobs (one with steps), number displays and a reset button.
//! This file is the state and the events: it knows nothing about winit or drawing, so it is
//! tested on its own. `controls_render.rs` draws it.

use toolkit::digits::Style;
use toolkit::geometry::Rect;
use toolkit::input::{DoubleClick, Input, WidgetId};
use toolkit::knob::Knob;
use toolkit::layout::{Cross, Insets, Item, Size, Stack};
use toolkit::slider::{Slider, Track};
use toolkit::value::Range;
use toolkit::widgets::{Button, Toggle};

pub const TOGGLE: WidgetId = WidgetId(1);
pub const SLIDER: WidgetId = WidgetId(2);
pub const KNOB_A: WidgetId = WidgetId(3);
pub const KNOB_B: WidgetId = WidgetId(4);
pub const RESET: WidgetId = WidgetId(5);

/// Sizes in points.
const MARGIN: f32 = 16.0;
const GAP: f32 = 14.0;
const ROW: f32 = 32.0;
const KNOB: f32 = 96.0;
const TOGGLE_WIDTH: f32 = 60.0;
const RESET_WIDTH: f32 = 56.0;
pub const THUMB: f32 = 18.0;
pub const DIGITS_HEIGHT: f32 = 22.0;
/// How far a drag must go (points) to turn a knob from one end to the other.
const KNOB_TRAVEL: f32 = 160.0;
/// One wheel notch, and how many pixels of a touchpad scroll make one notch.
const PIXELS_PER_NOTCH: f32 = 40.0;
const DOUBLE_CLICK_MS: u64 = 400;
const DOUBLE_CLICK_DISTANCE: f32 = 6.0;

pub fn digits_style() -> Style {
    Style::new(DIGITS_HEIGHT)
}

/// How a number is written next to a control.
pub fn digits_text(value: f32, decimals: usize) -> String {
    toolkit::digits::format_value(value, decimals)
}

/// How tall the controls need to be, in points.
pub const HEIGHT: f32 = 2.0 * MARGIN + 2.0 * ROW + KNOB + DIGITS_HEIGHT + 3.0 * GAP;

/// Where everything is, in pixels.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub scale: f32,
    /// The area the controls were placed in.
    pub area: Rect,
    pub toggle: Rect,
    pub reset: Rect,
    pub slider: Rect,
    pub slider_value: Rect,
    pub knob: [Rect; 2],
    pub knob_value: [Rect; 2],
}

/// The layout of a window that holds only the controls, for its size in pixels.
pub fn plan(width: f32, height: f32, scale: f32) -> Plan {
    plan_in(Rect::new(0.0, 0.0, width, height), scale)
}

/// The layout of the controls inside `area` (pixels), at `scale`. The two knobs share the
/// width, each with its number under it, so the controls fit any width from about 240 points.
pub fn plan_in(area: Rect, scale: f32) -> Plan {
    // Room for "100.0" in the number displays, in points.
    let digits_w = digits_style().width("100.0", 1.0);
    let rows = Stack::column().gap(GAP).insets(Insets::all(MARGIN)).layout(
        area,
        scale,
        &[
            Item::stretch(Size::Fixed(ROW)),
            Item::stretch(Size::Fixed(ROW)),
            Item::stretch(Size::Fixed(KNOB)),
            Item::stretch(Size::Fixed(DIGITS_HEIGHT)),
            Item::stretch(Size::Fill(1.0)),
        ],
    );
    let top = Stack::row().layout(
        rows[0],
        scale,
        &[
            Item::new(Size::Fixed(TOGGLE_WIDTH), Cross::Center(ROW)),
            Item::stretch(Size::Fill(1.0)),
            Item::new(Size::Fixed(RESET_WIDTH), Cross::Center(ROW)),
        ],
    );
    let slider_row = Stack::row().gap(GAP).layout(
        rows[1],
        scale,
        &[
            Item::stretch(Size::Fill(1.0)),
            Item::new(Size::Fixed(digits_w), Cross::Center(DIGITS_HEIGHT)),
        ],
    );
    let knobs = Stack::row().gap(GAP).layout(
        rows[2],
        scale,
        &[
            Item::new(Size::Fill(1.0), Cross::Center(KNOB)),
            Item::new(Size::Fill(1.0), Cross::Center(KNOB)),
        ],
    );
    let values = Stack::row().gap(GAP).layout(
        rows[3],
        scale,
        &[
            Item::new(Size::Fill(1.0), Cross::Center(DIGITS_HEIGHT)),
            Item::new(Size::Fill(1.0), Cross::Center(DIGITS_HEIGHT)),
        ],
    );
    Plan {
        scale,
        area,
        toggle: top[0],
        reset: top[2],
        slider: slider_row[0],
        slider_value: slider_row[1],
        knob: [
            centred(knobs[0], KNOB, scale),
            centred(knobs[1], KNOB, scale),
        ],
        knob_value: [values[0], values[1]],
    }
}

/// A square of `points` in the middle of `cell` across its width, so a control is only
/// hit where it is drawn.
fn centred(cell: Rect, points: f32, scale: f32) -> Rect {
    let size = (points * scale).min(cell.w);
    Rect::new(cell.x + (cell.w - size) / 2.0, cell.y, size, cell.h).snapped()
}

impl Plan {
    /// The control under `pos`, if any.
    pub fn hit(&self, pos: (f32, f32)) -> Option<WidgetId> {
        [
            (TOGGLE, self.toggle),
            (RESET, self.reset),
            (SLIDER, self.slider),
            (KNOB_A, self.knob[0]),
            (KNOB_B, self.knob[1]),
        ]
        .into_iter()
        .find(|(_, rect)| rect.contains(pos))
        .map(|(id, _)| id)
    }

    /// The part of the slider the thumb's centre travels on, and the thumb's length.
    pub fn track(&self) -> (Track, f32) {
        let thumb = THUMB * self.scale;
        (
            Track {
                start: self.slider.x + thumb / 2.0,
                len: (self.slider.w - thumb).max(0.0),
            },
            thumb,
        )
    }

    fn knob_travel(&self) -> f32 {
        KNOB_TRAVEL * self.scale
    }
}

pub struct Panel {
    pub input: Input,
    pub toggle: Toggle,
    pub slider: Slider,
    pub knobs: [Knob; 2],
    pub reset: Button,
    double_click: DoubleClick,
    fine: bool,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            input: Input::new(),
            toggle: Toggle::new(TOGGLE, false),
            slider: Slider::new(SLIDER, Range::new(0.0, 100.0).with_default(50.0), 50.0),
            knobs: [
                Knob::new(KNOB_A, Range::new(0.0, 100.0).with_default(50.0), 50.0),
                Knob::new(
                    KNOB_B,
                    Range::new(0.0, 10.0).with_step(1.0).with_default(5.0),
                    5.0,
                ),
            ],
            reset: Button::new(RESET),
            double_click: DoubleClick::new(DOUBLE_CLICK_MS, DOUBLE_CLICK_DISTANCE),
            fine: false,
        }
    }

    /// Shift is held (a fine drag) or not.
    pub fn set_fine(&mut self, fine: bool) {
        self.fine = fine;
    }

    /// The pointer moved. Returns true when something looks different.
    pub fn pointer_moved(&mut self, plan: &Plan, pos: (f32, f32)) -> bool {
        let mut changed = self.input.pointer_moved(pos, plan.hit(pos));
        let (track, _) = plan.track();
        changed |= self.slider.drag(pos.0, track, self.fine).is_some();
        let travel = plan.knob_travel();
        for knob in &mut self.knobs {
            changed |= knob.drag(pos.1, travel, self.fine).is_some();
        }
        changed
    }

    /// The pointer left the window (while a button is held it keeps being reported outside).
    pub fn pointer_left(&mut self) -> bool {
        self.input.pointer_left()
    }

    /// The left button went down at `time_ms` (any steady clock).
    pub fn press(&mut self, plan: &Plan, pos: (f32, f32), time_ms: u64) -> bool {
        let under = plan.hit(pos);
        let mut changed = self.input.press(pos, under);
        let Some(id) = under else {
            return changed;
        };
        let draggable = matches!(id, SLIDER | KNOB_A | KNOB_B);
        if draggable && self.double_click.click(id, time_ms, pos) {
            // A double-click puts the control back to its default instead of dragging it.
            self.end_drags();
            changed |= self.input.cancel();
            return self.reset_one(id) | changed;
        }
        match id {
            SLIDER => {
                let (track, thumb) = plan.track();
                changed |= self.slider.press(pos.0, track, thumb).is_some();
            }
            KNOB_A => self.knobs[0].press(pos.1),
            KNOB_B => self.knobs[1].press(pos.1),
            _ => {}
        }
        if self.fine {
            // Shift was already held: the drag starts fine, so its first move is not lost to
            // switching speed (the position is anchored here, with no change).
            let (track, _) = plan.track();
            self.slider.drag(pos.0, track, true);
            for knob in &mut self.knobs {
                knob.drag(pos.1, plan.knob_travel(), true);
            }
        }
        changed
    }

    /// The left button came up.
    pub fn release(&mut self, plan: &Plan, pos: (f32, f32)) -> bool {
        let (released, mut changed) = self.input.release(pos, plan.hit(pos));
        self.end_drags();
        changed |= self.toggle.handle(released, true).is_some();
        if self.reset.clicked(released, true) {
            changed |= self.reset_all();
        }
        changed
    }

    /// The wheel turned by `notches` (positive = up) over `pos`.
    pub fn wheel(&mut self, plan: &Plan, pos: (f32, f32), notches: f32) -> bool {
        match plan.hit(pos) {
            Some(SLIDER) => self.slider.wheel(notches, self.fine).is_some(),
            Some(KNOB_A) => self.knobs[0].wheel(notches, self.fine).is_some(),
            Some(KNOB_B) => self.knobs[1].wheel(notches, self.fine).is_some(),
            _ => false,
        }
    }

    /// A touchpad scrolled by `pixels` (positive = up).
    pub fn wheel_pixels(&mut self, plan: &Plan, pos: (f32, f32), pixels: f32) -> bool {
        self.wheel(plan, pos, pixels / PIXELS_PER_NOTCH)
    }

    /// Forget everything held or hovered (the panel's window is going away or changing).
    pub fn cancel(&mut self) {
        self.end_drags();
        self.input.cancel();
        self.input.pointer_left();
    }

    fn end_drags(&mut self) {
        self.slider.release();
        for knob in &mut self.knobs {
            knob.release();
        }
    }

    fn reset_one(&mut self, id: WidgetId) -> bool {
        match id {
            SLIDER => self.slider.reset().is_some(),
            KNOB_A => self.knobs[0].reset().is_some(),
            KNOB_B => self.knobs[1].reset().is_some(),
            _ => false,
        }
    }

    fn reset_all(&mut self) -> bool {
        let mut changed = self.toggle.on;
        self.toggle.on = false;
        changed |= self.slider.reset().is_some();
        for knob in &mut self.knobs {
            changed |= knob.reset().is_some();
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCALES: [f32; 6] = [1.0, 1.25, 1.5, 1.75, 2.0, 2.5];

    fn plan_at(scale: f32) -> Plan {
        plan(380.0 * scale, 330.0 * scale, scale)
    }

    fn centre(r: Rect) -> (f32, f32) {
        (r.x + r.w / 2.0, r.y + r.h / 2.0)
    }

    fn click(panel: &mut Panel, plan: &Plan, pos: (f32, f32), time: u64) {
        panel.pointer_moved(plan, pos);
        panel.press(plan, pos, time);
        panel.release(plan, pos);
    }

    fn all_rects(p: &Plan) -> Vec<Rect> {
        let mut v = vec![p.toggle, p.reset, p.slider, p.slider_value];
        v.extend(p.knob);
        v.extend(p.knob_value);
        v
    }

    #[test]
    fn everything_is_inside_the_window_on_whole_pixels_and_does_not_overlap() {
        for scale in SCALES {
            let p = plan_at(scale);
            let rects = all_rects(&p);
            for (i, a) in rects.iter().enumerate() {
                assert!(a.x >= 0.0 && a.y >= 0.0, "at {scale}: {a:?}");
                assert!(a.x + a.w <= 380.0 * scale && a.y + a.h <= 330.0 * scale);
                assert!([a.x, a.y, a.w, a.h].iter().all(|v| *v == v.round()));
                for b in &rects[i + 1..] {
                    let overlap =
                        a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
                    assert!(!overlap, "at {scale}: {a:?} {b:?}");
                }
            }
        }
    }

    #[test]
    fn what_is_drawn_is_what_is_hit() {
        let p = plan_at(1.5);
        assert_eq!(p.hit(centre(p.toggle)), Some(TOGGLE));
        assert_eq!(p.hit(centre(p.reset)), Some(RESET));
        assert_eq!(p.hit(centre(p.slider)), Some(SLIDER));
        assert_eq!(p.hit(centre(p.knob[0])), Some(KNOB_A));
        assert_eq!(p.hit(centre(p.knob[1])), Some(KNOB_B));
        assert_eq!(p.hit(centre(p.slider_value)), None);
        assert_eq!(p.hit((1.0, 1.0)), None);
    }

    #[test]
    fn the_digits_have_room_for_the_text() {
        for scale in SCALES {
            let p = plan_at(scale);
            let needed = digits_style().width("100.0", scale);
            assert!(p.slider_value.w >= needed, "at {scale}");
        }
    }

    #[test]
    fn a_click_on_the_toggle_flips_it_but_letting_go_elsewhere_does_not() {
        let p = plan_at(1.0);
        let mut panel = Panel::new();
        click(&mut panel, &p, centre(p.toggle), 0);
        assert!(panel.toggle.on);
        panel.pointer_moved(&p, centre(p.toggle));
        panel.press(&p, centre(p.toggle), 1000);
        panel.pointer_moved(&p, (2.0, 2.0));
        panel.release(&p, (2.0, 2.0));
        assert!(panel.toggle.on, "dragged off before letting go: no click");
    }

    #[test]
    fn a_quick_second_click_on_the_toggle_flips_it_back() {
        let p = plan_at(1.0);
        let mut panel = Panel::new();
        click(&mut panel, &p, centre(p.toggle), 0);
        click(&mut panel, &p, centre(p.toggle), 100);
        assert!(!panel.toggle.on, "two clicks, two flips");
    }

    #[test]
    fn pressing_on_the_slider_track_jumps_the_thumb_and_dragging_moves_it() {
        let p = plan_at(1.0);
        let (track, _) = p.track();
        let y = centre(p.slider).1;
        let mut panel = Panel::new();
        panel.pointer_moved(&p, (track.start + track.len * 0.2, y));
        assert!(panel.press(&p, (track.start + track.len * 0.2, y), 0));
        assert!((panel.slider.value() - 20.0).abs() < 1.0);
        assert!(panel.pointer_moved(&p, (track.start + track.len * 0.7, y)));
        assert!((panel.slider.value() - 70.0).abs() < 1.0);
        panel.release(&p, (track.start + track.len * 0.7, y));
        let before = panel.slider.value();
        panel.pointer_moved(&p, (track.start + track.len * 0.9, y));
        assert_eq!(panel.slider.value(), before, "no drag after the release");
    }

    #[test]
    fn a_slider_drag_goes_on_outside_the_window_and_stops_at_the_end() {
        let p = plan_at(1.0);
        let (track, _) = p.track();
        let y = centre(p.slider).1;
        let mut panel = Panel::new();
        let start = centre(p.slider);
        panel.pointer_moved(&p, start);
        panel.press(&p, start, 0);
        assert!(
            !panel.pointer_left(),
            "nothing to redraw: the button is held"
        );
        panel.pointer_moved(&p, (track.start + track.len + 500.0, y));
        assert_eq!(panel.slider.value(), 100.0);
    }

    #[test]
    fn dragging_up_on_a_knob_turns_it_up_and_shift_makes_it_slower() {
        let p = plan_at(1.0);
        let start = centre(p.knob[0]);
        let mut panel = Panel::new();
        panel.pointer_moved(&p, start);
        panel.press(&p, start, 0);
        assert!(panel.pointer_moved(&p, (start.0, start.1 - 16.0)));
        let normal = panel.knobs[0].value() - 50.0;
        assert!(
            (normal - 10.0).abs() < 0.5,
            "16 of 160 points = 10: {normal}"
        );
        panel.release(&p, (start.0, start.1 - 16.0));

        let mut fine = Panel::new();
        fine.set_fine(true);
        fine.pointer_moved(&p, start);
        fine.press(&p, start, 0);
        fine.pointer_moved(&p, (start.0, start.1 - 16.0));
        let slow = fine.knobs[0].value() - 50.0;
        assert!(slow < normal / 5.0 && slow > 0.0, "{slow} vs {normal}");
    }

    #[test]
    fn pressing_shift_halfway_through_a_drag_does_not_make_the_value_jump() {
        let p = plan_at(1.0);
        let start = centre(p.knob[0]);
        let mut panel = Panel::new();
        panel.pointer_moved(&p, start);
        panel.press(&p, start, 0);
        panel.pointer_moved(&p, (start.0, start.1 - 16.0));
        let before = panel.knobs[0].value();
        panel.set_fine(true);
        panel.pointer_moved(&p, (start.0, start.1 - 16.0));
        assert_eq!(panel.knobs[0].value(), before);
        panel.pointer_moved(&p, (start.0, start.1 - 32.0));
        let gained = panel.knobs[0].value() - before;
        assert!(gained > 0.0 && gained < 3.0, "{gained}");
    }

    #[test]
    fn the_stepped_knob_moves_in_whole_steps() {
        let p = plan_at(1.0);
        let start = centre(p.knob[1]);
        let mut panel = Panel::new();
        panel.pointer_moved(&p, start);
        panel.press(&p, start, 0);
        panel.pointer_moved(&p, (start.0, start.1 - 32.0));
        assert_eq!(panel.knobs[1].value(), 7.0);
    }

    #[test]
    fn a_double_click_on_a_slider_or_knob_goes_back_to_the_default() {
        let p = plan_at(1.0);
        let mut panel = Panel::new();
        panel.slider.set(90.0);
        let pos = centre(p.slider);
        // 90 puts the thumb away from the centre: click the centre quickly twice.
        click(&mut panel, &p, pos, 0);
        assert!((panel.slider.value() - 90.0).abs() > 1.0 || panel.slider.value() == 50.0);
        click(&mut panel, &p, pos, 150);
        assert_eq!(panel.slider.value(), 50.0);

        panel.knobs[0].set(10.0);
        let k = centre(p.knob[0]);
        click(&mut panel, &p, k, 2000);
        click(&mut panel, &p, k, 2100);
        assert_eq!(panel.knobs[0].value(), 50.0);
    }

    #[test]
    fn the_reset_button_puts_everything_back() {
        let p = plan_at(1.0);
        let mut panel = Panel::new();
        panel.toggle.on = true;
        panel.slider.set(10.0);
        panel.knobs[0].set(90.0);
        panel.knobs[1].set(2.0);
        click(&mut panel, &p, centre(p.reset), 0);
        assert!(!panel.toggle.on);
        assert_eq!(panel.slider.value(), 50.0);
        assert_eq!(panel.knobs[0].value(), 50.0);
        assert_eq!(panel.knobs[1].value(), 5.0);
    }

    #[test]
    fn the_wheel_changes_the_control_under_the_pointer_only() {
        let p = plan_at(1.0);
        let mut panel = Panel::new();
        assert!(panel.wheel(&p, centre(p.slider), 2.0));
        assert!((panel.slider.value() - 52.0).abs() < 1e-3);
        assert!(panel.wheel(&p, centre(p.knob[1]), 1.0));
        assert_eq!(panel.knobs[1].value(), 6.0);
        assert!(!panel.wheel(&p, centre(p.toggle), 1.0));
        assert!(!panel.wheel(&p, (1.0, 1.0), 1.0));
        assert!(panel.wheel_pixels(&p, centre(p.slider), 40.0));
    }

    #[test]
    fn moving_over_a_control_says_the_look_changed_only_when_it_did() {
        let p = plan_at(1.0);
        let mut panel = Panel::new();
        let t = centre(p.toggle);
        assert!(panel.pointer_moved(&p, t));
        assert!(!panel.pointer_moved(&p, (t.0 + 1.0, t.1)));
        assert!(panel.pointer_moved(&p, (1.0, 1.0)));
    }

    #[test]
    fn the_controls_fit_the_smallest_panel_width_and_the_stated_height() {
        // The detached panel window is 320 points wide with 12 points around the panel;
        // the controls must fit that, and their height is what `HEIGHT` says.
        for scale in SCALES {
            let area = Rect::new(0.0, 0.0, 296.0 * scale, HEIGHT * scale);
            let p = plan_in(area, scale);
            for r in all_rects(&p) {
                assert!(r.x >= 0.0 && r.x + r.w <= area.w + 1.0, "at {scale}: {r:?}");
                assert!(r.y + r.h <= area.h + 1.0, "at {scale}: {r:?}");
            }
        }
    }

    #[test]
    fn the_controls_move_with_the_area_they_are_placed_in() {
        let at_origin = plan_in(Rect::new(0.0, 0.0, 300.0, 300.0), 1.0);
        let moved = plan_in(Rect::new(40.0, 70.0, 300.0, 300.0), 1.0);
        assert_eq!(moved.area, Rect::new(40.0, 70.0, 300.0, 300.0));
        assert_eq!(moved.toggle.x, at_origin.toggle.x + 40.0);
        assert_eq!(moved.toggle.y, at_origin.toggle.y + 70.0);
        assert_eq!(moved.knob[1].x, at_origin.knob[1].x + 40.0);
        // A click is found where the control now is, and not where it was.
        assert_eq!(moved.hit(centre(moved.slider)), Some(SLIDER));
        assert_eq!(moved.hit(centre(at_origin.slider)), None);
    }

    #[test]
    fn each_number_sits_under_its_own_knob() {
        for scale in SCALES {
            let p = plan_at(scale);
            for i in 0..2 {
                assert!(p.knob_value[i].y >= p.knob[i].y + p.knob[i].h);
                let (k, v) = (centre(p.knob[i]).0, centre(p.knob_value[i]).0);
                assert!((k - v).abs() <= 1.0, "at {scale}: knob {i}");
            }
        }
    }

    #[test]
    fn cancel_lets_go_of_everything_held() {
        let p = plan_at(1.0);
        let mut panel = Panel::new();
        let start = centre(p.slider);
        panel.pointer_moved(&p, start);
        panel.press(&p, start, 0);
        panel.cancel();
        assert_eq!(panel.input.captured(), None);
        // The drag is over: moving the pointer no longer changes the value.
        let before = panel.slider.value();
        panel.pointer_moved(&p, (start.0 + 50.0, start.1));
        assert_eq!(panel.slider.value(), before);
    }
}
