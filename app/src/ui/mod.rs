//! The settings window.

mod app;
mod preview;
mod theme;

use eframe::egui;
use std::path::PathBuf;

pub fn run(config_path: PathBuf) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 640.0])
            .with_min_inner_size([560.0, 560.0])
            .with_app_id("predator-rgb"), // matches predator-rgb.desktop so the taskbar shows the right icon
        ..Default::default()
    };
    eframe::run_native(
        "Predator RGB",
        options,
        Box::new(|cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(app::App::new(config_path)))
        }),
    )
}
