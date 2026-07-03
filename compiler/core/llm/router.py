from __future__ import annotations

import json
import asyncio
from typing import Any, Dict, List, Optional

from ..scene_manager import SceneManager
from ..executors.skeleton_executor import SkeletonExecutor
from ..executors.mesh_executor import MeshExecutor
from ..executors.motion_executor import MotionExecutor
from ..executors.texture_executor import TextureExecutor
from ..executors.scene_editor import SceneEditor


class LLMClient:
    """Calls any OpenAI-compatible LLM endpoint."""

    def __init__(self, endpoint: str = "http://127.0.0.1:8082/v1/chat"):
        self.endpoint = endpoint

    async def chat(self, system: str, messages: List[dict]) -> dict:
        import aiohttp
        payload = {
            "model": "muse-llm",
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

        self.executors = {
            "generate_skeleton": SkeletonExecutor(),
            "generate_mesh": MeshExecutor(),
            "generate_motion": MotionExecutor(),
            "generate_texture": TextureExecutor(),
            "edit_scene": SceneEditor(),
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
        messages = request.get("messages", [])

        scene_context = self._format_scene_context()
        full_system = self.system_prompt.replace("{scene_context}", scene_context)

        llm_output = await self.llm.chat(full_system, messages)

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

        return {
            "reply": reply,
            "actions": action_results,
            "entities": new_entities,
        }

    def _format_scene_context(self) -> str:
        if not self.scene.entities:
            return "The scene is empty."
        lines = []
        for eid, entity in self.scene.entities.items():
            lines.append(f"- Entity {eid}: type={entity.entity_type}, label='{entity.label}'")
        return "\n".join(lines)
