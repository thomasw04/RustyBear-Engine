use bimap::BiMap;
use rayon::prelude::*;

use crate::context::VisContext;
use crate::render::texture::{Texture2D, TextureArray};
use crate::utils::{FileUtils, Guid, GuidGenerator};
use std::collections::HashMap;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};

pub enum AssetType {
    TextureArray(TextureArray),
    Texture2D(Texture2D),
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
        let (in_sender, in_receiver): (
            Sender<(String, Guid, usize)>,
            Receiver<(String, Guid, usize)>,
        ) = mpsc::channel();
        let (out_sender, out_receiver): (
            Sender<(Guid, Result<AssetType, String>)>,
            Receiver<(Guid, Result<AssetType, String>)>,
        ) = mpsc::channel();

        rayon::spawn(move || {
            let context = context.clone();

            let mut what = what::What::new(max_size, loc);

            while let Ok((path, guid, priority)) = in_receiver.recv() {
                let out_sender = out_sender.clone();
                let context = context.clone();

                match what.load_asset(path, priority) {
                    Ok(asset) => {
                        rayon::spawn(move || {
                            let asset = AssetManager::load_asset(&context, asset);
                            let _ = out_sender.send((guid, Ok(asset)));
                        });
                    }
                    Err(_error) => {
                        //TODO: Better error return type.
                        let _ = out_sender.send((guid, Err("Failed to load asset.".to_string())));
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

    pub fn update(&mut self) {
        loop {
            match self.asset_receiver.try_recv() {
                Ok(content_result) => {
                    if let (guid, Ok(content)) = content_result {
                        self.gpu_cache.insert(guid, content);
                    } else if let (guid, Err(error)) = content_result {
                        log::error!("{}", error);
                    }
                }
                Err(_) => break,
            }
        }
    }

    pub fn request_asset(&mut self, guid: Guid, priority: usize) {
        let path = if let Some(path) = self.asset_path(guid) {
            path
        } else {
            log::error!("Failed to request asset. Asset not found.");
            return;
        };

        if self.gpu_cache.contains_key(&guid) {
            return;
        }

        if let Err(error) = self.request_sender.send((path.to_owned(), guid, priority)) {
            log::error!(
                "Failed to send asset request. Is the asset manager online? Error: {}",
                error
            );
        } else {
            log::info!("Requested asset: {}", path);
        }
    }

    pub fn get_asset(&self, guid: Guid, priority: usize) -> Option<&AssetType> {
        if let Some(asset) = self.gpu_cache.get(&guid) {
            return Some(asset);
        } else {
            self.request_asset(guid, priority);
            self.update();
            todo!("Implement asset waiters.")
        }
    }

    pub fn delete_asset(&mut self, guid: Guid) {
        self.gpu_cache.remove(&guid);
    }

    fn load_asset(context: &VisContext, asset: what::Asset) -> AssetType {
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
                let texture =
                    TextureArray::new(context, texture_array.size, texture_array.data.len() as u32);

                let image_data = &texture_array.data;

                image_data.par_iter().enumerate().for_each(|(i, image)| {
                    if let Ok(image) = image::load_from_memory(image) {
                        let rgba = image.to_rgba8();
                        texture.upload(context, &rgba, i as u32);
                    } else {
                        log::error!("Failed to load texture. Loading error texture instead...");
                        texture.upload_error_texture(context, i as u32);
                    }
                });

                AssetType::TextureArray(texture)
            }
            _ => todo!("Implement other asset types."),
        }
    }
}
