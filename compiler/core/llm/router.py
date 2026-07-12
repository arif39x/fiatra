from __future__ import annotations

import json
import asyncio
from typing import Any, Dict, List, Optional

from ..scene_manager import SceneManager
from ..executors.material_executor import MaterialExecutor
from ..executors.mesh_executor import MeshExecutor
from ..executors.motion_executor import MotionExecutor
from ..executors.primitive_executor import PrimitiveExecutor
from ..executors.scene_editor import SceneEditor
from ..executors.skeleton_executor import SkeletonExecutor
from ..executors.texture_executor import TextureExecutor


class LLMClient:
    """Calls any OpenAI-compatible LLM endpoint."""

    def __init__(self, endpoint: str = "http://127.0.0.1:8082/v1/chat"):
        self.endpoint = endpoint

    async def chat(self, system: str, messages: List[dict]) -> dict:
        import aiohttp
        payload = {
            "model": "initial-llm",
            "system": system,
            "messages": messages,
            "response_format": {"type": "json_object"},
            "temperature": 0.2,
        }
        async with aiohttp.ClientSession() as session:
            async with session.post(self.endpoint, json=payload) as resp:
                result = await resp.json()
                content = result["choices"][0]["message"]["content"]
                content = content.strip()
                if content.startswith("```"):
                    content = content.split("\n", 1)[1]
                    content = content.rsplit("```", 1)[0]
                return json.loads(content)


class LLMRouter:
    """
    One router. One prompt. No hardcoded knowledge.

    The system prompt teaches the LLM what output schemas are available.
    The LLM decides everything. The executors just validate and convert.
    """

    def __init__(self, system_prompt: str, llm_endpoint: str):
        self.system_prompt = system_prompt
        self.llm = LLMClient(llm_endpoint)
        self.scene = SceneManager()
        self.active_ws: List[Any] = []
        self.message_history: List[dict] = []

        self.executors = {
            "generate_skeleton": SkeletonExecutor(),
            "generate_mesh": MeshExecutor(),
            "generate_motion": MotionExecutor(),
            "generate_texture": TextureExecutor(),
            "edit_scene": SceneEditor(),
            "create_primitive": PrimitiveExecutor(),
            "assign_material": MaterialExecutor(),
        }

    def register_ws(self, ws):
        self.active_ws.append(ws)

    async def broadcast(self, msg: dict):
        for ws in self.active_ws:
            try:
                await ws.send_json(msg)
            except Exception:
                self.active_ws.remove(ws)

    async def process(self, request: dict) -> dict:
        user_message = request.get("user_message", "")
        if user_message:
            self.message_history.append({"role": "user", "content": user_message})

        scene_context = self._format_scene_context()
        full_system = self.system_prompt.replace("{scene_context}", scene_context)

        llm_output = await self.llm.chat(full_system, self.message_history)

        reply = llm_output.get("reply", "")
        actions = llm_output.get("actions", [])
        new_entities = []
        action_results = []

        for action in actions:
            action_type = action.get("type", "")
            params = action.get("params", {})

            await self.broadcast({
                "type": "Progress",
                "action": action_type,
                "progress": 0.0,
                "message": f"Starting {action_type}..."
            })

            executor = self.executors.get(action_type)
            if executor is None:
                await self.broadcast({"type": "Progress", "action": action_type, "progress": 0.0, "message": f"Unknown action: {action_type}"})
                continue

            try:
                result = executor.execute(params)
            except Exception as e:
                await self.broadcast({"type": "Progress", "action": action_type, "progress": 0.0, "message": f"Error: {e}"})
                continue

            entity = self.scene.add_entity(action_type, result)
            new_entities.append(entity.to_dict())
            action_results.append({
                "action_type": action_type,
                "status": "done",
                "entity_id": entity.id,
            })

            await self.broadcast({
                "type": "Progress",
                "action": action_type,
                "progress": 1.0,
                "entity": entity.to_dict(),
            })

        self.message_history.append({"role": "assistant", "content": reply})

        return {
            "reply": reply,
            "actions": action_results,
            "entities": new_entities,
        }

    def sync_scene(self, entities: list):
        for ent in entities:
            eid = ent.get("entity_id")
            if eid is None:
                eid = self.scene._next_id
                self.scene._next_id += 1
            else:
                if eid >= self.scene._next_id:
                    self.scene._next_id = eid + 1
            label = ent.get("label", "")
            entity_type = ent.get("entity_type", "primitive")
            data = {
                "position": ent.get("position", [0.0, 0.0, 0.0]),
                "rotation": ent.get("rotation", [0.0, 0.0, 0.0]),
                "scale": ent.get("scale", [1.0, 1.0, 1.0]),
                "color": ent.get("color", [0.8, 0.8, 0.8]),
            }
            from ..scene_manager import SceneEntity
            existing = self.scene.entities.get(eid)
            if existing is None:
                self.scene.entities[eid] = SceneEntity(eid, entity_type, data, label)
            else:
                existing.label = label
                existing.entity_type = entity_type
                if isinstance(existing.data, dict):
                    existing.data.update(data)

    def _format_scene_context(self) -> str:
        if not self.scene.entities:
            return "The scene is empty."
        lines = []
        for eid, entity in self.scene.entities.items():
            data = entity.data
            if isinstance(data, dict):
                pos = data.get("position", data.get("translation", "?"))
                color = data.get("material", {}).get("albedo", "?") if isinstance(data.get("material"), dict) else "?"
                lines.append(f"- Entity {eid}: type={entity.entity_type}, label='{entity.label}', pos={pos}, color={color}")
            else:
                lines.append(f"- Entity {eid}: type={entity.entity_type}, label='{entity.label}'")
        return "\n".join(lines)
