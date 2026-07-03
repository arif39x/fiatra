<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="asset/logo.jpg">
    <img alt="Fiatra logo" src="asset/logo.jpg" width="160">
  </picture>
</p>

<h1 align="center">Fiatra</h1>

<p align="center">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-1.75+-DEA584?logo=rust">
  <img alt="Python" src="https://img.shields.io/badge/Python-3.11+-3776AB?logo=python">
  <img alt="WebGPU" src="https://img.shields.io/badge/WebGPU-WGSL-FF6B35">
  <img alt="License" src="https://img.shields.io/badge/license-MIT-green">
  <img alt="Status" src="https://img.shields.io/badge/status-alpha-orange">
</p>

**Fiatra** is an AI Character & Animation Studio — a desktop application that takes text prompts and produces rigged, animated, exportable 3D characters, all with a real-time WGPU preview.

---

## Features

1. **Text-to-Character** — prompt → rigged, skinned mesh + base idle animation
2. **Text-to-Motion** — prompt → BVH/animation clip on the character's skeleton
3. **AI Pose Staging** — two hand-set poses + prompt → generated in-between frames
4. **Style Transfer** — existing animation + style prompt → restyled animation
5. **Auto-Retarget** — external animation (Mixamo/asset-store) → adapted to Fiatra's skeleton
6. **Live Preview + Export** — real-time WGPU viewport with egui controls; export to FBX/GLB/ONNX

---

![Fiatra GUI](asset/fiatrafirstgui.png)

---

## Architecture

```
fiat/
├── client/              # Rust / WGPU desktop app
│   ├── src/app.rs       # Application state, event loop, WGPU + egui setup
│   ├── src/core/        # Skeleton, math, ECS, validation
│   ├── src/animation/   # Playback, blending, IK, retarget
│   ├── src/render/      # Skinned mesh, PBR, WGSL shaders, export
│   ├── src/network.rs   # WebSocket client
│   └── src/ui/          # egui panels (editor, chat, scene, inspector, toolbar, gen status)
├── compiler/            # Python / FastAPI backend
│   ├── api.py           # Generation and job endpoints
│   └── core/
│       ├── animation/   # Skeleton, motion, retarget, BVH import
│       ├── ml/          # Model orchestration, pose interpolation, text-to-mesh/motion
│       ├── llm/         # LLM router for chat
│       ├── executors/   # Job executors
│       └── jobs.py      # Async job queue with progress streaming
├── asset/               # Base skeletons, bone maps, style library (populated on first run)
├── start_universe.sh    # Launch script
└── README.md
```

---

## Quick Start

**Prerequisites:** Rust 1.75+, Python 3.11+

```bash
# Set up the compiler
cd compiler
python3 -m venv venv && source venv/bin/activate
pip install -r requirements.txt
cd ..

# Launch Fiatra
./start_universe.sh
```

---

## How It Works

| Layer | Technology | Role |
|-------|-----------|------|
| **Animation** | Rust (custom math) | Skeleton, FK, IK, blending, retargeting |
| **Rendering** | Rust / WGPU | GPU skinning, PBR shaders, egui overlay |
| **Backend** | Python / FastAPI | ML orchestration, async job queue |
| **Network** | WebSocket | Real-time progress streaming |

All ML inference runs server-side (Python). The Rust client consumes generated data (mesh, motion, poses) over HTTP/WebSocket.

---

## License

MIT
