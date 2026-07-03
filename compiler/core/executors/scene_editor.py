from __future__ import annotations

from typing import Any, Dict


class SceneEditor:
    """
    Applies scene modifications from the LLM's structured params.
    Knows NOTHING about 'sunset' or 'sunrise'. The LLM defines exact values.
    """

    def execute(self, params: dict) -> dict:
        result = {}

        lighting = params.get("lighting")
        if lighting:
            result["lighting"] = {
                "type": lighting.get("type", "directional"),
                "direction": lighting.get("direction", [0.3, -1.0, 0.3]),
                "position": lighting.get("position", [0.0, 5.0, 0.0]),
                "color": lighting.get("color", [1.0, 1.0, 1.0]),
                "intensity": lighting.get("intensity", 1.0),
                "ambient": lighting.get("ambient", [0.1, 0.1, 0.1]),
            }

        transforms = params.get("entity_transforms")
        if transforms:
            result["entity_transforms"] = transforms

        materials = params.get("materials")
        if materials:
            result["materials"] = materials

        if params.get("clear_scene", False):
            result["clear_scene"] = True

        return result
