//! The window layer: draws the layout and reports what the user did.
//! The rules (a panel is docked or detached, never both) live in `dock::layout::Layout`.

use dock::geometry::{is_outside_window, is_over_window, to_screen, window_origin_under_cursor};
use dock::layout::{Layout, PanelId};
use eframe::egui;
use egui_dock::{DockArea, DockState, NodePath, TabViewer};

use crate::panels;

/// Size of a detached panel's window, in points.
const DETACHED_SIZE: [f32; 2] = [320.0, 420.0];
/// How far below the new window's top edge the cursor sits when a tab is dropped.
const TITLE_OFFSET: f32 = 15.0;
/// How far past the window edge a tab must be dropped to count as "dragged out".
const DRAG_OUT_MARGIN: f32 = 8.0;

pub struct DockApp {
    /// Source of truth for where each panel is.
    layout: Layout,
    /// How the docked panels are arranged in the main window (tabs, splits).
    dock: DockState<PanelId>,
    /// The tab under the pointer when the mouse button went down. egui_dock does not
    /// say which tab is being dragged, so this is how a drag-out finds its panel.
    pressed_tab: Option<PanelId>,
    /// Whether the primary mouse button was down in the previous frame.
    was_down: bool,
}

impl DockApp {
    pub fn new() -> Self {
        let ids = panels::ALL.to_vec();
        Self {
            layout: Layout::new(ids.clone()),
            dock: DockState::new(ids),
            pressed_tab: None,
            was_down: false,
        }
    }

    fn detach(&mut self, id: PanelId, position: Option<(f32, f32)>) {
        if self.layout.detach(id, position).is_err() {
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

    /// Watches the mouse: remembers which tab the button went down on, and when the
    /// button is released outside the window, detaches that panel where it was dropped.
    /// The pointer keeps reporting positions outside the window while the button is held.
    fn handle_drag_out(&mut self, ctx: &egui::Context, hovered_tab: Option<PanelId>) {
        let (down, pressed, pos, inner) = ctx.input(|i| {
            (
                i.pointer.primary_down(),
                i.pointer.primary_pressed(),
                i.pointer.latest_pos(),
                i.viewport().inner_rect,
            )
        });

        if pressed {
            self.pressed_tab = hovered_tab;
        }

        let released = self.was_down && !down;
        self.was_down = down;
        if !released {
            return;
        }
        let Some(id) = self.pressed_tab.take() else {
            return;
        };
        let (Some(pos), Some(inner)) = (pos, inner) else {
            return;
        };

        let point = (pos.x, pos.y);
        let size = (inner.width(), inner.height());
        if is_outside_window(point, size, DRAG_OUT_MARGIN) {
            let cursor = to_screen((inner.min.x, inner.min.y), point);
            let origin = window_origin_under_cursor(cursor, DETACHED_SIZE[0], TITLE_OFFSET);
            self.detach(id, Some(origin));
        }
    }

    /// Each detached panel is its own OS window (an egui immediate viewport).
    fn show_detached_windows(&mut self, ctx: &egui::Context) {
        let mut to_redock: Vec<PanelId> = Vec::new();
        // Where the main window is on the screen (called from the main window's frame).
        let main_rect = ctx.input(|i| i.viewport().inner_rect);

        for window in self.layout.detached().to_vec() {
            let id = window.panel;
            let viewport_id = egui::ViewportId::from_hash_of(("detached-panel", id.0));
            let mut builder = egui::ViewportBuilder::default()
                .with_title(panels::title(id))
                .with_inner_size(DETACHED_SIZE);
            if let Some((x, y)) = window.position {
                builder = builder.with_position([x, y]);
            }

            ctx.show_viewport_immediate(viewport_id, builder, |ui, _class| {
                // Drag this strip and let go over the main window to dock the panel.
                // The pointer keeps reporting positions outside this window while the
                // button is held, so the drop can be tested against the main window.
                let handle = ui
                    .add(
                        egui::Label::new(
                            egui::RichText::new(
                                "Drag here, then let go over the main window to dock",
                            )
                            .strong(),
                        )
                        // Not selectable: dragging must not select the text.
                        .selectable(false)
                        .sense(egui::Sense::drag()),
                    )
                    .on_hover_cursor(egui::CursorIcon::Grab);
                if handle.drag_stopped() {
                    let (pos, inner) =
                        ui.input(|i| (i.pointer.latest_pos(), i.viewport().inner_rect));
                    if let (Some(pos), Some(inner), Some(main)) = (pos, inner, main_rect) {
                        let cursor = to_screen((inner.min.x, inner.min.y), (pos.x, pos.y));
                        let origin = (main.min.x, main.min.y);
                        if is_over_window(cursor, origin, (main.width(), main.height())) {
                            to_redock.push(id);
                        }
                    }
                }
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
    /// The tab the pointer is over this frame, if any.
    hovered_tab: Option<PanelId>,
}

impl TabViewer for Viewer {
    type Tab = PanelId;

    fn id(&mut self, tab: &mut PanelId) -> egui::Id {
        egui::Id::unique(*tab)
    }

    fn title(&mut self, tab: &mut PanelId) -> egui::WidgetText {
        panels::title(*tab).into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut PanelId) {
        panels::show(*tab, ui);
    }

    fn on_tab_button(&mut self, tab: &mut PanelId, response: &egui::Response) {
        if response.hovered() {
            self.hovered_tab = Some(*tab);
        }
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
            hovered_tab: None,
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
            self.detach(id, None);
        }

        let ctx = ui.ctx().clone();
        self.handle_drag_out(&ctx, viewer.hovered_tab);
        self.show_detached_windows(&ctx);
    }
}
