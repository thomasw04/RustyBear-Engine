use rayon::prelude::*;

use crate::context::VisContext;
use crate::render::texture::{Texture2D, TextureArray};
use crate::utils::FileUtils;
use std::collections::HashMap;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};

pub enum AssetType {
    TextureArray(TextureArray),
    Texture2D(Texture2D),
}

pub struct AssetManager {
    asset_waiters: HashMap<String, Receiver<Result<AssetType, String>>>,
    gpu_cache: HashMap<String, AssetType>,

    request_sender: Sender<(PathBuf, usize)>,
    asset_receiver: Receiver<Result<AssetType, String>>,
}

impl AssetManager {
    pub fn new(context: Arc<VisContext>, loc: Option<what::Location>, max_size: usize) -> Self {
        let (in_sender, in_receiver): (Sender<(PathBuf, usize)>, Receiver<(PathBuf, usize)>) =
            mpsc::channel();
        let (out_sender, out_receiver): (
            Sender<Result<AssetType, String>>,
            Receiver<Result<AssetType, String>>,
        ) = mpsc::channel();

        rayon::spawn(move || {
            let context = context.clone();

            let mut what = what::What::new(max_size, loc);

            while let Ok((path, priority)) = in_receiver.recv() {
                let out_sender = out_sender.clone();
                let context = context.clone();

                match what.load_asset(path.to_string_lossy(), priority) {
                    Ok(asset) => {
                        rayon::spawn(move || {
                            let asset = AssetManager::load_asset(&context, asset);
                            let _ = out_sender.send(Ok(asset));
                        });
                    }
                    Err(_error) => {
                        //TODO: Better error return type.
                        let _ = out_sender.send(Err("Failed to load asset.".to_string()));
                    }
                }
            }
        });

        AssetManager {
            asset_waiters: HashMap::new(),
            gpu_cache: HashMap::new(),

            request_sender: in_sender,
            asset_receiver: out_receiver,
        }
    }

    pub fn update(&mut self) {
        self.asset_waiters.retain(|path, receiver| {
            if let Ok(content_result) = receiver.try_recv() {
                if let Ok(content) = content_result {
                    self.gpu_cache.insert(path.clone(), content);
                } else {
                    log::error!("{}", content_result.err().unwrap());
                }

                return false;
            }
            true
        });
    }

    pub fn get_asset(&self, path: &Path) -> Option<&AssetType> {
        self.gpu_cache.get(FileUtils::pts(path))
    }

    pub fn delete_asset(&mut self, path: &Path) {
        self.gpu_cache.remove(FileUtils::pts(path));
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
