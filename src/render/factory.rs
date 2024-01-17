use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use crate::assets::assets::Assets;
use crate::assets::assets::GenPtr;
use crate::assets::assets::Ptr;
use crate::assets::shader::Shader;
use crate::context::VisContext;
use crate::utils::Guid;
use hashbrown::HashMap;
use smallvec::SmallVec;

use super::types::MaterialLayout;
use super::types::{FragmentShader, Mesh, PipelineBaseConfig, VertexBuffer, VertexShader};

pub struct RenderPipelineConfig<'a> {
    pub vertex_shader: &'a Ptr<Shader>,
    pub fragment_shader: &'a Ptr<Shader>,
    pub vertex_layout: &'a [wgpu::VertexBufferLayout<'a>],
    pub bind_layouts: Vec<&'a wgpu::BindGroupLayout>,
    pub base_config: PipelineBaseConfig,
}

impl<'a> RenderPipelineConfig<'a> {
    //TODO: make additional bind groups as the camera more generic.
    pub fn new(
        material: &'a impl MaterialLayout, mesh: Option<&'a impl Mesh>,
        config: Option<PipelineBaseConfig>, camera_layout: Option<&'a wgpu::BindGroupLayout>,
    ) -> Self {
        let vertex_layout = mesh.map(|m| VertexBuffer::layout(m)).unwrap_or(&[]);

        //This should be something like a stack allocated container as it will never be bigger than a few.
        let mut bind_layouts = Vec::with_capacity(1 + material.layouts().len());

        for layout in material.layouts() {
            bind_layouts.push(layout);
        }

        if let Some(camera_layout) = camera_layout {
            bind_layouts.push(camera_layout);
        }

        Self {
            vertex_shader: VertexShader::ptr(material),
            fragment_shader: FragmentShader::ptr(material),
            vertex_layout,
            bind_layouts,
            base_config: config.unwrap_or_default(),
        }
    }
}

pub struct RenderPipelineBuilder<'a> {
    vertex_shader: &'a Ptr<Shader>,
    fragment_shader: &'a Ptr<Shader>,
    vertex_layout: &'a [wgpu::VertexBufferLayout<'a>],
    bind_layouts: Vec<&'a wgpu::BindGroupLayout>,
    base_config: PipelineBaseConfig,
}

impl<'a> RenderPipelineBuilder<'a> {
    pub fn new(
        vertex_shader: &'a impl VertexShader, fragment_shader: &'a impl FragmentShader,
    ) -> Self {
        Self {
            vertex_shader: vertex_shader.ptr(),
            fragment_shader: fragment_shader.ptr(),
            vertex_layout: &[],
            bind_layouts: Vec::with_capacity(0),
            base_config: PipelineBaseConfig::default(),
        }
    }

    pub fn with_config(mut self, base_config: PipelineBaseConfig) -> Self {
        self.base_config = base_config;
        self
    }

    pub fn with_vertex_buffer(mut self, vertex_layout: &'a [wgpu::VertexBufferLayout<'a>]) -> Self {
        self.vertex_layout = vertex_layout;
        self
    }

    pub fn with_bind_groups(mut self, bind_layouts: &[&'a wgpu::BindGroupLayout]) -> Self {
        self.bind_layouts = Vec::from(bind_layouts);
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

pub struct PipelineResult<'a>(
    Result<&'a wgpu::RenderPipeline, (&'a mut PipelineFactory, &'a RenderPipelineConfig<'a>)>,
);

impl<'a> PipelineResult<'a> {
    pub fn or_create(self, context: &VisContext, assets: &mut Assets) -> &'a wgpu::RenderPipeline {
        match self.0 {
            Ok(pipeline) => pipeline,
            Err((factory, config)) => factory.insert(context, assets, &config),
        }
    }
}

pub struct PipelineFactory {
    cache: HashMap<u64, Vec<wgpu::RenderPipeline>>,
    lookup: HashMap<u64, Vec<(Guid, Guid, PipelineBaseConfig)>>,
}

impl Default for PipelineFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineFactory {
    pub fn new() -> Self {
        Self { cache: HashMap::new(), lookup: HashMap::new() }
    }

    pub fn get(&self, config: &RenderPipelineConfig) -> PipelineResult {
        let hash = Self::hash_pipeline(config);

        if let Some(pipelines) = self.cache.get(&hash) {
            for (idx, pipeline) in pipelines.iter().enumerate() {
                if !self.compatible_pipeline(hash, idx, config) {
                    continue;
                }

                return PipelineResult(Ok(pipeline));
            }
        }

        PipelineResult(Err(config))
    }

    pub fn insert(
        &mut self, context: &VisContext, assets: &mut Assets, config: &RenderPipelineConfig,
    ) -> &wgpu::RenderPipeline {
        let hash = Self::hash_pipeline(config);
        //Create a new pipeline if there are no compatible pipelines in the cache.
        let pipeline = self.create(context, assets, config, true);

        //Add the pipeline to the cache and lookup table.
        if let Some(pipeline) = pipeline {
            self.lookup.entry(hash).or_default().push((
                config.vertex_shader.inner(),
                config.fragment_shader.inner(),
                config.base_config,
            ));

            self.cache.entry(hash).or_default().push(pipeline);
        }

        //Return the newly created pipeline.
        self.cache.get(&hash).unwrap().last().unwrap()
    }

    //Create a new pipeline. Returns None if not all assets where loaded.
    //If wait is true waits until the asset is fully loaded. Returns None if there was an error.
    fn create(
        &self, context: &VisContext, assets: &mut Assets, config: &RenderPipelineConfig, wait: bool,
    ) -> Option<wgpu::RenderPipeline> {
        let pipeline_layout =
            context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &config.bind_layouts,
                push_constant_ranges: &[],
            });

        let color_state = &[Some(wgpu::ColorTargetState {
            format: context.format,
            blend: config.base_config.blend,
            write_mask: config.base_config.write_mask,
        })];

        if wait {
            assets.wait_for(config.vertex_shader);
            assets.wait_for(config.fragment_shader);
        }

        let vertex_shader = assets.try_get(config.vertex_shader);
        let fragment_shader = assets.try_get(config.fragment_shader);

        match (vertex_shader, fragment_shader) {
            (Some(vertex_shader), Some(fragment_shader)) => {
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
                        module: vertex_shader.module(),
                        entry_point: "vertex_main",
                        buffers: config.vertex_layout,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: fragment_shader.module(),
                        entry_point: "fragment_main",
                        targets: color_state,
                    }),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: config.base_config.samples,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                };

                Some(context.device.create_render_pipeline(&pipeline_desc))
            }
            _ => None,
        }
    }

    //Hash a pipeline config.
    fn hash_pipeline(config: &RenderPipelineConfig) -> u64 {
        let mut hasher = DefaultHasher::new();

        config.vertex_shader.hash(&mut hasher);
        config.fragment_shader.hash(&mut hasher);
        config.base_config.hash(&mut hasher);
        hasher.finish()
    }

    //Check if a pipeline is compatible with the given config.
    fn compatible_pipeline(&self, hash: u64, idx: usize, config: &RenderPipelineConfig) -> bool {
        if let Some(pipelines) = self.lookup.get(&hash) {
            if let Some(pipeline) = pipelines.get(idx) {
                if pipeline.0 == config.vertex_shader.inner()
                    && pipeline.1 == config.fragment_shader.inner()
                    && pipeline.2 == config.base_config
                {
                    return true;
                }
            }
        }
        false
    }
}

pub struct BindGroupConfig<'a> {
    entries: &'a [GenPtr],
}

impl<'a> BindGroupConfig<'a> {
    pub fn new(entries: &[GenPtr]) -> Self {
        Self { entries }
    }
}

pub struct BindGroupFactory {
    cache: HashMap<u64, Vec<wgpu::BindGroup>>,
    lookup: HashMap<u64, Vec<Vec<GenPtr>>>,
}

impl Default for BindGroupFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl BindGroupFactory {
    pub fn new() -> Self {
        Self { cache: HashMap::new(), lookup: HashMap::new() }
    }

    fn is_compatible(&self, target: &[GenPtr], config: &BindGroupConfig) -> bool {
        if target.len() != config.entries.len() {
            return false;
        }

        for (idx, entry) in config.entries.iter().enumerate() {
            if target[idx] != *entry {
                return false;
            }
        }

        true
    }

    fn create(
        &self, context: &VisContext, assets: &Assets, config: &BindGroupConfig,
    ) -> Option<wgpu::BindGroup> {
        let mut layout_entries = SmallVec::<[wgpu::BindGroupLayoutEntry; 16]>::new();
        let mut group_entries = SmallVec::<[wgpu::BindGroupEntry; 16]>::new();

        for (i, entry) in config.entries.iter().enumerate() {
            if let Some(asset) = assets.try_get_entry(entry) {
                group_entries.push(asset.group_entry(i as u32));
                layout_entries.push(asset.layout_entry(i as u32));
            }
        }

        let layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &layout_entries,
        });

        let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &group_entries,
        });

        Some(bind_group)
    }

    pub fn get_for(
        &mut self, context: &VisContext, assets: &mut Assets, config: &BindGroupConfig,
    ) -> &wgpu::BindGroup {
        let mut hasher = DefaultHasher::new();
        config.entries.hash(&mut hasher);
        let hash = hasher.finish();

        if let Some(bind_groups) = self.lookup.get(&hash) {
            for (idx, bind_group) in bind_groups.iter().enumerate() {
                if self.is_compatible(bind_group.as_slice(), config) {
                    return self.cache.get(&hash).unwrap().get(idx).unwrap();
                }
            }
        }

        let bind_group = self.create(context, assets, config);

        if let Some(bind_group) = bind_group {
            self.cache.entry(hash).or_default().push(bind_group);
        }

        self.cache.get(&hash).unwrap().last().unwrap()
    }
}
