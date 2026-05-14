use egui::{
    Align, CentralPanel, Color32, FontId, Layout, RichText, ScrollArea, SidePanel, TextEdit,
};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;

pub struct EditorState {
    pub equation: String,
    pub log: String,
    pub ws_tx: Arc<Mutex<Option<UnboundedSender<String>>>>,
    pub templates: Vec<(&'static str, &'static str)>,
}

impl EditorState {
    pub fn new(ws_tx: Arc<Mutex<Option<UnboundedSender<String>>>>) -> Self {
        Self {
            equation: String::from("sqrt(x**2 + y**2 + z**2) - 10.0"),
            log: String::from("[ Rationalist terminal initialized ]\n"),
            ws_tx,
            templates: vec![
                ("Null Plane", "z - 0"),
                ("Sphere", "sqrt(x**2 + y**2 + z**2) - 10.0"),
                ("Infinite Cylinder", "sqrt(x**2 + y**2) - 5.0"),
                (
                    "Torus (Donut)",
                    "(sqrt(x**2 + y**2) - 10.0)**2 + z**2 - 4.0",
                ),
                ("Rippled Ground", "z - sin(x/5.0) * cos(y/5.0) * 2.0"),
                (
                    "Twisted Column",
                    "sqrt(x**2 + (y - sin(z/5.0)*3.0)**2) - 4.0",
                ),
                (
                    "Sphere with Hole",
                    "Max(sqrt(x**2 + y**2 + z**2) - 10.0, -(sqrt(x**2 + y**2) - 4.0))",
                ),
                (
                    "Orbital Tracking",
                    "sqrt((x - state.x)**2 + (y - state.y)**2 + (z - state.z)**2) - 5.0",
                ),
            ],
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context) {
        let mut visual_style = (*ctx.style()).clone();
        visual_style.visuals.window_fill = Color32::from_black_alpha(200);
        ctx.set_style(visual_style);

        SidePanel::left("template_panel")
            .resizable(false)
            .default_width(180.0)
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.label(
                    RichText::new("STRUCTURE TEMPLATES")
                        .strong()
                        .color(Color32::from_rgb(100, 200, 255)),
                );
                ui.separator();
                ui.add_space(5.0);

                ScrollArea::vertical().show(ui, |ui| {
                    for (name, eq) in &self.templates {
                        if ui
                            .add(egui::Button::new(*name).fill(Color32::TRANSPARENT))
                            .clicked()
                        {
                            self.equation = eq.to_string();
                        }
                        ui.add_space(2.0);
                    }
                });
            });

        SidePanel::right("log_panel")
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.label(
                    RichText::new("SYSTEM LOG")
                        .strong()
                        .color(Color32::from_rgb(100, 255, 100)),
                );
ui.separator();

                ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    ui.add(
                        TextEdit::multiline(&mut self.log)
                            .font(FontId::monospace(10.0))
                            .text_color(Color32::from_rgb(0, 255, 150))
                            .desired_rows(20)
                            .desired_width(f32::INFINITY)
                            .interactive(false),
                    );
                });
            });

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(5.0);
                ui.label(
                    RichText::new("Rationalist").strong().size(18.0)
                );
                
                ui.add_space(5.0);
            });
            ui.separator();

            ui.add_space(10.0);

            let edit_height = ui.available_height() - 60.0;

            ui.group(|ui| {
                ScrollArea::vertical()
                    .max_height(edit_height)
                    .show(ui, |ui| {
                        ui.add(
                            TextEdit::multiline(&mut self.equation)
                                .font(FontId::monospace(14.0))
                                .desired_rows(12)
                                .desired_width(f32::INFINITY)
                                .margin(egui::vec2(10.0, 10.0))
                                .hint_text("Enter SDF equation..."),
                        );
                    });
            });

            ui.add_space(10.0);

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new(" INJECT INTO UNIVERSE ").strong().size(14.0),
                        )
                        .fill(Color32::from_rgb(50, 100, 200)),
                    )
                    .clicked()
                {
                    self.compile_and_send();
                }
            });
        });
    }

    fn compile_and_send(&mut self) {
        let payload = serde_json::json!({ "equation": self.equation.trim() }).to_string();
        let guard = self.ws_tx.lock().unwrap();
        if let Some(tx) = guard.as_ref() {
            match tx.send(payload) {
                Ok(_) => self.log.push_str("[TX] structure broadcasted\n"),
                Err(e) => self.log.push_str(&format!("[ERR] uplink failed: {e}\n")),
            }
        } else {
            self.log.push_str("[WARN] relay disconnected\n");
        }
    }

    pub fn push_log(&mut self, msg: &str) {
        self.log.push_str(msg);
        self.log.push('\n');
    }
}
