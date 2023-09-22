use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::config::ProjectConfiguration;
use crate::context::Context;
use crate::render::texture::{CubeTexture, Texture2D};
use crate::utils::FileUtils;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

pub enum AssetType {
    Raw(Vec<u8>),
    CubeTexture(CubeTexture),
    Texture2D(Texture2D),
}

pub struct AssetManager {
    file_waiters: HashMap<String, Receiver<Result<AssetType, String>>>,
    file_cache: HashMap<String, AssetType>,
    root_folder: PathBuf,

    request_sender: Sender<PathBuf>,
    asset_receiver: Receiver<Result<AssetType, String>>,
}

impl AssetManager {
    pub fn new(config: &ProjectConfiguration) -> Self {
        let (in_sender, in_receiver): (Sender<PathBuf>, Receiver<PathBuf>) = mpsc::channel();
        let (out_sender, out_receiver): (
            Sender<Result<AssetType, String>>,
            Receiver<Result<AssetType, String>>,
        ) = mpsc::channel();

        rayon::spawn(move || loop {
            if let Ok(path) = in_receiver.recv() {
                if let Ok(content) = fs::read(path) {}
            } else {
                break;
            }
        });

        AssetManager {
            file_waiters: HashMap::new(),
            file_cache: HashMap::new(),
            root_folder: PathBuf::from(config.data_folder.clone()),

            request_sender: in_sender,
            asset_receiver: out_receiver,
        }
    }

    pub fn update(&mut self, context: &Context) {
        self.file_waiters.retain(|path, receiver| {
            if let Ok(content_result) = receiver.try_recv() {
                if let Ok(content) = content_result {
                    self.file_cache.insert(path.clone(), content);
                } else {
                    log::error!("{}", content_result.err().unwrap());
                }

                return false;
            }
            true
        });
    }

    pub fn get_file(&self, path: &Path) -> Option<&AssetType> {
        self.file_cache.get(FileUtils::pts(path))
    }

    pub fn delete_file(&mut self, path: &Path) {
        self.file_cache.remove(FileUtils::pts(path));
    }

    pub fn preload_resource(resource: &Path) -> Vec<Vec<u8>> {
        todo!()
    }

    pub fn load_cube_texture(&mut self, context: &Context, folder: &Path) {
        let path_str = FileUtils::pts(folder);

        if folder.is_absolute() {
            log::error!(
                "Did you specify an absolute path? Asset paths must be relative. {}",
                path_str
            );
            return;
        }

        let full_path = self.root_folder.join(folder);

        if self.file_waiters.contains_key(path_str) {
            return;
        }

        if !full_path.exists() {
            log::error!("The requested asset does not exist. {}", path_str);
            return;
        }

        let (sender, receiver): (
            Sender<Result<AssetType, String>>,
            Receiver<Result<AssetType, String>>,
        ) = mpsc::channel();

        let thread_sender = sender.clone();

        rayon::spawn(move || {
            let bytes = AssetManager::preload_resource(folder);
            let dimension_result = image::image_dimensions(paths[0]);

            if let Ok(dimension) = dimension_result {
                //If texture is not a cube fail.
                if dimension.0 != dimension.1 {
                    thread_sender.send(Err("Invalid cube texture. Width != height.".to_string()));
                    return;
                }

                let texture = CubeTexture::new(context, dimension.0);

                let mut successful = true;

                (0..5).into_par_iter().for_each(|layer| {
                    let content_result = fs::read(paths[0]);

                    if let Ok(content) = content_result {
                        if let Ok(image) = image::load_from_memory(&content) {
                            let rgba = image.to_rgba8();

                            if dimension != rgba.dimensions() {
                                successful = false;
                                return;
                            }

                            texture.upload(context, &rgba, layer);
                        }
                    }
                });

                if successful {
                    thread_sender.send(Ok(AssetType::CubeTexture(texture)));
                } else {
                    thread_sender.send(Err("Invalid cube texture. Size not matching.".to_string()));
                }
            }
        });

        self.file_waiters.insert(path_str.to_string(), receiver);
    }

    pub fn load_file(&mut self, path: &Path) {
        let path_str = FileUtils::pts(path);

        if path.is_absolute() {
            log::error!(
                "Did you specify an absolute path? Asset paths must be relative. {}",
                path_str
            );
            return;
        }

        let full_path = self.root_folder.join(path);

        if self.file_waiters.contains_key(path_str) {
            return;
        }

        if !full_path.exists() {
            log::error!("The requested asset does not exist. {}", path_str);
            return;
        }

        let (sender, receiver): (
            Sender<Result<AssetType, String>>,
            Receiver<Result<AssetType, String>>,
        ) = mpsc::channel();

        let thread_sender = sender.clone();
        rayon::spawn(move || {
            let content_result = fs::read(full_path);

            if let Ok(content) = content_result {
                //If we are not listening anymore we are not interested in the result thus just discarding it.
                let _result = thread_sender.send(Ok(AssetType::Raw(content)));
            }
        });

        self.file_waiters.insert(path_str.to_string(), receiver);
    }
}
