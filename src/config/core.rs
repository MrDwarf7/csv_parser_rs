use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;

use crate::cli::{Cli, OutputType};
use crate::prelude::{Deserialize, Serialize, *};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(rename = "source")]
    pub source: PathBuf,

    #[serde(rename = "output_type", default)]
    pub output_type: OutputType,

    #[serde(rename = "output_path", default)]
    pub output_path: PathBuf,

    pub fields: Vec<String>,

    pub filter_by: HashMap<String, Vec<String>>,
}

impl Config {
    pub fn new(cli: &Cli) -> Result<Self> {
        let config = match &cli.config_file {
            Some(user_provided_file) => {
                if user_provided_file.exists() {
                    Self::try_from(user_provided_file.to_path_buf())?
                } else {
                    eprintln!("Config file provided does not exist, generating a new one...");
                    Self::write_as_default()?
                }
            }
            None => Self::try_from(Self::write_as_default()?.to_string().as_str())?,
        };

        Self::handle_overrides(cli, config)
    }

    fn handle_overrides(cli: &Cli, mut con: Config) -> Result<Config> {
        // let base_path_dir = crate::config::current_dir()?;

        if let Some(user_provided_config_file) = &cli.config_file {
            if !user_provided_config_file.exists() {
                eprintln!("Config file provided does not exist, generating a new one...");
                // Self::write_as_default()?;
            }
            con = Self::try_from(user_provided_config_file.to_path_buf())?;
        }

        if let Some(output_type) = &cli.output_type {
            if *output_type != con.output_type {
                con.output_type = *output_type;
            }
        }

        if let Some(source) = &cli.source {
            con.source = source.clone();
        }

        if let Some(output_path) = &cli.output_path {
            if *output_path != con.output_path {
                // con.output_path = Self::make_output_path(con.source.clone(), output_path.clone())?;
                con.output_path = make_output_path(output_path.clone(), con.source.clone())
                    .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, e)))?;
            }
        }

        Ok(con)
    }

    fn write_as_default() -> Result<Self> {
        let def = Self::default();
        let current_dir = crate::config::current_dir()?;
        let config_filepath = config_file(current_dir, def.clone())?;
        std::fs::write(&config_filepath, def.to_string())?;
        Ok(def)
    }
}

fn config_file(current_dir: PathBuf, def_config: Config) -> Result<PathBuf> {
    let config_folder = current_dir.join(DEFAULT_CONFIG_DIR);
    if !config_folder.exists() {
        std::fs::create_dir_all(&config_folder)?;
    }
    let config_file = config_folder.join(DEFAULT_CONFIG_FILE);
    if !config_file.exists() || config_file.metadata()?.len() == 0 {
        std::fs::write(&config_file, def_config.to_string())?;
        let msg = "Config file could not be found or had no content, one has been generated for you at:";
        eprintln!("{}\n{:?}", msg, config_file.to_path_buf());
        return Ok(current_dir);
    }

    Ok(config_file)
}

trait PathBufExt {
    fn pop_if_dir(&mut self);

    fn pop_if_extension(&mut self) -> Self;
}

impl PathBufExt for PathBuf {
    fn pop_if_dir(&mut self) {
        if self.ends_with("\\") || self.ends_with("/") {
            self.pop();
        }
    }

    fn pop_if_extension(&mut self) -> Self {
        if self.extension().is_some() {
            self.set_extension("");
        }
        self.clone()
    }
}

fn make_output_path(mut user_output: PathBuf, user_source: PathBuf) -> Result<PathBuf> {
    if user_output.exists() {
        // 'User is affected by confusion!'
        if user_output == user_source {
            user_output.set_extension("csv");
            return Ok(user_output);
        }

        user_output.pop_if_dir();
        user_output.pop_if_extension();
        user_output.push(user_source.file_name().ok_or_else(|| {
            Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "No file name found in source path"))
        })?);
    }

    if user_output.extension().is_none() {
        user_output.set_extension("csv");
    }

    // Path is a `simple filename` like 'file' or 'file.csv' -- user assumes output will be in the same directory as the source
    if user_output.is_relative() {
        user_output = user_source
            .parent()
            .ok_or_else(|| {
                Error::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "No parent directory found for source path",
                ))
            })?
            .join(user_output);
    }

    Ok(user_output)
}

impl TryFrom<PathBuf> for Config {
    type Error = Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        let builder = config::Config::builder().add_source(config::File::from(path));
        let config = builder.build().map_err(Error::ConfigParse)?;
        let config: Config = config.try_deserialize().map_err(Error::ConfigParse)?;

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::try_from(DEFAULT_FILLER).unwrap()
    }
}

// This is to be able to serialize the Config struct from the DEFAULT_FILLER string
impl TryFrom<&str> for Config {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        let builder = config::Config::builder().add_source(config::File::from_str(s, config::FileFormat::Json));
        let config = builder.build().map_err(Error::ConfigParse)?;
        let config: Config = config.try_deserialize().map_err(Error::ConfigParse)?;

        Ok(config)
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod output_tests {
    use std::fs;

    use tempdir::TempDir;

    use super::*;

    #[test]
    fn test_make_output_path_absolute_server_path() {
        let temp_dir = TempDir::new("test").unwrap();
        let source_path = temp_dir.path().join("source.txt");
        fs::write(&source_path, "test").unwrap();

        let output_path = PathBuf::from("\\some\\server\\path\\here\\file"); //This will likely fail on non-windows systems
        let result = make_output_path(output_path.clone(), source_path);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("C:\\some\\server\\path\\here\\file.csv"));
    }

    #[test]
    fn test_make_output_path_absolute_server_path_csv() {
        let temp_dir = TempDir::new("test").unwrap();
        let source_path = temp_dir.path().join("source.txt");
        fs::write(&source_path, "test").unwrap();

        let output_path = PathBuf::from("C:\\some\\server\\path\\here\\file.csv"); //This will likely fail on non-windows systems
        let result = make_output_path(output_path.clone(), source_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("C:\\some\\server\\path\\here\\file.csv"));
    }

    #[test]
    fn test_make_output_path_relative_path() {
        let temp_dir = TempDir::new("test").unwrap();
        let source_path = temp_dir.path().join("source.txt");
        fs::write(&source_path, "test").unwrap();

        let output_path = PathBuf::from("file");
        let result = make_output_path(output_path, source_path.clone());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path().join("file.csv"));
    }

    #[test]
    fn test_make_output_path_relative_path_csv() {
        let temp_dir = TempDir::new("test").unwrap();
        let source_path = temp_dir.path().join("source.txt");
        fs::write(&source_path, "test").unwrap();

        let output_path = PathBuf::from("file.csv");
        let result = make_output_path(output_path, source_path.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path().join("file.csv"));
    }

    #[test]
    // #[should_panic]
    fn test_make_output_path_existing_file() {
        let temp_dir = TempDir::new("test").unwrap();

        let source_path = temp_dir.path().join("source.csv");
        fs::write(&source_path, "test").unwrap();

        let output_path = temp_dir.path().join("file.csv");
        fs::write(&output_path, "test").unwrap();

        let result = make_output_path(output_path.clone(), source_path.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path().join("file").join("source.csv"));
    }

    #[test]
    fn test_make_output_path_absolute_server_path_txt() {
        let temp_dir = TempDir::new("test").unwrap();
        let source_path = temp_dir.path().join("source.txt");
        fs::write(&source_path, "test").unwrap();

        let output_path = PathBuf::from("/some/server/path/here/file.txt"); //Using a more portable path format
        let result = make_output_path(output_path, source_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("C:/some/server/path/here/file.txt"));
    }

    #[test]
    fn test_make_output_path_relative_path_txt() {
        let temp_dir = TempDir::new("test").unwrap();
        let source_path = temp_dir.path().join("source.txt");
        fs::write(&source_path, "test").unwrap();

        let output_path = PathBuf::from("file.txt");
        let result = make_output_path(output_path, source_path.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path().join("file.txt"));
    }
    #[test]
    fn test_make_output_path_same_as_source() {
        let temp_dir = TempDir::new("test").unwrap();
        let source_path = temp_dir.path().join("source.txt");
        fs::write(&source_path, "test").unwrap();

        let output_path = source_path.clone();
        let result = make_output_path(output_path, source_path.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path().join("source.csv"));
    }
}
