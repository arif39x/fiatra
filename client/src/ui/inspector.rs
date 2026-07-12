use crate::core::ecs::{EntityId, MaterialComponent, TransformComponent};
use crate::core::scene::Scene;
use crate::core::undo::{EditCommand, MaterialSnapshot, TransformSnapshot, UndoStack};

pub struct Inspector {
    pub open: bool,
}

impl Inspector {
    pub fn new() -> Self {
        Self { open: true }
    }

    pub fn draw(&mut self, ctx: &egui::Context, scene: &mut Scene, selected: Option<EntityId>, undo: &mut UndoStack) {
        let Some(entity) = selected else { return };

        egui::Window::new("Inspector")
            .id(egui::Id::new("inspector_panel"))
            .default_width(260.0)
            .show(ctx, |ui| {
                let mut changed = false;
                let mut snapshot_transform: Option<TransformSnapshot> = None;
                let mut snapshot_material: Option<MaterialSnapshot> = None;

                if let Some(transform) = scene.world.get::<TransformComponent>(entity).copied() {
                    let mut pos = [transform.position.0, transform.position.1, transform.position.2];
                    let mut rot = [transform.rotation.0, transform.rotation.1, transform.rotation.2];
                    let mut scale = [transform.scale.0, transform.scale.1, transform.scale.2];

                    ui.label("Transform");
                    ui.horizontal(|ui| {
                        ui.label("X"); changed |= ui.add(egui::DragValue::new(&mut pos[0]).speed(0.05).prefix(" ")).changed();
                        ui.label("Y"); changed |= ui.add(egui::DragValue::new(&mut pos[1]).speed(0.05).prefix(" ")).changed();
                        ui.label("Z"); changed |= ui.add(egui::DragValue::new(&mut pos[2]).speed(0.05).prefix(" ")).changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label("RX"); changed |= ui.add(egui::DragValue::new(&mut rot[0]).speed(0.05).prefix(" ")).changed();
                        ui.label("RY"); changed |= ui.add(egui::DragValue::new(&mut rot[1]).speed(0.05).prefix(" ")).changed();
                        ui.label("RZ"); changed |= ui.add(egui::DragValue::new(&mut rot[2]).speed(0.05).prefix(" ")).changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label("SX"); changed |= ui.add(egui::DragValue::new(&mut scale[0]).speed(0.05).prefix(" ")).changed();
                        ui.label("SY"); changed |= ui.add(egui::DragValue::new(&mut scale[1]).speed(0.05).prefix(" ")).changed();
                        ui.label("SZ"); changed |= ui.add(egui::DragValue::new(&mut scale[2]).speed(0.05).prefix(" ")).changed();
                    });

                    if changed {
                        if let Some(t) = scene.world.get_mut::<TransformComponent>(entity) {
                            t.position = (pos[0], pos[1], pos[2]);
                            t.rotation = (rot[0], rot[1], rot[2]);
                            t.scale = (scale[0], scale[1], scale[2]);
                            snapshot_transform = Some(TransformSnapshot {
                                entity,
                                prev: transform,
                                current: *t,
                            });
                        }
                    }
                }

                if let Some(material) = scene.world.get::<MaterialComponent>(entity).copied() {
                    let mut color = [material.albedo.0, material.albedo.1, material.albedo.2];
                    let mut metallic = material.metallic;
                    let mut roughness = material.roughness;

                    ui.separator();
                    ui.label("Material");
                    if ui.color_edit_button_rgb(&mut color).changed() {
                        changed = true;
                    }
                    ui.horizontal(|ui| {
                        ui.label("Metallic");
                        changed |= ui.add(egui::Slider::new(&mut metallic, 0.0..=1.0).text("")).changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label("Roughness");
                        changed |= ui.add(egui::Slider::new(&mut roughness, 0.0..=1.0).text("")).changed();
                    });

                    if changed {
                        if let Some(m) = scene.world.get_mut::<MaterialComponent>(entity) {
                            let prev = *m;
                            m.albedo = (color[0], color[1], color[2]);
                            m.metallic = metallic;
                            m.roughness = roughness;
                            snapshot_material = Some(MaterialSnapshot {
                                entity,
                                prev,
                                current: *m,
                            });
                        }
                    }

                    if let Some(snap) = snapshot_material {
                        undo.push(EditCommand::Material(snap));
                    }
                }

                if let Some(snap) = snapshot_transform {
                    undo.push(EditCommand::Transform(snap));
                }
            });
    }
}
