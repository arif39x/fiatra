# 04 — 7-Day Roadmap

Build a **working prototype** of Aetherium Studio — not Ursina, not Muse, but a minimal-viable blend that renders multiple objects, supports chat AND manual interaction, and is LLM-native.

---

## Day 1: Static Mesh Pipeline

**Goal:** Render ANY mesh (not just skinned characters) with a simple WGSL shader.

| Task | What | Inspired By |
|---|---|---|
| 1.1 | Write `static_mesh.wgsl` — vertex shader takes `model_matrix`, fragment takes `albedo` + `metallic` + `roughness` | Ursina's unlit/lighting shaders |
| 1.2 | Create `StaticRenderer` in Rust — manages a pipeline for non-skinned meshes | Muse's `SkinRenderer` (refactor) |
| 1.3 | Add primitive generators: `create_cube()`, `create_sphere(segments)`, `create_plane()`, `create_quad()` | Ursina's procedural model classes |
| 1.4 | Wire into draw loop: render a hardcoded list of 3 primitives (cube + sphere + plane) | |

**Verification:** Three colored primitives visible on screen. Orbit camera works. Depth sorting correct.

---

## Day 2: Entity + Scene Graph

**Goal:** Every visible thing is an ECS Entity with a transform, mesh, and material.

| Task | What | Inspired By |
|---|---|---|
| 2.1 | Extend `EcsWorld`: `spawn()`, `despawn()`, parent/child via optional `parent_id` on `TransformComponent` | Ursina's `Entity.parent` |
| 2.2 | Add `MeshComponent { mesh_type: MeshType, data: Option<MeshData> }` and `MaterialComponent { albedo, metallic, roughness }` | Ursina's `Entity.model`, `Entity.color` |
| 2.3 | Write a `Scene` struct that owns the ECS world and provides `add_entity()`, `remove_entity()`, `find_by_id()` | Ursina's `scene` singleton |
| 2.4 | Rewrite draw loop: iterate `Query<TransformComponent + MeshComponent>`, compute world matrix from parent chain, submit draw calls | Ursina's scene graph traversal |

**Verification:** Can spawn 5+ entities at different positions with different shapes/colors. Parent-child transforms work (child moves with parent).

---

## Day 3: Scene Panel + Selection

**Goal:** Users can see all entities in a tree and click to select them.

| Task | What | Inspired By |
|---|---|---|
| 3.1 | egui `ScenePanel`: tree view of all entities, grouped by parent/child, click to select | Ursina's level editor, Unity Hierarchy |
| 3.2 | Add `Selected` component to ECS. Highlight selected entity with outline or wireframe overlay | |
| 3.3 | Raycast from mouse into scene: intersect with entity bounding spheres (simple) or mesh triangles (precise) | Ursina's `mouse.hovered_entity` + `raycast()` |
| 3.4 | Click in viewport → raycast → select nearest entity. Highlight it. Show in Scene Panel. | |

**Verification:** Click a cube → it highlights. Scene Panel shows tree. Click another → selection switches.

---

## Day 4: Inspector + Property Editing

**Goal:** Users can edit any entity's properties manually.

| Task | What | Inspired By |
|---|---|---|
| 4.1 | egui `Inspector` panel: shows selected entity's transform (position/rotation/scale sliders), material (color picker, roughness slider) | Ursina's property editing, Unity Inspector |
| 4.2 | Changes in inspector update ECS components in real-time | |
| 4.3 | Rotation uses Euler angles (user-friendly), stored as quaternion internally | Ursina's `entity.rotation` |
| 4.4 | Undo/redo stack: before any edit, snapshot affected components | |

**Verification:** Select cube → change position slider → cube moves. Change color → cube recolors. Undo reverts.

---

## Day 5: Transform Gizmo + Toolbar

**Goal:** Users can create and manipulate objects directly in the 3D viewport.

| Task | What | Inspired By |
|---|---|---|
| 5.1 | Translate gizmo: 3 colored arrows at selected entity's position. Drag arrow → move along that axis | Blender/Maya gizmos, Unity's `TransformHandle` |
| 5.2 | Toolbar: [Add Cube] [Add Sphere] [Add Plane] [Add Cylinder] [Add Light] [Delete] | Ursina's editor, basic 3D toolbars |
| 5.3 | Each toolbar button spawns a new entity at origin with default material | |
| 5.4 | Delete button removes selected entity (with undo) | |

**Verification:** Click "Add Cube" → cube appears. Select it → gizmo shows. Drag Y arrow → cube moves up. Delete → gone.

---

## Day 6: LLM Scene Integration

**Goal:** The LLM can read the scene AND write to it. Chat and manual modes converge.

| Task | What | Inspired By |
|---|---|---|
| 6.1 | Extend `SceneManager` (Python) to accept entity CRUD from the new scene system | Muse existing SceneManager |
| 6.2 | New executor: `create_primitive` — LLM outputs `{type: "create_primitive", params: {primitive: "cube", position: [0,1,0], color: [1,0,0], scale: [1,1,1]}}` | Ursina `Entity(model='cube', color=color.red)` |
| 6.3 | New executor: `assign_material` — LLM sets albedo, metallic, roughness on existing entities | |
| 6.4 | Scene context in system prompt now includes full entity list with positions, colors, types | Muse existing `{scene_context}` |
| 6.5 | Chat panel sends to LLM. LLM response creates/edits entities. Scene Panel updates. | |

**Verification:** Chat "add a blue sphere at (2,0,0)" → sphere appears at correct position with correct color. Chat "make the cube bigger" → cube scales up.

---

## Day 7: Polish + Hybrid Workflow

**Goal:** A seamless demo where chat and manual modes interleave freely.

| Task | What | Inspired By |
|---|---|---|
| 7.1 | Manual creation updates scene context for LLM. LLM responses update the scene for manual editing. **Full bidirectional sync.** | The core innovation |
| 7.2 | Add Quick Commands toolbar: "Add Primitive", "Randomize Colors", "Arrange in Grid" (each sends a pre-built prompt) | |
| 7.3 | Test flow: User places 3 cubes manually → types "arrange them in a circle" → LLM repositions → user drags one higher → "make a staircase to it" | |
| 7.4 | Fix edge cases: entity ID mapping between Python SceneManager and Rust ECS, undo with LLM actions, selection after LLM delete | |
| 7.5 | Write a demo script / scene file that loads on startup showing the capabilities | |

**Verification:** A full demo session where manual placement, chat prompts, and manual tweaks interleave without friction. No crashes.

---

## Summary

| Day | Theme | Files Changed | New Capability |
|---|---|---|---|
| 1 | Render engine | `render/static_mesh.wgsl`, `render/static_renderer.rs` | Arbitrary mesh rendering |
| 2 | Entity system | `ecs.rs`, `scene.rs`, `animation/` (minor) | Multi-object scene graph |
| 3 | Selection | `ui/scene_panel.rs`, `render/raycast.rs` | Click to select |
| 4 | Editing | `ui/inspector.rs`, `core/undo.rs` | Property editing |
| 5 | Gizmo | `ui/toolbar.rs`, `ui/gizmo.rs` | Direct manipulation |
| 6 | LLM sync | `compiler/core/executors/primitive_executor.py` | Chat creates objects |
| 7 | Integration | All of the above | Hybrid workflow demo |
