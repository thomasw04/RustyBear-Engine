use std::num::NonZeroU64;

use crate::context::VisContext;

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

    pub fn group_entry(&self, idx: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: idx,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &self.buffer,
                offset: 0,
                size: NonZeroU64::new(self.buffer.size()),
            }),
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
