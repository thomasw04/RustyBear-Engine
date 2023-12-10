use crate::context::VisContext;

pub struct TextureArray {
    extend: wgpu::Extent3d,
    texture: wgpu::Texture,
}

impl TextureArray {
    pub fn new(context: &VisContext, size: u32, layers: u32) -> Self {
        let extend = wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: layers,
        };

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

        TextureArray { extend, texture }
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
            wgpu::ImageCopyTextureBase {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: layer,
                },
                aspect: wgpu::TextureAspect::All,
            },
            buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.extend.width),
                rows_per_image: Some(self.extend.height),
            },
            self.extend,
        );
    }

    pub fn texture_view(&self) -> wgpu::TextureView {
        self.texture
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn extend(&self) -> wgpu::Extent3d {
        self.extend
    }
}

pub struct Texture2D {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl Texture2D {
    pub fn new(
        context: &VisContext,
        name: Option<&str>,
        bytes: &[u8],
        format: image::ImageFormat,
    ) -> Result<Texture2D, Texture2D> {
        if let Ok(image) = image::load_from_memory_with_format(bytes, format) {
            //Potentially also support other color formats e.g. rgba16 (Need to do more research on this)
            let rgba = image.to_rgba8();
            let dim = rgba.dimensions();

            let extend = wgpu::Extent3d {
                width: dim.0,
                height: dim.1,
                depth_or_array_layers: 1,
            };

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
                &rgba,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * dim.0),
                    rows_per_image: Some(dim.1),
                },
                extend,
            );

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

            Ok(Texture2D {
                texture,
                view,
                sampler,
            })
        } else {
            log::error!(
                "Failed to parse image {}. Did you choose a supported format?",
                name.unwrap_or("[UNKNOWN_IMAGE]")
            );

            Err(Texture2D::error_texture(context))
        }
    }

    pub fn error_texture(context: &VisContext) -> Texture2D {
        if let Ok(image) = image::load_from_memory_with_format(
            include_bytes!("../../resources/error.png"),
            image::ImageFormat::Png,
        ) {
            let rgba = image.to_rgba8();
            let dim = rgba.dimensions();

            let extend = wgpu::Extent3d {
                width: dim.0,
                height: dim.1,
                depth_or_array_layers: 1,
            };

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

            let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

            Texture2D {
                texture,
                view,
                sampler,
            }
        } else {
            //For devs: Of course this can also happen while engine development. E.g. broken png in resources/
            panic!("Fatal. Error texture should always be loadable. This suggest you messed with the executable. Abort.");
        }
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}
