//! Controls as small state machines: input in, "clicked" or "changed" out, and how they look.

use crate::input::{Input, Released, WidgetId};

/// How a control is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Look {
    Normal,
    Hover,
    Pressed,
    Disabled,
}

fn look(input: &Input, id: WidgetId, enabled: bool) -> Look {
    if !enabled {
        Look::Disabled
    } else if input.pressed(id) {
        Look::Pressed
    } else if input.hovered(id) {
        Look::Hover
    } else {
        Look::Normal
    }
}

/// A push button: it has no state of its own, only clicks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Button {
    pub id: WidgetId,
}

impl Button {
    pub fn new(id: WidgetId) -> Self {
        Self { id }
    }

    pub fn look(&self, input: &Input, enabled: bool) -> Look {
        look(input, self.id, enabled)
    }

    /// True when `released` is a click on this button.
    pub fn clicked(&self, released: Released, enabled: bool) -> bool {
        enabled && released == Released::Click(self.id)
    }
}

/// An on/off switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Toggle {
    pub id: WidgetId,
    pub on: bool,
}

impl Toggle {
    pub fn new(id: WidgetId, on: bool) -> Self {
        Self { id, on }
    }

    pub fn look(&self, input: &Input, enabled: bool) -> Look {
        look(input, self.id, enabled)
    }

    /// Flips on a click and returns the new state; `None` when `released` is not a click on it.
    pub fn handle(&mut self, released: Released, enabled: bool) -> Option<bool> {
        if enabled && released == Released::Click(self.id) {
            self.on = !self.on;
            Some(self.on)
        } else {
            None
        }
    }
}
