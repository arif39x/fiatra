use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use egui::{RichText, Layout, Align};

use crate::ui::style::*;

pub struct StatusBar {
    pub fps: f32,
    pub ws_connected: Arc<AtomicBool>,
    pub selection: String,
}

impl StatusBar {
    pub fn new(ws_connected: Arc<AtomicBool>) -> Self {
        Self { fps: 0.0, ws_connected, selection: String::new() }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let connected = self.ws_connected.load(Ordering::Relaxed);
            let (dot, col) = if connected { ("●", GREEN) } else { ("○", TEXT_MUTED) };
            ui.label(RichText::new(dot).size(10.0).color(col));
            ui.label(RichText::new(if connected { "Connected" } else { "Disconnected" }).size(10.0).color(TEXT_MUTED));
            ui.separator();

            let fps_color = if self.fps > 55.0 { GREEN } else if self.fps > 30.0 { YELLOW } else { RED };
            ui.label(RichText::new(format!("{:.0} FPS", self.fps)).size(10.0).color(fps_color));

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if !self.selection.is_empty() {
                    ui.label(RichText::new(&self.selection).size(10.0).color(TEXT_DIM));
                }
            });
        });
    }
}
