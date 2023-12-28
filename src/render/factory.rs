use std::{ascii::AsciiExt, collections::HashMap};

use crate::{context::VisContext, utils::Guid};

use super::types::{
    BindGroup, FragmentShader, IndexBuffer, PipelineHash, VertexBuffer, VertexShader,
};

struct PipelineConfig<'a> {
    vertex_buffer: wgpu::Buffer,
    index_buffer: Option<wgpu::Buffer>,
    bind_groups: Vec<wgpu::BindGroup>,
    pipeline_layout_desc: wgpu::PipelineLayoutDescriptor<'a>,
    pipeline_desc: wgpu::RenderPipelineDescriptor<'a>,
    uid: PipelineHash,
}

impl<'a> PipelineConfig<'a> {
    pub fn new() -> Self {
        todo!("Implement PipelineConfig::new")
    }

    pub fn with_shaders(vertex: impl VertexShader, fragment: impl FragmentShader) -> Self {
        todo!("Implement PipelineConfig::with_shaders")
    }

    /* T must be a type that derives a wgpu_macros::VertexLayout */
    pub fn with_buffer(
        vertex_buffer: impl VertexBuffer,
        index_buffer: Option<impl IndexBuffer>,
    ) -> Self {
        todo!("Implement PipelineConfig::with_vertex_buffer")
    }

    pub fn with_bind_groups(bind_groups: &[impl BindGroup]) -> Self {
        todo!("Implement PipelineConfig::with_bind_groups")
    }

    pub fn with_primitive_state(primitive_state: wgpu::PrimitiveState) -> Self {
        todo!("Implement PipelineConfig::with_primitive_state")
    }

    pub fn with_multisampled(&mut self, multisampled: Option<u32>) -> Self {
        todo!("Implement PipelineConfig::multisampled")
    }
}

struct PipelineFactory {
    cache: HashMap<PipelineHash, wgpu::RenderPipeline>,
}

impl<'a> PipelineFactory {
    pub fn new(context: &VisContext) -> Self {
        todo!("Implement PipelineFactory::new")
    }

    /*Returns a pipeline for the given config or creates a new one if not exists. */
    pub fn get(
        &'a mut self,
        config: &PipelineConfig<'a>,
        context: &VisContext,
    ) -> &'a wgpu::RenderPipeline {
        if self.cache.contains_key(&config.uid) {
            return self.cache.get(&config.uid).unwrap();
        } else {
            let pipeline_layout = context
                .device
                .create_pipeline_layout(&config.pipeline_layout_desc);

            let mut pipeline_desc = config.pipeline_desc.clone();
            pipeline_desc.layout = Some(&pipeline_layout);

            let pipeline = context.device.create_render_pipeline(&pipeline_desc);

            self.cache.insert(config.uid, pipeline);

            self.cache.get(&config.uid).unwrap()
        }
    }
}

fn shader_id(vertex: impl VertexShader, fragment: impl FragmentShader) -> u64 {
    // 2xu64 + 1xu64
}

fn buffer_id(vertex_buffer: impl VertexBuffer, index_buffer: Option<impl IndexBuffer>) -> u64 {
    // 1xu64
}

fn bind_group_id(bind_groups: &[impl BindGroup]) -> u64 {
    // 1xu64
}

fn primitive_state_id(primitive_state: wgpu::PrimitiveState) -> u64 {
    // 1xu64
}

fn multisampled_id(multisampled: Option<u32>) -> u64 {
    // 1xu32
}

fn gen_pipeline_id(config: &PipelineConfig) -> u64 {
    config.

    todo!("Implement hash_pipeline")
}
