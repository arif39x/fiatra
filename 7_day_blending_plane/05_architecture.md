# 05 — Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│                        AETHERIUM STUDIO                              │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────────┐    ┌─────────────────────────────────────┐  │
│  │     CLIENT (Rust)    │    │     COMPILER (Python/FastAPI)       │  │
│  │     WGPU + egui      │    │     ML + LLM Orchestration          │  │
│  ├─────────────────────┤    ├─────────────────────────────────────┤  │
│  │                     │    │                                     │  │
│  │  ┌───────────────┐  │    │  ┌─────────────────────────────┐   │  │
│  │  │  Viewport      │  │    │  │  LLM Router                │   │  │
│  │  │  (WGPU)        │  │    │  │  ┌─────────┐ ┌───────────┐│   │  │
│  │  │  ┌───────────┐ │  │    │  │  │ Prompt  │ │ Executors ││   │  │
│  │  │  │ Static    │ │  │    │  │  │ Engine  │ │ ┌───────┐││   │  │
│  │  │  │ Mesh      │ │  │    │  │  │ (system │ │ │create_ │││   │  │
│  │  │  │ Pipeline  │ │  │    │  │  │  prompt │ │ │primitive│││   │  │
│  │  │  └───────────┘ │  │    │  │  │ +scene  │ │ ├───────┤││   │  │
│  │  │  ┌───────────┐ │  │    │  │  │ context)│ │ │assign_ │││   │  │
│  │  │  │ Skin      │ │  │    │  │  └─────────┘ │ │material│││   │  │
│  │  │  │ Pipeline  │ │  │    │  │              │ ├───────┤││   │  │
│  │  │  └───────────┘ │  │    │  │  ┌─────────┐ │ │edit_  │││   │  │
│  │  │  ┌───────────┐ │  │    │  │  │Scene    │ │ │scene  │││   │  │
│  │  │  │ PostFX    │ │  │    │  │  │Manager  │ │ ├───────┤││   │  │
│  │  │  │ (optional) │ │  │    │  │  │(tracks  │ │ │generate│││   │  │
│  │  │  └───────────┘ │  │    │  │  │ entities)│ │ │_skele │││   │  │
│  │  └───────────────┘  │    │  │  └─────────┘ │ │ton    │││   │  │
│  │                     │    │  │              │ ├───────┤││   │  │
│  │  ┌───────────────┐  │    │  │  ┌─────────┐ │ │generate│││   │  │
│  │  │  UI (egui)     │  │    │  │  │ Job     │ │ │_motion │││   │  │
│  │  │  ┌───────────┐ │  │    │  │  │ Queue   │ │ ├───────┤││   │  │
│  │  │  │ Chat      │ │  │    │  │  │ (async) │ │ │generate│││   │  │
│  │  │  │ Panel     │ │  │    │  │  └─────────┘ │ │_mesh  │││   │  │
│  │  │  ├───────────┤ │  │    │  │              │ └───────┘││   │  │
│  │  │  │ Scene     │ │  │    │  │  ┌─────────┐ └───────────┘│   │  │
│  │  │  │ Panel     │ │  │    │  │  │ Model   │              │   │  │
│  │  │  ├───────────┤ │  │    │  │  │ Registry│              │   │  │
│  │  │  │ Inspector │ │  │    │  │  └─────────┘              │   │  │
│  │  │  ├───────────┤ │  │    │  └─────────────────────────────┘   │  │
│  │  │  │ Toolbar   │ │  │    │                                     │  │
│  │  │  └───────────┘ │  │    └─────────────────────────────────────┘  │
│  │  └───────────────┘  │            ▲                                 │
│  │                     │            │ WebSocket (ws://)               │
│  │  ┌───────────────┐  │            │ JSON messages                  │
│  │  │  SCENE (ECS)   │◄├────────────┘                                 │
│  │  │  ┌───────────┐ │  │                                             │
│  │  │  │ Entities  │ │  │    ┌─────────────────────────────────────┐  │
│  │  │  │ Components│ │  │    │     ASSET PIPELINE                  │  │
│  │  │  │ Systems   │ │  │    ├─────────────────────────────────────┤  │
│  │  │  └───────────┘ │  │    │ Skeleton lib │ Mesh lib │ Textures  │  │
│  │  └───────────────┘  │    └─────────────────────────────────────┘  │
│  │                     │                                             │
│  │  ┌───────────────┐  │    ┌─────────────────────────────────────┐  │
│  │  │  INPUT         │  │    │     EXPORT PIPELINE                │  │
│  │  │  (winit/egui)  │  │    ├─────────────────────────────────────┤  │
│  │  │  keyboard      │  │    │ GLB │ FBX │ USD │ glTF │ ONNX      │  │
│  │  │  mouse         │  │    └─────────────────────────────────────┘  │
│  │  │  gizmo drag    │  │                                             │
│  │  └───────────────┘  │                                             │
│  └─────────────────────┘                                             │
└──────────────────────────────────────────────────────────────────────┘
```

---

## Data Flow: Chat → Scene

```
User: "Add a red cube at (1, 2, 0)"
  │
  ▼
Chat Panel → WebSocket → Compiler LLM Router
  │
  ▼
System Prompt (includes scene context + available commands)
  │
  ▼
LLM outputs: { "reply": "Added a red cube...", "actions": [
    { "type": "create_primitive", "params": { "primitive": "cube",
      "position": [1,2,0], "scale": [1,1,1], "color": [1,0,0] } }
  ]}
  │
  ▼
PrimitiveExecutor.execute(params)
  │
  ▼
Returns: { "entity_id": 5, "type": "cube", "position": [1,2,0], ... }
  │
  ▼
SceneManager.add_entity() → updates scene context
  │
  ▼
WebSocket → Client
  │
  ▼
Client Handler: deserialize → ECSWorld.spawn() + add components
  │
  ▼
Draw Loop: query TransformComponent + MeshComponent → StaticRenderer.draw()
  │
  ▼
GPU: Static Mesh WGSL → pixel on screen
```

## Data Flow: Manual → Scene

```
User clicks "Add Cube" in toolbar
  │
  ▼
egui button callback → ECSWorld.spawn()
  → adds TransformComponent (identity)
  → adds MeshComponent (Cube, generated mesh data)
  → adds MaterialComponent (white, 0.0 metallic, 0.5 roughness)
  │
  ▼
Scene Panel updates (new entity appears in tree)
  │
  ▼
WebSocket → Compiler SceneManager.add_entity()
  (syncs scene context so LLM knows about it)
  │
  ▼
Draw Loop picks up the new entity → renders it
```

## Data Flow: Hybrid (The Magic)

```
1. User places 3 cubes manually (toolbar clicks)
       │
       ▼
2. Scene context sent to LLM on next chat:
   "Scene has: Entity 1 (cube, pos=(0,0,0), color=white),
                Entity 2 (cube, pos=(1,0,0), color=white),
                Entity 3 (cube, pos=(2,0,0), color=white)"
       │
       ▼
3. User types: "Make them red, green, blue in a tower"
       │
       ▼
4. LLM outputs:
   - edit_scene → entity 1 color → [1,0,0], pos → [0,0,0]
   - edit_scene → entity 2 color → [0,1,0], pos → [0,1,0]
   - edit_scene → entity 3 color → [0,0,1], pos → [0,2,0]
       │
       ▼
5. Scene updates. User drags the blue cube higher with gizmo.
       │
       ▼
6. Next chat: LLM sees updated positions. Consistency maintained.
```

---

## Key Components (New)

### Rust Client

| Module | New File | Purpose |
|---|---|---|
| `render/static_mesh.wgsl` | New | Vertex shader with model matrix, fragment with PBR |
| `render/static_renderer.rs` | New | Pipeline + draw calls for non-skinned meshes |
| `render/raycast.rs` | New | Screen-space ray → entity intersection |
| `render/gizmo.rs` | New | 3-axis transform handles |
| `scene/mod.rs` | New | Scene struct wrapping ECS + parent/child + serialization |
| `ui/scene_panel.rs` | New | egui tree view of all entities |
| `ui/inspector.rs` | New | egui property editor |
| `ui/toolbar.rs` | New | egui buttons for primitives + tools |
| `core/undo.rs` | New | Command pattern undo/redo stack |

### Python Compiler

| Module | New File | Purpose |
|---|---|---|
| `executors/primitive_executor.py` | New | Creates primitives from LLM params |
| `executors/material_executor.py` | New | Applies material changes from LLM |
| Expanded scene context in prompt | Edit | Include full entity state for LLM |

---

## Data Types: Entity Serialization (Client ↔ Server)

```json
{
  "entity_id": 5,
  "entity_type": "primitive",
  "label": "Red Cube",
  "data": {
    "primitive": "cube",
    "position": [1.0, 2.0, 0.0],
    "rotation": [0.0, 0.0, 0.0],
    "scale": [1.0, 1.0, 1.0],
    "material": {
      "albedo": [1.0, 0.0, 0.0],
      "metallic": 0.0,
      "roughness": 0.5,
      "ambient_occlusion": 1.0
    },
    "parent_id": null
  }
}
```
