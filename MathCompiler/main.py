from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from core.parser.lower_to_ir import parse_string_to_ir
from core.runtime.validation import validate_expr
from core.compiler.optimizer.passes import optimize
from core.compiler.optimizer.canonicalize import canonicalize
from core.compiler.type_checker import infer_type
from core.analysis.complexity import analyze_complexity
from core.compiler.wgsl.backend import WGSLBackend
from core.compiler.pass_manager import PassManager

app = FastAPI()

class EquationRequest(BaseModel):
    equation: str

# Initialize the compiler pipeline
pm = PassManager()
pm.add_pass("Type Inference", infer_type)
pm.add_pass("Semantic Validation", validate_expr)
pm.add_pass("Canonicalization", canonicalize)
pm.add_pass("Optimization", optimize)
pm.add_pass("Complexity Analysis", analyze_complexity)

backend = WGSLBackend()

def to_wgsl_module(wgsl_expr: str) -> str:
    # This wraps the generated expression into the full shader template
    return f"""
struct State {{
    x: f32,
    y: f32,
    z: f32,
    padding: f32,
}}

@group(0) @binding(0)
var<uniform> state: State;

struct VertexOutput {{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {{
    var out: VertexOutput;
    let x = f32((in_vertex_index << 1) & 2u);
    let y = f32(in_vertex_index & 2u);
    out.clip_position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);
    return out;
}}

fn opU(d1: vec2<f32>, d2: vec2<f32>) -> vec2<f32> {{
    if (d1.x < d2.x) {{ return d1; }}
    return d2;
}}

// Safe power function to prevent NaN on negative bases (common in SDFs)
fn safe_pow(base: f32, exp: f32) -> f32 {{
    return pow(abs(base), exp);
}}

fn map(p: vec3<f32>) -> vec2<f32> {{
    let x = p.x;
    let y = p.y;
    let z = p.z;
    
    // The compiled expression is injected here (replacing pow with safe_pow)
    let dist = {wgsl_expr.replace("pow(", "safe_pow(")};
    
    return vec2<f32>(dist, 1.0); // 1.0 = Default Material
}}

fn calcNormal(p: vec3<f32>) -> vec3<f32> {{
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        map(p + e.xyy).x - map(p - e.xyy).x,
        map(p + e.yxy).x - map(p - e.yxy).x,
        map(p + e.yyx).x - map(p - e.yyx).x
    ));
}}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {{
    let uv = in.uv * 2.0 - 1.0;

    // Use state for camera or object offset
    let ro = vec3<f32>(state.x, state.y, state.z + 100.0);
    let rd = normalize(vec3<f32>(uv.x, uv.y, -1.5));
var t = 0.0;
for (var i = 0; i < 256; i = i + 1) {
    let p = ro + rd * t;
    let res = map(p);
    let d = res.x;

    if (d < 0.001) {
        let n = calcNormal(p);
        let lightDir = normalize(vec3<f32>(1.0, 1.0, 1.0));
        let diff = max(dot(n, lightDir), 0.1);

        var col = vec3<f32>(0.5, 0.7, 1.0) * diff;

        // Fog / Depth darkening
        col = col * exp(-0.001 * t);

        return vec4<f32>(col, 1.0);
    }
    t = t + d;
    if (t > 2000.0) {
        break;
    }
}
    }}

    // Sky gradient
    let sky = mix(vec3<f32>(0.02, 0.05, 0.1), vec3<f32>(0.1, 0.2, 0.3), uv.y * 0.5 + 0.5);
    return vec4<f32>(sky, 1.0);
}}
"""

@app.post("/compile_sdf")
async def compile_sdf(req: EquationRequest):
    try:
        # 1. Parsing
        ir_expr = parse_string_to_ir(req.equation)
        
        # 2. Run Compiler Pipeline (Type Check, Validate, Optimize)
        final_ir = pm.run(ir_expr)
        
        # 3. Lowering to WGSL AST and Emission
        wgsl_ast = backend.compile_expr(final_ir)
        wgsl_expr_string = wgsl_ast.emit()
        
        # 4. Wrap into full module
        full_code = to_wgsl_module(wgsl_expr_string)

        return {"status": "success", "wgsl": full_code}
        
    except Exception as e:
        raise HTTPException(status_code=400, detail=f"Compilation error: {str(e)}")
