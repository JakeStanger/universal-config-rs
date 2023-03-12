use thiserror::Error;

#[derive(Error, Debug)]
pub enum UniversalConfig {
    #[error("unable to locate user config or home directories")]
    MissingUserDir,
    #[error("unable to find any valid config file within the config directory")]
    FileNotFound,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Encoding(#[from] std::str::Utf8Error),
    #[error(transparent)]
    Deserialization(#[from] Deserialization),
}

#[derive(Error, Debug)]
pub enum Deserialization {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    Corn(#[from] libcorn::error::Error),
    #[error("unsupported file extension: '{0}'")]
    UnsupportedExtension(String),
}

pub type Result<T> = std::result::Result<T, UniversalConfig>;
