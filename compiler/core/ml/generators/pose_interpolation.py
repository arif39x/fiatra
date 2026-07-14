from __future__ import annotations

import math
from typing import List, Optional

from ...animation.math import Quaternion
from ...animation.motion import MotionClip
from ...animation.skeleton import Pose, Skeleton


def procedural_interpolate(
    pose_a: Pose,
    pose_b: Pose,
    num_frames: int,
    easing: str = "linear",
) -> MotionClip:
    if pose_a.skeleton.joint_count() != pose_b.skeleton.joint_count():
        raise ValueError("Poses must share the same skeleton")

    skeleton = pose_a.skeleton
    frames: List[List[Quaternion]] = []
    root_positions: List[tuple[float, float, float]] = []

    for i in range(num_frames):
        t = i / max(num_frames - 1, 1)
        t = _apply_easing(t, easing)
        rotations = [
            pose_a.joint_rotations[j].slerp(pose_b.joint_rotations[j], t)
            for j in range(skeleton.joint_count())
        ]
        root_positions.append(
            (
                pose_a.root_translation[0]
                + (pose_b.root_translation[0] - pose_a.root_translation[0]) * t,
                pose_a.root_translation[1]
                + (pose_b.root_translation[1] - pose_a.root_translation[1]) * t,
                pose_a.root_translation[2]
                + (pose_b.root_translation[2] - pose_a.root_translation[2]) * t,
            )
        )
        frames.append(rotations)

    return MotionClip(
        skeleton=skeleton,
        frames=frames,
        root_positions=root_positions,
        fps=30.0,
        loop=False,
    )


def _apply_easing(t: float, easing: str) -> float:
    if easing == "linear":
        return t
    elif easing == "ease_in":
        return t * t
    elif easing == "ease_out":
        return 1.0 - (1.0 - t) * (1.0 - t)
    elif easing == "ease_in_out":
        if t < 0.5:
            return 2.0 * t * t
        return 1.0 - (-2.0 * t + 2.0) ** 2 * 0.5
    elif easing == "smoothstep":
        return t * t * (3.0 - 2.0 * t)
    return t
