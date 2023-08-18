

struct CameraUniform {
    view_projection: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) texture_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_coords: vec2<f32>,
};


@vertex
fn vs_main(
    mesh: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.texture_coords = mesh.texture_coords;
    out.clip_position = camera.view_projection * vec4<f32>(mesh.position, 1.0);
    return out;
}

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, in.texture_coords);
}