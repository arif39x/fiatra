use crate::core::ecs::{EntityId, MeshType};
use crate::core::scene::Scene;

pub struct Toolbar {
    pub open: bool,
}

impl Toolbar {
    pub fn new() -> Self {
        Self { open: true }
    }

    pub fn draw(&mut self, ctx: &egui::Context, scene: &mut Scene, selected: &mut Option<EntityId>) {
        egui::Window::new("Toolbar")
            .id(egui::Id::new("toolbar_window"))
            .default_width(160.0)
            .show(ctx, |ui| {
                ui.label("Primitives");
                if ui.button("Add Cube").clicked() {
                    let id = scene.spawn_primitive(MeshType::Cube);
                    *selected = Some(id);
                }
                if ui.button("Add Sphere").clicked() {
                    let id = scene.spawn_primitive(MeshType::Sphere(16));
                    *selected = Some(id);
                }
                if ui.button("Add Plane").clicked() {
                    let id = scene.spawn_primitive(MeshType::Plane);
                    *selected = Some(id);
                }
                if ui.button("Add Cylinder").clicked() {
                    let id = scene.spawn_primitive(MeshType::Cylinder);
                    *selected = Some(id);
                }

                ui.separator();
                ui.label("Actions");
                if ui.button("Delete Selected").clicked() {
                    if let Some(id) = *selected {
                        scene.remove_entity(id);
                        *selected = None;
                    }
                }
            });
    }
}
