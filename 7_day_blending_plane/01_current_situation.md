# 01 — Current Situation

## Muse (Today)

Muse is an **AI Character & Animation Studio** — a Rust/WGPU desktop app with a Python ML backend.

### What Exists

```
muse/
├── client/               # Rust / WGPU
│   ├── src/core/         # ECS, math, skeleton, validation
│   ├── src/animation/    # Playback, blending, IK, retarget
│   ├── src/render/       # Skinned mesh pipeline (skin.wgsl), camera, export
│   └── src/ui/           # egui: Chat, gen-status, logs, export
├── compiler/             # Python / FastAPI
│   ├── api.py            # WebSocket + HTTP endpoints
│   └── core/
│       ├── animation/    # Math, skeleton, motion, retarget
│       ├── ml/           # Model registry, pose interpolation, style transfer
│       ├── llm/          # Router + prompt — LLM outputs structured actions
│       ├── executors/    # skeleton, mesh, motion, texture, scene_editor
│       └── scene_manager.py  # Tracks entities by ID
└── asset/                # Skeletons, bone maps, styles
```

### Muse + Points

| # | Point | Detail |
|---|-------|--------|
| +1 | **Rust/WGPU rendering** | Native performance, modern GPU API, cross-platform, no GC |
| +2 | **LLM-native architecture** | LLM isn't bolted on — it's the primary input. Router + executors pattern is clean |
| +3 | **Async job queue** | Long ML tasks don't block. WebSocket progress streaming to client |
| +4 | **ML pipeline** | Model registry, text-to-mesh, text-to-motion, style transfer, retargeting |
| +5 | **ECS exists** | Entity-component-system is already defined (`ecs.rs`) — just underutilized |
| +6 | **Export pipeline** | GLB/FBX export already wired (though partial) |
| +7 | **egui UI** | Immediate-mode GUI, inspectable, dockable, easy to extend |
| +8 | **WebSocket architecture** | Client-server split enables future multi-user / cloud rendering |

### Muse − Points

| # | Point | Detail |
|---|-------|--------|
| −1 | **Single object only** | Renders one skinned character. No multi-object scene |
| −2 | **No primitives** | Cannot create a cube, sphere, plane, or any basic shape |
| −3 | **No manual interaction** | Chat-only input. No click-select, no drag-move, no gizmos |
| −4 | **No scene hierarchy** | No parent/child, no outliner, no grouping |
| −5 | **Disconnected ECS** | ECS exists but doesn't drive rendering. Entities are just data |
| −6 | **No 2D support** | No sprites, no UI-in-3D-world, no orthographic view |
| −7 | **No physics** | No collision, no raycast, no gravity |
| −8 | **No animation system** | Only plays pre-baked clips. No tweening, sequencing, or procedural animation |

---

## Ursina (Inspiration)

Ursina is a **Python game engine** built on Panda3D — designed for rapid 3D prototyping.

### Ursina + Points

| # | Point | Detail |
|---|-------|--------|
| +1 | **Entity simplicity** | `Entity(model='cube', color=color.red)` — one line, a visible object |
| +2 | **Rich primitives** | Cube, sphere, plane, quad, circle, cylinder, cone, capsule, terrain, grid |
| +3 | **Scene graph** | Parent/child transforms, world-space/local-space duality |
| +4 | **UI prefabs** | 40+ prefabs: Text, Button, Slider, InputField, WindowPanel, DropdownMenu... |
| +5 | **Camera system** | Perspective/orthographic, UI overlay layer, post-processing shaders |
| +6 | **Animation** | Tweening (`animate_x(3, duration=1)`), sequences, `@every`, `@after` |
| +7 | **Input system** | `held_keys` dict, mouse singleton, key rebinding, gamepad support |
| +8 | **Collision/physics** | Raycast, boxcast, spherecast + optional Bullet physics |
| +9 | **Editor camera** | Orbit/fly camera built-in, plus first-person / platformer controllers |

### Ursina − Points

| # | Point | Detail |
|---|-------|--------|
| −1 | **Python-only** | GIL-bound, single-threaded, not suitable for heavy computation |
| −2 | **Panda3D legacy** | Built on an aging C++ engine (CMU/Disney 2000s era) |
| −3 | **No LLM/AI** | Zero AI integration. Dialogue system is simple branching text |
| −4 | **No ML pipeline** | No way to generate meshes, textures, or motion from prompts |
| −5 | **No export** | No built-in GLB/FBX/USD export for production pipelines |
| −6 | **No async** | All code runs on the main thread. Long operations freeze the engine |
| −7 | **No client-server** | No networking architecture. Multiplayer is DIY |
| −8 | **No plugin system** | No WASM, no scripting sandbox, no package manager |

---

## The Gap

```
                     Ursina territory
                    ┌─────────────────┐
                    │  Any object      │
                    │  Scene graph     │
                    │  UI prefabs      │
                    │  Primitives      │
                    │  Animation       │
                    │  Physics         │
                    └────────┬────────┘
                             │
               BLEND ZONE    │    ← What we will build
                             │
                    ┌────────▼────────┐
                    │  LLM-native     │
                    │  Rust/WGPU      │
                    │  Async pipeline │
                    │  ML generation  │
                    │  Export pipeline│
                    └─────────────────┘
                     Muse territory
```

Muse has the **engine** (Rust/WGPU, async, LLM). Ursina has the **design** (entity simplicity, primitives, scene graph, UI). Neither alone is the answer. Their blend is.
