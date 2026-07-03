from __future__ import annotations

from typing import Any, Dict


class MaterialExecutor:
    def execute(self, params: Dict[str, Any]) -> dict:
        entity_id = params.get("entity_id")
        color = params.get("color", [0.8, 0.8, 0.8])
        metallic = params.get("metallic", 0.0)
        roughness = params.get("roughness", 0.5)

        return {
            "entity_id": entity_id,
            "material": {
                "albedo": color,
                "metallic": metallic,
                "roughness": roughness,
            },
        }
