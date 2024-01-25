use crate::assets::{assets::Ptr, shader::Shader};

#[repr(C)]
#[derive(wgpu_macros::VertexLayout, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex2D {
    pub position: [f32; 3],
    pub texture_coords: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteUniform {
    pub transform: [[f32; 4]; 4],
    pub color: [f32; 4],
}

impl Default for SpriteUniform {
    fn default() -> Self {
        SpriteUniform {
            transform: glam::Mat4::IDENTITY.to_cols_array_2d(),
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_projection: [[f32; 4]; 4],
}

impl Default for CameraUniform {
    fn default() -> Self {
        CameraUniform { view_projection: glam::Mat4::IDENTITY.to_cols_array_2d() }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SplitCameraUniform {
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
}

impl Default for SplitCameraUniform {
    fn default() -> Self {
        SplitCameraUniform {
            view: glam::Mat4::IDENTITY.to_cols_array_2d(),
            projection: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}

#[derive(Hash, PartialEq, Eq, Copy, Clone, Debug)]
pub struct PipelineBaseConfig {
    pub cull: bool,
    pub polygon_mode: wgpu::PolygonMode,
    pub blend: Option<wgpu::BlendState>,
    pub write_mask: wgpu::ColorWrites,
    pub samples: u32,
}

impl Default for PipelineBaseConfig {
    fn default() -> Self {
        Self {
            cull: true,
            polygon_mode: wgpu::PolygonMode::Fill,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
            samples: 4,
        }
    }
}

pub trait BindGroupEntry {
    fn group_entry(&self, binding: u32) -> wgpu::BindGroupEntry;
    fn layout_entry(&self, binding: u32) -> wgpu::BindGroupLayoutEntry;
}

pub trait BindLayout {
    fn layouts(&self) -> &[wgpu::BindGroupLayout];
}

pub trait BindGroup: BindLayout {
    fn groups(&self) -> &[wgpu::BindGroup];
}

pub trait VertexShader {
    fn ptr(&self) -> &Ptr<Shader>;
}

pub trait FragmentShader {
    fn ptr(&self) -> &Ptr<Shader>;
}

pub trait VertexLayout {
    fn layout(&self) -> &[wgpu::VertexBufferLayout];
}

pub trait VertexBuffer: VertexLayout {
    fn buffer(&self) -> Option<&wgpu::Buffer>;
}

pub trait IndexBuffer {
    fn buffer(&self) -> Option<(&wgpu::Buffer, wgpu::IndexFormat)>;
}

pub trait MaterialLayout: VertexShader + FragmentShader + BindLayout {
    fn base_config(&self) -> Option<PipelineBaseConfig>;
}
pub trait Material: MaterialLayout + BindGroup {}

pub trait Mesh: VertexBuffer + IndexBuffer {}
