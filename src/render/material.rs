use crate::render::types::BindGroupEntry;

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
    bind_layout: [wgpu::BindGroupLayout; 1],
    bind_group: [wgpu::BindGroup; 1],

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
                    buffer.layout_entry(0),
                    texture.layout_entry(1),
                    sampler.layout_entry(2),
                ],
            });

        let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_layout,
            entries: &[buffer.group_entry(0), texture.group_entry(1), sampler.group_entry(2)],
        });

        SkyboxMaterial {
            shader,
            bind_layout: [bind_layout],
            bind_group: [bind_group],
            buffer,
            uniform,
        }
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
    fn groups(&self) -> &[wgpu::BindGroup] {
        &self.bind_group
    }

    fn layouts(&self) -> &[wgpu::BindGroupLayout] {
        &self.bind_layout
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

pub struct GenericMaterial {
    //Shader
    shader: ShaderVariant,

    //Bind group layout and bind group
    bind_layout: [wgpu::BindGroupLayout; 1],
    bind_group: [wgpu::BindGroup; 1],
}

impl GenericMaterial {
    pub fn new(
        context: &VisContext, shader: ShaderVariant, entries: &[wgpu::BindGroupLayoutEntry],
        groups: &[wgpu::BindGroupEntry],
    ) -> Self {
        //TODO: Maybe find a better way to split the BindGroupEntries in layout and group entries. We dont want to allocate two new vectors for each material.
        let bind_layout = context
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: None, entries });

        let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_layout,
            entries: groups,
        });

        GenericMaterial { shader, bind_layout: [bind_layout], bind_group: [bind_group] }
    }
}

impl Material for GenericMaterial {}

impl BindGroup for GenericMaterial {
    fn groups(&self) -> &[wgpu::BindGroup] {
        &self.bind_group
    }

    fn layouts(&self) -> &[wgpu::BindGroupLayout] {
        &self.bind_layout
    }
}

impl FragmentShader for GenericMaterial {
    fn ptr(&self) -> &Ptr<Shader> {
        self.shader.fragment()
    }
}

impl VertexShader for GenericMaterial {
    fn ptr(&self) -> &Ptr<Shader> {
        self.shader.vertex()
    }
}
