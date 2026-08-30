use std::path::PathBuf;

use eframe::egui;
use sheetz::state::SheetzApp;

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
        Box::new(move |cc| {
            sheetz::fonts::install(&cc.egui_ctx);
            Ok(Box::new(SheetzApp::new(path)))
        }),
    )
}
