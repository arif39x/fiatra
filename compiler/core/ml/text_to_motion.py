from __future__ import annotations

import json
import math
import os
from typing import Optional

from ..animation.math import Quaternion
from ..animation.motion import MotionClip
from ..animation.skeleton import Skeleton


def _load_skeleton() -> Skeleton:
    path = os.path.join(os.path.dirname(__file__), "..", "..", "..", "asset", "base_skeletons", "humanoid.json")
    with open(path) as f:
        return Skeleton.from_dict(json.load(f))


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

    def idx(sub: str) -> int:
        for n, name in enumerate(names):
            if sub in name:
                return n
        return -1

    hip = idx("hips")
    spine = idx("spine")
    chest = idx("chest")
    lul = idx("left_upper_leg")
    lll = idx("left_lower_leg")
    rul = idx("right_upper_leg")
    rll = idx("right_lower_leg")
    lua = idx("left_upper_arm")
    lla = idx("left_lower_arm")
    rua = idx("right_upper_arm")
    rla = idx("right_lower_arm")

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
            if lul >= 0:
                rots[lul] = Quaternion.from_axis_angle((1, 0, 0), swing)
            if rul >= 0:
                rots[rul] = Quaternion.from_axis_angle((1, 0, 0), -swing)
            if lll >= 0:
                rots[lll] = Quaternion.from_axis_angle((1, 0, 0), max(0.0, swing) * 0.4)
            if rll >= 0:
                rots[rll] = Quaternion.from_axis_angle((1, 0, 0), max(0.0, -swing) * 0.4)
            arm = -swing * 0.4
            if lua >= 0:
                rots[lua] = Quaternion.from_axis_angle((1, 0, 0), arm)
            if rua >= 0:
                rots[rua] = Quaternion.from_axis_angle((1, 0, 0), -arm)
            if hip >= 0:
                rots[hip] = Quaternion.from_axis_angle((0, 0, 1), math.sin(phase * speed) * 0.04)
            bounce = abs(math.sin(phase * speed)) * 0.04
            rp = (0.0, bounce, t * 1.5)
        elif motion_type == "run":
            speed = 2.5
            swing = math.sin(phase * speed) * 0.8
            if lul >= 0:
                rots[lul] = Quaternion.from_axis_angle((1, 0, 0), swing)
            if rul >= 0:
                rots[rul] = Quaternion.from_axis_angle((1, 0, 0), -swing)
            if lll >= 0:
                rots[lll] = Quaternion.from_axis_angle((1, 0, 0), max(0.0, swing) * 0.3)
            if rll >= 0:
                rots[rll] = Quaternion.from_axis_angle((1, 0, 0), max(0.0, -swing) * 0.3)
            arm = -swing * 0.5
            if lua >= 0:
                rots[lua] = Quaternion.from_axis_angle((1, 0, 0), arm)
            if rua >= 0:
                rots[rua] = Quaternion.from_axis_angle((1, 0, 0), -arm)
            if hip >= 0:
                rots[hip] = Quaternion.from_axis_angle((0, 0, 1), math.sin(phase * speed) * 0.06)
            bounce = abs(math.sin(phase * speed)) * 0.08
            rp = (0.0, bounce, t * 5.0)
        elif motion_type == "wave":
            if rua >= 0:
                rots[rua] = Quaternion.from_axis_angle((0, 0, 1), -1.2)
            if rla >= 0:
                rots[rla] = Quaternion.from_axis_angle((0, 0, 1), math.sin(phase * 3.0) * 0.8)
            if lua >= 0:
                rots[lua] = Quaternion.from_axis_angle((1, 0, 0), -0.3)
        else:
            breathe = math.sin(phase * 0.5) * 0.02
            if spine >= 0:
                rots[spine] = Quaternion.from_axis_angle((1, 0, 0), breathe)
            if chest >= 0:
                rots[chest] = Quaternion.from_axis_angle((1, 0, 0), breathe * 0.5)

        frames.append(rots)
        root_positions.append(rp)

    return MotionClip(
        skeleton=skeleton,
        frames=frames,
        root_positions=root_positions,
        fps=30.0,
        loop=motion_type != "wave",
    )
