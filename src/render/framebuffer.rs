use crate::context::Context;

pub struct Framebuffer {
    texture: wgpu::Texture,
    sample_count: u32,
}

impl Framebuffer {
    pub fn new(context: &Context, sample_count: u32) -> Self {
        let texture = context.graphics.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size: wgpu::Extent3d {
                width: context.surface_config.width,
                height: context.surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: context.surface_config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &context.surface_config.view_formats,
        });

        Framebuffer { texture, sample_count }
    }

    pub fn resize(&mut self, context: &Context, width: u32, height: u32) {
        if width > 0 && height > 0 {
            let samples = self.texture.sample_count();
            self.create_buffer(context, samples, width, height);
        }
    }

    pub fn change_sample_count(&mut self, context: &Context, sample_count: u32) -> bool {
        if context.features.texture_features.sample_count_supported(sample_count) {
            let width = self.texture.width();
            let height = self.texture.height();
            self.sample_count = sample_count;
            self.create_buffer(context, sample_count, width, height);
            return true;
        }

        false
    }

    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }

    fn create_buffer(&mut self, context: &Context, sample_count: u32, width: u32, height: u32) {
        self.texture = context.graphics.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: context.surface_config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &context.surface_config.view_formats,
        });
    }
}

impl From<&Framebuffer> for wgpu::TextureView {
    fn from(value: &Framebuffer) -> Self {
        value.texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
}
