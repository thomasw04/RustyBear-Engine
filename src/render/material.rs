use crate::context::Context;

pub struct Material {
    name: String,
    texture_count: u32,
    bind_group: wgpu::BindGroup,
    layout: wgpu::BindGroupLayout,
}

impl Material {
    pub fn new(
        context: &Context,
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
        context: &Context,
        textures: Vec<&wgpu::TextureView>,
        sampler: &wgpu::Sampler,
    ) {
        self.bind_group =
            Material::create_bind_group(context, &self.layout, textures, sampler, &self.name);
    }

    fn create_bind_group(
        context: &Context,
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

    pub fn recreate_layout(&mut self, context: &Context, texture_count: u32, name: &str) {
        self.layout = Material::create_layout(context, texture_count, name);
    }

    fn create_layout(context: &Context, texture_count: u32, name: &str) -> wgpu::BindGroupLayout {
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
