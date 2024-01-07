use std::path::Path;
use std::{io::BufReader, path::PathBuf};

use crate::utils::FileUtils;
use serde::{Deserialize, Serialize};

use super::error::ConfigError;

#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectConfiguration {
    pub project_name: String,
    pub author: Option<String>,
    pub version: Option<String>,

    //Root folder is simply the location of the project file.
    #[serde(skip_serializing, skip_deserializing)]
    pub location: Option<PathBuf>,

    pub data_folder: Option<PathBuf>,
    pub code_folder: Option<PathBuf>,
}

impl ProjectConfiguration {
    pub fn new(path: Option<PathBuf>) -> Self {
        ProjectConfiguration {
            project_name: "MyProject".to_string(),
            author: None,
            version: None,
            location: path,
            data_folder: None,
            code_folder: None,
        }
    }

    pub fn with_name<S: Into<String>>(mut self, name: S) -> ProjectConfiguration {
        self.project_name = name.into();
        self
    }

    pub fn with_author<S: Into<String>>(mut self, author: S) -> ProjectConfiguration {
        self.author = Some(author.into());
        self
    }

    pub fn with_version<S: Into<String>>(mut self, version: S) -> ProjectConfiguration {
        self.version = Some(version.into());
        self
    }

    pub fn with_data(mut self, path: PathBuf) -> ProjectConfiguration {
        self.data_folder = Some(path);
        self
    }

    pub fn with_code(mut self, path: PathBuf) -> ProjectConfiguration {
        self.code_folder = Some(path);
        self
    }
}

#[derive(Serialize, Deserialize)]
pub struct EngineConfiguration {
    pub project_file_extension: String,
}

impl Default for EngineConfiguration {
    fn default() -> Self {
        EngineConfiguration { project_file_extension: "rbe".to_string() }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ThemeConfiguration {
    pub background_music: String,
}

impl Default for ThemeConfiguration {
    fn default() -> Self {
        ThemeConfiguration { background_music: "default.mp3".to_string() }
    }
}

pub struct Config {
    engine_config: EngineConfiguration,
    theme_config: ThemeConfiguration,
    project_config: ProjectConfiguration,
}

impl Config {
    pub fn new(project_config: Option<ProjectConfiguration>) -> Self {
        let engine_config = Config::load_engine_config();
        let theme_config = Config::load_theme_config(&engine_config);
        let project_config = project_config.unwrap_or(ProjectConfiguration::new(None));

        Config { engine_config, theme_config, project_config }
    }

    pub fn exist_project(&self, path: &Path) -> bool {
        FileUtils::has_extension(path, self.engine_config.project_file_extension.as_str())
            || FileUtils::find_ext_in_dir(path, self.engine_config.project_file_extension.as_str())
                .is_some()
    }

    pub fn find_project(&mut self, path: &Path) -> Result<(), ConfigError> {
        if let Some(file_path) =
            if FileUtils::has_extension(path, self.engine_config.project_file_extension.as_str()) {
                Some(path.to_path_buf())
            } else {
                FileUtils::find_ext_in_dir(path, self.engine_config.project_file_extension.as_str())
            }
        {
            let file_path = file_path.as_path();
            let file = std::fs::File::open(file_path);

            match file {
                Err(error) => {
                    log::error!("Could not access {}. Please check if the file exists and I am permitted to open it. Message: {}", file_path.to_str().unwrap_or("ERR_NON_UTF8_PATH"), error);
                    return Err(ConfigError::Io(error));
                }
                Ok(file) => {
                    let reader = BufReader::new(file);

                    match serde_json::from_reader(reader) {
                        Err(error) => {
                            log::error!(
                                "Failed to parse {}. Message: {}",
                                file_path.to_str().unwrap_or("ERR_NON_UTF8_PATH"),
                                error
                            );
                            return Err(ConfigError::JsonError(error));
                        }
                        Ok(configuration) => {
                            self.project_config = configuration;
                            self.project_config.location = Some(path.to_path_buf());
                            return Ok(());
                        }
                    }
                }
            }
        }

        Err(ConfigError::NotFound)
    }

    pub fn create_project(&mut self, config: ProjectConfiguration) {
        let path = if let Some(path) = &config.location {
            if self.exist_project(path.as_path()) {
                log::error!("Project {} already exists.", path.display());
                return;
            }

            path.as_path()
        } else {
            log::error!(
                "Cannot create Project without location. Maybe you want to load a project instead?"
            );
            return;
        };

        if path.is_file() {
            log::error!("Invalid path {}. Project path must be a folder.", path.display());
            return;
        }

        if let Err(e) = std::fs::create_dir_all(path) {
            log::error!("Could not create project directory. Message: {}.", e);
            return;
        }

        if FileUtils::find_ext_in_dir(path, self.engine_config.project_file_extension.as_str())
            .is_some()
        {
            log::error!("Project {} already exists.", path.display());
            return;
        }

        let mut file_name = config.project_name.clone();
        file_name.push('.');
        file_name.push_str(self.engine_config.project_file_extension.as_str());

        let file_path = path.join(file_name);

        let file = std::fs::File::create(file_path.clone());

        if file.is_ok() {
            self.project_config = config;

            if let Err(e) = std::fs::write(
                file_path.clone(),
                serde_json::to_string(&self.project_config).unwrap_or("{}".to_string()),
            ) {
                log::error!("Could not create {}. {}", file_path.display(), e);
            }
        }
    }

    pub fn engine_config(&self) -> &EngineConfiguration {
        &self.engine_config
    }

    pub fn theme_config(&self) -> &ThemeConfiguration {
        &self.theme_config
    }

    pub fn project_config(&self) -> &ProjectConfiguration {
        &self.project_config
    }

    fn load_engine_config() -> EngineConfiguration {
        let config_folder = Path::new("config");
        let config = config_folder.join("config.json");

        if let Err(e) = std::fs::create_dir_all(config_folder) {
            log::error!("Could not create config directory. Message: {}. Defaulting... ", e);
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
            log::error!("Could not create themes directory. Message: {}. Defaulting... ", e);
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
