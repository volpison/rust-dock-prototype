//! The panels themselves: their ids, names and colours. There is no text drawing yet
//! (stage 5), so a panel is told apart by its colour here and by its window title.

use dock::layout::PanelId;

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

/// The colour of a panel's block, as red, green, blue, alpha.
pub fn color(id: PanelId) -> [u8; 4] {
    match id {
        TOOLS => [64, 110, 190, 255],
        LAYERS => [66, 150, 110, 255],
        PROPERTIES => [200, 130, 60, 255],
        _ => [120, 120, 120, 255],
    }
}
