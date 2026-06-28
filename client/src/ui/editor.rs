use crate::ui::style::*;
use crate::ui::templates::{Template, TEMPLATES};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;

use egui::{
    Align, CentralPanel, Color32, Context, FontId, Frame, Layout, Margin, RichText,
    ScrollArea, SidePanel, Stroke, TextEdit, Ui,
};

#[derive(Clone)]
pub enum LogLevel {
    Ok,
    Info,
    Warn,
    Err,
}

pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Default)]
pub struct PerformanceMetrics {
    pub fps: f32,
    pub compile_time_ms: u32,
    pub complexity: u32,
    pub march_steps: u32,
    pub entities: usize,
    pub tick_ms: u32,
}

#[derive(PartialEq)]
enum RightTab {
    Console,
    State,
}

pub struct EditorState {
    pub equation: String,
    pub logs: Vec<LogEntry>,
    pub ws_tx: Arc<Mutex<Option<UnboundedSender<String>>>>,
    templates: Vec<Template>,
    active_tab: RightTab,
    pub metrics: PerformanceMetrics,
    pub uniforms: Vec<(String, String)>,
    pub error_msg: Option<String>,
    pub compiling: bool,
}

impl EditorState {
    pub fn new(ws_tx: Arc<Mutex<Option<UnboundedSender<String>>>>) -> Self {
        Self {
            equation: String::from("sqrt(x*x + y*y + z*z) - 10.0"),
            logs: vec![LogEntry {
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                level: LogLevel::Info,
                message: String::from("Uclid initialized"),
            }],
            ws_tx,
            active_tab: RightTab::Console,
            metrics: PerformanceMetrics::default(),
            uniforms: Vec::new(),
            error_msg: None,
            compiling: false,
            templates: TEMPLATES.to_vec(),
        }
    }

    fn setup_style(ctx: &Context) {
        let mut style = (*ctx.style()).clone();
        style.visuals.window_fill = BG_PANEL;
        style.visuals.panel_fill = BG_SIDEBAR;
        style.visuals.extreme_bg_color = BG_CANVAS;
        style.visuals.widgets.noninteractive.bg_fill = BG_PANEL;
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_DIM);
        style.visuals.widgets.inactive.bg_fill = BG_CARD;
        style.visuals.widgets.hovered.bg_fill = BG_CARD_HOVER;
        style.visuals.widgets.active.bg_fill = ACCENT_STRONG;
        style.visuals.override_text_color = Some(TEXT);
        ctx.set_style(style);
    }

    pub fn draw(&mut self, ctx: &Context) {
        Self::setup_style(ctx);

        // ── Top Bar ──
        egui::TopBottomPanel::top("top_bar")
            .frame(Frame::none().fill(BG_CARD).inner_margin(Margin::symmetric(16.0, 0.0)))
            .height_range(44.0..=44.0)
            .show(ctx, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.label(RichText::new("Uclid").strong().size(14.0).color(Color32::WHITE));

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let fps_color = if self.metrics.fps > 30.0 {
                            GREEN
                        } else if self.metrics.fps > 15.0 {
                            YELLOW
                        } else {
                            RED
                        };
                        ui.label(
                            RichText::new(format!("{:.0} FPS", self.metrics.fps))
                                .font(FontId::monospace(11.0))
                                .color(fps_color),
                        );
                        ui.add_space(12.0);
                        ui.label(RichText::new("●").size(8.0).color(GREEN));
                    });
                });
            });

        // ── Left Sidebar: Templates ──
        SidePanel::left("templates")
            .frame(Frame::none().fill(BG_SIDEBAR).inner_margin(Margin::same(10.0)))
            .resizable(true)
            .default_width(200.0)
            .min_width(140.0)
            .show(ctx, |ui| {
                ui.label(RichText::new("TEMPLATES").font(FontId::monospace(10.0)).color(TEXT_MUTED));
                ui.add_space(10.0);

                ScrollArea::vertical().show(ui, |ui| {
                    for t in &self.templates {
                        let active = self.equation == t.equation;
                        let frame = Frame::none()
                            .fill(if active { ACCENT_FILL } else { BG_CARD })
                            .stroke(Stroke::new(0.5, if active { BORDER_ACTIVE } else { BORDER }))
                            .rounding(6.0)
                            .inner_margin(Margin::same(8.0));

                        let inner = frame.show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(t.name)
                                            .font(FontId::proportional(12.0))
                                            .color(if active { Color32::WHITE } else { TEXT }),
                                    );
                                    let (tag_bg, tag_fg) = match t.tag {
                                        "geometry" => (Color32::from_rgb(12, 30, 25), GREEN),
                                        "physics" => (Color32::from_rgb(26, 16, 36), Color32::from_rgb(218, 88, 133)),
                                        _ => (Color32::from_rgb(31, 22, 6), YELLOW),
                                    };
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.label(
                                            RichText::new(t.tag)
                                                .font(FontId::monospace(8.0))
                                                .color(tag_fg)
                                                .background_color(tag_bg),
                                        );
                                    });
                                });
                                ui.label(
                                    RichText::new(t.description)
                                        .font(FontId::monospace(9.0))
                                        .color(TEXT_MUTED),
                                );
                            });
                        });
                        let response = ui.interact(inner.response.rect, ui.next_auto_id(), egui::Sense::click());

                        if response.hovered() {
                            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                        }
                        if response.clicked() {
                            self.equation = t.equation.to_string();
                            self.error_msg = None;
                        }
                        ui.add_space(4.0);
                    }
                });
            });

        // ── Right Panel ──
        SidePanel::right("right_panel")
            .frame(Frame::none().fill(BG_SIDEBAR).inner_margin(Margin::ZERO))
            .resizable(true)
            .default_width(280.0)
            .min_width(180.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.style_mut().spacing.item_spacing.x = 0.0;
                    let tab_btn = |ui: &mut Ui, label: &str, tab: RightTab, current: &RightTab| {
                        let active = tab == *current;
                        let color = if active { Color32::WHITE } else { TEXT_DIM };
                        let btn = ui.add_sized(
                            [ui.available_width() / 2.0, 36.0],
                            egui::Button::new(RichText::new(label).font(FontId::monospace(10.0)).color(color))
                                .fill(Color32::TRANSPARENT),
                        );
                        if active {
                            let rect = btn.rect;
                            ui.painter().line_segment(
                                [rect.left_bottom(), rect.right_bottom()],
                                Stroke::new(2.0, ACCENT),
                            );
                        }
                        if btn.clicked() {
                            Some(tab)
                        } else {
                            None
                        }
                    };

                    if let Some(t) = tab_btn(ui, "CONSOLE", RightTab::Console, &self.active_tab) {
                        self.active_tab = t;
                    }
                    if let Some(t) = tab_btn(ui, "STATE", RightTab::State, &self.active_tab) {
                        self.active_tab = t;
                    }
                });

                ui.add_space(4.0);

                match self.active_tab {
                    RightTab::Console => {
                        ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                            ui.style_mut().spacing.item_spacing.y = 0.0;
                            for log in &self.logs {
                                let (tag, color, bg) = match log.level {
                                    LogLevel::Ok => ("OK", GREEN, GREEN.gamma_multiply(0.08)),
                                    LogLevel::Info => ("INFO", ACCENT, ACCENT.gamma_multiply(0.08)),
                                    LogLevel::Warn => ("WARN", YELLOW, YELLOW.gamma_multiply(0.08)),
                                    LogLevel::Err => ("ERR", RED, RED.gamma_multiply(0.08)),
                                };
                                Frame::none()
                                    .fill(bg)
                                    .inner_margin(Margin::symmetric(8.0, 4.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(&log.timestamp).font(FontId::monospace(9.0)).color(TEXT_MUTED));
                                            ui.label(RichText::new(tag).font(FontId::monospace(9.0)).color(color));
                                            ui.label(RichText::new(&log.message).font(FontId::monospace(11.0)).color(TEXT));
                                        });
                                    });
                            }
                        });
                    }
                    RightTab::State => {
                        ScrollArea::vertical().show(ui, |ui| {
                            ui.add_space(4.0);

                            ui.label(RichText::new("PERFORMANCE").font(FontId::monospace(9.0)).color(TEXT_MUTED));
                            ui.add_space(4.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.style_mut().spacing.item_spacing.x = 4.0;
                                let mini = |ui: &mut Ui, val: &str, label: &str, color: Color32| {
                                    let width = (ui.available_width() - 8.0).max(60.0) / 3.0;
                                    Frame::none()
                                        .fill(BG_PANEL)
                                        .stroke(Stroke::new(0.5, BORDER))
                                        .rounding(4.0)
                                        .inner_margin(Margin::symmetric(6.0, 6.0))
                                        .show(ui, |ui| {
                                            ui.vertical(|ui| {
                                                ui.set_min_width(width);
                                                ui.label(RichText::new(val).font(FontId::monospace(13.0)).color(color));
                                                ui.label(RichText::new(label).font(FontId::monospace(8.0)).color(TEXT_MUTED));
                                            });
                                        });
                                };
                                mini(ui, &format!("{:.0}", self.metrics.fps), "FPS", GREEN);
                                mini(ui, &format!("{}ms", self.metrics.compile_time_ms), "compile", TEXT);
                                mini(ui, &format!("{}ms", self.metrics.tick_ms), "tick", TEXT);
                            });

                            ui.add_space(12.0);

                            ui.label(RichText::new("SCENE").font(FontId::monospace(9.0)).color(TEXT_MUTED));
                            ui.add_space(4.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.style_mut().spacing.item_spacing.x = 4.0;
                                let mini = |ui: &mut Ui, val: &str, label: &str, color: Color32| {
                                    let width = (ui.available_width() - 8.0).max(60.0) / 3.0;
                                    Frame::none()
                                        .fill(BG_PANEL)
                                        .stroke(Stroke::new(0.5, BORDER))
                                        .rounding(4.0)
                                        .inner_margin(Margin::symmetric(6.0, 6.0))
                                        .show(ui, |ui| {
                                            ui.vertical(|ui| {
                                                ui.set_min_width(width);
                                                ui.label(RichText::new(val).font(FontId::monospace(13.0)).color(color));
                                                ui.label(RichText::new(label).font(FontId::monospace(8.0)).color(TEXT_MUTED));
                                            });
                                        });
                                };
                                mini(ui, &format!("{}", self.metrics.complexity), "complexity", YELLOW);
                                mini(ui, &format!("{}", self.metrics.march_steps), "steps", TEXT);
                                mini(ui, &format!("{}", self.metrics.entities), "entities", GREEN);
                            });

                            ui.add_space(12.0);

                            ui.label(RichText::new("UNIFORMS").font(FontId::monospace(9.0)).color(TEXT_MUTED));
                            ui.add_space(4.0);
                            if !self.uniforms.is_empty() {
                                for (key, val) in &self.uniforms {
                                    Frame::none()
                                        .fill(BG_PANEL)
                                        .stroke(Stroke::new(0.5, BORDER))
                                        .rounding(4.0)
                                        .inner_margin(Margin::symmetric(8.0, 5.0))
                                        .show(ui, |ui| {
                                            ui.set_min_width(ui.available_width());
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new(key).font(FontId::monospace(10.0)).color(TEXT_DIM));
                                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                    ui.label(RichText::new(val).font(FontId::monospace(10.0)).color(ACCENT));
                                                });
                                            });
                                        });
                                }
                            } else {
                                Frame::none()
                                    .fill(BG_PANEL)
                                    .stroke(Stroke::new(0.5, BORDER))
                                    .rounding(4.0)
                                    .inner_margin(Margin::symmetric(8.0, 8.0))
                                    .show(ui, |ui| {
                                        ui.set_min_width(ui.available_width());
                                        ui.label(RichText::new("No state data yet").font(FontId::monospace(9.0)).color(TEXT_MUTED));
                                    });
                            }
                        });
                    }
                }
            });

        // ── Central viewport (SDF raymarching output) ──
        CentralPanel::default()
            .frame(Frame::none().fill(Color32::TRANSPARENT))
            .show(ctx, |_ui| {});

        // ── Resizable Equation Panel ──
        egui::TopBottomPanel::bottom("equation_panel")
            .frame(Frame::none().fill(BG_PANEL).inner_margin(Margin::ZERO))
            .resizable(true)
            .default_height(56.0)
            .min_height(44.0)
            .show(ctx, |ui| {
                if let Some(err) = &self.error_msg.clone() {
                    Frame::none()
                        .fill(RED_BG)
                        .stroke(Stroke::new(0.5, RED))
                        .rounding(4.0)
                        .inner_margin(Margin::symmetric(10.0, 6.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("!").font(FontId::monospace(12.0)).color(RED));
                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(err)
                                        .font(FontId::monospace(10.0))
                                        .color(Color32::from_rgb(255, 180, 180)),
                                );
                            });
                        });
                    ui.add_space(4.0);
                }

                Frame::none()
                    .stroke(Stroke::new(0.5, BORDER))
                    .inner_margin(Margin::symmetric(14.0, 10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("f(x,y,z) =")
                                    .font(FontId::monospace(12.0))
                                    .color(ACCENT_STRONG),
                            );
                            ui.add_space(6.0);

                            let resp = TextEdit::multiline(&mut self.equation)
                                .font(FontId::monospace(14.0))
                                .desired_rows(1)
                                .desired_width(f32::INFINITY)
                                .hint_text("Enter an SDF equation...")
                                .show(ui);

                            if resp.response.lost_focus()
                                && ui.input(|i| {
                                    i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl
                                })
                            {
                                self.compile_and_send();
                            }

                            ui.add_space(6.0);

                            let label = if self.compiling { "..." } else { "Compile" };
                            let color = if self.compiling { TEXT_MUTED } else { ACCENT_TEXT };
                            let fill = if self.compiling { BG_CARD } else { ACCENT_FILL };
                            if ui
                                .add_sized(
                                    [72.0, 28.0],
                                    egui::Button::new(
                                        RichText::new(label)
                                            .font(FontId::monospace(11.0))
                                            .strong()
                                            .color(color),
                                    )
                                    .fill(fill)
                                    .stroke(Stroke::new(0.5, ACCENT_STRONG)),
                                )
                                .on_hover_text("Ctrl+Enter")
                                .clicked()
                                && !self.compiling
                            {
                                self.compile_and_send();
                            }
                        });
                    });
            });
    }

    fn compile_and_send(&mut self) {
        self.compiling = true;
        let payload = serde_json::json!({ "equation": self.equation.trim() }).to_string();
        let tx = {
            let guard = self.ws_tx.lock().unwrap();
            guard.clone()
        };
        if let Some(tx) = tx {
            match tx.send(payload) {
                Ok(_) => self.push_log(LogLevel::Ok, "Equation sent"),
                Err(e) => self.push_log(LogLevel::Err, &format!("Send failed: {e}")),
            }
        } else {
            self.push_log(LogLevel::Warn, "Not connected to server");
        }
    }

    pub fn finish_compiling(&mut self) {
        self.compiling = false;
    }

    pub fn push_log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push(LogEntry {
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            level,
            message: msg.to_string(),
        });
        if self.logs.len() > 200 {
            self.logs.remove(0);
        }
    }
}
