use std::collections::HashMap;

use crate::{context::VisContext, utils::Guid};

use super::types::{
    BindGroup, FragmentShader, IndexBuffer, VertexBuffer, VertexLayout, VertexShader,
};

struct PrimitiveConfig {
    cull: bool,
    polygon_mode: wgpu::PolygonMode,
}

struct RenderPipelineConfig<'a> {
    //Descriptors
    vertex_shader: Guid,
    fragment_shader: Guid,
    blend: Option<wgpu::BlendState>,
    write_mask: wgpu::ColorWrites,
    vertex_buffer: Option<(&'a wgpu::Buffer, &'a wgpu::VertexBufferLayout<'a>)>,
    bind_groups: Option<&'a [&'a wgpu::BindGroupEntry<'a>]>,
    primitive_config: Option<PrimitiveConfig>,

    //Data
    index_buffer: Option<(&'a wgpu::Buffer, wgpu::IndexFormat)>,
}

impl<'a> RenderPipelineConfig<'a> {
    pub fn new(vertex_shader: impl VertexShader, fragment_shader: impl FragmentShader) -> Self {
        Self {
            vertex_shader: vertex_shader.module(),
            fragment_shader: fragment_shader.module(),
            blend: fragment_shader.blend(),
            write_mask: fragment_shader.mask(),
            vertex_buffer: None,
            bind_groups: None,
            primitive_config: None,
            index_buffer: None,
        }
    }

    pub fn with_buffer(
        mut self,
        vertex_buffer: impl VertexLayout + VertexBuffer,
        index_buffer: Option<impl IndexBuffer>,
    ) -> Self {
        self.vertex_buffer = Some((vertex_buffer.buffer(), vertex_buffer.layout()));
        self.index_buffer =
            index_buffer.map(|index_buffer| (index_buffer.buffer(), index_buffer.format()));
        self
    }

    pub fn with_bind_groups(mut self, bind_groups: &'a [impl BindGroup]) -> Self {
        self.bind_groups = Some(
            bind_groups
                .iter()
                .map(|bind_group| bind_group.entry())
                .collect::<Vec<_>>()
                .as_slice(),
        );
        self
    }

    pub fn with_config(mut self, primitive_config: PrimitiveConfig) -> Self {
        self.primitive_config = Some(primitive_config);
        self
    }
}

struct PipelineFactory {
    shader_ids: HashMap<wgpu::ShaderModule, u32>,
    cache: HashMap<u64, wgpu::RenderPipeline>,
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

/*fn shader_id(vertex: impl VertexShader, fragment: impl FragmentShader) -> u64 {
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
}*/
