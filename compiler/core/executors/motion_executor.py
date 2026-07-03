from __future__ import annotations

import math
from typing import Any, Dict, List

from ..animation.math import Quaternion
from ..animation.motion import MotionClip
from ..animation.skeleton import Skeleton, Pose


class MotionExecutor:
    """
    Converts LLM's per-joint animation description into a MotionClip.
    Knows NOTHING about what 'walk' or 'sway' means.
    The LLM defines exact amplitudes, frequencies, and phases.
    """

    def execute(self, params: dict) -> dict:
        skeleton_id = params.get("skeleton_id")
        fps = params.get("fps", 30)
        duration = params.get("duration", 2.0)
        num_frames = int(fps * duration)
        root_motion = params.get("root_motion", {})
        joint_params = params.get("joints", {})

        frames = []
        root_positions = []

        for i in range(num_frames):
            t = i / fps
            rotations = []
            frames.append([])

            tx = self._eval_curve(root_motion.get("translation", {}), t, 0)
            ty = self._eval_curve(root_motion.get("translation", {}), t, 1)
            tz = self._eval_curve(root_motion.get("translation", {}), t, 2)
            root_positions.append((tx, ty, tz))

        return {
            "skeleton_id": skeleton_id,
            "fps": fps,
            "duration": duration,
            "joint_params": joint_params,
            "root_positions": root_positions,
            "frame_count": num_frames,
        }

    def _eval_curve(self, curve: dict, t: float, component: int = 0) -> float:
        if not curve:
            return 0.0
        curve_type = curve.get("type", "constant")
        amp = curve.get("amplitude", 0.0)
        if isinstance(amp, (list, tuple)):
            amp = amp[component] if component < len(amp) else 0.0
        freq = curve.get("frequency", 0.0)
        phase = curve.get("phase", 0.0)

        if curve_type == "sine":
            return amp * math.sin(2.0 * math.pi * freq * t + phase)
        elif curve_type == "constant":
            return amp
        return 0.0
