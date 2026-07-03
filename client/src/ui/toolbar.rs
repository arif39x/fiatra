use crate::core::ecs::{EntityId, MeshType};
use crate::core::scene::Scene;

pub struct Toolbar {
    pub open: bool,
}

impl Toolbar {
    pub fn new() -> Self {
        Self { open: true }
    }

    pub fn draw(&mut self, ctx: &egui::Context, scene: &mut Scene, selected: &mut Option<EntityId>) -> Option<String> {
        let mut quick_command: Option<String> = None;

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

                ui.separator();
                ui.label("Quick Commands");
                if ui.button("Arrange in Grid").clicked() {
                    quick_command = Some("Arrange all objects in a 3x3 grid".to_string());
                }
                if ui.button("Randomize Colors").clicked() {
                    quick_command = Some("Randomize the colors of all objects".to_string());
                }
                if ui.button("Arrange in Circle").clicked() {
                    quick_command = Some("Arrange all objects in a circle".to_string());
                }
                if ui.button("Clear Scene").clicked() {
                    quick_command = Some("Clear the scene and start fresh".to_string());
                }
            });

        quick_command
    }
}
