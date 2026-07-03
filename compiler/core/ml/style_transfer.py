from __future__ import annotations

import math
import random
from typing import Optional

from ..animation.math import Quaternion
from ..animation.motion import MotionClip


def _apply_presets(
    clip: MotionClip, style: str
) -> tuple[float, float, float, float]:
    speed = 1.0
    amplitude = 1.0
    noise = 0.0
    root_bounce = 0.0

    s = style.lower()
    if "bouncy" in s or "springy" in s:
        amplitude = 1.3
        root_bounce = 0.06
    elif "sneaky" in s or "stealth" in s or "creep" in s:
        speed = 0.5
        amplitude = 0.6
        root_bounce = -0.03
    elif "robotic" in s or "robot" in s or "stiff" in s:
        amplitude = 1.0
        root_bounce = 0.0
    elif "exaggerat" in s or "cartoon" in s or "overact" in s:
        amplitude = 1.8
        noise = 0.02
    elif "subtle" in s or "gentle" in s or "minimal" in s:
        amplitude = 0.4
        speed = 0.8
    elif "heavy" in s or "stomp" in s or "weighted" in s:
        amplitude = 1.2
        root_bounce = -0.04
    elif "float" in s or "light" in s or "airy" in s:
        amplitude = 0.7
        root_bounce = 0.04
    elif "sad" in s or "tired" in s or "depressed" in s:
        speed = 0.6
        amplitude = 0.7
        root_bounce = -0.05
    elif "happy" in s or "excited" in s or "energetic" in s:
        speed = 1.4
        amplitude = 1.3
        root_bounce = 0.04
        noise = 0.03

    return speed, amplitude, noise, root_bounce


def apply_style_transfer(
    source_clip: MotionClip,
    style_prompt: str,
    style_reference: Optional[MotionClip] = None,
) -> MotionClip:
    speed, amplitude, noise, root_bounce = _apply_presets(source_clip, style_prompt)

    new_frames = []
    new_root_positions = []
    jc = source_clip.skeleton.joint_count()

    for i, frame in enumerate(source_clip.frames):
        t = i / max(len(source_clip.frames) - 1, 1)

        time_scaled = i * speed
        src_idx = min(int(time_scaled), len(source_clip.frames) - 1)
        frac = time_scaled - src_idx
        src_idx = max(0, min(src_idx, len(source_clip.frames) - 2))

        new_rots = []
        for j in range(jc):
            if speed != 1.0 and src_idx < len(source_clip.frames) - 1:
                q = source_clip.frames[src_idx][j].slerp(
                    source_clip.frames[src_idx + 1][j], frac
                )
            else:
                q = frame[j]

            if amplitude != 1.0:
                angle = 2.0 * math.acos(max(-1.0, min(1.0, q.w)))
                if angle > 1e-6:
                    s = math.sin(angle * 0.5)
                    ax = q.x / s if s > 1e-6 else 0.0
                    ay = q.y / s if s > 1e-6 else 0.0
                    az = q.z / s if s > 1e-6 else 0.0
                    new_angle = angle * amplitude
                    q = Quaternion.from_axis_angle(
                        (ax, ay, az), new_angle
                    )

            if noise > 0:
                jitter = Quaternion.from_axis_angle(
                    (random.gauss(0, 1), random.gauss(0, 1), random.gauss(0, 1)),
                    noise,
                )
                q = (jitter * q).normalize()

            new_rots.append(q)
        new_frames.append(new_rots)

        if source_clip.root_positions:
            rp = source_clip.root_positions[min(i, len(source_clip.root_positions) - 1)]
            new_rp = (
                rp[0],
                rp[1] + root_bounce * math.sin(math.pi * t),
                rp[2] * speed,
            )
            new_root_positions.append(new_rp)
        else:
            new_root_positions.append((0.0, 0.0, 0.0))

    return MotionClip(
        skeleton=source_clip.skeleton,
        frames=new_frames,
        root_positions=new_root_positions,
        fps=source_clip.fps,
        loop=source_clip.loop,
    )
