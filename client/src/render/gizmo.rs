use crate::core::scene::Scene;
use crate::render::mesh::StaticVertex;

fn create_cylinder(radius: f32, height: f32, segments: u32) -> (Vec<StaticVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let half = height * 0.5;

    for i in 0..=segments {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
        let ca = angle.cos();
        let sa = angle.sin();
        vertices.push(StaticVertex {
            position: [ca * radius, -half, sa * radius],
            normal: [ca, 0.0, sa],
            uv: [i as f32 / segments as f32, 0.0],
        });
        vertices.push(StaticVertex {
            position: [ca * radius, half, sa * radius],
            normal: [ca, 0.0, sa],
            uv: [i as f32 / segments as f32, 1.0],
        });
    }
    for i in 0..segments {
        let a = i * 2;
        let b = a + 1;
        let c = (i + 1) * 2;
        let d = c + 1;
        indices.push(a); indices.push(c); indices.push(b);
        indices.push(b); indices.push(c); indices.push(d);
    }
    (vertices, indices)
}

fn create_cone(radius: f32, height: f32, segments: u32) -> (Vec<StaticVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    vertices.push(StaticVertex {
        position: [0.0, height * 0.5, 0.0],
        normal: [0.0, 1.0, 0.0],
        uv: [0.5, 1.0],
    });
    for i in 0..=segments {
        let angle = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
        let ca = angle.cos();
        let sa = angle.sin();
        vertices.push(StaticVertex {
            position: [ca * radius, -height * 0.5, sa * radius],
            normal: [ca * 0.5, 0.5, sa * 0.5],
            uv: [i as f32 / segments as f32, 0.0],
        });
    }
    for i in 0..segments {
        indices.push(0);
        indices.push(i + 1);
        indices.push(i + 2);
    }
    (vertices, indices)
}

pub fn create_axis_arrow(length: f32) -> (Vec<StaticVertex>, Vec<u32>) {
    let shaft_len = length * 0.7;
    let head_len = length * 0.3;
    let shaft_radius = 0.02;
    let head_radius = 0.06;
    let segs = 8;

    let (shaft_verts, shaft_idxs) = create_cylinder(shaft_radius, shaft_len, segs);
    let (head_verts, head_idxs) = create_cone(head_radius, head_len, segs);

    let shaft_offset = -shaft_len * 0.5;
    let head_offset = shaft_len * 0.5;

    let mut verts = Vec::new();
    let mut idxs = Vec::new();

    for mut v in shaft_verts {
        v.position[1] += shaft_offset;
        verts.push(v);
    }
    for i in shaft_idxs {
        idxs.push(i);
    }

    let base_idx = verts.len() as u32;
    for mut v in head_verts {
        v.position[1] += head_offset;
        verts.push(v);
    }
    for i in head_idxs {
        idxs.push(base_idx + i);
    }

    (verts, idxs)
}

pub fn collect_gizmo_data(scene: &Scene) -> Vec<(Vec<StaticVertex>, Vec<u32>, [f32; 16], [f32; 3], f32, f32)> {
    let mut results = Vec::new();
    let selected = scene.world.query::<crate::core::ecs::Selected>();
    if selected.is_empty() {
        return results;
    }
    let id = selected[0].0;
    let world_mat = scene.compute_world_matrix(id);
    let pos = [world_mat[12], world_mat[13], world_mat[14]];

    let arrow_len = 0.5;
    let (arrow_verts, arrow_idxs) = create_axis_arrow(arrow_len);

    let axes: [([f32; 3], [f32; 3]); 3] = [
        ([1.0, 0.0, 0.0], [1.0, 0.15, 0.15]),
        ([0.0, 1.0, 0.0], [0.15, 1.0, 0.15]),
        ([0.0, 0.0, 1.0], [0.15, 0.15, 1.0]),
    ];

    for (dir, color) in axes {
        let mat = [
            dir[0], dir[1], dir[2], 0.0,
            0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0,
            pos[0], pos[1], pos[2], 1.0,
        ];
        // Build a proper rotation matrix for the arrow direction
        let up = [0.0, 1.0, 0.0];
        let fwd = dir;
        let right = cross(fwd, up);
        let right_len = (right[0]*right[0] + right[1]*right[1] + right[2]*right[2]).sqrt();
        let (r, u, f) = if right_len > 0.01 {
            let rn = [right[0]/right_len, right[1]/right_len, right[2]/right_len];
            let u2 = cross(fwd, rn);
            (rn, u2, fwd)
        } else {
            ([1.0, 0.0, 0.0], [0.0, 0.0, fwd[1].signum()], fwd)
        };
        let rot_mat = [
            r[0], r[1], r[2], 0.0,
            u[0], u[1], u[2], 0.0,
            f[0], f[1], f[2], 0.0,
            pos[0], pos[1], pos[2], 1.0,
        ];
        results.push((arrow_verts.clone(), arrow_idxs.clone(), rot_mat, color, 0.2, 0.3));
    }

    results
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1]*b[2] - a[2]*b[1], a[2]*b[0] - a[0]*b[2], a[0]*b[1] - a[1]*b[0]]
}
