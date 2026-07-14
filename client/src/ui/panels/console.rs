use egui::{Frame, Margin, Rounding, RichText, ScrollArea, Color32, vec2};

use crate::ui::style::*;

#[derive(Clone)]
pub enum LogLevel {
    Ok,
    Info,
    Warn,
    Err,
}

#[derive(Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
}

impl LogEntry {
    fn tag(&self) -> &str {
        match self.level {
            LogLevel::Ok => "OK",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Err => "ERR",
        }
    }

    fn color(&self) -> Color32 {
        match self.level {
            LogLevel::Ok => GREEN,
            LogLevel::Info => ACCENT,
            LogLevel::Warn => YELLOW,
            LogLevel::Err => RED,
        }
    }
}

pub struct Console {
    pub logs: Vec<LogEntry>,
    pub filter: String,
    show_ok: bool,
    show_info: bool,
    show_warn: bool,
    show_err: bool,
}

impl Console {
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            filter: String::new(),
            show_ok: true,
            show_info: true,
            show_warn: true,
            show_err: true,
        }
    }

    pub fn push(&mut self, level: LogLevel, msg: &str) {
        self.logs.push(LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            level,
            message: msg.to_string(),
        });
        if self.logs.len() > 500 {
            self.logs.remove(0);
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Console").size(12.0).color(TEXT));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let toggle = |ui: &mut egui::Ui, label: &str, on: &mut bool, color: Color32| {
                    let fill = if *on { color.linear_multiply(0.15) } else { Color32::TRANSPARENT };
                    let btn = egui::Button::new(RichText::new(label).size(10.0).color(if *on { color } else { TEXT_MUTED })).fill(fill);
                    if ui.add(btn).clicked() { *on = !*on; }
                };
                toggle(ui, "OK", &mut self.show_ok, GREEN);
                toggle(ui, "INF", &mut self.show_info, ACCENT);
                toggle(ui, "WRN", &mut self.show_warn, YELLOW);
                toggle(ui, "ERR", &mut self.show_err, RED);
            });
        });
        ui.add_space(4.0);

        ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.style_mut().spacing.item_spacing = vec2(0.0, 1.0);
                for entry in &self.logs {
                    let show = match entry.level {
                        LogLevel::Ok => self.show_ok,
                        LogLevel::Info => self.show_info,
                        LogLevel::Warn => self.show_warn,
                        LogLevel::Err => self.show_err,
                    };
                    if !show { continue; }

                    Frame::none()
                        .rounding(Rounding::same(3.0))
                        .inner_margin(Margin::symmetric(8.0, 2.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&entry.timestamp).size(10.0).color(TEXT_MUTED));
                                ui.label(RichText::new(entry.tag()).size(10.0).color(entry.color()));
                                ui.label(RichText::new(&entry.message).size(11.0).color(TEXT_DIM));
                            });
                        });
                }
            });
    }
}
