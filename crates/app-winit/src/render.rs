//! Drawing with tiny-skia into a `Pixmap`. It only draws; where things are comes from `ui.rs`.
//! There is no text yet, so the buttons carry a small arrow instead of a word:
//! pointing right = "detach" (send the panel out), pointing left = "dock back".

use dock::layout::PanelId;
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Transform};

use crate::panels;
use crate::ui::{self, Rect};

const BACKGROUND: [u8; 4] = [30, 33, 41, 255];
const BUTTON: [u8; 4] = [20, 22, 28, 255];
const BUTTON_HOVER: [u8; 4] = [70, 76, 92, 255];
const ARROW: [u8; 4] = [240, 240, 240, 255];

/// The main window: one block per docked panel, each with a "detach" button.
/// `hover` is the panel whose button is under the pointer; `dragging` is the panel being
/// dragged, whose block is drawn dimmed.
pub fn draw_main(
    pm: &mut Pixmap,
    scale: f32,
    docked: &[PanelId],
    hover: Option<PanelId>,
    dragging: Option<PanelId>,
) {
    pm.fill(Color::from_rgba8(
        BACKGROUND[0],
        BACKGROUND[1],
        BACKGROUND[2],
        BACKGROUND[3],
    ));
    for row in ui::main_rows(pm.width() as f32, scale, docked) {
        let fill = block_color(row.panel, dragging == Some(row.panel));
        rounded_rect(pm, row.area, 10.0 * scale, fill);
        button(
            pm,
            row.button,
            scale,
            hover == Some(row.panel),
            Arrow::Right,
        );
    }
}

/// A detached window: a "dock back" button and the panel's block (dimmed while `dragging`).
pub fn draw_detached(pm: &mut Pixmap, scale: f32, panel: PanelId, hover: bool, dragging: bool) {
    pm.fill(Color::from_rgba8(
        BACKGROUND[0],
        BACKGROUND[1],
        BACKGROUND[2],
        BACKGROUND[3],
    ));
    let view = ui::detached_view(pm.width() as f32, pm.height() as f32, scale);
    rounded_rect(pm, view.content, 10.0 * scale, block_color(panel, dragging));
    button(pm, view.button, scale, hover, Arrow::Left);
}

/// The ghost that follows the pointer during a drag: the panel's colour in a light frame.
/// When the window is see-through (`see_through`) the corners are rounded and show what is
/// behind; otherwise the window is a plain rectangle, which looks right everywhere.
pub fn draw_ghost(pm: &mut Pixmap, scale: f32, panel: PanelId, see_through: bool) {
    let (w, h) = (pm.width() as f32, pm.height() as f32);
    if see_through {
        pm.fill(Color::TRANSPARENT);
        let whole = Rect {
            x: 0.0,
            y: 0.0,
            w,
            h,
        };
        rounded_rect(pm, whole, 10.0 * scale, ARROW);
    } else {
        pm.fill(Color::from_rgba8(ARROW[0], ARROW[1], ARROW[2], 255));
    }
    let inset = (3.0 * scale).round();
    let inner = Rect {
        x: inset,
        y: inset,
        w: (pm.width() as f32 - 2.0 * inset).max(0.0),
        h: (pm.height() as f32 - 2.0 * inset).max(0.0),
    };
    rounded_rect(pm, inner, 6.0 * scale, panels::color(panel));
}

/// The panel's colour, or that colour half way to the background while it is dragged.
fn block_color(panel: PanelId, dragging: bool) -> [u8; 4] {
    let color = panels::color(panel);
    if !dragging {
        return color;
    }
    let mix = |c: u8, b: u8| ((c as u16 + b as u16) / 2) as u8;
    [
        mix(color[0], BACKGROUND[0]),
        mix(color[1], BACKGROUND[1]),
        mix(color[2], BACKGROUND[2]),
        255,
    ]
}

enum Arrow {
    Left,
    Right,
}

fn button(pm: &mut Pixmap, rect: Rect, scale: f32, hover: bool, arrow: Arrow) {
    let fill = if hover { BUTTON_HOVER } else { BUTTON };
    rounded_rect(pm, rect, 8.0 * scale, fill);

    // A triangle in the middle of the button.
    let (cx, cy) = (rect.x + rect.w / 2.0, rect.y + rect.h / 2.0);
    let (half_h, half_w) = (10.0 * scale, 9.0 * scale);
    let points = match arrow {
        Arrow::Right => [
            (cx - half_w, cy - half_h),
            (cx - half_w, cy + half_h),
            (cx + half_w, cy),
        ],
        Arrow::Left => [
            (cx + half_w, cy - half_h),
            (cx + half_w, cy + half_h),
            (cx - half_w, cy),
        ],
    };
    triangle(pm, points, ARROW);
}

pub(crate) fn fill_path(pm: &mut Pixmap, builder: PathBuilder, rgba: [u8; 4]) {
    if let Some(path) = builder.finish() {
        let mut paint = Paint::default();
        paint.set_color_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]);
        paint.anti_alias = true;
        pm.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

fn triangle(pm: &mut Pixmap, points: [(f32, f32); 3], rgba: [u8; 4]) {
    let mut pb = PathBuilder::new();
    pb.move_to(points[0].0, points[0].1);
    pb.line_to(points[1].0, points[1].1);
    pb.line_to(points[2].0, points[2].1);
    pb.close();
    fill_path(pm, pb, rgba);
}

/// A filled rounded rectangle.
pub(crate) fn rounded_rect(pm: &mut Pixmap, rect: Rect, radius: f32, rgba: [u8; 4]) {
    let Rect { x, y, w, h } = rect;
    let r = radius.min(w / 2.0).min(h / 2.0).max(0.0);
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
    fill_path(pm, pb, rgba);
}
