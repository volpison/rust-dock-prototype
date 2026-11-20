//! The window layer: draws the layout and reports what the user did.
//! The rules (a panel is docked or detached, never both) live in `dock::layout::Layout`.

use dock::layout::{Layout, PanelId};
use eframe::egui;
use egui_dock::{DockArea, DockState, NodePath, TabViewer};

use crate::panels;

pub struct DockApp {
    /// Source of truth for where each panel is.
    layout: Layout,
    /// How the docked panels are arranged in the main window (tabs, splits).
    dock: DockState<PanelId>,
}

impl DockApp {
    pub fn new() -> Self {
        let ids = panels::ALL.to_vec();
        Self {
            layout: Layout::new(ids.clone()),
            dock: DockState::new(ids),
        }
    }

    fn detach(&mut self, id: PanelId) {
        if self.layout.detach(id, None).is_err() {
            return;
        }
        if let Some(path) = self.dock.find_tab(&id) {
            self.dock.remove_tab(path);
        }
    }

    fn redock(&mut self, id: PanelId) {
        if self.layout.redock(id).is_ok() {
            self.dock.push_to_focused_leaf(id);
        }
    }

    /// Each detached panel is its own OS window (an egui immediate viewport).
    fn show_detached_windows(&mut self, ctx: &egui::Context) {
        let mut to_redock: Vec<PanelId> = Vec::new();

        for window in self.layout.detached().to_vec() {
            let id = window.panel;
            let viewport_id = egui::ViewportId::from_hash_of(("detached-panel", id.0));
            let mut builder = egui::ViewportBuilder::default()
                .with_title(panels::title(id))
                .with_inner_size([320.0, 420.0]);
            if let Some((x, y)) = window.position {
                builder = builder.with_position([x, y]);
            }

            ctx.show_viewport_immediate(viewport_id, builder, |ui, _class| {
                if ui.button("Dock back").clicked() {
                    to_redock.push(id);
                }
                ui.separator();
                panels::show(id, ui);
                // Closing the window puts the panel back in the main window.
                if ui.input(|i| i.viewport().close_requested()) {
                    to_redock.push(id);
                }
            });
        }

        for id in to_redock {
            self.redock(id);
        }
    }
}

/// Tells egui_dock how to draw a panel, and collects "detach" requests.
struct Viewer {
    detach_requests: Vec<PanelId>,
}

impl TabViewer for Viewer {
    type Tab = PanelId;

    fn id(&mut self, tab: &mut PanelId) -> egui::Id {
        egui::Id::new(*tab)
    }

    fn title(&mut self, tab: &mut PanelId) -> egui::WidgetText {
        panels::title(*tab).into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut PanelId) {
        panels::show(*tab, ui);
    }

    fn context_menu(&mut self, ui: &mut egui::Ui, tab: &mut PanelId, _path: NodePath) {
        if ui.button("Detach to window").clicked() {
            self.detach_requests.push(*tab);
        }
    }

    // Closing a tab would lose the panel, and turning it into an in-window floating
    // panel would put it in a third place. Both would break the layout's rule.
    fn is_closeable(&self, _tab: &PanelId) -> bool {
        false
    }

    fn allowed_in_windows(&self, _tab: &mut PanelId) -> bool {
        false
    }
}

impl eframe::App for DockApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut viewer = Viewer {
            detach_requests: Vec::new(),
        };
        if self.layout.docked().is_empty() {
            // Empty state: every panel is in its own window.
            ui.centered_and_justified(|ui| {
                ui.label("All panels are in their own windows. Use \"Dock back\" in a panel's window, or close it, to bring it here.");
            });
        } else {
            DockArea::new(&mut self.dock).show_inside(ui, &mut viewer);
        }

        for id in viewer.detach_requests {
            self.detach(id);
        }

        let ctx = ui.ctx().clone();
        self.show_detached_windows(&ctx);
    }
}
