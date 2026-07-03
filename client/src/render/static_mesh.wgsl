struct Camera {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
};

struct Light {
    direction: vec3<f32>,
    padding: f32,
    color: vec3<f32>,
    ambient: vec3<f32>,
};

struct Material {
    albedo: vec4<f32>,
    metallic: f32,
    roughness: f32,
    ambient_occlusion: f32,
};

@group(0) @binding(0) var<uniform> model_matrix: mat4x4<f32>;
@group(0) @binding(1) var<uniform> material: Material;

@group(1) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(1) var<uniform> light: Light;

struct VSInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VSOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_pos: vec3<f32>,
};

@vertex
fn vs_main(input: VSInput) -> VSOutput {
    let world_pos = model_matrix * vec4<f32>(input.position, 1.0);
    let world_normal = (model_matrix * vec4<f32>(input.normal, 0.0)).xyz;
    var output: VSOutput;
    output.position = camera.view_proj * world_pos;
    output.normal = normalize(world_normal);
    output.uv = input.uv;
    output.world_pos = world_pos.xyz;
    return output;
}

@fragment
fn fs_main(input: VSOutput) -> @location(0) vec4<f32> {
    let ndotl = max(dot(normalize(input.normal), normalize(-light.direction)), 0.0);
    let ambient_term = light.ambient * material.albedo.rgb;
    let diffuse_term = material.albedo.rgb * light.color * ndotl;
    let final_color = ambient_term + diffuse_term;
    return vec4<f32>(final_color, 1.0);
}
