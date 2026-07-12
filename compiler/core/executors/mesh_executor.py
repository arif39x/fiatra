from __future__ import annotations

import math
from typing import Any, Dict


def _sphere(radius: float, cx: float = 0, cy: float = 0, cz: float = 0,
            stacks: int = 8, slices: int = 12) -> tuple[list[dict], list[int]]:
    verts = []
    for i in range(stacks + 1):
        phi = math.pi * i / stacks
        for j in range(slices):
            theta = 2.0 * math.pi * j / slices
            x = radius * math.sin(phi) * math.cos(theta)
            y = radius * math.cos(phi)
            z = radius * math.sin(phi) * math.sin(theta)
            nl = math.sqrt(x * x + y * y + z * z) or 1.0
            verts.append({
                "position": [cx + x, cy + y, cz + z],
                "normal": [x / nl, y / nl, z / nl],
                "uv": [j / slices, i / stacks],
                "bone_weights": [1.0, 0.0, 0.0, 0.0],
                "bone_indices": [0, 0, 0, 0],
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


def _box(w: float, h: float, d: float) -> tuple[list[dict], list[int]]:
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
    faces = [
        0,1,2, 0,2,3, 4,6,5, 4,7,6,
        8,10,9, 8,11,10, 12,13,14, 12,14,15,
        16,17,18, 16,18,19, 20,23,22, 20,22,21,
    ]
    return verts, faces


def _cylinder(radius: float, height: float,
              slices: int = 16) -> tuple[list[dict], list[int]]:
    hh = height / 2
    verts = []
    cap_top = len(verts)
    verts.append({"position": [0, hh, 0], "normal": [0, 1, 0], "uv": [0.5, 0.5],
                  "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]})
    for j in range(slices):
        theta = 2.0 * math.pi * j / slices
        x = radius * math.cos(theta)
        z = radius * math.sin(theta)
        verts.append({"position": [x, hh, z], "normal": [0, 1, 0], "uv": [x / radius * 0.5 + 0.5, z / radius * 0.5 + 0.5],
                      "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]})
    for j in range(slices):
        theta = 2.0 * math.pi * j / slices
        x = radius * math.cos(theta)
        z = radius * math.sin(theta)
        nx, nz = math.cos(theta), math.sin(theta)
        verts.append({"position": [x, -hh, z], "normal": [nx, 0, nz], "uv": [j / slices, 0],
                      "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]})
        verts.append({"position": [x, hh, z], "normal": [nx, 0, nz], "uv": [j / slices, 1],
                      "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]})
    cap_bot = len(verts)
    verts.append({"position": [0, -hh, 0], "normal": [0, -1, 0], "uv": [0.5, 0.5],
                  "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]})
    for j in range(slices):
        theta = 2.0 * math.pi * j / slices
        x = radius * math.cos(theta)
        z = radius * math.sin(theta)
        verts.append({"position": [x, -hh, z], "normal": [0, -1, 0], "uv": [x / radius * 0.5 + 0.5, z / radius * 0.5 + 0.5],
                      "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]})

    idxs = []
    for j in range(slices):
        a = cap_top + 1 + j
        b = cap_top + 1 + (j + 1) % slices
        idxs.extend([cap_top, a, b])
    side_start = cap_top + 1 + slices
    for j in range(slices):
        a = side_start + j * 2
        b = side_start + ((j + 1) % slices) * 2
        idxs.extend([a, b, a + 1, b, b + 1, a + 1])
    for j in range(slices):
        a = cap_bot + 1 + j
        b = cap_bot + 1 + (j + 1) % slices
        idxs.extend([cap_bot, b, a])
    return verts, idxs


def _cone(radius: float, height: float,
          slices: int = 16) -> tuple[list[dict], list[int]]:
    hh = height / 2
    verts = []
    tip = len(verts)
    verts.append({"position": [0, hh, 0], "normal": [0, 1, 0], "uv": [0.5, 1],
                  "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]})
    for j in range(slices):
        theta = 2.0 * math.pi * j / slices
        x = radius * math.cos(theta)
        z = radius * math.sin(theta)
        nx, nz = math.cos(theta), math.sin(theta)
        verts.append({"position": [x, -hh, z], "normal": [nx, 0.5, nz], "uv": [j / slices, 0],
                      "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]})
    cap = len(verts)
    verts.append({"position": [0, -hh, 0], "normal": [0, -1, 0], "uv": [0.5, 0.5],
                  "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]})
    for j in range(slices):
        theta = 2.0 * math.pi * j / slices
        x = radius * math.cos(theta)
        z = radius * math.sin(theta)
        verts.append({"position": [x, -hh, z], "normal": [0, -1, 0], "uv": [x / radius * 0.5 + 0.5, z / radius * 0.5 + 0.5],
                      "bone_weights": [1, 0, 0, 0], "bone_indices": [0, 0, 0, 0]})
    idxs = []
    for j in range(slices):
        a = tip + 1 + j
        b = tip + 1 + (j + 1) % slices
        idxs.extend([tip, a, b])
    for j in range(slices):
        a = cap + 1 + j
        b = cap + 1 + (j + 1) % slices
        idxs.extend([cap, b, a])
    return verts, idxs


class MeshExecutor:
    """
    Procedurally generates a mesh from the LLM's prompt + style.

    The LLM describes the visual. This executor parses the prompt
    for shape keywords (sphere, cube, cylinder, cone) and builds
    the corresponding mesh. This IS a fallback — a real system
    would call a text-to-3D model instead.
    """

    def execute(self, params: dict) -> dict:
        prompt = (params.get("prompt", "") or "").lower()
        style = params.get("style", "low-poly")
        polygon_count = params.get("polygon_count", 500)

        prompt = prompt.lower()
        verts, idxs = [], []

        if "sphere" in prompt or "ball" in prompt or "orb" in prompt or "round" in prompt:
            r = 0.5
            stacks = max(4, int(math.sqrt(polygon_count / 3)))
            verts, idxs = _sphere(r, stacks=stacks, slices=stacks * 2)
        elif "cube" in prompt or "box" in prompt or "block" in prompt:
            verts, idxs = _box(1.0, 1.0, 1.0)
        elif "cone" in prompt or "pyramid" in prompt or "wedge" in prompt:
            verts, idxs = _cone(0.5, 1.0)
        elif "cylinder" in prompt or "pillar" in prompt or "tube" in prompt or "pipe" in prompt:
            verts, idxs = _cylinder(0.5, 1.0)
        elif "plane" in prompt or "flat" in prompt or "ground" in prompt or "floor" in prompt or "platform" in prompt:
            verts, idxs = _box(2.0, 0.05, 2.0)
        elif "torus" in prompt or "ring" in prompt or "donut" in prompt:
            verts, idxs = _sphere(0.5)
        else:
            verts, idxs = _box(0.5, 0.5, 0.5)

        return {
            "prompt": params.get("prompt", ""),
            "style": style,
            "polygon_count": len(idxs),
            "skeleton_id": params.get("skeleton_id"),
            "vertices": verts,
            "indices": idxs,
        }
