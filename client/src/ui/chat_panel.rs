use egui::{
    Align, Color32, FontId, Frame, Layout, Margin, RichText, ScrollArea, TextEdit,
};

use super::style::*;

#[derive(Clone, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub text: String,
    pub entity_ids: Vec<u64>,
}

pub struct ChatPanel {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub processing: bool,
    pub pending_send: Option<String>,
}

impl ChatPanel {
    pub fn new() -> Self {
        Self {
            messages: vec![ChatMessage {
                role: MessageRole::System,
                text: "Describe anything you want to create in 3D.".to_string(),
                entity_ids: vec![],
            }],
            input: String::new(),
            processing: false,
            pending_send: None,
        }
    }

    pub fn draw(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("chat_bottom_panel")
            .frame(Frame::none().fill(BG_CARD))
            .height_range(200.0..=200.0)
            .show(ctx, |ui| {
                let max_msg_height = (ui.available_height() - 60.0).max(40.0);
                ScrollArea::vertical()
                    .id_source("chat_messages")
                    .stick_to_bottom(true)
                    .max_height(max_msg_height)
                    .show(ui, |ui| {
                        ui.style_mut().spacing.item_spacing.y = 4.0;
                        for msg in &self.messages {
                            match msg.role {
                                MessageRole::User => {
                                    Frame::none()
                                        .fill(ACCENT_STRONG.gamma_multiply(0.15))
                                        .rounding(egui::Rounding::same(6.0))
                                        .inner_margin(Margin::symmetric(8.0, 6.0))
                                        .show(ui, |ui| {
                                            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                                                ui.label(
                                                    RichText::new(&msg.text)
                                                        .font(FontId::monospace(12.0))
                                                        .color(TEXT),
                                                );
                                            });
                                        });
                                }
                                MessageRole::Assistant => {
                                    Frame::none()
                                        .fill(BG_SIDEBAR)
                                        .rounding(egui::Rounding::same(6.0))
                                        .inner_margin(Margin::symmetric(8.0, 6.0))
                                        .show(ui, |ui| {
                                            ui.label(
                                                RichText::new(&msg.text)
                                                    .font(FontId::monospace(12.0))
                                                    .color(TEXT),
                                            );
                                            if !msg.entity_ids.is_empty() {
                                                ui.horizontal_wrapped(|ui| {
                                                    for eid in &msg.entity_ids {
                                                        ui.label(
                                                            RichText::new(format!("#{}", eid))
                                                                .font(FontId::monospace(10.0))
                                                                .color(ACCENT),
                                                        );
                                                    }
                                                });
                                            }
                                        });
                                }
                                MessageRole::System => {
                                    ui.horizontal_centered(|ui| {
                                        ui.colored_label(TEXT_MUTED, &msg.text);
                                    });
                                }
                            }
                        }
                    });

                if self.processing {
                    ui.horizontal_centered(|ui| {
                        ui.label(
                            RichText::new("Processing...")
                                .font(FontId::monospace(11.0))
                                .color(TEXT_MUTED),
                        );
                    });
                } else {
                    ui.horizontal(|ui| {
                        let resp = TextEdit::multiline(&mut self.input)
                            .font(FontId::monospace(13.0))
                            .desired_width(f32::INFINITY)
                            .desired_rows(2)
                            .hint_text("Describe what to create...")
                            .show(ui);

                        let enter_send = ui.input(|i| i.key_pressed(egui::Key::Enter))
                            && !ui.input(|i| i.modifiers.shift);

                        let send_clicked = ui
                            .add_sized(
                                [72.0, 48.0],
                                egui::Button::new(
                                    RichText::new("Send")
                                        .font(FontId::monospace(12.0))
                                        .color(Color32::WHITE),
                                )
                                .fill(ACCENT_STRONG),
                            )
                            .clicked();

                        if send_clicked || enter_send {
                            self.send_message();
                        }
                    });
                }
            });
    }

    fn send_message(&mut self) {
        let input = self.input.trim().to_string();
        if input.is_empty() {
            return;
        }

        self.messages.push(ChatMessage {
            role: MessageRole::User,
            text: input.clone(),
            entity_ids: vec![],
        });

        self.pending_send = Some(input);
        self.processing = true;
        self.input.clear();
    }

    pub fn send_quick_command(&mut self, command: &str) {
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            text: command.to_string(),
            entity_ids: vec![],
        });

        self.pending_send = Some(command.to_string());
        self.processing = true;
    }

    pub fn receive_response(&mut self, reply: &str, entity_ids: &[u64]) {
        self.messages.push(ChatMessage {
            role: MessageRole::Assistant,
            text: reply.to_string(),
            entity_ids: entity_ids.to_vec(),
        });
        self.processing = false;
    }
}
