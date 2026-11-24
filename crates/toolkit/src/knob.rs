//! A knob: dragging up or down turns it. It does not matter where on the knob the pointer
//! went down, and the pointer may leave the knob while turning it.

use crate::input::WidgetId;
use crate::value::{Dragger, Range, FINE_FACTOR};

/// The knob's pointer sweeps this many degrees from its minimum to its maximum.
pub const SWEEP_DEGREES: f32 = 270.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Knob {
    pub id: WidgetId,
    pub range: Range,
    value: f32,
    drag: Option<Dragger>,
}

impl Knob {
    pub fn new(id: WidgetId, range: Range, value: f32) -> Self {
        Self {
            id,
            range,
            value: range.snap(value),
            drag: None,
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    /// Where the value sits between the ends: 0.0 to 1.0.
    pub fn unit(&self) -> f32 {
        self.range.to_unit(self.value)
    }

    /// The angle of the knob's pointer in degrees, 0 straight up, negative to the left:
    /// -135 at the minimum, +135 at the maximum.
    pub fn angle_degrees(&self) -> f32 {
        (self.unit() - 0.5) * SWEEP_DEGREES
    }

    pub fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    /// Set the value from outside. Returns the new value when it changed.
    pub fn set(&mut self, value: f32) -> Option<f32> {
        self.change_to(self.range.snap(value))
    }

    /// The button went down with the pointer at height `y` (pixels, growing downwards).
    pub fn press(&mut self, y: f32) {
        self.drag = Some(Dragger::start(-y, self.unit()));
    }

    /// The pointer moved to height `y` while the button is held. Moving up turns the knob up:
    /// `pixels_per_range` pixels of movement take it from one end to the other. `fine` (Shift)
    /// slows it down. Returns the new value if it changed.
    pub fn drag(&mut self, y: f32, pixels_per_range: f32, fine: bool) -> Option<f32> {
        let unit = self.drag.as_mut()?.unit_at(-y, pixels_per_range, fine);
        self.change_to(self.range.from_unit(unit))
    }

    /// The button came up (or the drag was lost).
    pub fn release(&mut self) {
        self.drag = None;
    }

    /// The wheel turned by `notches` (positive = up). Same rules as [`crate::slider::Slider::wheel`].
    pub fn wheel(&mut self, notches: f32, fine: bool) -> Option<f32> {
        let slow = if fine && self.range.step.is_none() {
            FINE_FACTOR
        } else {
            1.0
        };
        self.change_to(
            self.range
                .snap(self.value + notches * self.range.notch() * slow),
        )
    }

    /// Back to the default value (a double-click). Returns it if the value changed.
    pub fn reset(&mut self) -> Option<f32> {
        self.change_to(self.range.snap(self.range.default))
    }

    fn change_to(&mut self, value: f32) -> Option<f32> {
        if value == self.value {
            None
        } else {
            self.value = value;
            Some(value)
        }
    }
}
