use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::{borrow::Cow, hash::Hasher};

use hashbrown::HashMap;

use crate::context::VisContext;
use crate::utils::Guid;

use super::types::{
    BindGroup, FragmentShader, IndexBuffer, Material, PipelineBaseConfig, VertexBuffer,
    VertexShader,
};

struct RenderPipelineConfig<'a> {
    //Descriptors
    vertex_shader: (&'a wgpu::ShaderModule, Guid),
    fragment_shader: (&'a wgpu::ShaderModule, Guid),
    vertex_buffer: Option<(&'a wgpu::Buffer, &'a wgpu::VertexBufferLayout<'a>)>,
    bind_groups: Cow<'a, [&'a wgpu::BindGroup]>,
    bind_group_layouts: Cow<'a, [&'a wgpu::BindGroupLayout]>,
    base_config: Option<PipelineBaseConfig>,
    index_buffer: Option<(&'a wgpu::Buffer, wgpu::IndexFormat)>,
}

impl<'a> RenderPipelineConfig<'a> {
    pub fn new(
        vertex_shader: &'a impl VertexShader,
        fragment_shader: &'a impl FragmentShader,
    ) -> Self {
        Self {
            vertex_shader: (vertex_shader.module(), vertex_shader.guid()),
            fragment_shader: (fragment_shader.module(), fragment_shader.guid()),
            vertex_buffer: None,
            bind_groups: Cow::from(vec![]),
            bind_group_layouts: Cow::from(vec![]),
            base_config: None,
            index_buffer: None,
        }
    }

    pub fn from_material(material: &'a impl Material) -> Self {
        Self {
            vertex_shader: (VertexShader::module(material), VertexShader::guid(material)),
            fragment_shader: (
                FragmentShader::module(material),
                FragmentShader::guid(material),
            ),
            vertex_buffer: VertexBuffer::buffer(material),
            bind_groups: Cow::from(BindGroup::groups(material)),
            bind_group_layouts: Cow::from(BindGroup::layouts(material)),
            base_config: None,
            index_buffer: IndexBuffer::buffer(material),
        }
    }

    pub fn with_buffer(
        mut self,
        vertex_buffer: &'a impl VertexBuffer,
        index_buffer: &'a impl IndexBuffer,
    ) -> Self {
        self.vertex_buffer = vertex_buffer.buffer();
        self.index_buffer = index_buffer.buffer();
        self
    }

    pub fn with_bind_groups(mut self, bind_groups: &'a [impl BindGroup]) -> Self {
        self.bind_groups = Cow::from(
            bind_groups
                .iter()
                .flat_map(|group| group.groups())
                .cloned()
                .collect::<Vec<_>>(),
        );

        self.bind_group_layouts = Cow::from(
            bind_groups
                .iter()
                .flat_map(|group| group.layouts())
                .cloned()
                .collect::<Vec<_>>(),
        );

        self
    }

    pub fn with_config(mut self, base_config: PipelineBaseConfig) -> Self {
        self.base_config = Some(base_config);
        self
    }
}

struct PipelineFactory {
    cache: HashMap<u64, Vec<wgpu::RenderPipeline>>,
    lookup: HashMap<u64, Vec<(Guid, Guid, PipelineBaseConfig)>>,
}

impl PipelineFactory {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            lookup: HashMap::new(),
        }
    }

    //Create a new pipeline.
    pub fn create(
        &self,
        context: &VisContext,
        config: RenderPipelineConfig,
    ) -> wgpu::RenderPipeline {
        let base = config.base_config.unwrap_or(PipelineBaseConfig::default());

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &config.bind_group_layouts,
                    push_constant_ranges: &[],
                });

        let buffer = if let Some(vertex_buffer) = config.vertex_buffer {
            std::slice::from_ref(vertex_buffer.1)
        } else {
            &[]
        };

        let color_state = &[Some(wgpu::ColorTargetState {
            format: context.format,
            blend: base.blend,
            write_mask: base.write_mask,
        })];

        let pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: base.cull.then_some(wgpu::Face::Back),
                polygon_mode: base.polygon_mode,
                conservative: false,
                unclipped_depth: false,
            },
            vertex: wgpu::VertexState {
                module: config.vertex_shader.0,
                entry_point: "vertex_main",
                buffers: buffer,
            },
            fragment: Some(wgpu::FragmentState {
                module: config.fragment_shader.0,
                entry_point: "fragment_main",
                targets: color_state,
            }),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        };

        context.device.create_render_pipeline(&pipeline_desc)
    }

    //Hash a pipeline config.
    fn hash_pipeline(config: &RenderPipelineConfig) -> u64 {
        let mut hasher = DefaultHasher::new();

        config.vertex_shader.1.hash(&mut hasher);
        config.fragment_shader.1.hash(&mut hasher);
        config.base_config.hash(&mut hasher);
        hasher.finish()
    }

    //Check if a pipeline is compatible with the given config.
    fn compatible_pipeline(&self, hash: u64, idx: usize, config: &RenderPipelineConfig) -> bool {
        if let Some(pipelines) = self.lookup.get(&hash) {
            if let Some(pipeline) = pipelines.get(idx) {
                if pipeline.0 == config.vertex_shader.1
                    && pipeline.1 == config.fragment_shader.1
                    && pipeline.2 == config.base_config.unwrap_or(PipelineBaseConfig::default())
                {
                    return true;
                }
            }
        }
        false
    }

    fn insert_pipeline(
        &mut self,
        hash: u64,
        context: &VisContext,
        config: RenderPipelineConfig,
    ) -> &wgpu::RenderPipeline {
        self.lookup.entry(hash).or_default().push((
            config.vertex_shader.1,
            config.fragment_shader.1,
            config.base_config.unwrap_or_default(),
        ));

        //Create a new pipeline if there are no compatible pipelines in the cache.
        let pipeline = self.create(context, config);

        //Add the pipeline to the cache and lookup table.
        self.cache.entry(hash).or_default().push(pipeline);

        //Return the newly created pipeline.
        self.cache.get(&hash).unwrap().last().unwrap()
    }

    pub fn get(
        &mut self,
        context: &VisContext,
        config: RenderPipelineConfig,
    ) -> &wgpu::RenderPipeline {
        let hash = Self::hash_pipeline(&config);

        //Weird implementation because of: https://github.com/rust-lang/rfcs/blob/master/text/2094-nll.md#problem-case-3-conditional-control-flow-across-functions
        let mut index = None;

        if let Some(pipelines) = self.cache.get(&hash) {
            for (idx, _) in pipelines.iter().enumerate() {
                if !self.compatible_pipeline(hash, idx, &config) {
                    continue;
                }

                index = Some(idx);
                break;
            }
        }

        if index.is_none() {
            return self.insert_pipeline(hash, context, config);
        }
        return self.cache.get(&hash).unwrap().get(index.unwrap()).unwrap();
    }
}
