use std::io::{BufReader, Error};
use std::path::{Path, PathBuf};

use crate::utils::FileUtils;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ProjectConfiguration {
    pub project_name: String,
    pub author: Option<String>,
    pub version: Option<String>,
    pub root_folder: String,
    pub data_folder: String,
    pub code_folder: String,
}

impl ProjectConfiguration {
    pub fn new(path: &Path) -> Self {
        ProjectConfiguration {
            project_name: "MyProject".to_string(),
            author: None,
            version: None,
            root_folder: FileUtils::pts(path).to_string(),
            data_folder: "data".to_string(),
            code_folder: "code".to_string(),
        }
    }

    pub fn with_name(mut self, name: &str) -> ProjectConfiguration {
        self.project_name = String::from(name);
        return self;
    }

    pub fn with_author(mut self, author: &str) -> ProjectConfiguration {
        self.author = Some(String::from(author));
        return self;
    }

    pub fn with_version(mut self, version: &str) -> ProjectConfiguration {
        self.version = Some(String::from(version));
        return self;
    }

    pub fn with_data(mut self, path: &Path) -> ProjectConfiguration {
        self.data_folder = FileUtils::pts(path).to_string();
        return self;
    }

    pub fn with_code(mut self, path: &Path) -> ProjectConfiguration {
        self.code_folder = FileUtils::pts(path).to_string();
        return self;
    }
}

#[derive(Serialize, Deserialize)]
pub struct EngineConfiguration {
    pub project_file_extension: String,
}

impl Default for EngineConfiguration {
    fn default() -> Self {
        EngineConfiguration {
            project_file_extension: "rbe".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ThemeConfiguration {
    pub background_music: String,
}

impl Default for ThemeConfiguration {
    fn default() -> Self {
        ThemeConfiguration {
            background_music: "default.mp3".to_string(),
        }
    }
}

pub struct Config {
    engine_config: EngineConfiguration,
    theme_config: ThemeConfiguration,
    project_config: Option<ProjectConfiguration>,
}

impl Config {
    pub fn new() -> Self {
        let engine_config = Config::load_engine_config();
        let theme_config = Config::load_theme_config(&engine_config);

        Config {
            engine_config,
            theme_config,
            project_config: None,
        }
    }

    pub fn exist_project(&self, path: &Path) -> bool {
        FileUtils::has_extension(path, self.engine_config.project_file_extension.as_str())
            || FileUtils::find_ext_in_dir(path, self.engine_config.project_file_extension.as_str())
                .is_some()
    }

    pub fn find_project(&mut self, path: &Path) -> Option<&ProjectConfiguration> {
        let file_path: Option<PathBuf> =
            if FileUtils::has_extension(path, self.engine_config.project_file_extension.as_str()) {
                Some(path.to_path_buf())
            } else {
                FileUtils::find_ext_in_dir(path, self.engine_config.project_file_extension.as_str())
            };

        let file_result = std::fs::File::open(file_path);

        match file_result {
            Err(error) => {
                log::error!("Could not access {}. Please check if the file exists and I am permitted to open it. Message: {}", file_path.to_str().unwrap_or("ERR_NON_UTF8_PATH"), error);
                return None;
            }
            Ok(file) => {
                let reader = BufReader::new(file);
                let configuration_result = serde_json::from_reader(reader);

                if let Ok(configuration) = configuration_result {
                    self.project_config = Some(configuration);
                    return self.project_config.as_ref();
                }
            }
        }

        None
    }

    pub fn create_project(&mut self, config: &ProjectConfiguration) {
        let path = Path::new(config.root_folder.as_str());

        if path.is_file() {
            log::error!(
                "Invalid path {}. Project path must be a folder.",
                FileUtils::pts(path)
            );
            return;
        }

        if let Err(e) = std::fs::create_dir_all(path) {
            log::error!("Could not create project directory. Message: {}.", e);
            return;
        }

        if FileUtils::find_ext_in_dir(path, self.engine_config.project_file_extension.as_str()) {
            log::error!("Project {} already exists.", FileUtils::pts(path));
            return;
        }

        let mut file_name = String::from(config.project_name.clone());
        file_name.push('.');
        file_name.push_str(self.engine_config.project_file_extension.as_str());

        let file_path = path.join(file_name);

        let file = std::fs::File::create(file_path.clone());

        if file.is_ok() {
            self.project_config = Some((*config).clone());

            if let Err(e) = std::fs::write(
                file_path.clone(),
                serde_json::to_string(&self.project_config.unwrap()).unwrap_or("{}".to_string()),
            ) {
                log::error!("Could not create {}. {}", FileUtils::pts(file_path), e);
            }
        }
    }

    pub fn engine_config(&self) -> &EngineConfiguration {
        &self.engine_config
    }

    pub fn theme_config(&self) -> &ThemeConfiguration {
        &self.theme_config
    }

    pub fn project_config(&self) -> Option<&ProjectConfiguration> {
        self.project_config.as_ref()
    }

    fn load_engine_config() -> EngineConfiguration {
        let config_folder = Path::new("config");
        let config = config_folder.join("config.json");

        if let Err(e) = std::fs::create_dir_all(config_folder) {
            log::error!(
                "Could not create config directory. Message: {}. Defaulting... ",
                e
            );
            return EngineConfiguration::default();
        }

        let file = std::fs::File::open(config.clone());

        if file.is_err() {
            log::warn!(
                "Could not access {}. Creating and defaulting...",
                config.to_str().unwrap_or("ERR_NON_UTF8_PATH")
            );

            let default = EngineConfiguration::default();

            if let Err(e) = std::fs::write(
                config.clone(),
                serde_json::to_string(&default).unwrap_or("{}".to_string()),
            ) {
                log::error!(
                    "Could not create {}. {}",
                    config.to_str().unwrap_or("ERR_NON_UTF8_PATH"),
                    e
                );
            }

            return default;
        }

        let reader = BufReader::new(file.unwrap());

        let conf = serde_json::from_reader(reader);

        if conf.is_err() {
            log::error!(
                "Failed to parse {}. Defaulting...",
                config.to_str().unwrap_or("ERR_NON_UTF8_PATH")
            );
            return EngineConfiguration::default();
        }

        conf.unwrap()
    }

    fn load_theme_config(_engine_config: &EngineConfiguration) -> ThemeConfiguration {
        let themes_folder = Path::new("themes");
        let themes_config = themes_folder.join("config.json");

        if let Err(e) = std::fs::create_dir_all(themes_folder) {
            log::error!(
                "Could not create themes directory. Message: {}. Defaulting... ",
                e
            );
            return ThemeConfiguration::default();
        }

        let file = std::fs::File::open(themes_config.clone());

        if file.is_err() {
            log::warn!(
                "Could not access {}. Creating and defaulting...",
                themes_config.to_str().unwrap_or("ERR_NON_UTF8_PATH")
            );

            let default = ThemeConfiguration::default();

            if let Err(e) = std::fs::write(
                themes_config.clone(),
                serde_json::to_string(&default).unwrap_or("{}".to_string()),
            ) {
                log::error!(
                    "Could not create {}. {}",
                    themes_config.to_str().unwrap_or("ERR_NON_UTF8_PATH"),
                    e
                );
            }

            return default;
        }

        let reader = BufReader::new(file.unwrap());

        let conf = serde_json::from_reader(reader);

        if conf.is_err() {
            log::error!(
                "Failed to parse {}. Defaulting...",
                themes_config.to_str().unwrap_or("ERR_NON_UTF8_PATH")
            );
            return ThemeConfiguration::default();
        }

        conf.unwrap()
    }
}
