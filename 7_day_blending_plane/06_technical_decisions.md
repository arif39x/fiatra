# 06 — Technical Decisions

## Why Each Choice Was Made

---

### Decision 1: Separate Static Mesh Pipeline (Not Unified with Skin)

**Chosen:** New `static_mesh.wgsl` + `StaticRenderer` separate from `skin.wgsl` + `SkinRenderer`

**Rejected:** One universal pipeline with optional skinning

**Why:** The skinning pipeline has per-vertex bone weights + joint matrix uniform arrays — expensive for static objects (wasted uniform space, wasted vertex data). Separate pipelines are clearer, faster, and easier to debug. A future optimization can merge them via shader specialization.

**Tradeoff:** More code duplication (bind groups, pipeline layout) — but worth it for clarity and performance.

---

### Decision 2: ECS-Based Scene Graph (Not a Simple Vec<Entity>)

**Chosen:** Extend existing `EcsWorld` with parent/child via `TransformComponent.parent_id`

**Rejected:** `Vec<Entity>` with recursive transforms, or use a crate like `hecs`/`specs`

**Why:** ECS is already partially built in the Muse codebase. Adding parent_id + recursive world-matrix computation is ~50 lines. An external ECS crate adds dependency weight and API mismatch. The existing `EcsWorld` is minimal and correct — extend, don't replace.

**Tradeoff:** No archetype-based query optimization (yet). Fine for <1000 entities.

---

### Decision 3: Raycast-Based Selection (Not GPU Picking)

**Chosen:** CPU raycast against entity bounding volumes (sphere or AABB)

**Rejected:** GPU picking (render IDs to a separate buffer, read back)

**Why:** GPU picking requires:
- A separate render pass with an ID buffer
- Buffer readback (async, can be 1-2 frames delayed)
- Handling readback on resize/reconfig

CPU raycast against bounding spheres is O(n) but simple, synchronous, and accurate enough for selection. For <500 entities at 60fps, it's unnoticeable.

**Tradeoff:** Not pixel-perfect for concave meshes. Mitigation: raycast against mesh triangles for the selected entity (only one).

---

### Decision 4: egui for Gizmo (Not a Custom 2D Overlay)

**Chosen:** Gizmo drawn as 3D geometry rendered through the WGPU pipeline, picking via raycast

**Rejected:** egui 2D overlay on the viewport, or immediate-mode 2D drawing

**Why:** The gizmo needs to exist in 3D space (behind/occluded by objects, scaled by distance, depth-tested). egui is screen-space only. 3D geometry in the WGPU pipeline gives correct behavior. Inertial picking (raycast vs gizmo axes) is handled in the same raycast system.

**Tradeoff:** More complex than a 2D overlay, but correct 3D behavior.

---

### Decision 5: LLM Sees the Full Scene (Not Just Entity Counts)

**Chosen:** System prompt includes serialized scene: `"Entity 5: cube at (1,0,0) color=red...Entity 6: sphere at (0,2,0) color=blue..."`

**Rejected:** Only include entity IDs and types ("scene has 3 cubes, 1 sphere")

**Why:** The LLM needs exact positions and colors to reason about spatial relationships ("put the cube on top of the sphere", "make everything blue"). Including the full state (10-20 entities) is ~2KB of text — trivial for modern LLM context windows. Only truncate if >100 entities.

**Tradeoff:** Larger prompts. Mitigation: group identical entities ("10 cubes with varying positions").

---

### Decision 6: Python Compiler Stays (Not Everything in Rust)

**Chosen:** Keep Python/FastAPI compiler. Add new executors in Python.

**Rejected:** Rewrite compiler in Rust for a unified codebase

**Why:** The Python compiler already has:
- Async job queue
- ML model loading (PyTorch, etc.)
- LLM HTTP client
- WebSocket management

Rewriting these in Rust would take weeks with no user-facing benefit. The client-server split is clean — Rust owns the GPU, Python owns the AI. Keep it.

**Tradeoff:** Two runtimes to manage. But the start script (`start_universe.sh`) already handles this.

---

### Decision 7: No Physics Engine Yet (Raycast Only)

**Chosen:** Only implement `raycast()` for selection and gizmo interaction

**Rejected:** Integrate Bullet/Rapier physics engine

**Why:** Physics is a hard problem (broadphase, narrowphase, constraints, sleeping, CCD). The 7-day goal is a working prototype, not a game engine. Raycasting is ~50 lines of math. Physics would be thousands. Add physics post-MVP as a WASM plugin.

**Tradeoff:** No gravity, no collisions, no rigid bodies in the first prototype. Objects don't fall.

---

### Decision 8: WASM Plugins (Future, Not in 7 Days)

**Chosen:** Acknowledge as a post-MVP feature. Not implemented in 7-day plan.

**Rejected:** Build plugin system now

**Why:** WASM plugin systems require:
- A WASM runtime (wasmtime/wasmer)
- Stable ABI for scene access
- Sandboxed execution
- SDK generation

This is a 2-week project on its own. The 7-day plan focuses on the core engine + hybrid interaction.

**Tradeoff:** First public version is not extensible. But the architecture anticipates plugins (ECS is the API, WASM is the boundary).

---

### Decision 9: Undo/Redo via Command Pattern (Not Snapshot)

**Chosen:** Each edit creates a `Command` object with `execute()` and `undo()` methods. Stack of commands.

**Rejected:** Full scene snapshots before each edit

**Why:** Snapshots of the entire ECS world are memory-expensive (clone every component). Command pattern stores only "was X, now Y" — compact and composable. LLM actions also generate commands, so undo works for chat too.

**Tradeoff:** Requires manual implementation per edit type. Mitigation: only 4-5 edit types in the MVP (transform, material, add, delete).

---

### Decision 10: Bidirectional Sync via Entity ID Mapping

**Chosen:** Python `SceneManager` and Rust `EcsWorld` both use sequential integer IDs. The compiler tracks IDs and sends them to the LLM as `entity_5`. The client maps `entity_5` to its local ECS entity.

**Rejected:** Let the LLM generate UUIDs, or use the client as the source of truth

**Why:** Sequential integers are:
- Compact in the prompt ("entity_5" not "a1b2c3d4-e5f6-7890-abcd-ef1234567890")
- Easy for the LLM to reference
- Simple to keep in sync (compiler allocates, sends to client, client uses same number)

The compiler never deletes IDs — it marks them as "deleted" in the prompt. This prevents the LLM from referencing stale IDs.

**Tradeoff:** Need a sync protocol for initial connection (compiler sends full entity list to client on connect). ~50 lines of code.
