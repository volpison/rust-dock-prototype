//! Screen geometry for dropping a dragged tab. Pure arithmetic on `(x, y)` pairs, in the
//! same units egui uses (points). No UI, disk or network.

/// True when `point` (relative to the window's top-left corner) lies outside a window of
/// `size` by more than `margin` on at least one side. A margin stops a small overshoot
/// of the edge from counting as "dragged out".
pub fn is_outside_window(point: (f32, f32), size: (f32, f32), margin: f32) -> bool {
    point.0 < -margin || point.1 < -margin || point.0 > size.0 + margin || point.1 > size.1 + margin
}

/// Turn a position relative to a window into a position on the screen.
pub fn to_screen(window_origin: (f32, f32), point: (f32, f32)) -> (f32, f32) {
    (window_origin.0 + point.0, window_origin.1 + point.1)
}

/// Where to put a new window of `window_width` so that its title bar sits under the
/// cursor: centred horizontally, with the cursor `title_offset` below the window's top.
pub fn window_origin_under_cursor(
    cursor: (f32, f32),
    window_width: f32,
    title_offset: f32,
) -> (f32, f32) {
    (cursor.0 - window_width / 2.0, cursor.1 - title_offset)
}
