mod app;
mod http;
mod thread;
mod task;
mod state;
mod util;

use eframe::egui;
use app::App;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800., 600.]),
        ..Default::default()
    };
    eframe::run_native(
        "NossoChat",
        options,
        Box::new(|_| {
            Ok(Box::<App>::default())
        })
    )
}
