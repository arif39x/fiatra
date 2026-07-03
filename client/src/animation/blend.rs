use crate::core::skeleton::Pose;

#[allow(dead_code)]
pub fn crossfade(a: &Pose, b: &Pose, t: f32) -> Pose {
    let joints = a.joint_rotations.len().min(b.joint_rotations.len());
    let mut rotations = Vec::with_capacity(joints);
    for i in 0..joints {
        rotations.push(a.joint_rotations[i].slerp(b.joint_rotations[i], t));
    }
    let root = (
        a.root_translation.0 + (b.root_translation.0 - a.root_translation.0) * t,
        a.root_translation.1 + (b.root_translation.1 - a.root_translation.1) * t,
        a.root_translation.2 + (b.root_translation.2 - a.root_translation.2) * t,
    );
    Pose {
        skeleton: a.skeleton.clone(),
        joint_rotations: rotations,
        root_translation: root,
    }
}

#[allow(dead_code)]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
