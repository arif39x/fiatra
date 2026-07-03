# 03 — The New Thing: Aetherium Studio

Neither Muse (too narrow — characters only) nor Ursina (too old — Panda3D, no AI).

**Aetherium Studio** is a **modern-era, LLM-native 3D creation environment** — think Blender's power meets Ursina's simplicity meets ChatGPT's natural interface.

---

## Core Identity

| Dimension | Muse | Ursina | **Aetherium Studio** |
|---|---|---|---|
| What it makes | Characters | Any game | **Anything in 3D/2D** |
| Primary interface | Chat | Python code | **Chat + Gizmo + Voice** |
| User | AI researchers | Hobbyist devs | **Everyone** |
| Performance ceiling | High (Rust) | Low (Python) | **High (Rust + WASM plugins)** |
| AI integration | Built-in | None | **Native — LLM reads/writes the scene** |
| Extensibility | Hard (Rust compile) | Easy (Python) | **WASM sandbox + script nodes** |

---

## The Three Modes

### Mode 1: Chat (LLM-first)
```
"Create a cozy cottage with a red roof, a chimney, and a garden path"
  → LLM plans: cottage body (cube), roof (wedge), chimney (cylinder), path (plane)
  → Each primitive placed, colored, scaled
  → User: "Make the roof steeper"
  → LLM edits the roof's rotation
```

### Mode 2: Manual (Tool-first)
```
Toolbar: [Add Cube] [Add Sphere] [Add Plane] [Add Light] [Add Text]
Scene Panel: list of all objects, drag to reparent
Viewport: click to select, drag gizmo to move, right-click context menu
Inspector: edit position, rotation, scale, color, material, visibility
```

### Mode 3: Hybrid (Both, always)
```
Manual: User places a cube, colors it blue, adds a sphere on top
Chat: "Turn this into a snowman"
  → LLM sees: Entity 1 (cube, blue), Entity 2 (sphere, white)
  → LLM adds: Entity 3 (sphere for head), Entity 4 (cone for nose)
  → LLM edits: Entity 1 color → white, Entity 2 scale → (1.2, 1.2, 1.2)
  → Scene updates in real-time, user can then manually tweak
```

---

## What Makes It "Modern Era"

### 1. LLM is the Primary Computation Primitive
Not just a chat feature — the LLM **is the scripting language**. Every manual action can also be expressed as a prompt. Every prompt result can be manually tweaked. The two are symmetric.

### 2. Rust Performance Floor
No GC pauses, no GIL. WGPU runs on Vulkan/Metal/DX12. WASM plugins run in a sandboxed runtime. The engine doesn't dictate what you can build.

### 3. Scene is the API
The scene graph is not an implementation detail — it's the **public API**. The LLM serializes it to JSON. The user sees it in a tree panel. Plugins access it through a stable ABI. Export flattens it.

### 4. Progressive Complexity
| Expertise Level | Interface | Can Do |
|---|---|---|
| Beginner | Chat only | "Make a red cube" |
| Hobbyist | Chat + toolbar | Place objects, tweak colors |
| Designer | + gizmo + inspector | Precise layout, materials |
| Developer | + script nodes + WASM | Custom logic, procedural generation |
| Pro | + Rust SDK | Custom render pipelines, GPU compute |

### 5. Real-Time Collaboration
WebSocket architecture (inherited from Muse) enables:
- Two users editing the same scene
- One user prompting, another tweaking
- Cloud rendering of heavy ML tasks

---

## What It Is NOT

| NOT | Because |
|---|---|
| Not a game engine | No opinionated game loop, no built-in physics (opt-in via plugin) |
| Not a CAD tool | No NURBS, no parametric constraints (unless a plugin adds them) |
| Not a DCC tool | No Blender-style modal operators — all operations are async and undoable |
| Not a chat wrapper | LLM is one of three equal interfaces, not the only one |

---

## Name Justification: Aetherium

- **Aether** — the classical element representing the space beyond Earth. This tool is for creating **anything** in that space.
- **-ium** — a place of. Aetherium = "a place of creation in the void."
- The logo: a stylized diamond (primitive geometry) with a glowing center (the LLM core).

---

## Tagline

> **Aetherium Studio: Describe. Place. Refine.**
> *The 3D creation tool that works the way you think — by chat, by hand, or both.*
