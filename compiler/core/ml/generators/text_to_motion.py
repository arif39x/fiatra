from __future__ import annotations

import json
import math
import os
from typing import Optional, Set

from ...animation.math import Quaternion
from ...animation.motion import MotionClip
from ...animation.skeleton import Skeleton


def _load_skeleton() -> Skeleton:
    path = os.path.join(os.path.dirname(__file__), "..", "..", "..", "asset", "base_skeletons", "humanoid.json")
    with open(path) as f:
        return Skeleton.from_dict(json.load(f))


def _find_joint(names: list[str], keywords: Set[str]) -> int:
    for n, name in enumerate(names):
        nl = name.lower()
        for kw in keywords:
            if kw in nl:
                return n
    return -1


def generate_motion(
    prompt: str,
    target_skeleton: Optional[Skeleton] = None,
    seed: Optional[int] = None,
) -> MotionClip:
    skeleton = target_skeleton or _load_skeleton()
    names = [j.name.lower() for j in skeleton.joints]

    prompt_lower = prompt.lower()
    if "run" in prompt_lower:
        motion_type = "run"
        num_frames = 30
    elif "walk" in prompt_lower:
        motion_type = "walk"
        num_frames = 60
    elif "wave" in prompt_lower:
        motion_type = "wave"
        num_frames = 60
    else:
        motion_type = "idle"
        num_frames = 30

    root = _find_joint(names, {"hips", "root", "hip", "pelvis", "spine0"})
    spine = _find_joint(names, {"spine1", "spine_1", "chest"})
    chest = _find_joint(names, {"spine2", "spine_2", "chest", "upper_chest"})
    neck = _find_joint(names, {"neck"})
    head = _find_joint(names, {"head"})

    lul = _find_joint(names, {"left_upper_leg", "thigh_l", "upper_leg_l", "left_thigh", "left_hip"})
    lll = _find_joint(names, {"left_lower_leg", "shin_l", "lower_leg_l", "left_shin", "left_knee"})
    lfoot = _find_joint(names, {"left_foot", "foot_l", "left_ankle"})
    rul = _find_joint(names, {"right_upper_leg", "thigh_r", "upper_leg_r", "right_thigh", "right_hip"})
    rll = _find_joint(names, {"right_lower_leg", "shin_r", "lower_leg_r", "right_shin", "right_knee"})
    rfoot = _find_joint(names, {"right_foot", "foot_r", "right_ankle"})

    lua = _find_joint(names, {"left_upper_arm", "upper_arm_l", "left_arm", "arm_l", "left_shoulder"})
    lla = _find_joint(names, {"left_lower_arm", "lower_arm_l", "left_forearm", "forearm_l", "left_elbow"})
    lhand = _find_joint(names, {"left_hand", "hand_l"})
    rua = _find_joint(names, {"right_upper_arm", "upper_arm_r", "right_arm", "arm_r", "right_shoulder"})
    rla = _find_joint(names, {"right_lower_arm", "lower_arm_r", "right_forearm", "forearm_r", "right_elbow"})
    rhand = _find_joint(names, {"right_hand", "hand_r"})

    frames: list[list[Quaternion]] = []
    root_positions: list[tuple[float, float, float]] = []

    for i in range(num_frames):
        t = i / max(num_frames - 1, 1)
        phase = 2.0 * math.pi * t
        rots = [Quaternion.identity()] * skeleton.joint_count()
        rp = (0.0, 0.0, 0.0)

        if motion_type == "walk":
            speed = 1.0
            swing = math.sin(phase * speed) * 0.5
            for j, antiswing in [(lul, swing), (rul, -swing)]:
                if j >= 0:
                    rots[j] = Quaternion.from_axis_angle((1, 0, 0), antiswing)
            for j, s in [(lll, swing), (rll, -swing)]:
                if j >= 0:
                    rots[j] = Quaternion.from_axis_angle((1, 0, 0), max(0.0, s) * 0.4)
            for j, s in [(lfoot, swing * 0.5), (rfoot, -swing * 0.5)]:
                if j >= 0:
                    rots[j] = Quaternion.from_axis_angle((1, 0, 0), max(0.0, s) * 0.3)
            arm = -swing * 0.4
            for j, s in [(lua, arm), (rua, -arm)]:
                if j >= 0:
                    rots[j] = Quaternion.from_axis_angle((1, 0, 0), s)
            for j, s in [(lla, arm * 0.3), (rla, -arm * 0.3)]:
                if j >= 0:
                    rots[j] = Quaternion.from_axis_angle((1, 0, 0), s)
            if root >= 0:
                rots[root] = Quaternion.from_axis_angle((0, 0, 1), math.sin(phase * speed) * 0.04)
            if spine >= 0:
                rots[spine] = Quaternion.from_axis_angle((1, 0, 0), math.sin(phase * speed) * 0.02)
            bounce = abs(math.sin(phase * speed)) * 0.04
            rp = (0.0, bounce, t * 1.5)
        elif motion_type == "run":
            speed = 2.5
            swing = math.sin(phase * speed) * 0.8
            for j, antiswing in [(lul, swing), (rul, -swing)]:
                if j >= 0:
                    rots[j] = Quaternion.from_axis_angle((1, 0, 0), antiswing)
            for j, s in [(lll, swing), (rll, -swing)]:
                if j >= 0:
                    rots[j] = Quaternion.from_axis_angle((1, 0, 0), max(0.0, s) * 0.3)
            arm = -swing * 0.5
            for j, s in [(lua, arm), (rua, -arm)]:
                if j >= 0:
                    rots[j] = Quaternion.from_axis_angle((1, 0, 0), s)
            for j, s in [(lla, arm * 0.2), (rla, -arm * 0.2)]:
                if j >= 0:
                    rots[j] = Quaternion.from_axis_angle((1, 0, 0), s)
            if root >= 0:
                rots[root] = Quaternion.from_axis_angle((0, 0, 1), math.sin(phase * speed) * 0.06)
            if spine >= 0:
                rots[spine] = Quaternion.from_axis_angle((1, 0, 0), math.sin(phase * speed) * 0.03)
            bounce = abs(math.sin(phase * speed)) * 0.08
            rp = (0.0, bounce, t * 5.0)
        elif motion_type == "wave":
            for j in [rua, rla, rhand]:
                if j >= 0:
                    rots[j] = Quaternion.from_axis_angle((0, 0, 1), -1.2)
            if rla >= 0:
                rots[rla] = Quaternion.from_axis_angle((0, 0, 1), math.sin(phase * 3.0) * 0.8)
            for j in [lua, lla, lhand]:
                if j >= 0:
                    rots[j] = Quaternion.from_axis_angle((1, 0, 0), -0.3)
            if neck >= 0:
                rots[neck] = Quaternion.from_axis_angle((0, 0, 1), math.sin(phase * 2.0) * 0.1)
        else:
            breathe = math.sin(phase * 0.5) * 0.02
            if spine >= 0:
                rots[spine] = Quaternion.from_axis_angle((1, 0, 0), breathe)
            if chest >= 0:
                rots[chest] = Quaternion.from_axis_angle((1, 0, 0), breathe * 0.5)
            if head >= 0:
                rots[head] = Quaternion.from_axis_angle((0, 0, 1), math.sin(phase * 1.2) * 0.01)

        frames.append(rots)
        root_positions.append(rp)

    return MotionClip(
        skeleton=skeleton,
        frames=frames,
        root_positions=root_positions,
        fps=30.0,
        loop=motion_type != "wave",
    )
