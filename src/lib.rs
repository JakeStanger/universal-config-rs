mod error;

use crate::error::{Deserialization as DeserializationError, Result, UniversalConfig as Error};
use dirs::{config_dir, home_dir};
use serde::de::DeserializeOwned;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

pub enum Format {
    Json,
    Yaml,
    Toml,
    Corn,
}

/// Config
pub struct ConfigLoader<'a> {
    app_name: &'a str,
    file_name: &'a str,
    formats: &'a [Format],
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
            formats: &[Format::Json, Format::Yaml, Format::Toml, Format::Corn],
            config_dir: None,
        }
    }

    /// Specifies the file name to look for, excluding the extension.
    ///
    /// If not specified, defaults to "config".
    pub fn with_file_name(&mut self, file_name: &'a str) -> &Self {
        self.file_name = file_name;
        self
    }

    /// Specifies which file formats to search for, and in which order.
    ///
    /// If not specified, all formats are checked for
    /// in the order JSON, YAML, TOML, Corn.
    pub fn with_formats(&mut self, formats: &'a [Format]) -> &Self {
        self.formats = formats;
        self
    }

    /// Specifies which directory the config should be loaded from.
    ///
    /// If not specified, loads from `$XDG_CONFIG_DIR/<app_name>`
    /// or `$HOME/.<app_name>` if the config dir does not exist.
    pub fn with_config_dir(&mut self, dir: &'a str) -> &Self {
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

    /// Attempts to find a config file for the given app name
    /// in the app's config directory
    /// that matches any of the allowed formats.
    fn try_find_file(&self) -> Result<PathBuf> {
        let config_dir = self
            .config_dir
            .map(Into::into)
            .or_else(|| config_dir().map(|dir| dir.join(self.app_name)))
            .or_else(|| home_dir().map(|dir| dir.join(format!(".{}", self.app_name))))
            .ok_or(Error::MissingUserDir)?;

        let extensions = self.get_extensions();

        debug!("Using config dir: {}", config_dir.display());

        let file = extensions.into_iter().find_map(|extension| {
            let full_path = config_dir.join(format!("config.{extension}"));

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

    fn get_extensions(&self) -> Vec<&'static str> {
        let mut extensions = vec![];

        for format in self.formats {
            match format {
                Format::Json => extensions.push("json"),
                Format::Yaml => {
                    extensions.push("yaml");
                    extensions.push("yml");
                }
                Format::Toml => extensions.push("toml"),
                Format::Corn => extensions.push("corn"),
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
            _ => Err(DeserializationError::UnsupportedExtension(
                extension.to_string(),
            )),
        }?;

        Ok(res)
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
    fn test_find_load() {
        let mut config = ConfigLoader::new("universal-config");
        let res: ConfigContents = config
            .with_config_dir("test_configs")
            .find_and_load()
            .unwrap();
        assert_eq!(res.test, "hello world")
    }
}