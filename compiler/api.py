import json
import os
from typing import Any, Dict

from fastapi import FastAPI, HTTPException, WebSocket, WebSocketDisconnect
from pydantic import BaseModel

from core.animation.math import Quaternion
from core.animation.motion import MotionClip
from core.animation.retarget import build_bone_name_map, retarget_clip
from core.animation.skeleton import Pose, Skeleton
from core.jobs import JobQueue
from core.llm import LLMRouter, SYSTEM_PROMPT
from core.ml.model_registry import get_default_registry
from core.ml.pose_interpolation import procedural_interpolate
from core.ml.style_transfer import apply_style_transfer
from core.ml.text_to_mesh import generate_mesh
from core.ml.text_to_motion import generate_motion

app = FastAPI()

active_connections: list[WebSocket] = []

router = LLMRouter(system_prompt=SYSTEM_PROMPT, llm_endpoint="http://127.0.0.1:8082/v1/chat")


@app.websocket("/ws")
async def websocket_endpoint(ws: WebSocket):
    await ws.accept()
    active_connections.append(ws)
    router.register_ws(ws)
    try:
        while True:
            data = await ws.receive_text()
            msg = json.loads(data)
            if msg.get("type") == "chat":
                result = await router.process(msg)
                await ws.send_json({"type": "ChatReply", **result})
            else:
                job_request = msg.get("job_request")
                if job_request is None:
                    await ws.send_json({"type": "Error", "detail": "Missing job_request"})
                    continue
                job_id = await job_queue.enqueue(job_request["job_type"], job_request["params"])
                await ws.send_json({
                    "type": "JobUpdate",
                    "job": {"id": job_id, "status": "queued"},
                })
    except WebSocketDisconnect:
        try:
            active_connections.remove(ws)
        except ValueError:
            pass


@app.post("/chat")
async def chat_endpoint(req: dict):
    return await router.process(req)


@app.post("/jobs")
async def create_job(req: JobRequest):
    job_id = await job_queue.enqueue(req.job_type, req.params)
    return {"job_id": job_id, "status": "queued"}


@app.get("/jobs/{job_id}")
async def get_job(job_id: str):
    job = job_queue.get_job(job_id)
    if job is None:
        raise HTTPException(status_code=404, detail="Job not found")
    return {
        "id": job.id,
        "job_type": job.job_type,
        "status": job.status.value,
        "progress": job.progress,
        "error": job.error,
        "result": job.result,
    }


async def progress_callback(job_id: str, progress: float, message: str | None):
    for ws in active_connections:
        await ws.send_json({
            "type": "JobUpdate",
            "job": {"id": job_id, "progress": progress, "message": message},
        })


job_queue = JobQueue(progress_callback=progress_callback)
model_registry = get_default_registry()


def _pose_from_dict(data: dict) -> Pose:
    skel = Skeleton.from_dict(data["skeleton"])
    rotations = [Quaternion(q["w"], q["x"], q["y"], q["z"]) for q in data["joint_rotations"]]
    root = tuple(data.get("root_translation", [0.0, 0.0, 0.0]))
    return Pose(skeleton=skel, joint_rotations=rotations, root_translation=root)


def _load_target_skeleton() -> Skeleton:
    path = os.path.join(os.path.dirname(__file__), "..", "asset", "base_skeletons", "humanoid.json")
    with open(path) as f:
        return Skeleton.from_dict(json.load(f))


@app.post("/generate_character")
async def generate_character(params: Dict[str, Any]):
    _use, _endpoint = model_registry.resolve("text_to_mesh")
    prompt = params.get("prompt", "")
    seed = params.get("seed")
    mesh, skeleton_dict = generate_mesh(prompt, seed=seed)
    return {
        "status": "success",
        "mesh": mesh,
        "skeleton": skeleton_dict,
        "clip": {"frames": [], "fps": 30, "loop": True},
        "fallback_mode": _use == "fallback",
    }


@app.post("/generate_motion")
async def generate_motion(params: Dict[str, Any]):
    _use, _endpoint = model_registry.resolve("text_to_motion")
    prompt = params.get("prompt", "")
    seed = params.get("seed")
    clip = generate_motion(prompt, seed=seed)
    return {
        "status": "success",
        "clip": clip.to_dict(),
        "fallback_mode": _use == "fallback",
    }


@app.post("/stage_pose")
async def stage_pose(params: Dict[str, Any]):
    _use, _endpoint = model_registry.resolve("pose_interpolation")
    pose_a = _pose_from_dict(params["pose_a"])
    pose_b = _pose_from_dict(params["pose_b"])
    num_frames = params.get("num_frames", 60)
    easing = params.get("easing", "smoothstep")
    clip = procedural_interpolate(pose_a, pose_b, num_frames=num_frames, easing=easing)
    return {
        "status": "success",
        "clip": clip.to_dict(),
        "fallback_mode": _use == "fallback",
    }


@app.post("/style_transfer")
async def style_transfer_endpoint(params: Dict[str, Any]):
    _use, _endpoint = model_registry.resolve("style_transfer")
    clip_data = params.get("clip", {})
    style_prompt = params.get("style_prompt", "")
    source_clip = MotionClip.from_dict(clip_data) if clip_data else None
    if source_clip is None:
        return {"status": "error", "detail": "Missing clip data"}
    result = apply_style_transfer(source_clip, style_prompt)
    return {
        "status": "success",
        "clip": result.to_dict(),
        "fallback_mode": _use == "fallback",
    }


@app.post("/retarget")
async def retarget_animation(params: Dict[str, Any]):
    clip_data = params.get("clip", {})
    source_clip = MotionClip.from_dict(clip_data) if clip_data else None
    if source_clip is None:
        return {"status": "error", "detail": "Missing clip data"}
    target = _load_target_skeleton()
    known_maps = _load_bone_maps()
    source_name = params.get("source_name", "")
    if not source_name:
        src_names = [j.name.lower() for j in source_clip.skeleton.joints]
        for key in known_maps:
            known_joint = next(iter(known_maps[key].keys()), "").lower()
            if known_joint and any(known_joint[:4] in n for n in src_names):
                source_name = key
                break
    bone_map = build_bone_name_map(
        source_clip.skeleton, target,
        known_maps=known_maps, source_name=source_name,
    )
    result = retarget_clip(source_clip, target, bone_map)
    return {
        "status": "success",
        "clip": result.to_dict(),
        "source_name_detected": source_name,
    }


def _load_bone_maps() -> dict:
    maps_dir = os.path.join(os.path.dirname(__file__), "..", "asset", "mixamo_bone_maps")
    known = {}
    if not os.path.isdir(maps_dir):
        return known
    for fname in os.listdir(maps_dir):
        if fname.endswith(".json"):
            path = os.path.join(maps_dir, fname)
            with open(path) as f:
                data = json.load(f)
                known.update(data)
    return known


@app.post("/export")
async def export_asset(params: Dict[str, Any]):
    fmt = params.get("format", "glb")
    return {
        "status": "success",
        "format": fmt,
        "data": None,
        "message": f"Export to {fmt} requested",
    }
