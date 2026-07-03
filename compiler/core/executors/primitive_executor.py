from __future__ import annotations

from typing import Any, Dict


class PrimitiveExecutor:
    def execute(self, params: Dict[str, Any]) -> dict:
        primitive = params.get("primitive", "cube")
        position = params.get("position", [0.0, 0.0, 0.0])
        rotation = params.get("rotation", [0.0, 0.0, 0.0])
        scale = params.get("scale", [1.0, 1.0, 1.0])
        color = params.get("color", [0.8, 0.8, 0.8])
        metallic = params.get("metallic", 0.0)
        roughness = params.get("roughness", 0.5)

        return {
            "primitive": primitive,
            "position": position,
            "rotation": rotation,
            "scale": scale,
            "material": {
                "albedo": color,
                "metallic": metallic,
                "roughness": roughness,
            },
        }
