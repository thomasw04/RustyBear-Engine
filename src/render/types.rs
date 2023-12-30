use crate::utils::Guid;

#[repr(C)]
#[derive(wgpu_macros::VertexLayout, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex2D {
    pub position: [f32; 3],
    pub texture_coords: [f32; 2],
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

#[derive(Hash, PartialEq, Eq, Copy, Clone, Debug)]
pub struct PipelineBaseConfig {
    pub cull: bool,
    pub polygon_mode: wgpu::PolygonMode,
    pub blend: Option<wgpu::BlendState>,
    pub write_mask: wgpu::ColorWrites,
}

impl Default for PipelineBaseConfig {
    fn default() -> Self {
        Self {
            cull: true,
            polygon_mode: wgpu::PolygonMode::Fill,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        }
    }
}

pub trait BindGroup {
    fn groups(&self) -> &[&wgpu::BindGroup];
    fn layouts(&self) -> &[&wgpu::BindGroupLayout];
}

pub trait VertexShader {
    fn guid(&self) -> Guid;
    fn module(&self) -> &wgpu::ShaderModule;
}

pub trait FragmentShader {
    fn guid(&self) -> Guid;
    fn module(&self) -> &wgpu::ShaderModule;
}

pub trait VertexBuffer {
    fn buffer(&self) -> Option<(&wgpu::Buffer, &wgpu::VertexBufferLayout)>;
}

pub trait IndexBuffer {
    fn buffer(&self) -> Option<(&wgpu::Buffer, wgpu::IndexFormat)>;
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
