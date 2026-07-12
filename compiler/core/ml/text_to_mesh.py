from __future__ import annotations

import json
import math
import os
from typing import Any, Dict, Optional, Tuple

from ..animation.skeleton import Skeleton


def _load_skeleton() -> Skeleton:
    path = os.path.join(os.path.dirname(__file__), "..", "..", "..", "asset", "base_skeletons", "humanoid.json")
    with open(path) as f:
        return Skeleton.from_dict(json.load(f))


def _world_positions(skeleton: Skeleton) -> list[tuple[float, float, float]]:
    positions = [(0.0, 0.0, 0.0)] * skeleton.joint_count()
    for i, joint in enumerate(skeleton.joints):
        t = joint.local_transform.translation
        p = joint.parent_index
        if p < 0:
            positions[i] = t
        else:
            px, py, pz = positions[p]
            positions[i] = (px + t[0], py + t[1], pz + t[2])
    return positions


def _make_cylinder(
    p1: tuple, p2: tuple, radius: float, bone_idx: int, rings: int = 8
) -> tuple[list[dict], list[int]]:
    dx, dy, dz = p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]
    length = math.sqrt(dx * dx + dy * dy + dz * dz)
    if length < 1e-6:
        return [], []
    ax, ay, az = dx / length, dy / length, dz / length

    if abs(ax) < 0.9:
        ux, uy, uz = 1.0, 0.0, 0.0
    else:
        ux, uy, uz = 0.0, 1.0, 0.0
    rx = ay * uz - az * uy
    ry = az * ux - ax * uz
    rz = ax * uy - ay * ux
    rl = math.sqrt(rx * rx + ry * ry + rz * rz)
    rx /= rl
    ry /= rl
    rz /= rl
    sx = ay * rz - az * ry
    sy = az * rx - ax * rz
    sz = ax * ry - ay * rx

    verts = []
    for ring_t in (0.0, 1.0):
        cx = p1[0] + dx * ring_t
        cy = p1[1] + dy * ring_t
        cz = p1[2] + dz * ring_t
        for j in range(rings):
            theta = 2.0 * math.pi * j / rings
            ct, st = math.cos(theta), math.sin(theta)
            vx = cx + radius * (rx * ct + sx * st)
            vy = cy + radius * (ry * ct + sy * st)
            vz = cz + radius * (rz * ct + sz * st)
            nx, ny, nz = vx - cx, vy - cy, vz - cz
            nl = math.sqrt(nx * nx + ny * ny + nz * nz)
            verts.append({
                "position": [vx, vy, vz],
                "normal": [nx / nl, ny / nl, nz / nl],
                "uv": [j / rings, ring_t],
                "bone_weights": [1.0, 0.0, 0.0, 0.0],
                "bone_indices": [bone_idx, 0, 0, 0],
            })

    idxs = []
    for j in range(rings):
        a, b = j, (j + 1) % rings
        a2, b2 = a + rings, b + rings
        idxs.extend([a, b, a2, b, b2, a2])

    return verts, idxs


def _make_sphere(
    center: tuple, radius: float, bone_idx: int, stacks: int = 6, slices: int = 8
) -> tuple[list[dict], list[int]]:
    cx, cy, cz = center
    verts = []
    for i in range(stacks + 1):
        phi = math.pi * i / stacks
        for j in range(slices):
            theta = 2.0 * math.pi * j / slices
            x = radius * math.sin(phi) * math.cos(theta)
            y = radius * math.cos(phi)
            z = radius * math.sin(phi) * math.sin(theta)
            nl = math.sqrt(x * x + y * y + z * z)
            verts.append({
                "position": [cx + x, cy + y, cz + z],
                "normal": [x / nl, y / nl, z / nl],
                "uv": [j / slices, i / stacks],
                "bone_weights": [1.0, 0.0, 0.0, 0.0],
                "bone_indices": [bone_idx, 0, 0, 0],
            })

    idxs = []
    for i in range(stacks):
        for j in range(slices):
            a = i * slices + j
            b = a + slices
            aj = (j + 1) % slices
            a_next = i * slices + aj
            b_next = (i + 1) * slices + aj
            idxs.extend([a, a_next, b, a_next, b_next, b])

    return verts, idxs


def _primitive_from_prompt(prompt: str) -> dict | None:
    prompt_lower = prompt.lower()

    def _box(w, h, d):
        hw, hh, hd = w / 2, h / 2, d / 2
        verts = [
            {"position": [-hw, -hh, -hd], "normal": [0, 0, -1], "uv": [0, 0], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [ hw, -hh, -hd], "normal": [0, 0, -1], "uv": [1, 0], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [ hw,  hh, -hd], "normal": [0, 0, -1], "uv": [1, 1], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [-hw,  hh, -hd], "normal": [0, 0, -1], "uv": [0, 1], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [-hw, -hh,  hd], "normal": [0, 0, 1],  "uv": [0, 0], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [ hw, -hh,  hd], "normal": [0, 0, 1],  "uv": [1, 0], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [ hw,  hh,  hd], "normal": [0, 0, 1],  "uv": [1, 1], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [-hw,  hh,  hd], "normal": [0, 0, 1],  "uv": [0, 1], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [-hw, -hh, -hd], "normal": [0, -1, 0], "uv": [0, 0], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [ hw, -hh, -hd], "normal": [0, -1, 0], "uv": [1, 0], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [ hw, -hh,  hd], "normal": [0, -1, 0], "uv": [1, 1], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [-hw, -hh,  hd], "normal": [0, -1, 0], "uv": [0, 1], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [-hw,  hh, -hd], "normal": [0, 1, 0],  "uv": [0, 0], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [ hw,  hh, -hd], "normal": [0, 1, 0],  "uv": [1, 0], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [ hw,  hh,  hd], "normal": [0, 1, 0],  "uv": [1, 1], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [-hw,  hh,  hd], "normal": [0, 1, 0],  "uv": [0, 1], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [-hw, -hh, -hd], "normal": [-1, 0, 0], "uv": [0, 0], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [-hw, -hh,  hd], "normal": [-1, 0, 0], "uv": [1, 0], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [-hw,  hh,  hd], "normal": [-1, 0, 0], "uv": [1, 1], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [-hw,  hh, -hd], "normal": [-1, 0, 0], "uv": [0, 1], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [ hw, -hh, -hd], "normal": [1, 0, 0],  "uv": [0, 0], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [ hw, -hh,  hd], "normal": [1, 0, 0],  "uv": [1, 0], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [ hw,  hh,  hd], "normal": [1, 0, 0],  "uv": [1, 1], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
            {"position": [ hw,  hh, -hd], "normal": [1, 0, 0],  "uv": [0, 1], "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]},
        ]
        faces = [0,1,2, 0,2,3, 4,6,5, 4,7,6, 8,10,9, 8,11,10, 12,13,14, 12,14,15, 16,17,18, 16,18,19, 20,23,22, 20,22,21]
        return verts, faces

    if "sphere" in prompt_lower or "ball" in prompt_lower or "orb" in prompt_lower:
        return {"vertices": _make_sphere((0, 0, 0), 0.5, 0)[0], "indices": _make_sphere((0, 0, 0), 0.5, 0)[1]}
    if "cube" in prompt_lower or "box" in prompt_lower or "block" in prompt_lower:
        v, i = _box(1.0, 1.0, 1.0)
        return {"vertices": v, "indices": i}
    if "plane" in prompt_lower or "flat" in prompt_lower or "ground" in prompt_lower or "floor" in prompt_lower:
        v, i = _box(2.0, 0.05, 2.0)
        return {"vertices": v, "indices": i}
    if "humanoid" in prompt_lower or "human" in prompt_lower or "character" in prompt_lower or "person" in prompt_lower:
        return None
    return None


def generate_mesh(
    prompt: str,
    seed: Optional[int] = None,
) -> Tuple[dict, dict]:
    primitive = _primitive_from_prompt(prompt)
    if primitive is not None:
        return primitive, {}

    skeleton = _load_skeleton()
    wp = _world_positions(skeleton)
    names = [j.name.lower() for j in skeleton.joints]

    def idx(sub: str) -> int:
        for n, name in enumerate(names):
            if sub in name:
                return n
        return -1

    all_verts: list[dict] = []
    all_idxs: list[int] = []
    offset = 0

    body_parts = [
        {"kind": "sphere", "i": idx("head"), "r": 0.12},
        {"kind": "cyl", "a": idx("neck"), "b": idx("head"), "r": 0.08},
        {"kind": "cyl", "a": idx("chest"), "b": idx("neck"), "r": 0.14},
        {"kind": "cyl", "a": idx("spine"), "b": idx("chest"), "r": 0.13},
        {"kind": "cyl", "a": idx("hips"), "b": idx("spine"), "r": 0.15},
        {"kind": "cyl", "a": idx("left_upper_arm"), "b": idx("left_lower_arm"), "r": 0.06},
        {"kind": "cyl", "a": idx("left_lower_arm"), "b": idx("left_hand"), "r": 0.05},
        {"kind": "cyl", "a": idx("right_upper_arm"), "b": idx("right_lower_arm"), "r": 0.06},
        {"kind": "cyl", "a": idx("right_lower_arm"), "b": idx("right_hand"), "r": 0.05},
        {"kind": "cyl", "a": idx("left_upper_leg"), "b": idx("left_lower_leg"), "r": 0.09},
        {"kind": "cyl", "a": idx("left_lower_leg"), "b": idx("left_foot"), "r": 0.07},
        {"kind": "cyl", "a": idx("right_upper_leg"), "b": idx("right_lower_leg"), "r": 0.09},
        {"kind": "cyl", "a": idx("right_lower_leg"), "b": idx("right_foot"), "r": 0.07},
    ]

    for part in body_parts:
        if part["kind"] == "sphere":
            bi = part["i"]
            if bi < 0:
                continue
            v, idxs = _make_sphere(wp[bi], part["r"], bi)
        else:
            bi1, bi2 = part["a"], part["b"]
            if bi1 < 0 or bi2 < 0 or bi1 >= len(wp) or bi2 >= len(wp):
                continue
            v, idxs = _make_cylinder(wp[bi1], wp[bi2], part["r"], bi1)
        all_verts.extend(v)
        all_idxs.extend([i + offset for i in idxs])
        offset += len(v)

    mesh = {"vertices": all_verts, "indices": all_idxs}
    return mesh, skeleton.to_dict()
