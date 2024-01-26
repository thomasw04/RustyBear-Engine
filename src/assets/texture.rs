use once_cell::sync::OnceCell;

use crate::{context::VisContext, render::types::BindGroupEntry};

pub struct TextureArray {
    extend: wgpu::Extent3d,
    texture: wgpu::Texture,
    current_view: Option<wgpu::TextureView>,
    sampler: wgpu::Sampler,
}

impl TextureArray {
    pub fn new(context: &VisContext, size: u32, layers: u32) -> Self {
        let extend = wgpu::Extent3d { width: size, height: size, depth_or_array_layers: layers };

        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            mip_level_count: 1,
            sample_count: 1,
            size: extend,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        TextureArray { extend, texture, current_view: None, sampler }
    }

    pub fn upload_error_texture(&self, context: &VisContext, layer: u32) {
        if let Ok(image) = image::load_from_memory_with_format(
            include_bytes!("../../resources/error.png"),
            image::ImageFormat::Png,
        ) {
            let rgba = image.to_rgba8();
            self.upload(context, &rgba, layer);
        } else {
            panic!("Fatal. Error texture should always be loadable. This suggest you messed with the executable. Abort.");
        }
    }

    pub fn upload(&self, context: &VisContext, buffer: &[u8], layer: u32) {
        context.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: layer },
                aspect: wgpu::TextureAspect::All,
            },
            buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.extend.width),
                rows_per_image: Some(self.extend.height),
            },
            wgpu::Extent3d {
                width: self.extend.width,
                height: self.extend.height,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn finish_creation(&mut self) {
        self.current_view = Some(self.texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: Some(self.extend.depth_or_array_layers),
        }));
    }

    pub fn texture_view(&self) -> &wgpu::TextureView {
        self.current_view.as_ref().unwrap()
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn extend(&self) -> wgpu::Extent3d {
        self.extend
    }

    pub fn layout_entry(idx: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: idx,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::Cube,
                multisampled: false,
            },
            count: None,
        }
    }
}

impl BindGroupEntry for TextureArray {
    fn group_entry(&self, idx: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: idx,
            resource: wgpu::BindingResource::TextureView(self.texture_view()),
        }
    }

    fn layout_entry(&self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        Self::layout_entry(binding)
    }
}

pub struct Sampler {
    sampler: wgpu::Sampler,
}

impl Sampler {
    pub fn new(context: &VisContext) -> Self {
        let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self { sampler }
    }

    pub fn two_dim(context: &VisContext) -> Self {
        let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self { sampler }
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn layout_entry(idx: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: idx,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        }
    }
}

impl BindGroupEntry for Sampler {
    fn group_entry(&self, idx: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: idx,
            resource: wgpu::BindingResource::Sampler(&self.sampler),
        }
    }

    fn layout_entry(&self, idx: u32) -> wgpu::BindGroupLayoutEntry {
        Self::layout_entry(idx)
    }
}

pub struct Texture2D {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl Texture2D {
    pub fn new(
        context: &VisContext, name: Option<&str>, dim: (u32, u32), bytes: &[u8],
    ) -> Texture2D {
        let extend = wgpu::Extent3d { width: dim.0, height: dim.1, depth_or_array_layers: 1 };

        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: name,
            size: extend,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        context.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dim.0),
                rows_per_image: Some(dim.1),
            },
            extend,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Texture2D { texture, view }
    }

    pub fn error_texture(context: &VisContext) -> &Texture2D {
        static ERROR_TEXTURE: OnceCell<Texture2D> = OnceCell::new();

        ERROR_TEXTURE.get_or_init(|| {
            if let Ok(image) = image::load_from_memory_with_format(
                include_bytes!("../../resources/error.png"),
                image::ImageFormat::Png,
            ) {
                let rgba = image.to_rgba8();
                let dim = rgba.dimensions();
    
                let extend = wgpu::Extent3d { width: dim.0, height: dim.1, depth_or_array_layers: 1 };
    
                let texture = context.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("error_texture"),
                    size: extend,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
    
                context.queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &rgba,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * dim.0),
                        rows_per_image: Some(dim.1),
                    },
                    extend,
                );
    
                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    
                Texture2D { texture, view }
            } else {
                //For devs: Of course this can also happen while engine development. E.g. broken png in resources/
                panic!("Fatal. Error texture should always be loadable. This suggest you messed with the executable. Abort.");
            }
        })
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn layout_entry(idx: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: idx,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }
    }
}

impl BindGroupEntry for Texture2D {
    fn group_entry(&self, idx: u32) -> wgpu::BindGroupEntry {
        wgpu::BindGroupEntry {
            binding: idx,
            resource: wgpu::BindingResource::TextureView(self.view()),
        }
    }

    fn layout_entry(&self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        Self::layout_entry(binding)
    }
}
