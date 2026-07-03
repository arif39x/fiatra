# 07 — Comparison Table: Muse vs Ursina vs Aetherium

| Dimension | Muse | Ursina | Aetherium Studio (Blend) |
|---|---|---|---|
| **Rendering** | Rust/WGPU (cutting-edge) | Python/Panda3D (legacy) | Rust/WGPU + WGSL |
| **Primitives** | None | Cube, sphere, plane, quad, circle, cylinder, cone, capsule, grid, terrain | All of Ursina's + procedural LOD |
| **Max objects** | 1 (single character) | Unlimited (scene graph) | Unlimited (ECS + instancing) |
| **Performance** | Fast (native, GPU) | Slow (Python, GIL) | Fast (native + threaded) |
| **LLM integration** | Native (router + executors) | None | Native + bidirectional sync |
| **Chat input** | Yes | No | Yes (primary mode) |
| **Manual input** | No (chat only) | Yes (Python API + editor) | Yes (gizmo + toolbar + inspector) |
| **Hybrid workflow** | No | No | **Yes — the core innovation** |
| **Scene graph** | Partial (SceneManager) | Full (Entity.parent, world_position) | Full (ECS parent/child) |
| **UI toolkit** | Minimal (chat + log) | 40+ prefabs (Text, Button, Slider, WindowPanel...) | egui + ported Ursina prefabs |
| **Animation** | Motion clips (skinned only) | Tween, sequence, `@every`, `@after` | Both: motion clips + tween system |
| **Physics** | None | Raycast + Bullet | Raycast (MVP), Bullet (post-MVP) |
| **Camera** | Orbit (hardcoded) | EditorCamera, FPS, ortho, UI layer | Multi-mode orbit/ortho/FPS |
| **Export** | GLB/FBX (single character) | None | Full scene export (GLB/USD) |
| **Async** | Yes (Tokio + job queue) | No (blocking) | Yes |
| **ML pipeline** | Yes (text-to-mesh, text-to-motion, style transfer) | No | Yes + primitive executors |
| **Undo/redo** | No | No | Yes (command pattern) |
| **Plugin system** | No | Python (any import) | WASM (future) |
| **Learning curve** | Steep (Rust + AI) | Shallow (Python, concise) | Shallow for chat, progressive for experts |
| **Target user** | AI researchers | Hobbyist devs | Everyone |
| **Lines of code** | ~3,500 (Rust) + ~1,500 (Python) | ~25,000 (Python) | ~5,000 (Rust) + ~2,000 (Python) post-7-days |

---

## The Delta

Where Aetherium pulls ahead:

| Capability | Muse can't | Ursina can't | Aetherium can |
|---|---|---|---|
| Render a cube | ✗ | ✓ | ✓ |
| Render a character | ✓ | ✓ | ✓ |
| Render both at once | ✗ | ✓ | ✓ |
| Create via chat | ✓ | ✗ | ✓ |
| Click-to-move | ✗ | ✓ | ✓ |
| Chat "make it bigger" → it grows | ✓ | ✗ | ✓ |
| Drag it bigger manually AFTER | ✗ | ✓ | ✓ |
| Chat sees what you dragged | ✗ | ✗ | **✓** |
| Cycle: chat→tweak→chat→tweak→... | ✗ | ✗ | **✓** |
| Run at 144fps with 500 objects | ~ (1 char only) | ✗ (Python) | ✓ (WGPU + instancing) |
| Export full scene | ✗ | ✗ | ✓ |
