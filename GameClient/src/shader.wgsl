struct State {
    x: f32,
    y: f32,
    z: f32,
    padding: f32,
}

// We'll add a simple representation for force emitters to visualize them
struct Emitter {
    x: f32,
    y: f32,
    z: f32,
    amplitude: f32,
    sigma: f32,
}

@group(0) @binding(0)
var<uniform> state: State;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((in_vertex_index << 1) & 2u);
    let y = f32(in_vertex_index & 2u);
    out.clip_position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, y);
    return out;
}

fn map(p: vec3<f32>) -> vec2<f32> {
    let x = p.x;
    let y = p.y;
    let z = p.z;
    let sphere_pos = vec3<f32>(state.x, state.y, state.z);
    let dist = length(p - sphere_pos) - 10.0;
    return vec2<f32>(dist, 1.0);
}

fn get_force_intensity(p: vec3<f32>) -> f32 {
    // Hardcoded visualization for the default emitter at (0, 0, -20)
    let emitter_pos = vec3<f32>(0.0, 0.0, -20.0);
    let dist = length(p - emitter_pos);
    let sigma = 50.0;
    return exp(-pow(dist, 2.0) / (2.0 * pow(sigma, 2.0)));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - 1.0;
    
    let ro = vec3<f32>(0.0, 0.0, 200.0);
    let rd = normalize(vec3<f32>(uv.x, uv.y, -1.0));
    
    var t = 0.0;
    var force_acc = 0.0;
    
    for (var i = 0; i < 128; i = i + 1) {
        let p = ro + rd * t;
        let res = map(p);
        let d = res.x;
        let mat = res.y;
        
        // Accumulate force intensity for field visualization
        force_acc += get_force_intensity(p) * 0.05;

        if (d < 0.001) {
            var col = vec3<f32>(0.5, 0.8, 1.0);
            if (mat > 1.5) {
                col = vec3<f32>(1.0, 0.5, 0.2);
            }
            col = col * (1.0 - f32(i)/128.0);
            let force_glow = vec3<f32>(0.2, 0.4, 1.0) * force_acc;
            return vec4<f32>(col + force_glow, 1.0);
        }
        t = t + d;
        if (t > 1000.0) {
            break;
        }
    }
    
    // Background field visualization
    let field_col = vec3<f32>(0.0, 0.05, 0.1) * force_acc;
    return vec4<f32>(field_col, 1.0);
}
