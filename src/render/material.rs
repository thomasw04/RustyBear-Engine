use std::num::NonZeroU64;

use crate::context::VisContext;
use wgpu::util::DeviceExt;

use super::types::{CameraUniform, SplitCameraUniform};

pub struct Skybox {
    name: String,
    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    uniform: SplitCameraUniform,
}

impl Skybox {
    pub fn new(
        context: &VisContext,
        texture: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
        name: &str,
    ) -> Self {
        let uniform = SplitCameraUniform::default();
        let buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(name),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group = Skybox::create_bind_group(&buffer, context, texture, sampler, name);

        Skybox {
            name: String::from(name),
            bind_group,
            buffer,
            uniform,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn update_buffer(
        &mut self,
        context: &VisContext,
        view: [[f32; 4]; 4],
        projection: [[f32; 4]; 4],
    ) {
        self.uniform.view = view;
        self.uniform.projection = projection;
        context
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }

    pub fn create_pipeline(
        context: &VisContext,
        shader: &wgpu::ShaderModule,
    ) -> wgpu::RenderPipeline {
        context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("skybox_pipeline"),
                layout: Some(&Skybox::pipeline_layout(context)),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: context.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 4,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            })
    }

    fn create_bind_group(
        buffer: &wgpu::Buffer,
        context: &VisContext,
        texture: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
        name: &str,
    ) -> wgpu::BindGroup {
        let entries = [
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer,
                    offset: 0,
                    size: NonZeroU64::new(buffer.size()),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(texture),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ];

        context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(name),
                layout: &Skybox::create_layout(context),
                entries: &entries,
            })
    }

    fn pipeline_layout(context: &VisContext) -> wgpu::PipelineLayout {
        context
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("skybox_pipeline_layout"),
                bind_group_layouts: &[&Skybox::create_layout(context)],
                push_constant_ranges: &[],
            })
    }

    fn create_layout(context: &VisContext) -> wgpu::BindGroupLayout {
        context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::Cube,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            })
    }
}

pub struct Material {
    name: String,
    texture_count: u32,
    bind_group: wgpu::BindGroup,
    layout: wgpu::BindGroupLayout,
}

impl Material {
    pub fn new(
        context: &VisContext,
        textures: Vec<&wgpu::TextureView>,
        sampler: &wgpu::Sampler,
        name: &str,
    ) -> Material {
        let texture_count = textures.len() as u32;
        let layout = Material::create_layout(context, textures.len() as u32, name);
        let bind_group = Material::create_bind_group(context, &layout, textures, sampler, name);

        Material {
            name: String::from(name),
            texture_count,
            bind_group,
            layout,
        }
    }

    pub fn recreate_bind_group(
        &mut self,
        context: &VisContext,
        textures: Vec<&wgpu::TextureView>,
        sampler: &wgpu::Sampler,
    ) {
        self.bind_group =
            Material::create_bind_group(context, &self.layout, textures, sampler, &self.name);
    }

    fn create_bind_group(
        context: &VisContext,
        layout: &wgpu::BindGroupLayout,
        textures: Vec<&wgpu::TextureView>,
        sampler: &wgpu::Sampler,
        name: &str,
    ) -> wgpu::BindGroup {
        let mut entries = Vec::<wgpu::BindGroupEntry>::new();
        entries.reserve(textures.len() + 1);

        for i in 0..textures.len() {
            entries.push(wgpu::BindGroupEntry {
                binding: i as u32,
                resource: wgpu::BindingResource::TextureView(textures.get(i).unwrap()),
            });
        }

        entries.push(wgpu::BindGroupEntry {
            binding: textures.len() as u32,
            resource: wgpu::BindingResource::Sampler(sampler),
        });

        context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(name),
                layout,
                entries: &entries,
            })
    }

    pub fn recreate_layout(&mut self, context: &VisContext, texture_count: u32, name: &str) {
        self.layout = Material::create_layout(context, texture_count, name);
    }

    fn create_layout(
        context: &VisContext,
        texture_count: u32,
        name: &str,
    ) -> wgpu::BindGroupLayout {
        let mut entries = Vec::<wgpu::BindGroupLayoutEntry>::new();
        entries.reserve((texture_count + 1) as usize);

        for i in 0..texture_count {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: i,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            });
        }

        entries.push(wgpu::BindGroupLayoutEntry {
            binding: texture_count,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });

        context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(name),
                entries: &entries,
            })
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }
}
