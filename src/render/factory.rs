use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use crate::assets::assets::Assets;
use crate::assets::assets::GenPtr;
use crate::assets::shader::ShaderVariant;
use crate::context::VisContext;
use crate::utils::Guid;
use hashbrown::HashMap;
use smallvec::SmallVec;

use super::types::BindLayout;
use super::types::PipelineBaseConfig;
use super::types::VertexLayout;

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
struct PipelineConfigKey {
    vertex: Guid,
    fragment: Guid,
    base_config: PipelineBaseConfig,
}

pub struct RenderPipelineConfig<'a> {
    pub vertex_shader: &'a wgpu::ShaderModule,
    pub fragment_shader: &'a wgpu::ShaderModule,
    pub vertex_layout: &'a [wgpu::VertexBufferLayout<'a>],
    pub bind_layouts: SmallVec<[&'a wgpu::BindGroupLayout; 16]>,
    key: PipelineConfigKey,
}

impl<'a> RenderPipelineConfig<'a> {
    pub fn new(
        shader: &'a ShaderVariant<'a>, vertex_layout: Option<&'a impl VertexLayout>,
        bind_layout: &'a impl BindLayout, addi: &[&'a wgpu::BindGroupLayout],
    ) -> RenderPipelineConfig<'a> {
        let vertex_layout = vertex_layout.map(|a| a.layout()).unwrap_or(&[]);
        let mut bind_layouts = SmallVec::<[&'a wgpu::BindGroupLayout; 16]>::new();

        for layout in bind_layout.layouts() {
            bind_layouts.push(layout);
        }

        bind_layouts.extend_from_slice(addi);

        Self {
            vertex_shader: shader.vertex().module(),
            fragment_shader: shader.fragment().module(),
            vertex_layout,
            bind_layouts,
            key: PipelineConfigKey {
                vertex: shader.vertex_id().inner(),
                fragment: shader.fragment_id().inner(),
                base_config: PipelineBaseConfig::default(),
            },
        }
    }

    pub fn set_config(&mut self, base_config: PipelineBaseConfig) {
        self.key.base_config = base_config
    }
}

pub struct RenderPipelineBuilder<'a> {
    shader: &'a ShaderVariant<'a>,
    vertex_layout: &'a [wgpu::VertexBufferLayout<'a>],
    bind_layouts: SmallVec<[&'a wgpu::BindGroupLayout; 16]>,
    base_config: PipelineBaseConfig,
}

impl<'a> RenderPipelineBuilder<'a> {
    pub fn new(shader: &'a ShaderVariant<'a>) -> Self {
        Self {
            shader,
            vertex_layout: &[],
            bind_layouts: SmallVec::new(),
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
        self.bind_layouts = SmallVec::from(bind_layouts);
        self
    }

    pub fn build(self) -> RenderPipelineConfig<'a> {
        RenderPipelineConfig {
            vertex_shader: self.shader.vertex().module(),
            fragment_shader: self.shader.fragment().module(),
            vertex_layout: self.vertex_layout,
            bind_layouts: self.bind_layouts,
            key: PipelineConfigKey {
                vertex: self.shader.vertex_id().inner(),
                fragment: self.shader.fragment_id().inner(),
                base_config: self.base_config,
            },
        }
    }
}

pub struct PipelineFactory {
    cache: HashMap<PipelineConfigKey, wgpu::RenderPipeline>,
}

impl Default for PipelineFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineFactory {
    pub fn new() -> Self {
        Self { cache: HashMap::new() }
    }

    pub fn get(&self, config: &RenderPipelineConfig) -> Option<&wgpu::RenderPipeline> {
        self.cache.get(&config.key)
    }

    pub fn get_or_create(
        &mut self, context: &VisContext, config: &RenderPipelineConfig,
    ) -> &wgpu::RenderPipeline {
        self.cache.entry(config.key).or_insert_with(|| PipelineFactory::create(context, config))
    }

    fn create(context: &VisContext, config: &RenderPipelineConfig) -> wgpu::RenderPipeline {
        let pipeline_layout =
            context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &config.bind_layouts,
                push_constant_ranges: &[],
            });

        let color_state = &[Some(wgpu::ColorTargetState {
            format: context.format,
            blend: config.key.base_config.blend,
            write_mask: config.key.base_config.write_mask,
        })];

        let pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: config.key.base_config.cull.then_some(wgpu::Face::Back),
                polygon_mode: config.key.base_config.polygon_mode,
                conservative: false,
                unclipped_depth: false,
            },
            vertex: wgpu::VertexState {
                module: config.vertex_shader,
                entry_point: "vertex_main",
                buffers: config.vertex_layout,
            },
            fragment: Some(wgpu::FragmentState {
                module: config.fragment_shader,
                entry_point: "fragment_main",
                targets: color_state,
            }),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: config.key.base_config.samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        };

        context.device.create_render_pipeline(&pipeline_desc)
    }
}

pub struct BindGroupConfig<'a> {
    entries: &'a [GenPtr],
}

impl<'a> BindGroupConfig<'a> {
    pub fn new(entries: &'a [GenPtr]) -> Self {
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
        &self, context: &VisContext, assets: &mut Assets, config: &BindGroupConfig,
    ) -> Option<wgpu::BindGroup> {
        let mut layout_entries = SmallVec::<[wgpu::BindGroupLayoutEntry; 16]>::new();
        let mut group_entries = SmallVec::<[wgpu::BindGroupEntry; 16]>::new();

        for entry in config.entries.iter() {
            assets.wait_for(entry);
        }

        for (i, entry) in config.entries.iter().enumerate() {
            if let Some(asset) = assets.try_get_entry(entry) {
                group_entries.push(asset.group_entry(i as u32));
                layout_entries.push(asset.layout_entry(i as u32));
            } else {
                return None;
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

    pub fn prepare(&mut self, context: &VisContext, assets: &mut Assets, config: &BindGroupConfig) {
        let _ = self.get(context, assets, config);
    }

    pub fn try_get(&self, config: &BindGroupConfig) -> Option<&wgpu::BindGroup> {
        let mut hasher = DefaultHasher::new();
        config.entries.hash(&mut hasher);
        let hash = hasher.finish();

        if let Some(bind_groups) = self.lookup.get(&hash) {
            for (idx, bind_group) in bind_groups.iter().enumerate() {
                if self.is_compatible(bind_group.as_slice(), config) {
                    return self.cache.get(&hash).unwrap().get(idx);
                }
            }
        }

        None
    }

    pub fn get(
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
            self.lookup.entry(hash).or_default().push(config.entries.to_vec());
        } else {
            panic!("Failed to create bind group");
        }

        self.cache.get(&hash).unwrap().last().unwrap()
    }
}
