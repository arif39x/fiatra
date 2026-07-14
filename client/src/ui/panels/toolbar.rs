use egui::{RichText, vec2};

use crate::core::ecs::{EntityId, MeshType};
use crate::core::scene::Scene;
use crate::ui::style::*;

pub struct Toolbar {
    pub open: bool,
}

fn tb(ui: &mut egui::Ui, label: &str, tip: &str) -> egui::Response {
    let r = ui.add(
        egui::Button::new(RichText::new(label).size(11.0).color(TEXT_DIM))
            .fill(BG_CARD)
            .min_size(vec2(28.0, 22.0))
            .rounding(egui::Rounding::same(4.0)),
    );
    let r2 = r.on_hover_text(tip);
    r2
}

impl Toolbar {
    pub fn new() -> Self {
        Self { open: true }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, scene: &mut Scene, selected: &mut Option<EntityId>) -> Option<String> {
        ui.horizontal(|ui| {
            ui.style_mut().spacing.item_spacing = vec2(2.0, 0.0);

            if tb(ui, "N", "New scene").clicked() {
                scene.spawn_primitive(MeshType::Cube);
            }
            if tb(ui, "O", "Open scene").clicked() {
                if let Some(p) = rfd::FileDialog::new().add_filter("Scene", &["json"]).pick_file() {
                    let _ = scene.load_from_file(p.to_str().unwrap_or("scene.json"));
                }
            }
            if tb(ui, "S", "Save scene").clicked() {
                if let Some(p) = rfd::FileDialog::new()
                    .add_filter("Scene", &["json"])
                    .set_file_name("scene.json")
                    .save_file()
                {
                    let _ = scene.save_to_file(p.to_str().unwrap_or("scene.json"));
                }
            }

            ui.separator();
            tb(ui, "Mv", "Translate");
            tb(ui, "Rt", "Rotate");
            tb(ui, "Sc", "Scale");

            ui.separator();

            if tb(ui, "+", "Add cube").clicked() {
                *selected = Some(scene.spawn_primitive(MeshType::Cube));
            }
            if tb(ui, "●", "Add sphere").clicked() {
                *selected = Some(scene.spawn_primitive(MeshType::Sphere(16)));
            }
            if tb(ui, "—", "Add plane").clicked() {
                *selected = Some(scene.spawn_primitive(MeshType::Plane));
            }
            if tb(ui, "⭑", "Add cylinder").clicked() {
                *selected = Some(scene.spawn_primitive(MeshType::Cylinder));
            }

            ui.separator();

            if tb(ui, "✕", "Delete selected").clicked() {
                if let Some(id) = *selected {
                    scene.remove_entity(id);
                    *selected = None;
                }
            }
        });

        None
    }
}
