use bytemuck::ByteHash;

#[repr(C)]
#[derive(wgpu_macros::VertexLayout, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex2D {
    pub position: [f32; 3],
    pub texture_coords: [f32; 2],
}

impl VertexLayout for Vertex2D {
    fn layout(&self) -> (&wgpu::VertexBufferLayout, u64) {
        (&Vertex2D::LAYOUT, 0)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_projection: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SplitCameraUniform {
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
}

pub trait BindGroup {
    fn bind_group_entries(&self) -> (&str, &[wgpu::BindGroupEntry]);
}

pub trait VertexShader {
    fn module(&self) -> &wgpu::ShaderModule;
}

pub trait FragmentShader {
    fn module(&self) -> &wgpu::ShaderModule;
    fn mask(&self) -> wgpu::ColorWrites;
    fn blend(&self) -> Option<wgpu::BlendState>;
}

pub trait VertexLayout {
    fn layout(&self) -> (&wgpu::VertexBufferLayout, u64);
}

pub trait VertexBuffer {
    fn buffer(&self) -> &wgpu::Buffer;
}

pub trait IndexBuffer {
    fn buffer(&self) -> &wgpu::Buffer;
    fn format(&self) -> wgpu::IndexFormat;
}

/*An unique id that can grow if the number of entities grows. Should be allocated from a pool allocator. */
pub struct Uid<'a> {
    id: &'a [u8],
}

impl PartialEq for PipelineHash {
    fn eq(&self, other: &Self) -> bool {
        todo!("Implement PartialEq for PipelineHash")
    }
}

impl Eq for PipelineHash {}

impl Default for CameraUniform {
    fn default() -> Self {
        CameraUniform {
            view_projection: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}

impl Default for SplitCameraUniform {
    fn default() -> Self {
        SplitCameraUniform {
            view: glam::Mat4::IDENTITY.to_cols_array_2d(),
            projection: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}
