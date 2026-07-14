use crate::core::ecs::*;
use crate::core::scene::Scene;
use crate::ui::style::*;
use egui::{Frame, Margin, Rounding, RichText, ScrollArea, Color32};

pub struct ScenePanel {
    pub open: bool,
}

impl ScenePanel {
    pub fn new() -> Self {
        Self { open: true }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, scene: &mut Scene) -> Option<EntityId> {
        let mut clicked: Option<EntityId> = None;

        ui.horizontal(|ui| {
            ui.label(RichText::new("Scene").size(13.0).color(TEXT));
            if ui.button(RichText::new("+").size(13.0).color(ACCENT)).clicked() {
                clicked = Some(scene.spawn_primitive(MeshType::Cube));
            }
        });
        ui.add_space(4.0);

        ScrollArea::vertical().show(ui, |ui| {
            let mut ids: Vec<EntityId> = scene.world.query::<TransformComponent>().iter().map(|(id, _)| *id).collect();
            ids.sort();

            for id in &ids {
                let label = scene.world.get::<LabelComponent>(*id)
                    .map(|l| {
                        if l.name.is_empty() { format!("{} #{}", l.entity_type, id) } else { l.name.clone() }
                    })
                    .unwrap_or_else(|| format!("Entity #{}", id));
                let is_selected = scene.world.get::<Selected>(*id).is_some();
                let icon = match scene.world.get::<MeshComponent>(*id).and_then(|m| m.mesh_type.as_ref()) {
                    Some(MeshType::Cube) | Some(MeshType::Custom) => "▣",
                    Some(MeshType::Sphere(_)) => "◉",
                    Some(MeshType::Plane) | Some(MeshType::Quad) => "▢",
                    Some(MeshType::Cylinder) => "⬡",
                    None => "◇",
                };

                let bg = if is_selected { BG_HOVER } else { Color32::TRANSPARENT };
                Frame::none()
                    .fill(bg)
                    .rounding(Rounding::same(4.0))
                    .inner_margin(Margin::symmetric(8.0, 3.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(icon).size(11.0).color(TEXT_MUTED));
                            let resp = ui.selectable_label(is_selected, RichText::new(&label).size(12.0).color(if is_selected { TEXT } else { TEXT_DIM }));
                            if resp.clicked() {
                                clicked = Some(*id);
                            }
                        });
                    });
            }
        });

        clicked
    }
}
