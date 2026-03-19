mod app;
mod http;
mod models;
mod ui;

use app::ApiClientApp;
use egui::ViewportBuilder;

#[tokio::main]
async fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "API Client",
        options,
        Box::new(|cc| Ok(Box::new(ApiClientApp::new(cc)))),
    )
}
