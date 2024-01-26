use std::num::NonZeroU64;

use crate::{
    context::VisContext,
    render::types::{BindGroupEntry, IndexBuffer, VertexBuffer, VertexLayout},
};

use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct UniformBuffer {
    buffer: wgpu::Buffer,
    size: usize,
}

impl UniformBuffer {
    pub fn new(context: &VisContext, size: usize) -> Self {
        let buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self { buffer, size }
    }

    pub fn update_buffer(&mut self, context: &VisContext, data: &[u8]) {
        context.queue.write_buffer(&self.buffer, 0, data);
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn layout_entry(idx: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: idx,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }
}

impl BindGroupEntry for UniformBuffer {
    fn group_entry(&self, idx: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: idx,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &self.buffer,
                offset: 0,
                size: NonZeroU64::new(self.buffer.size()),
            }),
        }
    }

    fn layout_entry(&self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        Self::layout_entry(binding)
    }
}

pub struct Vertices<'a> {
    buffer: wgpu::Buffer,
    layout: [wgpu::VertexBufferLayout<'a>; 1],
}

impl<'a> Vertices<'a> {
    pub fn new(
        context: &VisContext, contents: &[u8], layout: wgpu::VertexBufferLayout<'a>,
    ) -> Self {
        let buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents,
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self { buffer, layout: [layout] }
    }

    pub fn update_buffer(&mut self, context: &VisContext, contents: &[u8]) {
        context.queue.write_buffer(&self.buffer, 0, contents);
    }
}

impl<'a> VertexLayout for Vertices<'a> {
    fn layout(&self) -> &[wgpu::VertexBufferLayout] {
        &self.layout
    }
}

impl<'a> VertexBuffer for Vertices<'a> {
    fn buffer(&self) -> Option<&wgpu::Buffer> {
        Some(&self.buffer)
    }
}

pub struct Indices {
    buffer: wgpu::Buffer,
    format: wgpu::IndexFormat,
}

impl Indices {
    pub fn new(context: &VisContext, contents: &[u8], format: wgpu::IndexFormat) -> Self {
        let buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents,
            usage: wgpu::BufferUsages::INDEX,
        });

        Self { buffer, format }
    }
}

impl IndexBuffer for Indices {
    fn buffer(&self) -> Option<(&wgpu::Buffer, wgpu::IndexFormat)> {
        Some((&self.buffer, self.format))
    }
}
