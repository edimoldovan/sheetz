mod app;
mod commands;
mod engine;
mod keymap;

use std::path::PathBuf;

use eframe::egui;

fn main() -> eframe::Result {
    let path = std::env::args().nth(1).map(PathBuf::from);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("Sheetz"),
        ..Default::default()
    };
    eframe::run_native(
        "sheetz",
        options,
        Box::new(move |_cc| Ok(Box::new(app::SheetzApp::new(path)))),
    )
}
