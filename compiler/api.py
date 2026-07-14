import json
import os
from typing import Any, Dict, Optional

from fastapi import FastAPI, HTTPException, WebSocket, WebSocketDisconnect
from pydantic import BaseModel

from core.animation.math import Quaternion
from core.animation.motion import MotionClip
from core.animation.retarget import build_bone_name_map, retarget_clip
from core.animation.skeleton import Pose, Skeleton
from core.jobs import JobQueue
from core.llm.prompt import SYSTEM_PROMPT
from core.ml.model_registry import get_default_registry
from core.ml.generators.pose_interpolation import procedural_interpolate
from core.ml.generators.style_transfer import apply_style_transfer
from core.ml.generators.text_to_mesh import generate_mesh
from core.ml.generators.text_to_motion import generate_motion
from core.websocket.handlers import LLMRouter

class JobRequest(BaseModel):
    job_type: str
    params: Dict[str, Any]

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
                scene_state = msg.get("scene_state", [])
                if scene_state:
                    router.sync_scene(scene_state)
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
    mesh_data = params.get("mesh")
    skeleton_data = params.get("skeleton")
    if mesh_data is None:
        return {"status": "error", "detail": "Missing mesh data"}
    if fmt == "glb":
        glb_bytes = _build_glb(mesh_data, skeleton_data)
        import base64
        return {
            "status": "success",
            "format": "glb",
            "data": base64.b64encode(glb_bytes).decode("ascii"),
            "filename": params.get("filename", "export.glb"),
        }
    return {
        "status": "error",
        "format": fmt,
        "data": None,
        "message": f"Export format {fmt} not supported on server",
    }


def _build_glb(mesh: dict, skeleton: Optional[dict] = None) -> bytes:
    import struct
    vertices = mesh.get("vertices", [])
    indices = mesh.get("indices", [])
    if not vertices or not indices:
        return b""

    v_count = len(vertices)
    i_count = len(indices)
    pos = []
    nrm = []
    for v in vertices:
        p = v.get("position", [0, 0, 0])
        n = v.get("normal", [0, 0, 1])
        pos.extend([float(p[0]), float(p[1]), float(p[2])])
        nrm.extend([float(n[0]), float(n[1]), float(n[2])])

    vert_bytes = struct.pack(f"<{len(pos)}f", *pos)
    norm_bytes = struct.pack(f"<{len(nrm)}f", *nrm)
    idx_bytes = struct.pack(f"<{i_count}I", *[int(i) for i in indices])

    VERT_OFF = 0
    NORM_OFF = len(vert_bytes)
    IDX_OFF = NORM_OFF + len(norm_bytes)
    total_len = IDX_OFF + len(idx_bytes)
    pad = (4 - total_len % 4) % 4
    total_len += pad

    vertex_count = v_count
    component_type_float = 5126
    component_type_uint = 5125

    accessors = [
        {"componentType": component_type_float, "count": vertex_count, "type": "VEC3", "byteOffset": VERT_OFF},
        {"componentType": component_type_float, "count": vertex_count, "type": "VEC3", "byteOffset": NORM_OFF},
        {"componentType": component_type_uint,  "count": i_count,      "type": "SCALAR", "byteOffset": IDX_OFF},
    ]

    accessor_idx = 0
    json_model: dict = {
        "asset": {"version": "2.0"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0}],
        "meshes": [{
            "primitives": [{
                "attributes": {
                    "POSITION": accessor_idx,
                    "NORMAL": accessor_idx + 1,
                },
                "indices": accessor_idx + 2,
            }]
        }],
        "accessors": [],
        "bufferViews": [],
        "buffers": [{"byteLength": total_len, "uri": "data:application/octet-stream;base64,"}],
    }

    for acc in accessors:
        json_model["accessors"].append({
            "bufferView": len(json_model["bufferViews"]),
            "componentType": acc["componentType"],
            "count": acc["count"],
            "type": acc["type"],
            "byteOffset": 0,
        })
        json_model["bufferViews"].append({
            "buffer": 0,
            "byteOffset": acc["byteOffset"],
            "byteLength": acc["count"] * (4 if acc["componentType"] == component_type_float else 4),
        })

    json_str = json.dumps(json_model, separators=(",", ":"))
    while len(json_str) % 4:
        json_str += " "
    json_bytes = json_str.encode("utf-8")

    header = struct.pack("<I", 0x46546C67)
    header += struct.pack("<I", 2)
    header += struct.pack("<I", 12 + 8 + len(json_bytes) + 8 + total_len)

    json_chunk_len = struct.pack("<I", len(json_bytes))
    json_chunk_type = struct.pack("<I", 0x4E4F534A)
    bin_chunk_len = struct.pack("<I", total_len)
    bin_chunk_type = struct.pack("<I", 0x004E4942)

    glb = header + json_chunk_len + json_chunk_type + json_bytes
    glb += bin_chunk_len + bin_chunk_type + vert_bytes + norm_bytes + idx_bytes
    glb += b"\x00" * pad
    return glb
