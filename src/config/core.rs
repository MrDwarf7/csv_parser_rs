use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;

use clap::ValueEnum;

use crate::cli::Cli;
use crate::prelude::{Deserialize, Serialize, *};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "source")]
    pub source: Option<PathBuf>,
    #[serde(rename = "output_type")]
    pub output: Option<Output>,
    pub threads: usize,
    pub fields: Vec<String>,
    pub filter_by: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, ValueEnum)]
pub enum Output {
    #[value(name = "STDOUT", alias = "stdout", alias = "Stdout", alias = "0")]
    #[serde(rename = "stdout")]
    Stdout,

    #[value(name = "CSV", alias = "csv", alias = "Csv", alias = "1")]
    #[serde(rename = "csv")]
    Csv,
}

impl Config {
    pub fn new(cli: &Cli) -> Result<Self> {
        #[rustfmt::skip]
        let filepath = Self::config_file(
            crate::config::config_dir()?
        )?;

        // TODO: double check logic order here when not feeling like death

        let mut output = None;
        if let Some(cli_output) = &cli.output {
            output = Some(*cli_output);
        }

        if let Some(source) = &cli.source {
            return Self::cli_source(source.to_path_buf(), filepath);
        }

        if let Some(config_file) = &cli.config_file {
            return Self::cli_config(config_file.to_path_buf(), filepath);
        }

        let mut config = Self::try_from(filepath)?;
        config.output = output;

        Ok(config)
    }

    fn cli_source(cli_source: PathBuf, standard_path: PathBuf) -> crate::Result<Self> {
        let mut s = Self::try_from(standard_path.clone())?;
        s.source = Some(cli_source);
        Ok(s)
    }

    fn cli_config(cli_config: PathBuf, standard_path: PathBuf) -> crate::Result<Self> {
        Ok(Self::try_from(cli_config).unwrap_or_else(|_| Self::try_from(standard_path).unwrap()))
    }

    fn config_file(path: PathBuf) -> crate::Result<PathBuf> {
        let folder = path.join(DEFAULT_CONFIG_DIR);
        if !folder.exists() {
            std::fs::create_dir_all(&folder)?;
        }

        let file = folder.join(DEFAULT_CONFIG_FILE);
        if !file.exists() || file.metadata()?.len() == 0 {
            std::fs::write(&file, Self::default().to_string())?;
            let msg = "Config file could not be found or had no content, one has been generated for you at:";
            println!("{}\n{:?}", msg, file.to_path_buf());
            std::process::exit(0);
        }

        Ok(file)
    }
}

impl TryFrom<&str> for Config {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        let builder = config::Config::builder().add_source(config::File::from_str(s, config::FileFormat::Json));
        let config = builder.build().map_err(Error::ConfigParse)?;
        let config: Config = config.try_deserialize().map_err(Error::ConfigParse)?;

        Ok(config)
    }
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

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", s)
    }
}
