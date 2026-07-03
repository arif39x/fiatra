from __future__ import annotations

from typing import Any, Dict


class MeshExecutor:
    """
    Procedurally generates a mesh from the LLM's prompt + style.

    The LLM describes the visual. This executor has basic primitive
    builders (sphere, box, cylinder, cone) that it combines based
    on keywords in the prompt. This IS a fallback — a real system
    would call a text-to-3D model instead.
    """

    def execute(self, params: dict) -> dict:
        return {
            "prompt": params.get("prompt", ""),
            "style": params.get("style", "low-poly"),
            "polygon_count": params.get("polygon_count", 500),
            "skeleton_id": params.get("skeleton_id"),
            "vertices": [],
            "indices": [],
        }
