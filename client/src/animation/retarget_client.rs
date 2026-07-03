use std::collections::HashMap;

use crate::core::math::Quaternion;
use crate::core::skeleton::Skeleton;
use crate::animation::playback::MotionClip;

#[allow(dead_code)]
pub struct RetargetMap {
    pub src_to_tgt: HashMap<usize, usize>,
}

#[allow(dead_code)]
pub fn apply_retarget(clip: &MotionClip, target: &Skeleton, map: &RetargetMap) -> MotionClip {
    let tgt_count = target.joint_count();
    let mut new_frames = Vec::with_capacity(clip.frame_count());

    for src_rotations in &clip.frames {
        let mut new_rotations = vec![Quaternion::identity(); tgt_count];
        for (&src_idx, &tgt_idx) in &map.src_to_tgt {
            if tgt_idx < tgt_count && src_idx < src_rotations.len() {
                let src_joint = &clip.skeleton.joints[src_idx];
                let tgt_joint = &target.joints[tgt_idx];

                let src_rest = src_joint.local_transform.rotation.to_quat();
                let tgt_rest = tgt_joint.local_transform.rotation.to_quat();

                let src_local = src_rotations[src_idx];
                let tgt_local = tgt_rest.inverse() * (src_rest * src_local * src_rest.inverse()) * tgt_rest;
                new_rotations[tgt_idx] = tgt_local.normalize();
            }
        }
        new_frames.push(new_rotations);
    }

    let new_root_positions = if !clip.root_positions.is_empty() {
        let scale = limb_scale_factor(&clip.skeleton, target, &map.src_to_tgt);
        clip.root_positions
            .iter()
            .map(|&(x, y, z)| (x * scale, y * scale, z * scale))
            .collect()
    } else {
        clip.root_positions.clone()
    };

    MotionClip {
        skeleton: target.clone(),
        frames: new_frames,
        root_positions: new_root_positions,
        fps: clip.fps,
        loop_: clip.loop_,
    }
}

#[allow(dead_code)]
fn limb_scale_factor(src: &Skeleton, tgt: &Skeleton, map: &HashMap<usize, usize>) -> f32 {
    let mut total = 0.0f32;
    let mut count = 0u32;
    for (&src_idx, &tgt_idx) in map {
        let src_len = bone_length(src, src_idx);
        let tgt_len = bone_length(tgt, tgt_idx);
        if src_len > 0.01 {
            total += tgt_len / src_len;
            count += 1;
        }
    }
    if count == 0 {
        1.0
    } else {
        total / count as f32
    }
}

#[allow(dead_code)]
fn bone_length(skeleton: &Skeleton, joint_index: usize) -> f32 {
    if joint_index >= skeleton.joint_count() {
        return 0.0;
    }
    let t = skeleton.joints[joint_index].local_transform.translation;
    (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt()
}
