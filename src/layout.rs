//! Where each panel lives: docked in the main window, or detached in its own window.
//!
//! Invariant: every panel is in exactly one place (docked or detached), never both,
//! never lost. All changes go through the methods below, which keep that true.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PanelId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetachedWindow {
    pub panel: PanelId,
    /// Screen position of the window, if known. `None` lets the OS decide
    /// (always the case on Wayland, which does not let apps place windows).
    pub position: Option<(f32, f32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    UnknownPanel(PanelId),
    AlreadyDetached(PanelId),
    AlreadyDocked(PanelId),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Layout {
    docked: Vec<PanelId>,
    detached: Vec<DetachedWindow>,
}

impl Layout {
    /// All panels start docked, in the given order. Duplicates are ignored.
    pub fn new(panels: impl IntoIterator<Item = PanelId>) -> Self {
        let mut docked = Vec::new();
        for p in panels {
            if !docked.contains(&p) {
                docked.push(p);
            }
        }
        Self { docked, detached: Vec::new() }
    }

    pub fn docked(&self) -> &[PanelId] {
        &self.docked
    }

    pub fn detached(&self) -> &[DetachedWindow] {
        &self.detached
    }

    pub fn is_docked(&self, id: PanelId) -> bool {
        self.docked.contains(&id)
    }

    pub fn is_detached(&self, id: PanelId) -> bool {
        self.detached.iter().any(|w| w.panel == id)
    }

    /// Move a docked panel into its own window.
    pub fn detach(&mut self, id: PanelId, position: Option<(f32, f32)>) -> Result<(), LayoutError> {
        if self.is_detached(id) {
            return Err(LayoutError::AlreadyDetached(id));
        }
        let index = self
            .docked
            .iter()
            .position(|p| *p == id)
            .ok_or(LayoutError::UnknownPanel(id))?;
        self.docked.remove(index);
        self.detached.push(DetachedWindow { panel: id, position });
        Ok(())
    }

    /// Move a detached panel back into the main window (e.g. when its window is closed).
    pub fn redock(&mut self, id: PanelId) -> Result<(), LayoutError> {
        if self.is_docked(id) {
            return Err(LayoutError::AlreadyDocked(id));
        }
        let index = self
            .detached
            .iter()
            .position(|w| w.panel == id)
            .ok_or(LayoutError::UnknownPanel(id))?;
        self.detached.remove(index);
        self.docked.push(id);
        Ok(())
    }

    /// Remember where the user moved a detached window.
    pub fn set_position(&mut self, id: PanelId, position: (f32, f32)) -> Result<(), LayoutError> {
        let window = self
            .detached
            .iter_mut()
            .find(|w| w.panel == id)
            .ok_or(LayoutError::UnknownPanel(id))?;
        window.position = Some(position);
        Ok(())
    }
}
