pub enum ConfigError {
    Io(std::io::Error),
    JsonError(serde_json::Error),
    NotFound,
}
