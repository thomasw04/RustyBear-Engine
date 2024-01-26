use crate::{
    assets::buffer::{Indices, Vertices},
    context::VisContext,
};

use super::types::{IndexBuffer, Mesh, VertexBuffer, VertexLayout};

pub struct GenericMesh<'a> {
    vertices: Vertices<'a>,
    indices: Indices,
    num_indices: u32,
}

impl<'a> GenericMesh<'a> {
    pub fn new(vertices: Vertices<'a>, indices: Indices, num_indices: u32) -> Self {
        Self { vertices, indices, num_indices }
    }

    pub fn update_vertices(&mut self, context: &VisContext, contents: &[u8]) {
        self.vertices.update_buffer(context, contents);
    }

    pub fn num_indices(&self) -> u32 {
        self.num_indices
    }
}

impl<'a> Mesh for GenericMesh<'a> {}

impl<'a> IndexBuffer for GenericMesh<'a> {
    fn buffer(&self) -> Option<(&wgpu::Buffer, wgpu::IndexFormat)> {
        self.indices.buffer()
    }
}

impl<'a> VertexLayout for GenericMesh<'a> {
    fn layout(&self) -> &[wgpu::VertexBufferLayout] {
        self.vertices.layout()
    }
}

impl<'a> VertexBuffer for GenericMesh<'a> {
    fn buffer(&self) -> Option<&wgpu::Buffer> {
        self.vertices.buffer()
    }
}
