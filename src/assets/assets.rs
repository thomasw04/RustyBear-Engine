use bimap::BiMap;
use hashbrown::HashMap;
use image::GenericImageView;
use indicatif::{ProgressBar, ProgressStyle};
use once_cell::sync::Lazy;
use rayon::prelude::*;

use crate::context::VisContext;
use crate::logging;
use crate::render::material::GenericMaterial;
use crate::render::types::BindGroupEntry;
use crate::utils::{Guid, GuidGenerator};

use std::any::Any;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};

use super::buffer::UniformBuffer;
use super::shader::Shader;
use super::texture::{Sampler, Texture2D, TextureArray};

pub enum AssetType {
    TextureArray(TextureArray),
    Texture2D(Texture2D),
    Shader(Shader),
    Uniforms(UniformBuffer),
    Sampler(Sampler),
    GenericMaterial(GenericMaterial),
}

static LOADING_STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
    ProgressStyle::with_template("{elapsed_precise} \u{1b}[32m[INFO]\u{1b}[0m    {wide_msg}")
        .unwrap()
});

static LOADING_SPINNER_STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
    ProgressStyle::with_template("{elapsed_precise} \u{1b}[32m[INFO]\u{1b}[0m {spinner} {wide_msg}")
        .unwrap()
});

pub static SPRITE_SHADER: Lazy<Ptr<Shader>> = Lazy::new(|| Ptr::new(Guid::new(0x1)));

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct GenPtr {
    guid: Guid,
}

impl<T> From<Ptr<T>> for GenPtr {
    fn from(ptr: Ptr<T>) -> Self {
        GenPtr { guid: ptr.guid }
    }
}

#[derive(Debug)]
pub struct Ptr<T> {
    guid: Guid,
    phantom: std::marker::PhantomData<T>,
}

impl<T> Hash for Ptr<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.guid.hash(state);
    }
}

impl<T> Clone for Ptr<T> {
    fn clone(&self) -> Self {
        Ptr { guid: self.guid, phantom: PhantomData }
    }
}

impl<T> Copy for Ptr<T> {}

impl<T> PartialEq for Ptr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.guid == other.guid
    }
}

impl<T> Eq for Ptr<T> {}

impl<T> Ptr<T> {
    pub fn new(guid: Guid) -> Self {
        Ptr { guid, phantom: std::marker::PhantomData }
    }

    pub fn is_dead(&self) -> bool {
        self.guid.is_dead()
    }

    pub fn inner(&self) -> Guid {
        self.guid
    }

    pub fn dead() -> Self {
        Ptr { guid: Guid::dead(), phantom: std::marker::PhantomData }
    }
}

pub struct Assets {
    gpu_cache: HashMap<Guid, AssetType>,
    path_cache: BiMap<Guid, String>,
    generator: GuidGenerator,

    request_sender: Sender<(String, Guid, usize)>,
    asset_receiver: Receiver<(Guid, Result<AssetType, String>)>,
}

impl Assets {
    pub fn new(context: Arc<VisContext>, loc: Option<what::Location>, max_size: usize) -> Self {
        type InChannel = (Sender<(String, Guid, usize)>, Receiver<(String, Guid, usize)>);
        type OutChannel = (
            Sender<(Guid, Result<AssetType, String>)>,
            Receiver<(Guid, Result<AssetType, String>)>,
        );

        let gpu_cache = HashMap::new();
        let path_cache = BiMap::new();
        let generator = GuidGenerator::new();

        let (in_sender, in_receiver): InChannel = mpsc::channel();
        let (out_sender, out_receiver): OutChannel = mpsc::channel();

        let mut assets = Assets {
            gpu_cache,
            path_cache,
            generator,

            request_sender: in_sender,
            asset_receiver: out_receiver,
        };

        assets.register_static(&context);

        rayon::spawn(move || {
            let context = context.clone();

            let mut what = what::What::new(max_size, loc);

            while let Ok((path, guid, priority)) = in_receiver.recv() {
                let out_sender = out_sender.clone();
                let context = context.clone();

                match what.load_asset(path.clone(), priority) {
                    Ok(asset) => {
                        rayon::spawn(move || {
                            if let Some(asset) = Self::load_asset(&context, asset, guid) {
                                let _ = out_sender.send((guid, Ok(asset)));
                                log::info!("Loaded asset: {}", path);
                            } else {
                                let _ =
                                    out_sender.send((guid, Err(format!("Failed to load asset."))));
                            }
                        });
                    }
                    Err(error) => {
                        let _ = out_sender
                            .send((guid, Err(format!("Failed to load asset. Error: {error:?}"))));
                    }
                }
            }
        });

        return assets;
    }

    fn register_static(&mut self, context: &VisContext) {
        let guid = SPRITE_SHADER.guid;
        let sprite_shader = Shader::new(
            context,
            guid,
            wgpu::ShaderSource::Wgsl(include_str!("sprite.wgsl").into()),
            what::ShaderStages::FRAGMENT | what::ShaderStages::VERTEX,
        )
        .unwrap();

        self.gpu_cache.insert(guid, AssetType::Shader(sprite_shader));
    }

    fn request_id<S: Into<String> + AsRef<str>>(&mut self, path: S) -> Guid {
        if let Some(guid) = self.path_cache.get_by_right(path.as_ref()) {
            *guid
        } else {
            let id = self.generator.generate();
            self.path_cache.insert(id, path.into());
            id
        }
    }

    pub fn exist(&self, ptr: &GenPtr) -> bool {
        self.gpu_cache.contains_key(&ptr.guid)
    }

    /*Register an already created asset in the asset manager. This is necessary when you need to reference assets via a Ptr<T>. */
    pub fn consume_asset<S: Into<String> + AsRef<str>, T>(
        &mut self, asset: AssetType, path: Option<S>,
    ) -> Ptr<T> {
        let guid =
            if let Some(path) = path { self.request_id(path) } else { self.generator.generate() };

        match asset {
            AssetType::TextureArray(texture_array) => {
                self.gpu_cache.insert(guid, AssetType::TextureArray(texture_array));
            }
            AssetType::Texture2D(texture) => {
                self.gpu_cache.insert(guid, AssetType::Texture2D(texture));
            }
            AssetType::Shader(mut shader) => {
                shader.change_guid(guid);
                self.gpu_cache.insert(guid, AssetType::Shader(shader));
            }
            AssetType::Uniforms(uniforms) => {
                self.gpu_cache.insert(guid, AssetType::Uniforms(uniforms));
            }
            AssetType::Sampler(sampler) => {
                self.gpu_cache.insert(guid, AssetType::Sampler(sampler));
            }
            AssetType::GenericMaterial(material) => {
                self.gpu_cache.insert(guid, AssetType::GenericMaterial(material));
            }
        }

        Ptr::new(guid)
    }

    pub fn asset_path(&self, id: Guid) -> Option<&String> {
        self.path_cache.get_by_left(&id)
    }

    pub fn update(&mut self) -> Result<(), Guid> {
        while let Ok(content_result) = self.asset_receiver.try_recv() {
            if let (guid, Ok(content)) = content_result {
                self.gpu_cache.insert(guid, content);
            } else if let (guid, Err(error)) = content_result {
                log::error!("{}", error);
                return Err(guid);
            }
        }

        Ok(())
    }

    pub fn wait_for(&mut self, ptr: &GenPtr) {
        let spinner = logging::install_bar(ProgressBar::new_spinner()).unwrap();
        spinner.set_style(LOADING_SPINNER_STYLE.clone());
        spinner.set_message(format!("Loading asset: {}", self.asset_path(ptr.guid).unwrap()));

        while !self.gpu_cache.contains_key(&ptr.guid) {
            if let Err(err) = self.update() {
                if ptr.guid == err {
                    return;
                }
            }

            spinner.tick();
        }

        spinner.finish_with_message("Done!");
    }

    pub fn request_asset<T, S: Into<String> + AsRef<str>>(
        &mut self, path: S, priority: usize,
    ) -> Ptr<T> {
        let path = path.as_ref();

        let guid = self.request_id(path);

        if self.gpu_cache.contains_key(&guid) {
            return Ptr::new(guid);
        }

        if let Err(error) = self.request_sender.send((path.to_owned(), guid, priority)) {
            log::error!(
                "Failed to send asset request. Is the asset manager online? Error: {}",
                error
            );
        } else {
            log::info!("Requested asset: {}", path);
        }

        Ptr::new(guid)
    }

    //This currently does expend the lifetime of the mutable borrow to the lifetime of the returned reference.
    //Won't get fixed until polonius is stable.
    //Use wait_for() instead.
    pub fn get<T: 'static>(&mut self, ptr: &Ptr<T>) -> Option<&T> {
        let here = self.gpu_cache.contains_key(&ptr.guid);

        if !here {
            self.wait_for(&(*ptr).into());
        }

        self.gpu_cache.get(&ptr.guid).and_then(|asset| match asset {
            AssetType::TextureArray(texture_array) => {
                (texture_array as &dyn Any).downcast_ref::<T>()
            }
            AssetType::Texture2D(texture) => (texture as &dyn Any).downcast_ref::<T>(),
            AssetType::Shader(shader) => (shader as &dyn Any).downcast_ref::<T>(),
            AssetType::Uniforms(uniforms) => (uniforms as &dyn Any).downcast_ref::<T>(),
            AssetType::Sampler(sampler) => (sampler as &dyn Any).downcast_ref::<T>(),
            AssetType::GenericMaterial(material) => (material as &dyn Any).downcast_ref::<T>(),
        })
    }

    pub fn try_get_mut<T: 'static>(&mut self, ptr: &Ptr<T>) -> Option<&mut T> {
        self.gpu_cache.get_mut(&ptr.guid).and_then(|asset| match asset {
            AssetType::TextureArray(texture_array) => {
                (texture_array as &mut dyn Any).downcast_mut::<T>()
            }
            AssetType::Texture2D(texture) => (texture as &mut dyn Any).downcast_mut::<T>(),
            AssetType::Shader(shader) => (shader as &mut dyn Any).downcast_mut::<T>(),
            AssetType::Uniforms(uniforms) => (uniforms as &mut dyn Any).downcast_mut::<T>(),
            AssetType::Sampler(sampler) => (sampler as &mut dyn Any).downcast_mut::<T>(),
            AssetType::GenericMaterial(material) => (material as &mut dyn Any).downcast_mut::<T>(),
        })
    }

    pub fn try_get<T: 'static>(&self, ptr: &Ptr<T>) -> Option<&T> {
        self.gpu_cache.get(&ptr.guid).and_then(|asset| match asset {
            AssetType::TextureArray(texture_array) => {
                (texture_array as &dyn Any).downcast_ref::<T>()
            }
            AssetType::Texture2D(texture) => (texture as &dyn Any).downcast_ref::<T>(),
            AssetType::Shader(shader) => (shader as &dyn Any).downcast_ref::<T>(),
            AssetType::Uniforms(uniforms) => (uniforms as &dyn Any).downcast_ref::<T>(),
            AssetType::Sampler(sampler) => (sampler as &dyn Any).downcast_ref::<T>(),
            AssetType::GenericMaterial(material) => (material as &dyn Any).downcast_ref::<T>(),
        })
    }

    pub fn try_get_entry(&self, ptr: &GenPtr) -> Option<&dyn BindGroupEntry> {
        self.gpu_cache.get(&ptr.guid).and_then(|asset| match asset {
            AssetType::TextureArray(texture_array) => Some(texture_array as &dyn BindGroupEntry),
            AssetType::Texture2D(texture) => Some(texture as &dyn BindGroupEntry),
            AssetType::Shader(_shader) => None,
            AssetType::Uniforms(uniforms) => Some(uniforms as &dyn BindGroupEntry),
            AssetType::Sampler(sampler) => Some(sampler as &dyn BindGroupEntry),
            AssetType::GenericMaterial(_) => None,
        })
    }

    pub fn delete_asset(&mut self, guid: Guid) {
        self.gpu_cache.remove(&guid);
    }

    fn load_asset(context: &VisContext, asset: what::Asset, guid: Guid) -> Option<AssetType> {
        match asset {
            what::Asset::Texture(texture) => {
                let texture_data = image::load_from_memory(&texture.data);

                match texture_data {
                    Ok(image) => {
                        let rgba = image.to_rgba8();

                        Some(AssetType::Texture2D(Texture2D::new(
                            context,
                            None,
                            image.dimensions(),
                            &rgba,
                        )))
                    }
                    Err(e) => {
                        log::error!(
                            "Failed to load texture. Error: {}. Loading error texture instead...",
                            e
                        );
                        None
                    }
                }
            }
            what::Asset::TextureArray(texture_array) => {
                let mut texture =
                    TextureArray::new(context, texture_array.size, texture_array.data.len() as u32);

                let image_data = &texture_array.data;

                image_data.par_iter().enumerate().for_each(|(i, image)| {
                    if let Some(spinner) = logging::install_bar(ProgressBar::new_spinner()) {
                        spinner.set_style(LOADING_STYLE.clone());
                        spinner.set_message(format!(
                            "Loading textures... [{}/{}]",
                            i + 1,
                            image_data.len()
                        ));

                        if let Ok(image) = image::load_from_memory(image) {
                            let rgba = image.to_rgba8();
                            texture.upload(context, &rgba, i as u32);
                        } else {
                            log::error!("Failed to load texture. Loading error texture instead...");
                            texture.upload_error_texture(context, i as u32);
                        }

                        //spinner.set_prefix("");
                        spinner.finish_with_message("Done!");
                    }
                });

                texture.finish_creation();

                Some(AssetType::TextureArray(texture))
            }
            what::Asset::Shader(shader) => {
                if let Ok(shader) = Shader::new(
                    context,
                    guid,
                    wgpu::ShaderSource::SpirV(shader.data.into()),
                    shader.stages,
                ) {
                    Some(AssetType::Shader(shader))
                } else {
                    log::error!("Failed to load shader. Loading error shader instead.");
                    None
                }
            }
            _ => todo!("Implement other asset types."),
        }
    }
}
