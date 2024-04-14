use thiserror::Error;

#[derive(Error, Debug)]
pub enum AssetError {
    #[error("Asset not found: {0}")]
    NotFound(String),
    #[error("Asset already exists: {0}")]
    Exists(String),
    #[error("Asset is not loaded: {0}")]
    NotLoaded(String),
    #[error("Asset is not ready: {0}")]
    NotReady(String),
    #[error("Asset is not valid: {0}")]
    NotValid(String),
    #[error("Asset type mismatch: {0}")]
    TypeMismatch(String),
}
