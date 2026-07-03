use serde::{Deserialize, Serialize};

use super::math::{Quaternion, Transform};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Joint {
    pub name: String,
    pub parent_index: i32,
    pub local_transform: TransformData,
    pub joint_limits: Option<JointLimits>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformData {
    pub translation: [f32; 3],
    pub rotation: QuaternionData,
    pub scale: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuaternionData {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl QuaternionData {
    #[allow(dead_code)]
    pub fn identity() -> Self {
        Self {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    #[allow(dead_code)]
    pub fn to_quat(&self) -> Quaternion {
        Quaternion {
            w: self.w,
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    #[allow(dead_code)]
    pub fn from_quat(q: Quaternion) -> Self {
        Self {
            w: q.w,
            x: q.x,
            y: q.y,
            z: q.z,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointLimits {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skeleton {
    pub name: String,
    pub joints: Vec<Joint>,
}

impl Skeleton {
    pub fn joint_count(&self) -> usize {
        self.joints.len()
    }

    #[allow(dead_code)]
    pub fn joint_names(&self) -> Vec<&str> {
        self.joints.iter().map(|j| j.name.as_str()).collect()
    }

    #[allow(dead_code)]
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.joints.iter().position(|j| j.name == name)
    }

    pub fn parent_indices(&self) -> Vec<i32> {
        self.joints.iter().map(|j| j.parent_index).collect()
    }

    #[allow(dead_code)]
    pub fn root_index(&self) -> usize {
        self.joints
            .iter()
            .position(|j| j.parent_index < 0)
            .unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for (i, j) in self.joints.iter().enumerate() {
            if j.parent_index >= i as i32 {
                errors.push(format!(
                    "Joint '{}' parent_index {} >= own index {}",
                    j.name, j.parent_index, i
                ));
            }
            if j.parent_index < -1 {
                errors.push(format!(
                    "Joint '{}' has invalid parent_index {}",
                    j.name, j.parent_index
                ));
            }
            if j.name.is_empty() {
                errors.push(format!("Joint at index {} has empty name", i));
            }
        }
        let roots = self.joints.iter().filter(|j| j.parent_index < 0).count();
        if roots == 0 {
            errors.push("Skeleton has no root joint".to_string());
        }
        if roots > 1 {
            errors.push(format!("Skeleton has {} root joints", roots));
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct Pose {
    pub skeleton: Skeleton,
    pub joint_rotations: Vec<Quaternion>,
    pub root_translation: (f32, f32, f32),
}

impl Pose {
    pub fn new(skeleton: &Skeleton) -> Self {
        let count = skeleton.joint_count();
        Self {
            skeleton: skeleton.clone(),
            joint_rotations: vec![Quaternion::identity(); count],
            root_translation: (0.0, 0.0, 0.0),
        }
    }

    pub fn local_transforms(&self) -> Vec<Transform> {
        let mut result = Vec::with_capacity(self.joint_rotations.len());
        for (i, rot) in self.joint_rotations.iter().enumerate() {
            let base = &self.skeleton.joints[i].local_transform;
            let t = if i > 0 {
                base.translation
            } else {
                [self.root_translation.0, self.root_translation.1, self.root_translation.2]
            };
            result.push(Transform {
                translation: (t[0], t[1], t[2]),
                rotation: *rot,
                scale: (base.scale[0], base.scale[1], base.scale[2]),
            });
        }
        result
    }
}
