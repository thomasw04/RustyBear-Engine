use crate::{
    assets::buffer::{Indices, Vertices},
    context::VisContext,
};

use super::types::{IndexBuffer, Mesh, Rect, Vertex2D, VertexBuffer, VertexLayout};

pub struct GenericMesh<'a> {
    vertices: Vertices<'a>,
    indices: Indices,
    num_indices: u64,
}

impl<'a> GenericMesh<'a> {
    pub fn new(vertices: Vertices<'a>, indices: Indices, num_indices: u64) -> Self {
        Self { vertices, indices, num_indices }
    }

    pub fn update_vertices(&mut self, context: &VisContext, offset: u64, contents: &[u8]) {
        self.vertices.update(context, offset, contents);
    }

    pub fn num_indices(&self) -> u64 {
        self.num_indices
    }

    pub fn size(&self) -> u64 {
        self.vertices.size()
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

pub struct Batch2D {
    mesh: GenericMesh<'static>,
}

impl Batch2D {
    pub fn from(context: &VisContext, rects: &[Rect]) -> Self {
        let content = bytemuck::cast_slice(rects);
        let indices =
            Indices::for_rect_array(context, rects.len() as u64, wgpu::IndexFormat::Uint16);

        Self {
            mesh: GenericMesh::new(
                Vertices::from(context, content, Vertex2D::LAYOUT),
                indices,
                rects.len() as u64 * 6,
            ),
        }
    }

    pub fn new(context: &VisContext, cnt: u64) -> Self {
        let content = vec![0; cnt as usize * std::mem::size_of::<Vertex2D>()];
        let indices = Indices::for_rect_array(context, cnt * 6, wgpu::IndexFormat::Uint16);

        Self {
            mesh: GenericMesh::new(
                Vertices::from(context, content.as_slice(), Vertex2D::LAYOUT),
                indices,
                0,
            ),
        }
    }

    /// This function adds a new rectangle to the batch.
    /// This is a lightweight operation and does not require any reallocation.
    /// Returns true if the rectangle was added successfully.
    /// Return false if the batch is full.
    pub fn add(&mut self, context: &VisContext, rect: Rect) -> bool {
        if self.mesh.size() <= self.mesh.num_indices() {
            return false;
        }

        let content = bytemuck::cast_slice(&[rect]);
        self.mesh.update_vertices(context, self.size(), content);
        self.mesh.num_indices += 6;
        return true;
    }

    pub fn replace(&mut self, context: &VisContext, rects: &[Rect]) {
        if rects.len() > self.mesh.size() as usize {
            self.mesh = GenericMesh::new(
                Vertices::from(context, bytemuck::cast_slice(rects), Vertex2D::LAYOUT),
                Indices::for_rect_array(context, rects.len() as u64, wgpu::IndexFormat::Uint16),
                rects.len() as u64 * 6,
            );
        } else {
            let content = bytemuck::cast_slice(rects);
            self.mesh.update_vertices(context, 0, content);
            self.mesh.num_indices = rects.len() as u64 * 6;
        }
    }

    pub fn update(&mut self, context: &VisContext, idx: u64, rect: Rect) {
        let content = bytemuck::cast_slice(&[rect]);
        self.mesh.update_vertices(context, idx * std::mem::size_of::<Vertex2D>() as u64, content);
    }

    pub fn update_bytes(&mut self, context: &VisContext, idx: u64, offset: u64, bytes: &[u8]) {
        self.mesh.update_vertices(
            context,
            idx * std::mem::size_of::<Vertex2D>() as u64 + offset,
            bytes,
        );
    }

    pub fn size(&self) -> u64 {
        self.size()
    }
}
