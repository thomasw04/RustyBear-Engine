use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use hashbrown::HashMap;

use crate::context::VisContext;
use crate::utils::Guid;

use super::types::{
    BindGroup, FragmentShader, Material, Mesh, PipelineBaseConfig, VertexBuffer, VertexShader,
};

struct RenderPipelineConfig<'a> {
    pub vertex_shader: (&'a wgpu::ShaderModule, Guid),
    pub fragment_shader: (&'a wgpu::ShaderModule, Guid),
    pub vertex_layout: &'a [wgpu::VertexBufferLayout<'a>],
    pub bind_layouts: &'a [&'a wgpu::BindGroupLayout],
    pub base_config: PipelineBaseConfig,
}

struct RenderPipelineBuilder<'a> {
    vertex_shader: (&'a wgpu::ShaderModule, Guid),
    fragment_shader: (&'a wgpu::ShaderModule, Guid),
    vertex_layout: &'a [wgpu::VertexBufferLayout<'a>],
    bind_layouts: &'a [&'a wgpu::BindGroupLayout],
    base_config: PipelineBaseConfig,
}

impl<'a> RenderPipelineConfig<'a> {
    pub fn new(
        material: &'a impl Material,
        mesh: &'a impl Mesh,
        config: Option<PipelineBaseConfig>,
    ) -> Self {
        Self {
            vertex_shader: (VertexShader::module(material), VertexShader::guid(material)),
            fragment_shader: (
                FragmentShader::module(material),
                FragmentShader::guid(material),
            ),
            vertex_layout: VertexBuffer::layout(mesh),
            bind_layouts: BindGroup::layouts(material),
            base_config: config.unwrap_or_default(),
        }
    }
}

impl<'a> RenderPipelineBuilder<'a> {
    pub fn new(
        vertex_shader: &'a impl VertexShader,
        fragment_shader: &'a impl FragmentShader,
    ) -> Self {
        Self {
            vertex_shader: (vertex_shader.module(), vertex_shader.guid()),
            fragment_shader: (fragment_shader.module(), fragment_shader.guid()),
            vertex_layout: &[],
            bind_layouts: &[],
            base_config: PipelineBaseConfig::default(),
        }
    }

    pub fn with_config(mut self, base_config: PipelineBaseConfig) -> Self {
        self.base_config = base_config;
        self
    }

    pub fn with_vertex_buffer(mut self, vertex_layout: &[wgpu::VertexBufferLayout<'a>]) -> Self {
        self.vertex_layout = vertex_layout;
        self
    }

    pub fn with_bind_groups(mut self, bind_layouts: &'a [&'a wgpu::BindGroupLayout]) -> Self {
        self.bind_layouts = bind_layouts;
        self
    }

    pub fn build(self) -> RenderPipelineConfig<'a> {
        RenderPipelineConfig {
            vertex_shader: self.vertex_shader,
            fragment_shader: self.fragment_shader,
            vertex_layout: self.vertex_layout,
            bind_layouts: self.bind_layouts,
            base_config: self.base_config,
        }
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

    pub fn for_object(
        &mut self,
        context: &VisContext,
        material: &impl Material,
        mesh: &impl Mesh,
        config: Option<PipelineBaseConfig>,
    ) -> &wgpu::RenderPipeline {
        self.get_for(context, &RenderPipelineConfig::new(material, mesh, config))
    }

    pub fn get_for(
        &mut self,
        context: &VisContext,
        config: &RenderPipelineConfig,
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

    //Create a new pipeline.
    fn create(&self, context: &VisContext, config: &RenderPipelineConfig) -> wgpu::RenderPipeline {
        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: config.bind_layouts,
                    push_constant_ranges: &[],
                });

        let pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: config.base_config.cull.then_some(wgpu::Face::Back),
                polygon_mode: config.base_config.polygon_mode,
                conservative: false,
                unclipped_depth: false,
            },
            vertex: wgpu::VertexState {
                module: config.vertex_shader.0,
                entry_point: "vertex_main",
                buffers: config.vertex_layout,
            },
            fragment: Some(wgpu::FragmentState {
                module: config.fragment_shader.0,
                entry_point: "fragment_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: context.format,
                    blend: config.base_config.blend,
                    write_mask: config.base_config.write_mask,
                })],
            }),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: config.base_config.samples,
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
                    && pipeline.2 == config.base_config
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
        config: &RenderPipelineConfig,
    ) -> &wgpu::RenderPipeline {
        self.lookup.entry(hash).or_default().push((
            config.vertex_shader.1,
            config.fragment_shader.1,
            config.base_config,
        ));

        //Create a new pipeline if there are no compatible pipelines in the cache.
        let pipeline = self.create(context, config);

        //Add the pipeline to the cache and lookup table.
        self.cache.entry(hash).or_default().push(pipeline);

        //Return the newly created pipeline.
        self.cache.get(&hash).unwrap().last().unwrap()
    }
}
