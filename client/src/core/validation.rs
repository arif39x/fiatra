use super::skeleton::Pose;

#[allow(dead_code)]
pub fn validate_pose(pose: &Pose) -> Vec<String> {
    let mut errors = pose.skeleton.validate();
    for (i, rot) in pose.joint_rotations.iter().enumerate() {
        let n = (rot.w * rot.w + rot.x * rot.x + rot.y * rot.y + rot.z * rot.z).sqrt();
        if (n - 1.0).abs() > 0.01 {
            errors.push(format!("Joint {} has non-unit quaternion (norm={})", i, n));
        }
    }
    errors
}
