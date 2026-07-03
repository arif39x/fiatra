use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;

pub type EntityId = u64;

// --- Scene Components ---

pub struct SkeletonComponent {
    pub skeleton: crate::core::skeleton::Skeleton,
}

#[derive(Clone)]
pub enum MeshType {
    Cube,
    Sphere(u32),
    Plane,
    Quad,
    Cylinder,
    Custom,
}

pub struct MeshComponent {
    pub mesh_data: Option<serde_json::Value>,
    pub mesh_type: Option<MeshType>,
}

impl MeshComponent {
    pub fn from_type(mesh_type: MeshType) -> Self {
        Self { mesh_data: None, mesh_type: Some(mesh_type) }
    }
}

pub struct MotionComponent {
    pub animator: crate::animation::playback::Animator,
    pub joint_params: Option<serde_json::Value>,
}

#[derive(Clone, Copy)]
pub struct TransformComponent {
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32),
    pub scale: (f32, f32, f32),
    pub parent_id: Option<EntityId>,
}

impl TransformComponent {
    pub fn identity() -> Self {
        Self { position: (0.0, 0.0, 0.0), rotation: (0.0, 0.0, 0.0), scale: (1.0, 1.0, 1.0), parent_id: None }
    }
}

pub enum LightType { Directional, Point, Ambient }

pub struct LightingComponent {
    pub light_type: LightType,
    pub direction: (f32, f32, f32),
    pub color: (f32, f32, f32),
    pub intensity: f32,
    pub ambient: (f32, f32, f32),
}

#[derive(Clone, Copy)]
pub struct MaterialComponent {
    pub albedo: (f32, f32, f32),
    pub metallic: f32,
    pub roughness: f32,
    pub ambient_occlusion: f32,
}

pub struct LabelComponent {
    pub name: String,
    pub entity_type: String,
}

pub struct Selected;

#[derive(Default)]
pub struct EcsWorld {
    next_id: EntityId,
    components: HashMap<TypeId, HashMap<EntityId, Box<dyn Any>>>,
}

impl EcsWorld {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self) -> EntityId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add<T: 'static>(&mut self, entity: EntityId, component: T) {
        let type_id = TypeId::of::<T>();
        let entries = self
            .components
            .entry(type_id)
            .or_insert_with(HashMap::new);
        entries.insert(entity, Box::new(component));
    }

    pub fn get<T: 'static>(&self, entity: EntityId) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&type_id)
            .and_then(|entries| entries.get(&entity))
            .and_then(|any| any.downcast_ref::<T>())
    }

    pub fn get_mut<T: 'static>(&mut self, entity: EntityId) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)
            .and_then(|entries| entries.get_mut(&entity))
            .and_then(|any| any.downcast_mut::<T>())
    }

    pub fn remove<T: 'static>(&mut self, entity: EntityId) -> bool {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)
            .and_then(|entries| entries.remove(&entity))
            .is_some()
    }

    pub fn query<T: 'static>(&self) -> Vec<(EntityId, &T)> {
        let type_id = TypeId::of::<T>();
        let mut results = Vec::new();
        if let Some(entries) = self.components.get(&type_id) {
            for (&id, any) in entries {
                if let Some(comp) = any.downcast_ref::<T>() {
                    results.push((id, comp));
                }
            }
        }
        results
    }

    pub fn query_mut<T: 'static>(&mut self) -> Vec<(EntityId, &mut T)> {
        let type_id = TypeId::of::<T>();
        let mut results = Vec::new();
        if let Some(entries) = self.components.get_mut(&type_id) {
            for (&id, any) in entries.iter_mut() {
                if let Some(comp) = any.downcast_mut::<T>() {
                    results.push((id, comp));
                }
            }
        }
        results
    }
}

pub trait Component: 'static {}

pub struct Query<'a, T: 'static> {
    world: &'a EcsWorld,
    _marker: PhantomData<T>,
}

impl<'a, T: 'static> Query<'a, T> {
    pub fn new(world: &'a EcsWorld) -> Self {
        Self {
            world,
            _marker: PhantomData,
        }
    }

    pub fn iter(&self) -> Vec<(EntityId, &T)> {
        self.world.query::<T>()
    }
}
