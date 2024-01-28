struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
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
    //Generate a big triangle over the screen.
    if mesh.vertex_index == 0u {
        out.clip_position = vec4<f32>(-1.0, -1.0, 0.0, 1.0); // bottom left
        out.texture_coords = vec2<f32>(0.0, 0.0);
    } else if mesh.vertex_index == 1u {
        out.clip_position = vec4<f32>(3.0, -1.0, 0.0, 1.0); // bottom right
        out.texture_coords = vec2<f32>(3.0, 0.0);
    } else {
        out.clip_position = vec4<f32>(-1.0, 3.0, 0.0, 1.0); // top left
        out.texture_coords = vec2<f32>(0.0, 3.0);
    }
    
    return out;
}

@group(0) @binding(0)
var<uniform> color: vec4<f32>;

@group(0) @binding(1)
var texture: texture_2d<f32>;
@group(0) @binding(2)
var texture_sampler: sampler;

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, in.texture_coords) * color;
}