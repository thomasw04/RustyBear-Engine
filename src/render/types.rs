#[repr(C)]
#[derive(wgpu_macros::VertexLayout, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex2D {
    pub position: [f32; 3],
    pub texture_coords: [f32; 2],
}

impl VertexLayout for Vertex2D {
    fn layout(&self) -> Option<&wgpu::VertexBufferLayout> {
        Some(&Vertex2D::LAYOUT)
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
    fn entries(&self) -> &[&wgpu::BindGroupEntry];
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
    fn layout(&self) -> Option<&wgpu::VertexBufferLayout>;
}

pub trait VertexBuffer: VertexLayout {
    fn buffer(&self) -> Option<&wgpu::Buffer>;
}

pub trait IndexBuffer {
    fn buffer(&self) -> Option<&wgpu::Buffer>;
    fn format(&self) -> Option<wgpu::IndexFormat>;
}

pub trait Material: VertexShader + FragmentShader + VertexBuffer + IndexBuffer + BindGroup {}

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
