def build_wgsl_module(wgsl_expr: str, max_steps: int) -> str:
    return f"""
struct State {{
    entities: array<vec4<f32>, 64>,
    count: u32,
    padding1: u32,
    padding2: u32,
    padding3: u32,
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

fn safe_pow(base: f32, exp: f32) -> f32 {{
    return pow(abs(base), exp);
}}

fn map(p: vec3<f32>) -> vec2<f32> {{
    let x = p.x;
    let y = p.y;
    let z = p.z;

    var final_dist = 1000000.0;
    let loop_count = max(1u, state.count);

    for (var i = 0u; i < loop_count; i = i + 1u) {{
        var state_x = 0.0;
        var state_y = 0.0;
        var state_z = 0.0;
        if (i < state.count) {{
            state_x = state.entities[i].x;
            state_y = state.entities[i].y;
            state_z = state.entities[i].z;
        }}

        let dist = {wgsl_expr};
        final_dist = min(final_dist, dist);
    }}

    return vec2<f32>(final_dist, 1.0);
}}

fn calcNormal(p: vec3<f32>) -> vec3<f32> {{
    let h = 0.001;
    let k = vec2<f32>(1.0, -1.0);
    return normalize(
        k.xyy * map(p + k.xyy * h).x +
        k.yyx * map(p + k.yyx * h).x +
        k.yxy * map(p + k.yxy * h).x +
        k.xxx * map(p + k.xxx * h).x
    );
}}

const AXIS_LEN = 25.0;
const AXIS_ARROW_LEN = 1.5;
const AXIS_ARROW_R = 0.35;

fn axis_line(p: vec3<f32>) -> vec3<f32> {{
    let w = 0.08;
    var col = vec3<f32>(0.0);

    // X-axis (red) - both directions, brighter in +x
    let dx = length(vec2<f32>(p.y, p.z));
    if (abs(p.x) < AXIS_LEN) {{
        let lx = 1.0 - smoothstep(0.0, w, dx);
        let bright_pos = smoothstep(0.0, 3.0, p.x) * 0.9;
        let bright_neg = smoothstep(0.0, -3.0, -p.x) * 0.15;
        col += vec3<f32>(lx * max(bright_pos, bright_neg), 0.0, 0.0);
    }}

    // Y-axis (green) - both directions, brighter in +y
    let dy = length(vec2<f32>(p.x, p.z));
    if (abs(p.y) < AXIS_LEN) {{
        let ly = 1.0 - smoothstep(0.0, w, dy);
        let bright_pos = smoothstep(0.0, 3.0, p.y) * 0.9;
        let bright_neg = smoothstep(0.0, -3.0, -p.y) * 0.15;
        col += vec3<f32>(0.0, ly * max(bright_pos, bright_neg), 0.0);
    }}

    // Z-axis (blue) - both directions, brighter in +z
    let dz = length(vec2<f32>(p.x, p.y));
    if (abs(p.z) < AXIS_LEN) {{
        let lz = 1.0 - smoothstep(0.0, w, dz);
        let bright_pos = smoothstep(0.0, 3.0, p.z) * 0.9;
        let bright_neg = smoothstep(0.0, -3.0, -p.z) * 0.15;
        col += vec3<f32>(0.0, 0.0, lz * max(bright_pos, bright_neg));
    }}

    // Arrow cones at positive ends (Blender-style)
    let xt = AXIS_LEN - p.x;
    if (xt > 0.0 && xt < AXIS_ARROW_LEN) {{
        let r = length(vec2<f32>(p.y, p.z));
        let cone_r = AXIS_ARROW_R * (1.0 - xt / AXIS_ARROW_LEN);
        let a = 1.0 - smoothstep(0.0, 0.04, r - cone_r);
        col += vec3<f32>(a, 0.0, 0.0);
    }}

    let yt = AXIS_LEN - p.y;
    if (yt > 0.0 && yt < AXIS_ARROW_LEN) {{
        let r = length(vec2<f32>(p.x, p.z));
        let cone_r = AXIS_ARROW_R * (1.0 - yt / AXIS_ARROW_LEN);
        let a = 1.0 - smoothstep(0.0, 0.04, r - cone_r);
        col += vec3<f32>(0.0, a, 0.0);
    }}

    let zt = AXIS_LEN - p.z;
    if (zt > 0.0 && zt < AXIS_ARROW_LEN) {{
        let r = length(vec2<f32>(p.x, p.y));
        let cone_r = AXIS_ARROW_R * (1.0 - zt / AXIS_ARROW_LEN);
        let a = 1.0 - smoothstep(0.0, 0.04, r - cone_r);
        col += vec3<f32>(0.0, 0.0, a);
    }}

    return col;
}}

fn axis_ticks(p: vec3<f32>) -> vec3<f32> {{
    let spacing = 10.0;
    let tick_len = 1.0;
    let tick_w = 0.08;

    let rx = round(p.x / spacing) * spacing;
    let ry = round(p.y / spacing) * spacing;
    let rz = round(p.z / spacing) * spacing;

    let dx = abs(p.x - rx);
    let dy = abs(p.y - ry);
    let dz = abs(p.z - rz);

    let major_w = 0.06;
    let minor_w = 0.03;

    var result = vec3<f32>(0.0);

    let is_even_x = select(0.0, 1.0, abs(round(fract(rx / spacing) * 2.0)) < 0.1);
    let is_even_y = select(0.0, 1.0, abs(round(fract(ry / spacing) * 2.0)) < 0.1);
    let is_even_z = select(0.0, 1.0, abs(round(fract(rz / spacing) * 2.0)) < 0.1);

    let mw = select(minor_w, major_w, is_even_x > 0.5);
    let near_x_tick = dx < mw && abs(p.y) < tick_len && abs(p.z) < tick_len && abs(rx) < AXIS_LEN;
    if (near_x_tick) {{
        result += vec3<f32>(is_even_x, is_even_x, is_even_x) * 0.6;
    }}

    let mw_y = select(minor_w, major_w, is_even_y > 0.5);
    let near_y_tick = dy < mw_y && abs(p.x) < tick_len && abs(p.z) < tick_len && abs(ry) < AXIS_LEN;
    if (near_y_tick) {{
        result += vec3<f32>(is_even_y, is_even_y, is_even_y) * 0.6;
    }}

    let mw_z = select(minor_w, major_w, is_even_z > 0.5);
    let near_z_tick = dz < mw_z && abs(p.x) < tick_len && abs(p.y) < tick_len && abs(rz) < AXIS_LEN;
    if (near_z_tick) {{
        result += vec3<f32>(is_even_z, is_even_z, is_even_z) * 0.6;
    }}

    return result;
}}

fn ground_grid(p: vec3<f32>) -> f32 {{
    let spacing = 10.0;
    let wx = abs(p.x - round(p.x / spacing) * spacing);
    let wz = abs(p.z - round(p.z / spacing) * spacing);
    let g = min(wx, wz);
    return exp(-g * g * 60.0) * 0.12;
}}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {{
    let uv = in.uv * 2.0 - 1.0;

    let ro = vec3<f32>(0.0, 0.0, 100.0);
    let rd = normalize(vec3<f32>(uv.x, uv.y, -1.5));

    var t = 0.0;
    var axis_acc = vec3<f32>(0.0);
    for (var i = 0; i < {max_steps}; i = i + 1) {{
        let p = ro + rd * t;
        axis_acc += axis_line(p) * 0.015;
        axis_acc += axis_ticks(p) * 0.02;

        let res = map(p);
        let d = res.x;

        if (d < 0.001 * t) {{
            let n = calcNormal(p);
            let lightDir = normalize(vec3<f32>(1.0, 1.0, 1.0));
            let diff = max(dot(n, lightDir), 0.1);

            var col = vec3<f32>(0.5, 0.7, 1.0) * diff;
            col = col * exp(-0.001 * t);
            col += axis_line(p) * 0.7;
            col += axis_ticks(p) * 0.5;

            return vec4<f32>(col, 1.0);
        }}
        t = t + d;
        if (t > 2000.0) {{
            break;
        }}
    }}

    let p_far = ro + rd * min(t, 2000.0);
    var sky = mix(vec3<f32>(0.02, 0.05, 0.1), vec3<f32>(0.1, 0.2, 0.3), uv.y * 0.5 + 0.5);
    sky += axis_line(p_far) * 0.4;
    sky += axis_ticks(p_far) * 0.3;
    sky += ground_grid(p_far);
    return vec4<f32>(sky + axis_acc, 1.0);
}}
"""
