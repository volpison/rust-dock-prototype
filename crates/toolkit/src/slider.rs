//! A slider: a thumb on a track. Works along either direction: the caller gives positions
//! along the track in pixels (x for a horizontal slider, y for a vertical one).

use crate::input::WidgetId;
use crate::value::{Dragger, Range};

/// The part of the track the thumb's centre travels on, in pixels along the slider's direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Track {
    /// Where the thumb's centre is at the minimum.
    pub start: f32,
    /// How far the centre travels to the maximum.
    pub len: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slider {
    pub id: WidgetId,
    pub range: Range,
    value: f32,
    drag: Option<Dragger>,
}

impl Slider {
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

    /// Where the thumb sits between the ends: 0.0 to 1.0.
    pub fn unit(&self) -> f32 {
        self.range.to_unit(self.value)
    }

    pub fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    /// Set the value from outside (held to the range and its steps). Returns the new value
    /// when it changed.
    pub fn set(&mut self, value: f32) -> Option<f32> {
        self.change_to(self.range.snap(value))
    }

    /// The button went down at `pos` along the track. On the thumb (within `thumb_len` of its
    /// centre) nothing moves and the thumb is grabbed where it is; elsewhere on the track the
    /// thumb jumps under the pointer and is grabbed there. Returns the new value if it changed.
    pub fn press(&mut self, pos: f32, track: Track, thumb_len: f32) -> Option<f32> {
        let unit = self.unit();
        let centre = track.start + unit * track.len;
        if (pos - centre).abs() <= thumb_len.max(0.0) / 2.0 {
            self.drag = Some(Dragger::start(pos, unit));
            return None;
        }
        let jumped = if track.len > 0.0 {
            ((pos - track.start) / track.len).clamp(0.0, 1.0)
        } else {
            unit
        };
        self.drag = Some(Dragger::start(pos, jumped));
        self.change_to(self.range.from_unit(jumped))
    }

    /// The pointer moved to `pos` along the track while the button is held; `fine` is true
    /// while Shift is held. Returns the new value if it changed.
    pub fn drag(&mut self, pos: f32, track: Track, fine: bool) -> Option<f32> {
        let unit = self.drag.as_mut()?.unit_at(pos, track.len, fine);
        self.change_to(self.range.from_unit(unit))
    }

    /// The button came up (or the drag was lost).
    pub fn release(&mut self) {
        self.drag = None;
    }

    /// The wheel turned by `notches` (positive = towards the maximum). With steps, one notch
    /// is one step and `fine` makes no difference; without, a notch is a hundredth of the range
    /// and `fine` makes it a tenth of that. Returns the new value if it changed.
    pub fn wheel(&mut self, notches: f32, fine: bool) -> Option<f32> {
        let slow = if fine && self.range.step.is_none() {
            crate::value::FINE_FACTOR
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
