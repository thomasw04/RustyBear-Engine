use crate::config::ProjectConfiguration;
use crate::context::Context;
use crate::utils::FileUtils;
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

struct AssetManager {
    file_waiters: HashMap<String, Receiver<Vec<u8>>>,
    file_cache: HashMap<String, Vec<u8>>,
    root_folder: PathBuf,
}

impl AssetManager {
    pub fn new(config: &ProjectConfiguration) -> Self {
        AssetManager {
            file_waiters: HashMap::new(),
            file_cache: HashMap::new(),
            root_folder: PathBuf::from(config.data_folder.clone()),
        }
    }

    pub fn update(&mut self, context: &Context) {
        self.file_waiters.retain(|path, receiver| {
            if let Some(content) = receiver.try_recv() {
                self.file_cache.insert(path.clone(), content);
                return false;
            }
            true
        });
    }

    pub fn get_file(&self, path: &Path) -> Option<&Vec<u8>> {
        self.file_cache.get(FileUtils::pts(path))
    }

    pub fn delete_file(&mut self, path: &Path) {
        self.file_cache.remove(FileUtils::pts(path));
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

        let (sender, receiver): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();

        let thread_sender = sender.clone();
        rayon::spawn(|handle| {
            let content_result = fs::read(full_path);

            if let Ok(content) = content_result {
                //If we are not listening anymore we are not interested in the result thus just discarding it.
                let _result = thread_sender.send(content);
            }
        });

        self.file_waiters.insert(path_str.to_string(), receiver);
    }
}
