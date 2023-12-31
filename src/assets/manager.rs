use bimap::BiMap;
use hashbrown::HashMap;
use indicatif::{ProgressBar, ProgressStyle};
use once_cell::sync::Lazy;
use rayon::prelude::*;

use crate::context::VisContext;
use crate::logging;
use crate::utils::{Guid, GuidGenerator};

use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};

use super::shader::Shader;
use super::texture::{Texture2D, TextureArray};

#[derive(Clone)]
pub enum AssetType {
    TextureArray(Arc<TextureArray>),
    Texture2D(Arc<Texture2D>),
    Shader(Arc<Shader>),
}

static LOADING_STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
    ProgressStyle::with_template("{elapsed_precise} \u{1b}[32m[INFO]\u{1b}[0m    {wide_msg}")
        .unwrap()
});

static LOADING_SPINNER_STYLE: Lazy<ProgressStyle> = Lazy::new(|| {
    ProgressStyle::with_template("{elapsed_precise} \u{1b}[32m[INFO]\u{1b}[0m {spinner} {wide_msg}")
        .unwrap()
});

pub struct StaticRegistry {
    static_cache: HashMap<String, AssetType>,
}

impl StaticRegistry {
    pub fn new(context: &VisContext) -> Self {
        let mut static_cache = HashMap::new();

        static_cache.insert(
            "skybox.wgsl".to_owned(),
            AssetType::Shader(Arc::new(
                Shader::new(
                    context,
                    Guid::new(0),
                    wgpu::ShaderSource::Wgsl(include_str!("skybox.wgsl").into()),
                    what::ShaderStages::VERTEX | what::ShaderStages::FRAGMENT,
                )
                .unwrap(),
            )),
        );

        StaticRegistry { static_cache }
    }

    pub fn get<S: AsRef<str>>(&self, path: S) -> Option<AssetType> {
        self.static_cache.get(path.as_ref()).cloned()
    }
}

pub struct AssetManager {
    gpu_cache: HashMap<Guid, AssetType>,
    path_cache: BiMap<Guid, String>,
    generator: GuidGenerator,

    request_sender: Sender<(String, Guid, usize)>,
    asset_receiver: Receiver<(Guid, Result<AssetType, String>)>,
}

impl AssetManager {
    pub fn new(context: Arc<VisContext>, loc: Option<what::Location>, max_size: usize) -> Self {
        type InChannel = (Sender<(String, Guid, usize)>, Receiver<(String, Guid, usize)>);
        type OutChannel = (
            Sender<(Guid, Result<AssetType, String>)>,
            Receiver<(Guid, Result<AssetType, String>)>,
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
                            let asset = AssetManager::load_asset(&context, asset, guid);
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

        AssetManager {
            gpu_cache: HashMap::new(),
            path_cache: BiMap::new(),
            generator: GuidGenerator::new(),

            request_sender: in_sender,
            asset_receiver: out_receiver,
        }
    }

    pub fn request_id<S: Into<String> + AsRef<str>>(&mut self, path: S) -> Guid {
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

    pub fn request_asset<S: Into<String> + AsRef<str>>(
        &mut self, path: S, priority: usize,
    ) -> Guid {
        let path = path.as_ref();

        let guid = self.request_id(path);

        if self.gpu_cache.contains_key(&guid) {
            return guid;
        }

        if let Err(error) = self.request_sender.send((path.to_owned(), guid, priority)) {
            log::error!(
                "Failed to send asset request. Is the asset manager online? Error: {}",
                error
            );
        } else {
            log::info!("Requested asset: {}", path);
        }

        guid
    }

    pub fn get_asset<S: Into<String> + AsRef<str>>(
        &mut self, path: S, priority: usize,
    ) -> (Option<AssetType>, Guid) {
        let guid = self.request_asset(path, priority);

        if let Some(spinner) = logging::install_bar(ProgressBar::new_spinner()) {
            spinner.set_style(LOADING_SPINNER_STYLE.clone());
            spinner.set_message(format!("Loading asset: {}", self.asset_path(guid).unwrap()));

            while !self.gpu_cache.contains_key(&guid) {
                if let Err(err) = self.update() {
                    if guid == err {
                        return (None, guid);
                    }
                }
                spinner.tick();
            }

            spinner.finish_with_message("Done!");

            (Some(self.gpu_cache.get(&guid).unwrap().clone()), guid)
        } else {
            while !self.gpu_cache.contains_key(&guid) {
                self.update();
            }

            (Some(self.gpu_cache.get(&guid).unwrap().clone()), guid)
        }
    }

    pub fn try_asset(&mut self, guid: Guid) -> Option<AssetType> {
        self.gpu_cache.get(&guid).cloned()
    }

    pub fn delete_asset(&mut self, guid: Guid) {
        self.gpu_cache.remove(&guid);
    }

    fn load_asset(context: &VisContext, asset: what::Asset, guid: Guid) -> AssetType {
        match asset {
            what::Asset::Texture(texture) => {
                let texture_data = image::load_from_memory(&texture.data);

                if let Ok(image) = texture_data {
                    let rgba = image.to_rgba8();

                    match Texture2D::new(context, guid, None, &rgba, image::ImageFormat::Png) {
                        Ok(texture) => AssetType::Texture2D(Arc::new(texture)),
                        Err(texture) => {
                            log::error!("Failed to load texture. Loading error texture instead.");
                            AssetType::Texture2D(Arc::new(texture))
                        }
                    }
                } else {
                    AssetType::Texture2D(Arc::new(Texture2D::error_texture(context)))
                }
            }
            what::Asset::TextureArray(texture_array) => {
                let mut texture = TextureArray::new(
                    context,
                    guid,
                    texture_array.size,
                    texture_array.data.len() as u32,
                );

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

                texture.finish_creation(context);

                AssetType::TextureArray(Arc::new(texture))
            }
            what::Asset::Shader(shader) => {
                if let Ok(shader) = Shader::new(
                    context,
                    guid,
                    wgpu::ShaderSource::SpirV(shader.data.into()),
                    shader.stages,
                ) {
                    return AssetType::Shader(Arc::new(shader));
                } else {
                    log::error!("Failed to load shader. Loading error shader instead.");
                    todo!("Implement error shader.")
                }
            }
            _ => todo!("Implement other asset types."),
        }
    }
}
