# 02 — Blend Strategy: Multi-Stage Fusion

Blending Muse and Ursina is not a single merge — it's a **3-stage fusion** where each stage unlocks the next. Each stage takes Ursina's design DNA and implements it using Muse's technology stack.

---

## Stage 1: Render Anything (Ursina's Entity Model → Muse's WGPU)

### Goal
Replace Muse's "skinned character only" renderer with a **general entity renderer** that can draw any mesh with any transform.

### What Ursina brings
- `Entity(model='cube', position=(x,y,z), rotation=(rx,ry,rz), scale=(sx,sy,sz))`
- Internal scene graph traversal per frame
- Each entity draws itself

### What Muse brings
- WGPU device + queue + surface
- GPU buffers for vertices/indices
- Camera uniform buffer

### Fusion
```
Ursina's Entity concept
        │
        ▼
Rust struct Entity {
    id: u64,
    model: ModelType,        // Cube | Sphere | Plane | Custom(Mesh)
    transform: Transform,    // position, rotation, scale
    material: Material,      // albedo, metallic, roughness
    parent: Option<EntityId>,
    children: Vec<EntityId>,
}
        │
        ▼
ECS Component: TransformComponent + MeshComponent + MaterialComponent
        │
        ▼
New WGSL shader (static_mesh.wgsl) that takes model_matrix + material
        │
        ▼
Per-frame: iterate all entities with MeshComponent, draw with their transform
```

### Output
**Multi-object scene rendering.** Cube + sphere + character all visible at once.

---

## Stage 2: Manual Interaction (Ursina's UI → Muse's egui)

### Goal
Add what Ursina has natively (click-to-select, drag, gizmos, scene panel) but using egui + raycasting.

### What Ursina brings
- Mouse hover detection on entities
- Editor camera (orbit/zoom/pan)
- Scene hierarchy panel

### What Muse brings
- egui immediate-mode GUI
- WebSocket ↔ chat architecture (extend to manual commands)

### Fusion
```
Ursina's click-to-select
        │
        ▼
Raycast from mouse position through camera → hit entity → select
        │
        ▼
egui ScenePanel: tree of all entities, click to select
egui Inspector: edit position/rotation/scale/color with sliders
egui Toolbar: "Add Cube", "Add Sphere", "Add Light" buttons
        │
        ▼
Transform gizmo: 3-axis arrow handles in the viewport (drag to move)
```

### Output
**Full manual creation workflow.** Users can create, select, move, rotate, scale, and edit any object — entirely without typing a prompt.

---

## Stage 3: Hybrid Orchestrator (Both → One)

### Goal
Make chat (LLM) and manual interaction **the same system**. The LLM can see and edit what the user places manually. The user can tweak what the LLM generates.

### What Ursina brings
- Entity state is always inspectable
- Scene can be modified at any time

### What Muse brings
- LLM router sees scene context (`{scene_context}` in the prompt)
- Executors validate and apply LLM actions

### Fusion
```
Manual action (click toolbar)
        │
        ▼
"Add Sphere at (0,2,0)" → creates ECS entity → SceneManager registers it
        │
        ▼
LLM next prompt includes: "Scene has: Entity 1 (sphere, pos=(0,2,0)), Entity 2 (cube, pos=(1,0,0))"
        │
        ▼
User types: "put the cube on top of the sphere"
        │
        ▼
LLM outputs: edit_scene → entity_transforms → entity_2.position = [0, 2.5, 0]
```

This is the **breakthrough mode**: the LLM is not a black box that generates from scratch — it's a **collaborator** that works with whatever the user has already built.

---

## The Blend Matrix

| Capability | Ursina DNA | Muse DNA | Blend |
|---|---|---|---|
| Object creation | `Entity(model='cube')` | LLM executors | Primitive factory + LLM can call it |
| Scene graph | Parent/child hierarchy | ECS `TransformComponent` | ECS-based scene graph with parent/child |
| Rendering | Panda3D scene traversal | WGPU per-entity draw | Static mesh WGSL shader, ECS-driven draw loop |
| UI | 40+ Python prefabs | egui panels | egui port of Ursina prefabs |
| Camera | `EditorCamera` + `camera.ui` | `OrbitCamera` struct | Extend OrbitCamera with ortho + UI layer |
| Animation | Tween, sequence, `@every` | Motion clips | Procedural tweens on any entity property |
| LLM | None | Router + executors + scene context | LLM can read AND write the scene graph |
| Input | `held_keys`, `mouse` | Winit events | Map Ursina input API to winit via egui |
| Physics | Raycast, Bullet | None | Raycast (gizmo + selection) first, Bullet later |
| Export | None | GLB/FBX | Export entire scene, not just one character |
