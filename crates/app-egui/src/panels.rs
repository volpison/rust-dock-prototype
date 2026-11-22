//! The panels themselves: their names and what they show.
//! Add a panel here (and to `ALL`); nothing else needs to change.

use dock::layout::PanelId;
use eframe::egui;

pub const TOOLS: PanelId = PanelId(1);
pub const LAYERS: PanelId = PanelId(2);
pub const PROPERTIES: PanelId = PanelId(3);

pub const ALL: [PanelId; 3] = [TOOLS, LAYERS, PROPERTIES];

pub fn title(id: PanelId) -> &'static str {
    match id {
        TOOLS => "Tools",
        LAYERS => "Layers",
        PROPERTIES => "Properties",
        _ => "Unknown panel",
    }
}

pub fn show(id: PanelId, ui: &mut egui::Ui) {
    ui.heading(title(id));
    ui.label("Placeholder content for this panel.");
}
