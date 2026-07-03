use crate::core::ecs::{EntityId, MeshType, TransformComponent, LabelComponent, MeshComponent, Selected};
use crate::core::scene::Scene;

pub struct ScenePanel {
    pub open: bool,
}

impl ScenePanel {
    pub fn new() -> Self {
        Self { open: true }
    }

    pub fn draw(&mut self, ctx: &egui::Context, scene: &mut Scene) -> Option<EntityId> {
        let mut clicked: Option<EntityId> = None;
        egui::Window::new("Scene")
            .id(egui::Id::new("scene_panel"))
            .default_width(220.0)
            .show(ctx, |ui| {
                let ids: Vec<EntityId> = scene.world.query::<TransformComponent>().iter().map(|(id, _)| *id).collect();
                for id in &ids {
                    let label = scene.world.get::<LabelComponent>(*id)
                        .map(|l| {
                            if l.name.is_empty() { format!("{} #{}", l.entity_type, id) } else { l.name.clone() }
                        })
                        .unwrap_or_else(|| format!("Entity #{}", id));

                    let mesh_label = scene.world.get::<MeshComponent>(*id)
                        .and_then(|m| m.mesh_type.as_ref())
                        .map(|mt| match mt {
                            MeshType::Cube => " [Cube]",
                            MeshType::Sphere(_) => " [Sphere]",
                            MeshType::Plane => " [Plane]",
                            MeshType::Quad => " [Quad]",
                            MeshType::Cylinder => " [Cylinder]",
                            MeshType::Custom => " [Mesh]",
                        })
                        .unwrap_or("");

                    let is_selected = scene.world.get::<Selected>(*id).is_some();
                    let response = ui.selectable_label(is_selected, format!("{}{}", label, mesh_label));
                    if response.clicked() {
                        clicked = Some(*id);
                    }
                }
            });
        clicked
    }
}
