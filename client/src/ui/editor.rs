use crate::core::ecs::*;
use crate::core::scene::Scene;
use crate::core::undo::{EditCommand, MaterialSnapshot, TransformSnapshot, UndoStack};
use crate::network::EntityData;
use crate::ui::chat_panel::ChatPanel;
use crate::ui::generation_status::GenerationStatusPanel;
use crate::ui::inspector::Inspector;
use crate::ui::scene_panel::ScenePanel;
use crate::ui::style::*;
use crate::ui::toolbar::Toolbar;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;

use egui::{
    Align, CentralPanel, Color32, Context, FontId, Frame, Layout, Margin, RichText,
    ScrollArea, SidePanel, Stroke,
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
}

pub struct EditorState {
    pub logs: Vec<LogEntry>,
    pub ws_tx: Arc<Mutex<Option<UnboundedSender<String>>>>,
    pub metrics: PerformanceMetrics,
    pub chat: ChatPanel,
    pub gen_status: GenerationStatusPanel,
    pub scene_panel: ScenePanel,
    pub inspector_panel: Inspector,
    pub toolbar_panel: Toolbar,
    pub undo: UndoStack,
    pub scene: Scene,
    pub selected_entity: Option<EntityId>,
    pub loaded_character: bool,
    pub loaded_motion: bool,
    pub character_mesh: Option<serde_json::Value>,
    pub character_skeleton: Option<serde_json::Value>,
    pub motion_clip: Option<serde_json::Value>,
    pub export_format: String,
    pub export_path: String,
    pub export_triggered: bool,
}

impl EditorState {
    pub fn new(ws_tx: Arc<Mutex<Option<UnboundedSender<String>>>>) -> Self {
        Self {
            logs: vec![LogEntry {
                timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                level: LogLevel::Info,
                message: String::from("Muse initialized"),
            }],
            ws_tx,
            metrics: PerformanceMetrics::default(),
            chat: ChatPanel::new(),
            gen_status: GenerationStatusPanel::new(),
            scene_panel: ScenePanel::new(),
            inspector_panel: Inspector::new(),
            toolbar_panel: Toolbar::new(),
            undo: UndoStack::new(),
            scene: Scene::new(),
            selected_entity: None,
            loaded_character: false,
            loaded_motion: false,
            character_mesh: None,
            character_skeleton: None,
            motion_clip: None,
            export_format: "glb".to_string(),
            export_path: "output/character.glb".to_string(),
            export_triggered: false,
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

        egui::TopBottomPanel::top("top_bar")
            .frame(Frame::none().fill(BG_CARD).inner_margin(Margin::symmetric(16.0, 0.0)))
            .height_range(44.0..=44.0)
            .show(ctx, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.label(RichText::new("Muse").strong().size(14.0).color(Color32::WHITE));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let fps_color = if self.metrics.fps > 30.0 { GREEN } else if self.metrics.fps > 15.0 { YELLOW } else { RED };
                        ui.label(RichText::new(format!("{:.0} FPS", self.metrics.fps)).font(FontId::monospace(11.0)).color(fps_color));
                    });
                });
            });

        SidePanel::right("right_panel")
            .frame(Frame::none().fill(BG_SIDEBAR).inner_margin(Margin::ZERO))
            .resizable(true)
            .default_width(280.0)
            .min_width(180.0)
            .show(ctx, |ui| {
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
            });

        CentralPanel::default()
            .frame(Frame::none().fill(Color32::TRANSPARENT))
            .show(ctx, |_ui| {});

        if let Some(clicked) = self.scene_panel.draw(ctx, &mut self.scene) {
            self.select_entity(clicked);
        }

        self.inspector_panel.draw(ctx, &mut self.scene, self.selected_entity, &mut self.undo);

        self.toolbar_panel.draw(ctx, &mut self.scene, &mut self.selected_entity);

        self.chat.draw(ctx);
        self.gen_status.draw(ctx);
        self.draw_export(ctx);
    }

    pub fn undo_last(&mut self) {
        if let Some(cmd) = self.undo.undo() {
            match cmd {
                EditCommand::Transform(snap) => {
                    if let Some(t) = self.scene.world.get_mut::<TransformComponent>(snap.entity) {
                        *t = snap.prev;
                    }
                }
                EditCommand::Material(snap) => {
                    if let Some(m) = self.scene.world.get_mut::<MaterialComponent>(snap.entity) {
                        *m = snap.prev;
                    }
                }
            }
        }
    }

    pub fn redo_last(&mut self) {
        // Re-apply the command (push it back, then push triggers apply on next edit)
        // For simplicity, we invert the undo
        if let Some(_cmd) = self.undo.redo() {
            self.push_log(LogLevel::Info, "Redo: re-apply not yet fully implemented");
        }
    }

    pub fn select_entity(&mut self, id: EntityId) {
        if let Some(old) = self.selected_entity {
            self.scene.world.remove::<Selected>(old);
        }
        self.scene.world.add(id, Selected);
        self.selected_entity = Some(id);
    }

    pub fn clear_selection(&mut self) {
        if let Some(old) = self.selected_entity {
            self.scene.world.remove::<Selected>(old);
        }
        self.selected_entity = None;
    }

    fn draw_export(&mut self, ctx: &Context) {
        egui::Window::new("Export")
            .id(egui::Id::new("export_window"))
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.label("Format:");
                egui::ComboBox::from_id_source("export_fmt")
                    .selected_text(&self.export_format.to_uppercase())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.export_format, "glb".to_string(), "GLB (glTF 2.0)");
                        ui.selectable_value(&mut self.export_format, "fbx".to_string(), "FBX (ASCII)");
                    });
                ui.add_space(4.0);
                ui.label("Path:");
                ui.text_edit_singleline(&mut self.export_path);
                ui.add_space(4.0);
                let has_data = self.character_mesh.is_some() && self.character_skeleton.is_some();
                if !has_data {
                    ui.colored_label(Color32::RED, "No character data loaded");
                }
                let btn = egui::Button::new(
                    RichText::new("Export").font(FontId::monospace(12.0)).color(Color32::WHITE),
                )
                .fill(if has_data { ACCENT_STRONG } else { BG_CARD })
                .min_size([ui.available_width(), 28.0].into());
                if ui.add(btn).clicked() && has_data {
                    self.export_triggered = true;
                }
            });
    }

    pub fn handle_entities(&mut self, entities: &[EntityData]) {
        for ed in entities {
            let eid = self.scene.world.spawn();
            self.scene.world.add(eid, TransformComponent::identity());
            self.scene.world.add(eid, LabelComponent {
                name: ed.label.clone(),
                entity_type: ed.entity_type.clone(),
            });

            match ed.entity_type.as_str() {
                "generate_skeleton" => {
                    if let Ok(skel) = serde_json::from_value::<crate::core::skeleton::Skeleton>(ed.data.clone()) {
                        self.scene.world.add(eid, SkeletonComponent { skeleton: skel });
                    }
                }
                "generate_mesh" => {
                    self.scene.world.add(eid, MeshComponent { mesh_data: None, mesh_type: None });
                }
                "generate_motion" => {
                    self.scene.world.add(eid, MotionComponent {
                        animator: crate::animation::playback::Animator::new(),
                        joint_params: Some(ed.data.clone()),
                    });
                }
                "edit_scene" => {
                    if let Some(_lighting) = ed.data.get("lighting") {
                    }
                    if let Some(_materials) = ed.data.get("materials") {
                    }
                }
                _ => {}
            }
        }
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
