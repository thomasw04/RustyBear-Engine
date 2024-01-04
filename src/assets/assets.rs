use bimap::BiMap;
use hashbrown::HashMap;
use indicatif::{ProgressBar, ProgressStyle};
use once_cell::sync::Lazy;
use rayon::prelude::*;

use crate::context::VisContext;
use crate::logging;
use crate::utils::{Guid, GuidGenerator};

use std::any::Any;
use std::hash::Hash;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};

use super::shader::Shader;
use super::texture::{Texture2D, TextureArray};

pub enum AssetType {
    TextureArray(TextureArray),
    Texture2D(Texture2D),
    Shader(Shader),
}

static LOADING_STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
    ProgressStyle::with_template("{elapsed_precise} \u{1b}[32m[INFO]\u{1b}[0m    {wide_msg}")
        .unwrap()
});

static LOADING_SPINNER_STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
    ProgressStyle::with_template("{elapsed_precise} \u{1b}[32m[INFO]\u{1b}[0m {spinner} {wide_msg}")
        .unwrap()
});

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Ptr<T> {
    guid: Guid,
    phantom: std::marker::PhantomData<T>,
}

impl<T> Hash for Ptr<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.guid.hash(state);
    }
}

impl<T> Ptr<T> {
    fn new(guid: Guid) -> Self {
        Ptr { guid, phantom: std::marker::PhantomData }
    }

    pub fn inner(&self) -> Guid {
        self.guid
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

        let mut gpu_cache = HashMap::new();
        let mut path_cache = BiMap::new();
        let mut generator = GuidGenerator::new();

        Self::add_static_asset(
            &mut gpu_cache,
            &mut path_cache,
            &mut generator,
            "static:skybox.wgsl".to_owned(),
            AssetType::Shader(
                Shader::new(
                    &context,
                    wgpu::ShaderSource::Wgsl(include_str!("skybox.wgsl").into()),
                    what::ShaderStages::VERTEX | what::ShaderStages::FRAGMENT,
                )
                .unwrap(),
            ),
        );

        Self::add_static_asset(
            &mut gpu_cache,
            &mut path_cache,
            &mut generator,
            "static:default.wgsl".to_owned(),
            AssetType::Shader(
                Shader::new(
                    &context,
                    wgpu::ShaderSource::Wgsl(include_str!("default.wgsl").into()),
                    what::ShaderStages::VERTEX | what::ShaderStages::FRAGMENT,
                )
                .unwrap(),
            ),
        );

        let (in_sender, in_receiver): InChannel = mpsc::channel();
        let (out_sender, out_receiver): OutChannel = mpsc::channel();

        rayon::spawn(move || {
            let context = context.clone();

            let mut what = what::What::new(max_size, loc);

            while let Ok((path, guid, priority)) = in_receiver.recv() {
                let out_sender = out_sender.clone();
                let context = context.clone();

                match what.load_asset(path.clone(), priority) {
                    Ok(asset) => {
                        rayon::spawn(move || {
                            let asset = Self::load_asset(&context, asset, guid);
                            let _ = out_sender.send((guid, Ok(asset)));
                            log::info!("Loaded asset: {}", path);
                        });
                    }
                    Err(error) => {
                        let _ = out_sender
                            .send((guid, Err(format!("Failed to load asset. Error: {error:?}"))));
                    }
                }
            }
        });

        Assets {
            gpu_cache,
            path_cache,
            generator,

            request_sender: in_sender,
            asset_receiver: out_receiver,
        }
    }

    fn add_static_asset(
        cache: &mut HashMap<Guid, AssetType>, path_cache: &mut BiMap<Guid, String>,
        gen: &mut GuidGenerator, path: String, asset: AssetType,
    ) {
        let guid = gen.generate();
        cache.insert(guid, asset);
        path_cache.insert(guid, path);
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

    pub fn wait_for<T>(&mut self, ptr: &Ptr<T>) {
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
    //Won't get fixed until polonios is stable.
    //Use wait_for() instead.
    pub fn get<T: 'static>(&mut self, ptr: &Ptr<T>) -> Option<&T> {
        let here = self.gpu_cache.contains_key(&ptr.guid);

        if !here {
            self.wait_for(ptr);
        }

        self.gpu_cache.get(&ptr.guid).and_then(|asset| match asset {
            AssetType::TextureArray(texture_array) => {
                (texture_array as &dyn Any).downcast_ref::<T>()
            }
            AssetType::Texture2D(texture) => (texture as &dyn Any).downcast_ref::<T>(),
            AssetType::Shader(shader) => (shader as &dyn Any).downcast_ref::<T>(),
        })
    }

    pub fn try_get<T: 'static>(&self, ptr: &Ptr<T>) -> Option<&T> {
        self.gpu_cache.get(&ptr.guid).and_then(|asset| match asset {
            AssetType::TextureArray(texture_array) => {
                (texture_array as &dyn Any).downcast_ref::<T>()
            }
            AssetType::Texture2D(texture) => (texture as &dyn Any).downcast_ref::<T>(),
            AssetType::Shader(shader) => (shader as &dyn Any).downcast_ref::<T>(),
        })
    }

    pub fn delete_asset(&mut self, guid: Guid) {
        self.gpu_cache.remove(&guid);
    }

    fn load_asset(context: &VisContext, asset: what::Asset, _guid: Guid) -> AssetType {
        match asset {
            what::Asset::Texture(texture) => {
                let texture_data = image::load_from_memory(&texture.data);

                if let Ok(image) = texture_data {
                    let rgba = image.to_rgba8();

                    match Texture2D::new(context, None, &rgba, image::ImageFormat::Png) {
                        Ok(texture) => AssetType::Texture2D(texture),
                        Err(texture) => {
                            log::error!("Failed to load texture. Loading error texture instead.");
                            AssetType::Texture2D(texture)
                        }
                    }
                } else {
                    AssetType::Texture2D(Texture2D::error_texture(context))
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

                AssetType::TextureArray(texture)
            }
            what::Asset::Shader(shader) => {
                if let Ok(shader) = Shader::new(
                    context,
                    wgpu::ShaderSource::SpirV(shader.data.into()),
                    shader.stages,
                ) {
                    AssetType::Shader(shader)
                } else {
                    log::error!("Failed to load shader. Loading error shader instead.");
                    todo!("Implement error shader.")
                }
            }
            _ => todo!("Implement other asset types."),
        }
    }
}
