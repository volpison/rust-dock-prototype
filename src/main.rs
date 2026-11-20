//! Window layer entry point. All placement rules live in the `dock` library (src/layout.rs).

mod app;
mod panels;

fn main() -> eframe::Result {
    eframe::run_native(
        "Dock prototype",
        eframe::NativeOptions::default(),
        Box::new(|_creation_context| Ok(Box::new(app::DockApp::new()))),
    )
}
