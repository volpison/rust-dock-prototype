//! A value between two ends, and how dragging changes it. Shared by sliders and knobs.

/// How much slower a "fine" drag (Shift held) moves the value.
pub const FINE_FACTOR: f32 = 0.1;

/// The ends of a value, an optional step, and the value it goes back to on reset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range {
    pub min: f32,
    pub max: f32,
    /// When set, values are multiples of it counted from `min` (and `max` itself).
    pub step: Option<f32>,
    pub default: f32,
}

impl Range {
    /// From `min` to `max`, no step, default at `min`.
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            min,
            max,
            step: None,
            default: min,
        }
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    pub fn with_default(mut self, default: f32) -> Self {
        self.default = self.snap(default);
        self
    }

    /// `value` held between the ends.
    pub fn clamp(&self, value: f32) -> f32 {
        if self.max <= self.min {
            return self.min;
        }
        if value.is_nan() {
            return self.min;
        }
        value.clamp(self.min, self.max)
    }

    /// `value` held between the ends and moved to the nearest step.
    pub fn snap(&self, value: f32) -> f32 {
        let value = self.clamp(value);
        match self.step {
            Some(step) if step > 0.0 => {
                let snapped = self.min + ((value - self.min) / step).round() * step;
                self.clamp(snapped)
            }
            _ => value,
        }
    }

    /// Where `value` sits between the ends: 0.0 at `min`, 1.0 at `max`.
    pub fn to_unit(&self, value: f32) -> f32 {
        if self.max <= self.min {
            return 0.0;
        }
        ((self.clamp(value) - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }

    /// The value at `unit` (held between 0.0 and 1.0), moved to the nearest step.
    pub fn from_unit(&self, unit: f32) -> f32 {
        let unit = if unit.is_nan() {
            0.0
        } else {
            unit.clamp(0.0, 1.0)
        };
        self.snap(self.min + unit * (self.max - self.min))
    }

    /// How far one wheel notch moves the value: a step, or a hundredth of the range.
    pub fn notch(&self) -> f32 {
        match self.step {
            Some(step) if step > 0.0 => step,
            _ => (self.max - self.min) / 100.0,
        }
    }
}

/// A drag in progress. The value follows how far the pointer has moved since an anchor, so
/// pressing on a control never makes the value jump, and switching to a fine drag halfway
/// does not either: the anchor is moved to where the pointer is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dragger {
    anchor_pos: f32,
    anchor_unit: f32,
    fine: bool,
}

impl Dragger {
    /// The drag began with the pointer at `pos` (along the drag's direction, in pixels) and
    /// the value at `unit` (see [`Range::to_unit`]).
    pub fn start(pos: f32, unit: f32) -> Self {
        Self {
            anchor_pos: pos,
            anchor_unit: unit.clamp(0.0, 1.0),
            fine: false,
        }
    }

    /// The value's position (0.0 to 1.0) for a pointer at `pos`, when moving `pixels_per_range`
    /// pixels takes the value across its whole range. `fine` slows it down. The position is
    /// not snapped: a drag stays smooth even when the value has steps.
    pub fn unit_at(&mut self, pos: f32, pixels_per_range: f32, fine: bool) -> f32 {
        if fine != self.fine {
            self.anchor_unit = self.unit_for(pos, pixels_per_range);
            self.anchor_pos = pos;
            self.fine = fine;
        }
        self.unit_for(pos, pixels_per_range)
    }

    fn unit_for(&self, pos: f32, pixels_per_range: f32) -> f32 {
        if pixels_per_range <= 0.0 || pixels_per_range.is_nan() {
            return self.anchor_unit;
        }
        let factor = if self.fine { FINE_FACTOR } else { 1.0 };
        (self.anchor_unit + (pos - self.anchor_pos) / pixels_per_range * factor).clamp(0.0, 1.0)
    }
}
