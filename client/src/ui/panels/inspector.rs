use egui::{Frame, Margin, Rounding, RichText, CollapsingHeader, vec2};

use crate::core::ecs::*;
use crate::core::scene::Scene;
use crate::core::undo::{EditCommand, MaterialSnapshot, TransformSnapshot, UndoStack};
use crate::ui::style::*;

pub struct Inspector {
    pub open: bool,
}

impl Inspector {
    pub fn new() -> Self {
        Self { open: true }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, scene: &mut Scene, selected: Option<EntityId>, undo: &mut UndoStack) {
        let Some(entity) = selected else { return };

        let label = scene.world.get::<LabelComponent>(entity)
            .map(|l| if l.name.is_empty() { format!("Entity #{}", entity) } else { l.name.clone() })
            .unwrap_or_else(|| format!("Entity #{}", entity));
        ui.label(RichText::new(&label).size(13.0).color(TEXT));
        ui.add_space(8.0);

        CollapsingHeader::new("Transform")
            .default_open(true)
            .show(ui, |ui| {
                section_frame(ui, |ui| {
                    if let Some(transform) = scene.world.get::<TransformComponent>(entity).copied() {
                        let mut pos = [transform.position.0, transform.position.1, transform.position.2];
                        let mut rot = [transform.rotation.0, transform.rotation.1, transform.rotation.2];
                        let mut scale = [transform.scale.0, transform.scale.1, transform.scale.2];
                        let mut changed = false;

                        ui.label(RichText::new("Position").size(11.0).color(TEXT_DIM));
                        changed |= drag3(ui, &mut pos, 0.05);

                        ui.label(RichText::new("Rotation").size(11.0).color(TEXT_DIM));
                        changed |= drag3(ui, &mut rot, 0.05);

                        ui.label(RichText::new("Scale").size(11.0).color(TEXT_DIM));
                        changed |= drag3(ui, &mut scale, 0.05);

                        if changed {
                            if let Some(t) = scene.world.get_mut::<TransformComponent>(entity) {
                                let prev = *t;
                                t.position = (pos[0], pos[1], pos[2]);
                                t.rotation = (rot[0], rot[1], rot[2]);
                                t.scale = (scale[0], scale[1], scale[2]);
                                undo.push(EditCommand::Transform(TransformSnapshot { entity, prev, current: *t }));
                            }
                        }
                    }
                });
            });

        CollapsingHeader::new("Material")
            .default_open(true)
            .show(ui, |ui| {
                section_frame(ui, |ui| {
                    if let Some(material) = scene.world.get::<MaterialComponent>(entity).copied() {
                        let mut color = [material.albedo.0, material.albedo.1, material.albedo.2];
                        let mut metallic = material.metallic;
                        let mut roughness = material.roughness;
                        let mut changed = false;

                        if ui.color_edit_button_rgb(&mut color).changed() { changed = true; }
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Metallic").size(11.0).color(TEXT_DIM));
                            changed |= ui.add(egui::Slider::new(&mut metallic, 0.0..=1.0)).changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Roughness").size(11.0).color(TEXT_DIM));
                            changed |= ui.add(egui::Slider::new(&mut roughness, 0.0..=1.0)).changed();
                        });

                        if changed {
                            if let Some(m) = scene.world.get_mut::<MaterialComponent>(entity) {
                                let prev = *m;
                                m.albedo = (color[0], color[1], color[2]);
                                m.metallic = metallic;
                                m.roughness = roughness;
                                undo.push(EditCommand::Material(MaterialSnapshot { entity, prev, current: *m }));
                            }
                        }
                    }
                });
            });
    }
}

fn section_frame(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    Frame::none()
        .fill(BG_CARD)
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::symmetric(10.0, 8.0))
        .show(ui, add_contents);
}

fn drag3(ui: &mut egui::Ui, v: &mut [f32; 3], speed: f32) -> bool {
    ui.horizontal(|ui| {
        ui.style_mut().spacing.item_spacing = vec2(2.0, 0.0);
        let mut changed = false;
        changed |= ui.add(egui::DragValue::new(&mut v[0]).speed(speed).prefix("X ").custom_formatter(|n, _| format!("{:.2}", n))).changed();
        changed |= ui.add(egui::DragValue::new(&mut v[1]).speed(speed).prefix("Y ").custom_formatter(|n, _| format!("{:.2}", n))).changed();
        changed |= ui.add(egui::DragValue::new(&mut v[2]).speed(speed).prefix("Z ").custom_formatter(|n, _| format!("{:.2}", n))).changed();
        changed
    }).inner
}
