use crate::core::math::multiply_mat4;

pub struct OrbitCamera {
    pub pitch: f32,
    pub yaw: f32,
    pub distance: f32,
    pub target: [f32; 3],
    pub fov_y: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl OrbitCamera {
    pub fn new(aspect: f32) -> Self {
        Self {
            pitch: 0.4,
            yaw: 0.6,
            distance: 3.5,
            target: [0.0, 0.9, 0.0],
            fov_y: std::f32::consts::PI / 3.0,
            aspect,
            near: 0.05,
            far: 100.0,
        }
    }

    pub fn eye(&self) -> [f32; 3] {
        let (sp, cp) = self.pitch.sin_cos();
        let (sy, cy) = self.yaw.sin_cos();
        [
            self.target[0] + self.distance * cy * sp,
            self.target[1] + self.distance * cp,
            self.target[2] + self.distance * sy * sp,
        ]
    }

    pub fn view_matrix(&self) -> [f32; 16] {
        look_at(self.eye(), self.target, [0.0, 1.0, 0.0])
    }

    pub fn proj_matrix(&self) -> [f32; 16] {
        perspective_reverse_z(self.fov_y, self.aspect, self.near, self.far)
    }

    pub fn view_proj(&self) -> [f32; 16] {
        multiply_mat4(&self.proj_matrix(), &self.view_matrix())
    }
}

fn perspective_reverse_z(fov_y: f32, aspect: f32, near: f32, far: f32) -> [f32; 16] {
    let f = 1.0 / (fov_y * 0.5).tan();
    [
        f / aspect, 0.0, 0.0, 0.0,
        0.0, f, 0.0, 0.0,
        0.0, 0.0, near / (near - far), -1.0,
        0.0, 0.0, near * far / (near - far), 0.0,
    ]
}

fn look_at(eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> [f32; 16] {
    let fwd = normalize(sub(target, eye));
    let right = normalize(cross(fwd, up));
    let up2 = cross(right, fwd);
    [
        right[0], up2[0], -fwd[0], 0.0,
        right[1], up2[1], -fwd[1], 0.0,
        right[2], up2[2], -fwd[2], 0.0,
        -dot(right, eye), -dot(up2, eye), dot(fwd, eye), 1.0,
    ]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-8 { return [0.0, 1.0, 0.0]; }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
