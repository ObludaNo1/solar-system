// Vertex shader

@group(0) @binding(0)
var<uniform> view_proj_mat: mat4x4<f32>;
@group(1) @binding(0)
var<uniform> model_mat: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = view_proj_mat * model_mat * vec4<f32>(model.position, 1.0);
    out.tex_coords = model.tex_coords;
    return out;
}

// Fragment shader

@group(2) @binding(0)
var tex_data: texture_2d<f32>;
@group(2) @binding(1)
var tex_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tex_data, tex_sampler, in.tex_coords);
}