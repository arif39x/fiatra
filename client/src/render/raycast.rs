use crate::core::ecs::{EntityId, MeshType, TransformComponent};
use crate::core::scene::Scene;
use crate::render::camera::OrbitCamera;

pub struct Ray {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

pub fn screen_to_ray(x: f32, y: f32, screen_w: f32, screen_h: f32, camera: &OrbitCamera) -> Ray {
    let ndc_x = (x / screen_w) * 2.0 - 1.0;
    let ndc_y = 1.0 - (y / screen_h) * 2.0;
    let inv_view_proj = match invert_4x4(&camera.view_proj()) {
        Some(m) => m,
        None => return Ray { origin: [0.0; 3], direction: [0.0, 0.0, -1.0] },
    };
    let near = homogeneous_transform(&inv_view_proj, ndc_x, ndc_y, 0.0);
    let far = homogeneous_transform(&inv_view_proj, ndc_x, ndc_y, 1.0);
    let origin = near;
    let dir = [
        far[0] - near[0],
        far[1] - near[1],
        far[2] - near[2],
    ];
    let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    if len < 1e-8 {
        return Ray { origin, direction: [0.0, 0.0, -1.0] };
    }
    Ray {
        origin,
        direction: [dir[0] / len, dir[1] / len, dir[2] / len],
    }
}

fn homogeneous_transform(m: &[f32; 16], x: f32, y: f32, z: f32) -> [f32; 3] {
    let w = 1.0 / (m[3] * x + m[7] * y + m[11] * z + m[15]);
    [
        (m[0] * x + m[4] * y + m[8] * z + m[12]) * w,
        (m[1] * x + m[5] * y + m[9] * z + m[13]) * w,
        (m[2] * x + m[6] * y + m[10] * z + m[14]) * w,
    ]
}

pub fn ray_vs_sphere(ray: &Ray, center: [f32; 3], radius: f32) -> Option<f32> {
    let oc = [
        ray.origin[0] - center[0],
        ray.origin[1] - center[1],
        ray.origin[2] - center[2],
    ];
    let a = ray.direction[0] * ray.direction[0]
        + ray.direction[1] * ray.direction[1]
        + ray.direction[2] * ray.direction[2];
    let b = 2.0 * (oc[0] * ray.direction[0] + oc[1] * ray.direction[1] + oc[2] * ray.direction[2]);
    let c = oc[0] * oc[0] + oc[1] * oc[1] + oc[2] * oc[2] - radius * radius;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return None;
    }
    let t = (-b - disc.sqrt()) / (2.0 * a);
    if t < 0.0 { None } else { Some(t) }
}

pub fn pick_entity(
    mouse_x: f32,
    mouse_y: f32,
    screen_w: f32,
    screen_h: f32,
    camera: &OrbitCamera,
    scene: &Scene,
) -> Option<(EntityId, f32)> {
    let ray = screen_to_ray(mouse_x, mouse_y, screen_w, screen_h, camera);
    let mut closest: Option<(EntityId, f32)> = None;
    let transforms = scene.world.query::<TransformComponent>();
    for (id, _t) in &transforms {
        let world_mat = scene.compute_world_matrix(*id);
        let pos = [world_mat[12], world_mat[13], world_mat[14]];
        let scale_x = (world_mat[0] * world_mat[0] + world_mat[1] * world_mat[1] + world_mat[2] * world_mat[2]).sqrt();
        let radius = 0.5 * scale_x.max(0.01);
        if let Some(dist) = ray_vs_sphere(&ray, pos, radius) {
            match closest {
                Some((_, best_d)) if dist < best_d => closest = Some((*id, dist)),
                None => closest = Some((*id, dist)),
                _ => {}
            }
        }
    }
    closest
}

fn invert_4x4(m: &[f32; 16]) -> Option<[f32; 16]> {
    let a00 = m[0]; let a01 = m[1]; let a02 = m[2]; let a03 = m[3];
    let a10 = m[4]; let a11 = m[5]; let a12 = m[6]; let a13 = m[7];
    let a20 = m[8]; let a21 = m[9]; let a22 = m[10]; let a23 = m[11];
    let a30 = m[12]; let a31 = m[13]; let a32 = m[14]; let a33 = m[15];
    let b00 = a00 * a11 - a01 * a10;
    let b01 = a00 * a12 - a02 * a10;
    let b02 = a00 * a13 - a03 * a10;
    let b03 = a01 * a12 - a02 * a11;
    let b04 = a01 * a13 - a03 * a11;
    let b05 = a02 * a13 - a03 * a12;
    let b06 = a20 * a31 - a21 * a30;
    let b07 = a20 * a32 - a22 * a30;
    let b08 = a20 * a33 - a23 * a30;
    let b09 = a21 * a32 - a22 * a31;
    let b10 = a21 * a33 - a23 * a31;
    let b11 = a22 * a33 - a23 * a32;
    let det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;
    if det.abs() < 1e-8 { return None; }
    let inv_det = 1.0 / det;
    Some([
        (a11 * b11 - a12 * b10 + a13 * b09) * inv_det,
        (-a01 * b11 + a02 * b10 - a03 * b09) * inv_det,
        (a31 * b05 - a32 * b04 + a33 * b03) * inv_det,
        (-a21 * b05 + a22 * b04 - a23 * b03) * inv_det,
        (-a10 * b11 + a12 * b08 - a13 * b07) * inv_det,
        (a00 * b11 - a02 * b08 + a03 * b07) * inv_det,
        (-a30 * b05 + a32 * b02 - a33 * b01) * inv_det,
        (a20 * b05 - a22 * b02 + a23 * b01) * inv_det,
        (a10 * b10 - a11 * b08 + a13 * b06) * inv_det,
        (-a00 * b10 + a01 * b08 - a03 * b06) * inv_det,
        (a30 * b04 - a31 * b02 + a33 * b00) * inv_det,
        (-a20 * b04 + a21 * b02 - a23 * b00) * inv_det,
        (-a10 * b09 + a11 * b07 - a12 * b06) * inv_det,
        (a00 * b09 - a01 * b07 + a02 * b06) * inv_det,
        (-a30 * b03 + a31 * b01 - a32 * b00) * inv_det,
        (a20 * b03 - a21 * b01 + a22 * b00) * inv_det,
    ])
}
