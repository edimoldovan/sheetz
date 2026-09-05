use std::path::PathBuf;

use eframe::egui;
use sheetz::mcp;
use sheetz::state::SheetzApp;

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // `sheetz mcp` is the stdio shim an MCP client launches. It is a
    // convenience, never a requirement: the server itself comes up with the
    // GUI, however the GUI was started.
    if args.first().map(String::as_str) == Some("mcp") {
        if let Err(e) = mcp::proto::stdio_shim() {
            eprintln!("sheetz mcp: {e}");
            std::process::exit(1);
        }
        return Ok(());
    }

    // `sheetz register` wires this install into the MCP clients it finds.
    if args.first().map(String::as_str) == Some("register") {
        let (done, skipped) = mcp::register::register_all();
        if done.is_empty() {
            println!("Registered with nothing.");
        } else {
            println!("Registered with: {}", done.join(", "));
        }
        for note in skipped {
            println!("Skipped {note}");
        }
        return Ok(());
    }

    let path = args
        .into_iter()
        .find(|a| !a.starts_with('-'))
        .map(PathBuf::from);

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
            let mut app = SheetzApp::new(path);

            // Start the assistant bridge unconditionally: launching from a
            // desktop launcher must be exactly as capable as a terminal start.
            let rx = mcp::bridge::install();
            mcp::bridge::set_wake(cc.egui_ctx.clone());
            app.mcp_rx = Some(rx);
            app.mcp_serving = mcp::proto::spawn_server().is_some();

            Ok(Box::new(app))
        }),
    )
}
