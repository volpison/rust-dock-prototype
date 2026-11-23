//! Pointer state for one window: which control the pointer is over, and which one has the
//! button held down ("captured": it keeps the pointer until the button is let go, even when
//! the pointer leaves it or the window).
//!
//! The caller says which control is under the pointer (`under`); this file keeps the rules.
//! Every call that can change what a control should look like returns `true`, so the caller
//! redraws only then.

/// Names a control. Chosen by the caller; only has to be different for each control in a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(pub u32);

/// What letting go of the button meant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Released {
    /// The button had not gone down on a control.
    Nothing,
    /// The button went down on this control and came up over it: a click.
    Click(WidgetId),
    /// The button went down on this control and came up somewhere else: not a click.
    Cancelled(WidgetId),
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Input {
    pointer: Option<(f32, f32)>,
    under: Option<WidgetId>,
    captured: Option<WidgetId>,
}

impl Input {
    pub fn new() -> Self {
        Self::default()
    }

    /// The pointer position in the window, if it is known (`None` once it has left).
    pub fn pointer(&self) -> Option<(f32, f32)> {
        self.pointer
    }

    /// The control that has the button held down, if any.
    pub fn captured(&self) -> Option<WidgetId> {
        self.captured
    }

    /// True when the pointer is over `id` and nothing else has the button held down.
    pub fn hovered(&self, id: WidgetId) -> bool {
        self.under == Some(id) && self.captured.map_or(true, |c| c == id)
    }

    /// True when `id` has the button held down and the pointer is still over it. Dragging off
    /// a control stops it looking pressed; coming back makes it look pressed again.
    pub fn pressed(&self, id: WidgetId) -> bool {
        self.captured == Some(id) && self.under == Some(id)
    }

    /// The pointer moved to `pos`, over the control `under` (if any). Returns true when what
    /// a control looks like changed.
    pub fn pointer_moved(&mut self, pos: (f32, f32), under: Option<WidgetId>) -> bool {
        let before = self.look_key();
        self.pointer = Some(pos);
        self.under = under;
        before != self.look_key()
    }

    /// The pointer left the window. While a button is held the pointer keeps being reported
    /// outside the window, so nothing is forgotten then. Returns true when a look changed.
    pub fn pointer_left(&mut self) -> bool {
        if self.captured.is_some() {
            return false;
        }
        let before = self.look_key();
        self.pointer = None;
        self.under = None;
        before != self.look_key()
    }

    /// The button went down at `pos`, over `under`. A control under the pointer captures it.
    /// A press while another control already has the button is ignored. Returns true when a
    /// look changed.
    pub fn press(&mut self, pos: (f32, f32), under: Option<WidgetId>) -> bool {
        let before = self.look_key();
        self.pointer = Some(pos);
        self.under = under;
        if self.captured.is_none() {
            self.captured = under;
        }
        before != self.look_key()
    }

    /// The button came up at `pos`, over `under`. Ends the capture and says what it meant.
    /// Use [`Input::pointer_moved`]'s result and this one's `changed` together to redraw.
    pub fn release(&mut self, pos: (f32, f32), under: Option<WidgetId>) -> (Released, bool) {
        let before = self.look_key();
        self.pointer = Some(pos);
        self.under = under;
        let released = match self.captured.take() {
            None => Released::Nothing,
            Some(id) if under == Some(id) => Released::Click(id),
            Some(id) => Released::Cancelled(id),
        };
        (released, before != self.look_key())
    }

    /// Forget a held button without a click (the window lost the pointer, closed, ...).
    /// Returns true when a look changed.
    pub fn cancel(&mut self) -> bool {
        let before = self.look_key();
        self.captured = None;
        before != self.look_key()
    }

    /// Everything that decides how a control looks: who is hovered, who is pressed.
    fn look_key(&self) -> (Option<WidgetId>, Option<WidgetId>) {
        let hovered = self.under.filter(|id| self.hovered(*id));
        let pressed = self.captured.filter(|id| self.pressed(*id));
        (hovered, pressed)
    }
}
