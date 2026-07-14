use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;

use egui::{CentralPanel, Color32, Context, Frame, Margin, RichText, SidePanel, TopBottomPanel, vec2};

use crate::core::ecs::*;
use crate::core::scene::Scene;
use crate::core::undo::{EditCommand, UndoStack};
use crate::network::EntityData;
use crate::ui::panels::chat_panel::ChatPanel;
use crate::ui::panels::console::{Console, LogLevel};
use crate::ui::panels::generation_status::GenerationStatusPanel;
use crate::ui::panels::inspector::Inspector;
use crate::ui::panels::scene_panel::ScenePanel;
use crate::ui::panels::status_bar::StatusBar;
use crate::ui::panels::toolbar::Toolbar;
use crate::ui::style::*;

#[derive(PartialEq)]
enum BottomTab {
    Console,
    Jobs,
}

pub struct PerformanceMetrics {
    pub fps: f32,
}

pub struct EditorState {
    pub ws_tx: Arc<Mutex<Option<UnboundedSender<String>>>>,
    pub metrics: PerformanceMetrics,
    pub chat: ChatPanel,
    pub gen_status: GenerationStatusPanel,
    pub scene_panel: ScenePanel,
    pub inspector_panel: Inspector,
    pub toolbar_panel: Toolbar,
    pub console: Console,
    pub status_bar: StatusBar,
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
    pub scene_sync_pending: bool,
    pub pending_texture: Option<(String, Vec<u8>)>,
    pub ws_connected: Arc<AtomicBool>,
    bottom_tab: BottomTab,
}

impl EditorState {
    pub fn new(ws_tx: Arc<Mutex<Option<UnboundedSender<String>>>>) -> Self {
        let ws_connected = Arc::new(AtomicBool::new(false));
        Self {
            ws_tx,
            metrics: PerformanceMetrics { fps: 0.0 },
            chat: ChatPanel::new(),
            gen_status: GenerationStatusPanel::new(),
            scene_panel: ScenePanel::new(),
            inspector_panel: Inspector::new(),
            toolbar_panel: Toolbar::new(),
            console: Console::new(),
            status_bar: StatusBar::new(ws_connected.clone()),
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
            scene_sync_pending: false,
            pending_texture: None,
            ws_connected,
            bottom_tab: BottomTab::Console,
        }
    }

    pub fn draw(&mut self, ctx: &Context) {
        setup_style(ctx);
        self.status_bar.fps = self.metrics.fps;

        TopBottomPanel::bottom("status_bar")
            .min_height(22.0)
            .max_height(22.0)
            .frame(Frame::none().fill(BG_CARD))
            .show(ctx, |ui| {
                self.status_bar.draw(ui);
            });

        let gen_cancel = self.gen_status.on_cancel.take().or_else(|| {
            let ws_tx = self.ws_tx.clone();
            Some(Box::new(move |job_id: String| {
                if let Some(tx) = ws_tx.lock().unwrap().as_ref() {
                    let msg = serde_json::json!({ "type": "cancel_job", "job_id": job_id });
                    let _ = tx.send(msg.to_string());
                }
            }) as Box<dyn FnMut(String) + Send>)
        });
        self.gen_status.on_cancel = gen_cancel;

        TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(200.0)
            .min_height(80.0)
            .frame(Frame::none().fill(BG_PANEL))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.style_mut().spacing.item_spacing = vec2(0.0, 0.0);
                    let mut tab = |ui: &mut egui::Ui, label: &str, id: BottomTab| {
                        let active = self.bottom_tab == id;
                        let fill = if active { BG_HOVER } else { Color32::TRANSPARENT };
                        let r = Frame::none()
                            .fill(fill)
                            .rounding(egui::Rounding { nw: 4.0, ne: 4.0, sw: 0.0, se: 0.0 })
                            .inner_margin(Margin::symmetric(10.0, 4.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new(label).size(11.0).color(if active { TEXT } else { TEXT_DIM }));
                            }).response;
                        if ui.interact(r.rect, ui.auto_id_with(label), egui::Sense::click()).clicked() {
                            self.bottom_tab = id;
                        }
                    };
                    tab(ui, "Console", BottomTab::Console);
                    tab(ui, "Jobs", BottomTab::Jobs);
                });
                ui.separator();

                match self.bottom_tab {
                    BottomTab::Console => self.console.draw(ui),
                    BottomTab::Jobs => self.gen_status.draw(ui),
                }
            });

        SidePanel::left("scene_panel")
            .resizable(true)
            .default_width(220.0)
            .min_width(160.0)
            .frame(Frame::none().fill(BG_PANEL))
            .show(ctx, |ui| {
                if let Some(clicked) = self.scene_panel.draw(ui, &mut self.scene) {
                    self.select_entity(clicked);
                }
            });

        SidePanel::right("inspector_panel")
            .resizable(true)
            .default_width(320.0)
            .min_width(240.0)
            .frame(Frame::none().fill(BG_PANEL))
            .show(ctx, |ui| {
                self.chat.draw(ui);
                ui.separator();
                self.inspector_panel.draw(ui, &mut self.scene, self.selected_entity, &mut self.undo);
            });

        TopBottomPanel::top("toolbar")
            .min_height(30.0)
            .frame(Frame::none().fill(BG_PANEL))
            .show(ctx, |ui| {
                self.toolbar_panel.draw(ui, &mut self.scene, &mut self.selected_entity);
            });

        CentralPanel::default()
            .frame(Frame::none().fill(Color32::TRANSPARENT))
            .show(ctx, |_ui| {});

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
        if let Some(cmd) = self.undo.redo() {
            match cmd {
                EditCommand::Transform(snap) => {
                    if let Some(t) = self.scene.world.get_mut::<TransformComponent>(snap.entity) {
                        *t = snap.current;
                    }
                }
                EditCommand::Material(snap) => {
                    if let Some(m) = self.scene.world.get_mut::<MaterialComponent>(snap.entity) {
                        *m = snap.current;
                    }
                }
            }
        }
    }

    pub fn select_entity(&mut self, id: EntityId) {
        if let Some(old) = self.selected_entity {
            self.scene.world.remove::<Selected>(old);
        }
        self.scene.world.add(id, Selected);
        self.selected_entity = Some(id);
        if let Some(l) = self.scene.world.get::<LabelComponent>(id) {
            let label = if l.name.is_empty() { format!("#{}", id) } else { l.name.clone() };
            self.status_bar.selection = format!("Selected: {}", label);
        }
    }

    pub fn clear_selection(&mut self) {
        if let Some(old) = self.selected_entity {
            self.scene.world.remove::<Selected>(old);
        }
        self.selected_entity = None;
        self.status_bar.selection.clear();
    }

    fn draw_export(&mut self, ctx: &Context) {
        egui::Window::new("Export")
            .id("export_window".into())
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
                    ui.colored_label(RED, "No character data loaded");
                }
                let btn = egui::Button::new(RichText::new("Export").size(12.0).color(TEXT))
                    .fill(if has_data { ACCENT } else { BG_CARD })
                    .min_size([ui.available_width(), 28.0].into());
                if ui.add(btn).clicked() && has_data {
                    self.export_triggered = true;
                }
            });
    }

    pub fn push_log(&mut self, level: LogLevel, msg: &str) {
        self.console.push(level, msg);
    }

    pub fn handle_entities(&mut self, entities: &[EntityData]) {
        for ed in entities {
            let eid = if let Some(uid) = ed.entity_id {
                if self.scene.world.is_alive(uid) { uid }
                else {
                    let nid = self.scene.world.spawn();
                    self.scene.world.add(nid, TransformComponent::identity());
                    self.scene.world.add(nid, LabelComponent { name: ed.label.clone(), entity_type: ed.entity_type.clone() });
                    nid
                }
            } else {
                let nid = self.scene.world.spawn();
                self.scene.world.add(nid, TransformComponent::identity());
                self.scene.world.add(nid, LabelComponent { name: ed.label.clone(), entity_type: ed.entity_type.clone() });
                nid
            };

            match ed.entity_type.as_str() {
                "generate_skeleton" => {
                    if let Ok(skel) = serde_json::from_value::<crate::core::skeleton::Skeleton>(ed.data.clone()) {
                        if let Some(s) = self.scene.world.get_mut::<SkeletonComponent>(eid) {
                            s.skeleton = skel;
                        } else {
                            self.scene.world.add(eid, SkeletonComponent { skeleton: skel });
                        }
                    }
                }
                "generate_mesh" => {
                    self.scene.world.add(eid, MeshComponent { mesh_data: Some(ed.data.clone()), mesh_type: None });
                }
                "generate_texture" => {
                    if let Some(width) = ed.data.get("width").and_then(|v| v.as_u64()) {
                        if let Some(height) = ed.data.get("height").and_then(|v| v.as_u64()) {
                            if let Some(data) = ed.data.get("data").and_then(|v| v.as_array()) {
                                let mut bytes = Vec::with_capacity(data.len());
                                for val in data { bytes.push(val.as_u64().unwrap_or(255) as u8); }
                                if bytes.len() as u64 == width * height * 4 {
                                    self.pending_texture = Some((ed.label.clone(), bytes));
                                }
                            }
                        }
                    }
                }
                "generate_motion" => {
                    self.scene.world.add(eid, MotionComponent {
                        animator: crate::animation::playback::Animator::new(),
                        joint_params: Some(ed.data.clone()),
                    });
                }
                "edit_scene" => {
                    if let Some(materials) = ed.data.get("materials") {
                        if let Some(mat_map) = materials.as_object() {
                            for (key, mat_val) in mat_map {
                                if let Some(id_str) = key.strip_prefix("entity_") {
                                    if let Ok(target_id) = id_str.parse::<u64>() {
                                        if let Some(albedo) = mat_val.get("albedo").and_then(|v| v.as_array()) {
                                            if albedo.len() >= 3 {
                                                let (r, g, b) = (
                                                    albedo[0].as_f64().unwrap_or(0.8) as f32,
                                                    albedo[1].as_f64().unwrap_or(0.8) as f32,
                                                    albedo[2].as_f64().unwrap_or(0.8) as f32,
                                                );
                                                if let Some(mat) = self.scene.world.get_mut::<MaterialComponent>(target_id) { mat.albedo = (r, g, b); }
                                            }
                                        }
                                        if let Some(metallic) = mat_val.get("metallic").and_then(|v| v.as_f64()) {
                                            if let Some(mat) = self.scene.world.get_mut::<MaterialComponent>(target_id) { mat.metallic = metallic as f32; }
                                        }
                                        if let Some(roughness) = mat_val.get("roughness").and_then(|v| v.as_f64()) {
                                            if let Some(mat) = self.scene.world.get_mut::<MaterialComponent>(target_id) { mat.roughness = roughness as f32; }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                "create_primitive" => {
                    let primitive = ed.data.get("primitive").and_then(|v| v.as_str()).unwrap_or("cube");
                    let pos = ed.data.get("position").and_then(|v| v.as_array())
                        .map(|a| (a[0].as_f64().unwrap_or(0.0) as f32, a[1].as_f64().unwrap_or(0.0) as f32, a[2].as_f64().unwrap_or(0.0) as f32))
                        .unwrap_or((0.0, 0.0, 0.0));
                    let rot = ed.data.get("rotation").and_then(|v| v.as_array())
                        .map(|a| (a[0].as_f64().unwrap_or(0.0) as f32, a[1].as_f64().unwrap_or(0.0) as f32, a[2].as_f64().unwrap_or(0.0) as f32))
                        .unwrap_or((0.0, 0.0, 0.0));
                    let scale = ed.data.get("scale").and_then(|v| v.as_array())
                        .map(|a| (a[0].as_f64().unwrap_or(1.0) as f32, a[1].as_f64().unwrap_or(1.0) as f32, a[2].as_f64().unwrap_or(1.0) as f32))
                        .unwrap_or((1.0, 1.0, 1.0));

                    let mesh_type = match primitive {
                        "sphere" => MeshType::Sphere(16),
                        "plane" => MeshType::Plane,
                        "cylinder" => MeshType::Cylinder,
                        _ => MeshType::Cube,
                    };
                    self.scene.world.add(eid, MeshComponent { mesh_data: None, mesh_type: Some(mesh_type) });
                    if let Some(t) = self.scene.world.get_mut::<TransformComponent>(eid) {
                        t.position = pos; t.rotation = rot; t.scale = scale;
                    }
                    let color = ed.data.get("color").and_then(|v| v.as_array())
                        .or_else(|| ed.data.get("material").and_then(|m| m.get("albedo")).and_then(|v| v.as_array()));
                    let (cr, cg, cb) = color
                        .map(|c| (c[0].as_f64().unwrap_or(0.8) as f32, c[1].as_f64().unwrap_or(0.8) as f32, c[2].as_f64().unwrap_or(0.8) as f32))
                        .unwrap_or((0.8, 0.8, 0.8));
                    let metallic = ed.data.get("metallic").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    let roughness = ed.data.get("roughness").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32;
                    self.scene.world.add(eid, MaterialComponent { albedo: (cr, cg, cb), metallic, roughness, ambient_occlusion: 1.0 });
                }
                "assign_material" => {
                    let target_id = ed.data.get("entity_id").and_then(|v| v.as_u64());
                    if let Some(tid) = target_id {
                        if let Some(c) = ed.data.get("color").and_then(|v| v.as_array()).filter(|c| c.len() >= 3) {
                            let (r, g, b) = (c[0].as_f64().unwrap_or(0.8) as f32, c[1].as_f64().unwrap_or(0.8) as f32, c[2].as_f64().unwrap_or(0.8) as f32);
                            if let Some(mat) = self.scene.world.get_mut::<MaterialComponent>(tid) { mat.albedo = (r, g, b); }
                            if let Some(mat) = self.scene.world.get_mut::<MaterialComponent>(tid) {
                                if let Some(metallic) = ed.data.get("metallic").and_then(|v| v.as_f64()) { mat.metallic = metallic as f32; }
                                if let Some(roughness) = ed.data.get("roughness").and_then(|v| v.as_f64()) { mat.roughness = roughness as f32; }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
