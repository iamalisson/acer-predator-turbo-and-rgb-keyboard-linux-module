//! Window state and layout.

use super::{
    preview,
    theme::{self, MUTED, card},
};
use crate::{
    config::{self, Config, Direction, Effect},
    device,
};
use eframe::egui::{self, RichText};
use std::path::PathBuf;

pub struct App {
    config: Config,
    config_path: PathBuf,
    /// `None` until the first change, then the outcome of the last apply/save.
    status: Option<Result<(), String>>,
    model: String,
}

impl App {
    pub fn new(config_path: PathBuf) -> Self {
        let model = std::fs::read_to_string("/sys/class/dmi/id/product_name").unwrap_or_default();
        Self {
            config: config::load(&config_path),
            config_path,
            status: None,
            model: model.trim().to_owned(),
        }
    }

    /// Sends the current settings to the keyboard and saves them.
    fn commit(&mut self) {
        // shortcut: writes to the device on every change (incl. color drags); throttle if it ever lags
        let result = device::apply(&self.config)
            .map_err(|e| format!("Keyboard not reachable. Is the facer module loaded? ({e})"))
            .and_then(|()| {
                config::save(&self.config_path, &self.config)
                    .map_err(|e| format!("Applied, but saving failed: {e}"))
            });
        self.status = Some(result);
    }

    fn header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Predator RGB").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(&self.model).color(MUTED));
            });
        });
        ui.label(RichText::new("Keyboard backlight · 4 zones").color(MUTED));
    }

    fn status_line(&self, ui: &mut egui::Ui) {
        let (dot, message) = match &self.status {
            None => (MUTED, "Changes apply instantly"),
            Some(Ok(())) => (theme::OK, "Applied and saved"),
            Some(Err(e)) => (theme::ERROR, e.as_str()),
        };
        ui.horizontal(|ui| {
            ui.label(RichText::new("●").color(dot));
            ui.label(RichText::new(message).small().color(MUTED));
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::BG).inner_margin(20))
            .show(ui, |ui| {
                self.header(ui);
                ui.add_space(12.0);

                let time = ui.input(|i| i.time) as f32;
                preview::show(ui, preview::zone_colors(&self.config, time));
                if self.config.effect != Effect::Static && self.config.speed > 0 {
                    ui.ctx().request_repaint();
                }
                ui.add_space(18.0);

                let before = self.config.clone();
                effect_picker(ui, &mut self.config);
                settings(ui, &mut self.config);
                card(ui, "Startup", |ui| {
                    ui.checkbox(
                        &mut self.config.apply_on_boot,
                        "Restore these settings after reboot",
                    );
                });
                if self.config != before {
                    self.commit();
                }

                self.status_line(ui);
            });
    }
}

fn effect_icon(effect: Effect) -> &'static str {
    match effect {
        Effect::Static => "■",
        Effect::Breath => "◐",
        Effect::Neon => "✦",
        Effect::Wave => "≈",
        Effect::Shifting => "⇄",
        Effect::Zoom => "◎",
    }
}

fn effect_picker(ui: &mut egui::Ui, config: &mut Config) {
    card(ui, "Effect", |ui| {
        let spacing = ui.spacing().item_spacing.x;
        let width = (ui.available_width() - 5.0 * spacing) / 6.0;
        ui.horizontal(|ui| {
            for effect in Effect::ALL {
                let selected = config.effect == effect;
                let text = RichText::new(format!("{}\n{}", effect_icon(effect), effect.name()))
                    .color(if selected {
                        egui::Color32::WHITE
                    } else {
                        MUTED
                    });
                if ui
                    .add(egui::Button::selectable(selected, text).min_size(egui::vec2(width, 56.0)))
                    .clicked()
                {
                    config.effect = effect;
                }
            }
        });
    });
}

fn settings(ui: &mut egui::Ui, c: &mut Config) {
    let title = if c.effect == Effect::Static {
        "Zone colors"
    } else {
        "Effect settings"
    };
    card(ui, title, |ui| {
        egui::Grid::new("settings")
            .num_columns(2)
            .spacing([16.0, 12.0])
            .show(ui, |ui| {
                if c.effect == Effect::Static {
                    ui.label("Zones");
                    ui.horizontal(|ui| {
                        for rgb in &mut c.zones {
                            ui.color_edit_button_srgb(rgb);
                        }
                        if ui.button("Match zone 1").clicked() {
                            c.zones = [c.zones[0]; 4];
                        }
                    });
                    ui.end_row();
                } else {
                    if c.effect.uses_color() {
                        ui.label("Color");
                        ui.color_edit_button_srgb(&mut c.color);
                        ui.end_row();
                    }
                    ui.label("Speed");
                    let suffix = if c.speed == 0 { "  (frozen)" } else { "" };
                    ui.add(egui::Slider::new(&mut c.speed, 0..=9).suffix(suffix));
                    ui.end_row();

                    ui.label("Direction");
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut c.direction,
                            Direction::RightToLeft,
                            "← Right to left",
                        );
                        ui.selectable_value(
                            &mut c.direction,
                            Direction::LeftToRight,
                            "Left to right →",
                        );
                    });
                    ui.end_row();
                }
                ui.label("Brightness");
                ui.add(egui::Slider::new(&mut c.brightness, 0..=100).suffix("%"));
                ui.end_row();
            });
    });
}
