from __future__ import annotations

from typing import Any, Dict, Optional


class SceneEntity:
    def __init__(self, entity_id: int, entity_type: str, data: Any, label: str = ""):
        self.id = entity_id
        self.entity_type = entity_type
        self.data = data
        self.label = label

    def to_dict(self) -> dict:
        return {
            "entity_id": self.id,
            "entity_type": self.entity_type,
            "label": self.label,
            "data": self.data if hasattr(self.data, "to_dict") else self._serialize(self.data),
        }

    def _serialize(self, obj: Any) -> Any:
        if hasattr(obj, "to_dict"):
            return obj.to_dict()
        if isinstance(obj, dict):
            return {k: self._serialize(v) for k, v in obj.items()}
        if isinstance(obj, list):
            return [self._serialize(v) for v in obj]
        return obj


class SceneManager:
    """
    Owns all entities in the scene. The LLM references entities by ID.
    The manager tracks what exists so the LLM can be told about it.
    """

    def __init__(self):
        self.entities: Dict[int, SceneEntity] = {}
        self._next_id: int = 1

    def add_entity(self, entity_type: str, data: Any, label: str = "") -> SceneEntity:
        entity_id = self._next_id
        self._next_id += 1
        if not label:
            label = f"{entity_type}_{entity_id}"
        entity = SceneEntity(entity_id, entity_type, data, label)
        self.entities[entity_id] = entity
        return entity

    def get_entity(self, entity_id: int) -> Optional[SceneEntity]:
        return self.entities.get(entity_id)

    def remove_entity(self, entity_id: int) -> bool:
        if entity_id in self.entities:
            del self.entities[entity_id]
            return True
        return False

    def clear(self):
        self.entities.clear()

    def get_entities_by_type(self, entity_type: str) -> list:
        return [e for e in self.entities.values() if e.entity_type == entity_type]

    def to_dict_list(self) -> list:
        return [e.to_dict() for e in self.entities.values()]
