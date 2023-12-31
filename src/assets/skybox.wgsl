struct CameraUniform {
    view: mat4x4<f32>,
    inverse_projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) eye_direction: vec3<f32>,
};


@vertex
fn vertex_main(
    mesh: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    if mesh.vertex_index == 0u {
        out.clip_position = vec4<f32>(-1.0, -1.0, 0.0, 1.0); // bottom left
    } else if mesh.vertex_index == 1u {
        out.clip_position = vec4<f32>(3.0, -1.0, 0.0, 1.0); // bottom right
    } else {
        out.clip_position = vec4<f32>(-1.0, 3.0, 0.0, 1.0); // top left
    }

    let small_view = mat3x3(camera.view[0].xyz, camera.view[1].xyz, camera.view[2].xyz);
    let inverse_view = transpose(small_view);
    let unprojected = (camera.inverse_projection * out.clip_position).xyz;
    out.eye_direction = inverse_view * unprojected;
    return out;
}

@group(0) @binding(1)
var texture: texture_cube<f32>;
@group(0) @binding(2)
var texture_sampler: sampler;

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, in.eye_direction);
}