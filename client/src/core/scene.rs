use crate::core::ecs::*;
use crate::core::math::{multiply_mat4, Quaternion, Transform};
use crate::render::mesh::{create_cube, create_plane, create_sphere, StaticVertex};
use crate::render::static_renderer::translation_matrix;

pub struct Scene {
    pub world: EcsWorld,
}

impl Scene {
    pub fn new() -> Self {
        Self { world: EcsWorld::new() }
    }

    pub fn spawn_primitive(&mut self, mesh_type: MeshType) -> EntityId {
        let id = self.world.spawn();
        self.world.add(id, TransformComponent::identity());
        self.world.add(id, MeshComponent::from_type(mesh_type));
        self.world.add(id, MaterialComponent {
            albedo: (0.8, 0.8, 0.8),
            metallic: 0.0,
            roughness: 0.5,
            ambient_occlusion: 1.0,
        });
        self.world.add(id, LabelComponent {
            name: String::new(),
            entity_type: String::new(),
        });
        id
    }

    pub fn remove_entity(&mut self, id: EntityId) {
        self.world.remove::<TransformComponent>(id);
        self.world.remove::<MeshComponent>(id);
        self.world.remove::<MaterialComponent>(id);
        self.world.remove::<LabelComponent>(id);
        self.world.remove::<Selected>(id);
    }

    pub fn entity_count(&self) -> usize {
        self.world.query::<TransformComponent>().len()
    }

    pub fn compute_world_matrix(&self, id: EntityId) -> [f32; 16] {
        let world = &self.world;
        let local = match world.get::<TransformComponent>(id) {
            Some(t) => *t,
            None => return crate::render::static_renderer::identity_matrix(),
        };

        let rotation = Quaternion::from_euler(
            local.rotation.0,
            local.rotation.1,
            local.rotation.2,
        );
        let transform = Transform {
            translation: local.position,
            rotation,
            scale: local.scale,
        };
        let local_mat = transform.to_matrix();

        match local.parent_id {
            None => local_mat,
            Some(pid) => {
                let parent_mat = self.compute_world_matrix(pid);
                multiply_mat4(&parent_mat, &local_mat)
            }
        }
    }

    pub fn collect_render_data(&self, selected: Option<EntityId>) -> Vec<(MeshType, [f32; 16], (f32, f32, f32), f32, f32)> {
        let transforms = self.world.query::<TransformComponent>();
        let meshes = self.world.query::<MeshComponent>();
        let materials = self.world.query::<MaterialComponent>();

        let mut results = Vec::new();

        for (id, _transform) in &transforms {
            let mesh = match meshes.iter().find(|(mid, _)| *mid == *id) {
                Some((_, m)) => m,
                None => continue,
            };
            let mesh_type = match &mesh.mesh_type {
                Some(mt) => mt.clone(),
                None => continue,
            };
            let material = materials.iter().find(|(mid, _)| *mid == *id).map(|(_, m)| *m);
            let world_mat = self.compute_world_matrix(*id);
            let (r, g, b, metallic, roughness) = match material {
                Some(m) => (m.albedo.0, m.albedo.1, m.albedo.2, m.metallic, m.roughness),
                None => (0.8, 0.8, 0.8, 0.0, 0.5),
            };
            let (r, g, b) = if Some(*id) == selected {
                (r.min(1.0) * 1.5, g.min(1.0) * 1.5, b.min(1.0) * 1.5)
            } else {
                (r, g, b)
            };
            results.push((mesh_type, world_mat, (r, g, b), metallic, roughness));
        }

        results
    }
}
