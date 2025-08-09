// Vertex shader

@group(0) @binding(0)
var<uniform> view_proj_mat: mat4x4<f32>;
@group(1) @binding(0)
var<uniform> model_mat: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) colour: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) colour: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = view_proj_mat * model_mat * vec4<f32>(model.position, 1.0);
    out.colour = model.colour;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.colour, 1.0);
}