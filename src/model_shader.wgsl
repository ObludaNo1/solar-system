// Vertex shader

@group(0) @binding(0)
var<uniform> view_proj_mat: mat4x4<f32>;
@group(1) @binding(0)
var<uniform> model_mat: mat4x4<f32>;
@group(1) @binding(1)
var<uniform> normal_mat: mat3x3<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    var world_position = model_mat * vec4<f32>(model.position, 1.0);
    out.clip_position = view_proj_mat * world_position;
    // ?? world position.z should be always 1, since model matrix should not change W
    out.position = world_position.xyz / world_position.w;
    out.tex_coords = model.tex_coords;
    out.normal = normal_mat * model.normal;
    return out;
}

// Fragment shader

const LIGHT_POS: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);

@group(2) @binding(0)
var tex_data: texture_2d<f32>;
@group(2) @binding(1)
var tex_sampler: sampler;
// // TODO lightning coefficients
// @group(2) @binding(2)
// var ambient_coef: f32;
// @group(2) @binding(3)
// var diffuse_coef: f32;
// @group(2) @binding(4)
// var specular_coef: f32;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var normal = normalize(in.normal);
    var light_dir = normalize(LIGHT_POS - in.position);
    var diffuse = max(dot(normal, light_dir), 0.0);

    var texel = textureSample(tex_data, tex_sampler, in.tex_coords);
    return vec4<f32>(texel.rgb * diffuse, texel.a);
}