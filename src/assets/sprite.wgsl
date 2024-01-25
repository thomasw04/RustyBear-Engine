

struct CameraUniform {
    view_projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> transform: mat4x4<f32>;

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
fn vertex_main(
    mesh: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.texture_coords = mesh.texture_coords;
    out.clip_position = camera.view_projection * transform * vec4<f32>(mesh.position, 1.0);
    return out;
}

@group(0) @binding(1)
var<uniform> color: vec4<f32>;

@group(0) @binding(2)
var texture: texture_2d<f32>;
@group(0) @binding(3)
var texture_sampler: sampler;

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, in.texture_coords) * color;
}