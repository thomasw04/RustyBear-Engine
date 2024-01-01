use std::borrow::Cow;

use crate::{
    assets::{
        assets::Ptr,
        buffer::UniformBuffer,
        shader::{Shader, ShaderVariant},
        texture::{Sampler, TextureArray},
    },
    context::VisContext,
};

use super::types::{BindGroup, FragmentShader, Material, SplitCameraUniform, VertexShader};

pub struct SkyboxMaterial {
    //Shader
    shader: ShaderVariant,

    //Bind group layout and bind group
    bind_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,

    //Buffer and uniform
    buffer: UniformBuffer,
    uniform: SplitCameraUniform,
}

impl SkyboxMaterial {
    pub fn new(context: &VisContext, shader: ShaderVariant, texture: &TextureArray) -> Self {
        let uniform = SplitCameraUniform::default();
        let buffer = UniformBuffer::new(context, std::mem::size_of::<SplitCameraUniform>());

        let sampler = Sampler::new(context);

        let bind_layout =
            context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    UniformBuffer::layout_entry(0),
                    TextureArray::layout_entry(1),
                    Sampler::layout_entry(2),
                ],
            });

        let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_layout,
            entries: &[buffer.group_entry(0), texture.group_entry(1), sampler.group_entry(2)],
        });

        SkyboxMaterial { shader, bind_layout, bind_group, buffer, uniform }
    }

    pub fn update_buffer(
        &mut self, context: &VisContext, view: [[f32; 4]; 4], projection: [[f32; 4]; 4],
    ) {
        self.uniform.view = view;
        self.uniform.projection = projection;
        self.buffer.update_buffer(context, bytemuck::cast_slice(&[self.uniform]));
    }
}

impl Material for SkyboxMaterial {}

impl BindGroup for SkyboxMaterial {
    fn groups(&self) -> Cow<'_, [&wgpu::BindGroup]> {
        //TODO: Find a way to avoid this heap allocation.
        Cow::Owned(vec![&self.bind_group])
    }

    fn layouts(&self) -> Cow<'_, [&wgpu::BindGroupLayout]> {
        //TODO: Find a way to avoid this heap allocation.
        Cow::Owned(vec![&self.bind_layout])
    }
}

impl FragmentShader for SkyboxMaterial {
    fn ptr(&self) -> &Ptr<Shader> {
        self.shader.fragment()
    }
}

impl VertexShader for SkyboxMaterial {
    fn ptr(&self) -> &Ptr<Shader> {
        self.shader.vertex()
    }
}

/*pub struct Material {
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
}*/
