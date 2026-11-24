//! Draws the controls window (`controls.rs`) with tiny-skia. Only shapes: circles and arcs are
//! filled polygons, so nothing here needs more than the calls `render.rs` already uses.

use tiny_skia::{Color, Pixmap};
use toolkit::digits::{layout as digit_layout, Align};
use toolkit::geometry::Rect;
use toolkit::widgets::Look;

use crate::controls::{digits_style, digits_text, Panel, Plan};
use crate::render::{fill_path, rounded_rect};

const BACKGROUND: [u8; 4] = [30, 33, 41, 255];
const SURFACE: [u8; 4] = [20, 22, 28, 255];
const TRACK: [u8; 4] = [58, 63, 76, 255];
const ACCENT: [u8; 4] = [255, 170, 60, 255];
const HANDLE: [u8; 4] = [235, 235, 240, 255];
const DIGITS_UNLIT: [u8; 4] = [48, 52, 62, 255];

/// A colour made lighter for hover and darker for pressed.
fn shade(rgba: [u8; 4], look: Look) -> [u8; 4] {
    let change = |c: u8| -> u8 {
        match look {
            Look::Hover => (c as u16 + (255 - c as u16) / 6) as u8,
            Look::Pressed => (c as f32 * 0.8) as u8,
            Look::Disabled => c / 2,
            Look::Normal => c,
        }
    };
    [change(rgba[0]), change(rgba[1]), change(rgba[2]), rgba[3]]
}

/// A window that holds only the controls (`--controls`).
pub fn draw(pm: &mut Pixmap, panel: &Panel, plan: &Plan) {
    pm.fill(Color::from_rgba8(
        BACKGROUND[0],
        BACKGROUND[1],
        BACKGROUND[2],
        BACKGROUND[3],
    ));
    draw_controls(pm, panel, plan);
}

/// The controls only, on whatever is already drawn (a panel's card).
pub fn draw_controls(pm: &mut Pixmap, panel: &Panel, plan: &Plan) {
    let scale = plan.scale;

    // Toggle: a pill with a round handle at the end it is on.
    let look = panel.toggle.look(&panel.input, true);
    let t = plan.toggle;
    let track = if panel.toggle.on { ACCENT } else { TRACK };
    rounded_rect(pm, t, t.h / 2.0, shade(track, look));
    let handle_r = t.h / 2.0 - 3.0 * scale;
    let handle_x = if panel.toggle.on {
        t.x + t.w - t.h / 2.0
    } else {
        t.x + t.h / 2.0
    };
    circle(pm, handle_x, t.y + t.h / 2.0, handle_r, HANDLE);

    // Reset button: a button with a ring on it.
    let look = panel.reset.look(&panel.input, true);
    let r = plan.reset;
    let fill = shade(SURFACE, look);
    rounded_rect(pm, r, 8.0 * scale, fill);
    let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    arc(
        pm,
        (cx, cy),
        4.5 * scale,
        8.0 * scale,
        (-150.0, 150.0),
        HANDLE,
    );

    // Slider: a groove, the filled part, and a round thumb.
    let look = panel.slider.look_with(&panel.input);
    let s = plan.slider;
    let (track_geo, thumb) = plan.track();
    let groove_h = 6.0 * scale;
    let groove = Rect::new(s.x, s.y + (s.h - groove_h) / 2.0, s.w, groove_h);
    rounded_rect(pm, groove, groove_h / 2.0, TRACK);
    let thumb_x = track_geo.start + panel.slider.unit() * track_geo.len;
    let filled = Rect::new(s.x, groove.y, (thumb_x - s.x).max(0.0), groove_h);
    rounded_rect(pm, filled, groove_h / 2.0, ACCENT);
    circle(
        pm,
        thumb_x,
        s.y + s.h / 2.0,
        thumb / 2.0,
        shade(HANDLE, look),
    );

    // Knobs: a ring showing the value, a body and a pointer.
    for (i, knob) in panel.knobs.iter().enumerate() {
        let rect = plan.knob[i];
        let look = knob.look_with(&panel.input);
        let (cx, cy) = (rect.x + rect.w / 2.0, rect.y + rect.h / 2.0);
        let outer = rect.w.min(rect.h) / 2.0 - 2.0 * scale;
        let ring_in = outer - 8.0 * scale;
        let sweep = toolkit::knob::SWEEP_DEGREES / 2.0;
        arc(pm, (cx, cy), ring_in, outer, (-sweep, sweep), TRACK);
        if knob.unit() > 0.0 {
            arc(
                pm,
                (cx, cy),
                ring_in,
                outer,
                (-sweep, knob.angle_degrees()),
                ACCENT,
            );
        }
        let body = ring_in - 4.0 * scale;
        circle(pm, cx, cy, body, shade(SURFACE, look));
        // The pointer: a thin quad from near the centre to near the edge of the body.
        let a = knob.angle_degrees().to_radians();
        let (dx, dy) = (a.sin(), -a.cos());
        let (px, py) = (-dy, dx);
        let w = 1.5 * scale;
        let (from, to) = (body * 0.35, body * 0.9);
        let quad = [
            (cx + dx * from + px * w, cy + dy * from + py * w),
            (cx + dx * to + px * w, cy + dy * to + py * w),
            (cx + dx * to - px * w, cy + dy * to - py * w),
            (cx + dx * from - px * w, cy + dy * from - py * w),
        ];
        polygon(pm, &quad, HANDLE);
    }

    // Number displays.
    digits(
        pm,
        plan.slider_value,
        scale,
        &digits_text(panel.slider.value(), 1),
    );
    digits(
        pm,
        plan.knob_value[0],
        scale,
        &digits_text(panel.knobs[0].value(), 1),
    );
    digits(
        pm,
        plan.knob_value[1],
        scale,
        &digits_text(panel.knobs[1].value(), 0),
    );
}

/// The look of a slider or knob, from the pointer state.
trait LookWith {
    fn look_with(&self, input: &toolkit::input::Input) -> Look;
}

impl LookWith for toolkit::slider::Slider {
    fn look_with(&self, input: &toolkit::input::Input) -> Look {
        if input.pressed(self.id) {
            Look::Pressed
        } else if input.hovered(self.id) {
            Look::Hover
        } else {
            Look::Normal
        }
    }
}

impl LookWith for toolkit::knob::Knob {
    fn look_with(&self, input: &toolkit::input::Input) -> Look {
        if input.pressed(self.id) {
            Look::Pressed
        } else if input.hovered(self.id) {
            Look::Hover
        } else {
            Look::Normal
        }
    }
}

fn digits(pm: &mut Pixmap, area: Rect, scale: f32, text: &str) {
    let bars = digit_layout(text, area, scale, Align::End, digits_style());
    for bar in bars.unlit {
        rounded_rect(pm, bar, 1.0 * scale, DIGITS_UNLIT);
    }
    for bar in bars.lit {
        rounded_rect(pm, bar, 1.0 * scale, ACCENT);
    }
}

fn polygon(pm: &mut Pixmap, points: &[(f32, f32)], rgba: [u8; 4]) {
    let mut pb = tiny_skia::PathBuilder::new();
    for (i, (x, y)) in points.iter().enumerate() {
        if i == 0 {
            pb.move_to(*x, *y);
        } else {
            pb.line_to(*x, *y);
        }
    }
    pb.close();
    fill_path(pm, pb, rgba);
}

fn circle(pm: &mut Pixmap, cx: f32, cy: f32, r: f32, rgba: [u8; 4]) {
    if r <= 0.0 {
        return;
    }
    let points: Vec<(f32, f32)> = (0..48)
        .map(|i| {
            let a = i as f32 / 48.0 * std::f32::consts::TAU;
            (cx + a.cos() * r, cy + a.sin() * r)
        })
        .collect();
    polygon(pm, &points, rgba);
}

/// A band between two radii from one angle to another (degrees, 0 = up, clockwise).
fn arc(
    pm: &mut Pixmap,
    centre: (f32, f32),
    r_in: f32,
    r_out: f32,
    degrees: (f32, f32),
    rgba: [u8; 4],
) {
    let (a0, a1) = degrees;
    if a1 <= a0 || r_out <= r_in {
        return;
    }
    let steps = (((a1 - a0) / 5.0).ceil() as usize).max(1);
    let at = |step: usize, r: f32| {
        let a = (a0 + (a1 - a0) * step as f32 / steps as f32).to_radians();
        (centre.0 + a.sin() * r, centre.1 - a.cos() * r)
    };
    let mut points: Vec<(f32, f32)> = (0..=steps).map(|i| at(i, r_out)).collect();
    points.extend((0..=steps).rev().map(|i| at(i, r_in)));
    polygon(pm, &points, rgba);
}
