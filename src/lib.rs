#![doc = include_str!("../README.md")]

mod error;

use crate::error::{
    DeserializationError, Result, SerializationError, UniversalConfigError as Error,
    UniversalConfigError,
};
use dirs::{config_dir, home_dir};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Supported config formats.
pub enum Format {
    /// `.json` file
    #[cfg(feature = "json")]
    Json,
    /// `.yaml` or `.yml` files.
    #[cfg(feature = "yaml")]
    Yaml,
    /// `.toml` files.
    #[cfg(feature = "toml")]
    Toml,
    /// `.corn` files.
    #[cfg(feature = "corn")]
    Corn,
    /// `.xml` files.
    #[cfg(feature = "xml")]
    Xml,
}

impl Format {
    const fn extension(&self) -> &str {
        match self {
            #[cfg(feature = "json")]
            Self::Json => "json",
            #[cfg(feature = "yaml")]
            Self::Yaml => "yaml",
            #[cfg(feature = "toml")]
            Self::Toml => "toml",
            #[cfg(feature = "corn")]
            Self::Corn => "corn",
            #[cfg(feature = "xml")]
            Self::Xml => "xml",
        }
    }
}

/// The main loader struct.
///
/// Create a new loader and configure as appropriate
/// to load your config file.
pub struct ConfigLoader<'a> {
    /// The name of your program, used when determining the directory path.
    app_name: &'a str,
    /// The name of the file (*excluding* extension) to search for.
    /// Defaults to `config`.
    file_name: &'a str,
    /// Allowed file formats.
    /// Defaults to all formats.
    /// Set to disable formats you do not wish to allow.
    formats: &'a [Format],
    /// The directory to load the config file from.
    /// Defaults to your system config dir (`$XDG_CONFIG_DIR` on Linux),
    /// or your home dir if that does not exist.
    config_dir: Option<&'a str>,
}

impl<'a> ConfigLoader<'a> {
    /// Creates a new config loader for the provided app name.
    /// Uses a default file name of "config" and all formats.
    #[must_use]
    pub const fn new(app_name: &'a str) -> ConfigLoader<'a> {
        Self {
            app_name,
            file_name: "config",
            formats: &[
                #[cfg(feature = "json")]
                Format::Json,
                #[cfg(feature = "yaml")]
                Format::Yaml,
                #[cfg(feature = "toml")]
                Format::Toml,
                #[cfg(feature = "corn")]
                Format::Corn,
                #[cfg(feature = "xml")]
                Format::Xml,
            ],
            config_dir: None,
        }
    }

    /// Specifies the file name to look for, excluding the extension.
    ///
    /// If not specified, defaults to "config".
    pub fn with_file_name(mut self, file_name: &'a str) -> Self {
        self.file_name = file_name;
        self
    }

    /// Specifies which file formats to search for, and in which order.
    ///
    /// If not specified, all formats are checked for
    /// in the order JSON, YAML, TOML, Corn.
    pub fn with_formats(mut self, formats: &'a [Format]) -> Self {
        self.formats = formats;
        self
    }

    /// Specifies which directory the config should be loaded from.
    ///
    /// If not specified, loads from `$XDG_CONFIG_DIR/<app_name>`
    /// or `$HOME/.<app_name>` if the config dir does not exist.
    pub fn with_config_dir(mut self, dir: &'a str) -> Self {
        self.config_dir = Some(dir);
        self
    }

    /// Attempts to locate a config file on disk and load it.
    ///
    /// # Errors
    ///
    /// Will return a `UniversalConfigError` if any error occurs
    /// when looking for, reading, or deserializing a config file.
    pub fn find_and_load<T: DeserializeOwned>(&self) -> Result<T> {
        let file = self.try_find_file()?;
        debug!("Found file at: '{}", file.display());
        Self::load(&file)
    }

    /// Attempts to find the directory in which the config file is stored.
    fn get_config_dir(&self) -> std::result::Result<PathBuf, UniversalConfigError> {
        self.config_dir
            .map(Into::into)
            .or_else(|| config_dir().map(|dir| dir.join(self.app_name)))
            .or_else(|| home_dir().map(|dir| dir.join(format!(".{}", self.app_name))))
            .ok_or(Error::MissingUserDir)
    }

    /// Attempts to find a config file for the given app name
    /// in the app's config directory
    /// that matches any of the allowed formats.
    fn try_find_file(&self) -> Result<PathBuf> {
        let config_dir = self.get_config_dir()?;

        let extensions = self.get_extensions();

        debug!("Using config dir: {}", config_dir.display());

        let file = extensions.into_iter().find_map(|extension| {
            let full_path = config_dir.join(format!("{}.{extension}", self.file_name));

            if Path::exists(&full_path) {
                Some(full_path)
            } else {
                None
            }
        });

        file.ok_or(Error::FileNotFound)
    }

    /// Loads the file at the given path,
    /// deserializing it into a new `T`.
    ///
    /// The type is automatically determined from the file extension.
    ///
    /// # Errors
    ///
    /// Will return a `UniversalConfigError` if unable to read or deserialize the file.
    pub fn load<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T> {
        let str = fs::read_to_string(&path)?;

        let extension = path
            .as_ref()
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();

        let config = Self::deserialize(&str, extension)?;
        Ok(config)
    }

    /// Gets a list of supported and enabled file extensions.
    fn get_extensions(&self) -> Vec<&'static str> {
        let mut extensions = vec![];

        for format in self.formats {
            match format {
                #[cfg(feature = "json")]
                Format::Json => extensions.push("json"),
                #[cfg(feature = "yaml")]
                Format::Yaml => {
                    extensions.push("yaml");
                    extensions.push("yml");
                }
                #[cfg(feature = "toml")]
                Format::Toml => extensions.push("toml"),
                #[cfg(feature = "corn")]
                Format::Corn => extensions.push("corn"),
                #[cfg(feature = "xml")]
                Format::Xml => extensions.push("xml"),
            }
        }

        extensions
    }

    /// Attempts to deserialize the provided input into `T`,
    /// based on the provided file extension.
    fn deserialize<T: DeserializeOwned>(
        str: &str,
        extension: &str,
    ) -> std::result::Result<T, DeserializationError> {
        let res = match extension {
            #[cfg(feature = "json")]
            "json" => serde_json::from_str(str).map_err(DeserializationError::from),
            #[cfg(feature = "toml")]
            "toml" => toml::from_str(str).map_err(DeserializationError::from),
            #[cfg(feature = "yaml")]
            "yaml" | "yml" => serde_yaml::from_str(str).map_err(DeserializationError::from),
            #[cfg(feature = "corn")]
            "corn" => libcorn::from_str(str).map_err(DeserializationError::from),
            #[cfg(feature = "xml")]
            "xml" => serde_xml_rs::from_str(str).map_err(DeserializationError::from),
            _ => Err(DeserializationError::UnsupportedExtension(
                extension.to_string(),
            )),
        }?;

        Ok(res)
    }

    /// Saves the provided configuration into a file of the specified format.
    ///
    /// The file is stored in the app's configuration directory.
    /// Directories are automatically created if required.
    ///
    /// # Errors
    ///
    /// If the provided config cannot be serialised into the format, an error will be returned.
    /// The `.corn` format is not supported, and the function will error if specified.
    ///
    /// If a valid config dir cannot be found, an error will be returned.
    ///
    /// If the file cannot be written to the specified path, an error will be returned.
    pub fn save<T: Serialize>(&self, config: &T, format: &Format) -> Result<()> {
        let str = match format {
            #[cfg(feature = "json")]
            Format::Json => serde_json::to_string_pretty(config).map_err(SerializationError::from),
            #[cfg(feature = "yaml")]
            Format::Yaml => serde_yaml::to_string(config).map_err(SerializationError::from),
            #[cfg(feature = "toml")]
            Format::Toml => toml::to_string_pretty(config).map_err(SerializationError::from),
            #[cfg(feature = "corn")]
            Format::Corn => Err(SerializationError::UnsupportedExtension("corn".to_string())),
            #[cfg(feature = "xml")]
            Format::Xml => serde_xml_rs::to_string(config).map_err(SerializationError::from),
        }?;

        let config_dir = self.get_config_dir()?;
        let file_name = format!("{}.{}", self.file_name, format.extension());
        let full_path = config_dir.join(file_name);

        fs::create_dir_all(config_dir)?;
        fs::write(full_path, str)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct ConfigContents {
        test: String,
    }

    #[test]
    fn test_json() {
        let res: ConfigContents = ConfigLoader::load("test_configs/config.json").unwrap();
        assert_eq!(res.test, "hello world")
    }

    #[test]
    fn test_yaml() {
        let res: ConfigContents = ConfigLoader::load("test_configs/config.yaml").unwrap();
        assert_eq!(res.test, "hello world")
    }

    #[test]
    fn test_toml() {
        let res: ConfigContents = ConfigLoader::load("test_configs/config.toml").unwrap();
        assert_eq!(res.test, "hello world")
    }

    #[test]
    fn test_corn() {
        let res: ConfigContents = ConfigLoader::load("test_configs/config.corn").unwrap();
        assert_eq!(res.test, "hello world")
    }

    #[test]
    fn test_xml() {
        let res: ConfigContents = ConfigLoader::load("test_configs/config.xml").unwrap();
        assert_eq!(res.test, "hello world")
    }

    #[test]
    fn test_find_load() {
        let mut config = ConfigLoader::new("universal-config");
        let res: ConfigContents = config
            .with_config_dir("test_configs")
            .find_and_load()
            .unwrap();
        assert_eq!(res.test, "hello world")
    }
}
