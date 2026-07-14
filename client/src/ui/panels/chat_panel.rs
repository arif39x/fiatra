use egui::{Frame, Margin, Rounding, RichText, ScrollArea, vec2, TextEdit, Layout, Align};

use crate::ui::style::*;

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

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("AI Assistant").size(12.0).color(TEXT));
        ui.add_space(4.0);

        ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.style_mut().spacing.item_spacing = vec2(0.0, 4.0);
                for msg in &self.messages {
                    match msg.role {
                        MessageRole::User => {
                            Frame::none()
                                .fill(ACCENT.linear_multiply(0.12))
                                .rounding(Rounding::same(6.0))
                                .inner_margin(Margin::symmetric(10.0, 6.0))
                                .show(ui, |ui| {
                                    ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                                        ui.label(RichText::new(&msg.text).size(12.0).color(TEXT));
                                    });
                                });
                        }
                        MessageRole::Assistant => {
                            Frame::none()
                                .fill(BG_CARD)
                                .rounding(Rounding::same(6.0))
                                .inner_margin(Margin::symmetric(10.0, 6.0))
                                .show(ui, |ui| {
                                    ui.label(RichText::new(&msg.text).size(12.0).color(TEXT_DIM));
                                    if !msg.entity_ids.is_empty() {
                                        ui.horizontal_wrapped(|ui| {
                                            for eid in &msg.entity_ids {
                                                ui.label(RichText::new(format!("#{}", eid)).size(10.0).color(ACCENT));
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

        ui.add_space(4.0);
        if self.processing {
            ui.horizontal_centered(|ui| {
                ui.label(RichText::new("Processing...").size(11.0).color(TEXT_MUTED));
            });
        } else {
            ui.horizontal(|ui| {
                TextEdit::multiline(&mut self.input)
                    .desired_rows(1)
                    .hint_text("Describe what to create...")
                    .show(ui);
                let send = ui.add_sized(
                    [60.0, 32.0],
                    egui::Button::new(RichText::new("Send").size(11.0).color(TEXT))
                        .fill(ACCENT)
                        .rounding(Rounding::same(4.0)),
                ).clicked();
                let enter = ui.input(|i| i.key_pressed(egui::Key::Enter))
                    && !ui.input(|i| i.modifiers.shift);
                if send || enter {
                    self.send_message();
                }
            });
        }
    }

    fn send_message(&mut self) {
        let input = self.input.trim().to_string();
        if input.is_empty() { return; }
        self.messages.push(ChatMessage { role: MessageRole::User, text: input.clone(), entity_ids: vec![] });
        self.pending_send = Some(input);
        self.processing = true;
        self.input.clear();
    }

    pub fn send_quick_command(&mut self, command: &str) {
        self.messages.push(ChatMessage { role: MessageRole::User, text: command.to_string(), entity_ids: vec![] });
        self.pending_send = Some(command.to_string());
        self.processing = true;
    }

    pub fn receive_response(&mut self, reply: &str, entity_ids: &[u64]) {
        self.messages.push(ChatMessage { role: MessageRole::Assistant, text: reply.to_string(), entity_ids: entity_ids.to_vec() });
        self.processing = false;
    }
}
