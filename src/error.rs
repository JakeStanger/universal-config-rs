use thiserror::Error;

#[derive(Error, Debug)]
pub enum UniversalConfigError {
    #[error("unable to locate user config or home directories")]
    MissingUserDir,
    #[error("unable to find any valid config file within the config directory")]
    FileNotFound,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Encoding(#[from] std::str::Utf8Error),
    #[error(transparent)]
    Deserialization(#[from] DeserializationError),
    #[error(transparent)]
    Serialization(#[from] SerializationError),
}

#[derive(Error, Debug)]
pub enum DeserializationError {
    #[cfg(feature = "json")]
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[cfg(feature = "yaml")]
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),

    #[cfg(feature = "toml")]
    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    #[cfg(feature = "corn")]
    #[error(transparent)]
    Corn(#[from] corn::error::Error),

    #[cfg(feature = "xml")]
    #[error(transparent)]
    Xml(#[from] serde_xml_rs::Error),

    #[cfg(feature = "ron")]
    #[error(transparent)]
    Ron(#[from] ron::de::SpannedError),

    #[error("unsupported file extension: '{0}'")]
    UnsupportedExtension(String),
}

#[derive(Error, Debug)]
pub enum SerializationError {
    #[cfg(feature = "json")]
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[cfg(feature = "yaml")]
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),

    #[cfg(feature = "toml")]
    #[error(transparent)]
    Toml(#[from] toml::ser::Error),

    #[cfg(feature = "corn")]
    #[error(transparent)]
    Corn(#[from] corn::error::Error),

    #[cfg(feature = "xml")]
    #[error(transparent)]
    Xml(#[from] serde_xml_rs::Error),

    #[cfg(feature = "ron")]
    #[error(transparent)]
    Ron(#[from] ron::Error),

    #[error("unsupported file extension: '{0}'")]
    UnsupportedExtension(String),
}

pub type Result<T> = std::result::Result<T, UniversalConfigError>;
