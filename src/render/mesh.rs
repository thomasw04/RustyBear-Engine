use super::types::{IndexBuffer, Mesh, VertexBuffer};

pub struct GenericMesh {}

impl Mesh for GenericMesh {}

impl IndexBuffer for GenericMesh {
    fn buffer(&self) -> Option<(&wgpu::Buffer, wgpu::IndexFormat)> {
        todo!()
    }
}
impl VertexBuffer for GenericMesh {
    fn layout(&self) -> &[wgpu::VertexBufferLayout] {
        todo!()
    }

    fn buffer(&self) -> Option<&wgpu::Buffer> {
        todo!()
    }
}
