use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::path::PathBuf;

use config::builder::DefaultState;

use crate::cli::{Cli, OutputType};
use crate::prelude::{Deserialize, Serialize, *};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(rename = "source")]
    pub source: PathBuf,

    #[serde(rename = "output_type", default)]
    pub output_type: OutputType,

    #[serde(rename = "output_path", default)]
    pub output_path: PathBuf,

    #[serde(rename = "has_headers", default)]
    pub has_headers: bool,

    pub fields: Vec<String>,

    pub unique_fields: Vec<String>,

    pub filter_by: HashMap<String, Vec<String>>,
}

impl Config {
    pub fn new(cli: Cli) -> Result<Self> {
        let config_from_cli = Self::try_from(cli);

        let mut config = match config_from_cli {
            Ok(c) => c,
            Err(e) => {
                return Err(e);
            }
        };

        if config.output_path.ends_with(".csv") {
            Ok(config)
        } else {
            config.output_path.set_extension("csv");
            Ok(config)
        }

        // let output_path = make_output_path(finished_config.output_path.clone(), finished_config.source.clone())?;
        // println!("Config::new:: output_path: {:#?}", &output_path);
        // finished_config.output_path = output_path;
        // Ok(finished_config)
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

impl TryFrom<PathBuf> for Config {
    type Error = Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        let builder = config::Config::builder().add_source(config::File::from(path));
        let config = builder
            .build()
            .map_err(Error::ConfigParse)
            .expect("Config::try_from:: builder.build()");
        let config: Config = config.try_deserialize().map_err(Error::ConfigParse)?;

        Ok(config)
    }
}

impl TryFrom<Cli> for Config {
    type Error = Error;

    fn try_from(cli: Cli) -> Result<Self> {
        let default_config_base = Config::default();

        let builder = config::Config::builder()
            .add_source(config::Config::try_from(&default_config_base).map_err(Error::ConfigParse)?);

        let mut builder = cli_valid(builder, &cli)?;

        let config_file = config_file(crate::config::current_dir()?, Config::default())?;
        // and finally - we attempt to parse the config file

        if let Some(cli_config_file) = &cli.config_file {
            builder = builder.set_override("config_file", cli_config_file.to_str().unwrap())?;
            builder = builder.add_source(config::File::from(cli_config_file.clone()));
        } else {
            builder = builder.set_override("config_file", config_file.to_str().unwrap())?;
            builder = builder.add_source(config::File::from(config_file));
        }

        let config = builder.build().map_err(Error::ConfigParse)?;
        let mut config: Config = config.try_deserialize().map_err(Error::ConfigParse)?;

        // remove any keys & values that start with __ as these are the 'default' filler keys
        config.fields.retain(|f| !f.starts_with("__"));
        config.filter_by.retain(|k, _| !k.starts_with("__"));

        Ok(config)
    }
}

fn cli_valid(builder: config::ConfigBuilder<DefaultState>, cli: &Cli) -> Result<config::ConfigBuilder<DefaultState>> {
    let mut builder = builder;
    // handling anything that came in via the CLI
    if let Some(source) = &cli.source {
        builder = builder.set_override(
            "source",
            source
                .to_str()
                .ok_or_else(|| Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "No source path found")))?,
        )?;
    }
    if let Some(output_type) = &cli.output_type {
        builder = builder.set_override("output_type", output_type.to_string().as_str())?
    }
    if let Some(output_path) = &cli.output_path {
        builder = builder.set_override(
            "output_path",
            output_path
                .to_str()
                .ok_or_else(|| Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "No output path found")))?,
        )?;
    };
    Ok(builder)
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
    #[allow(clippy::write_with_newline)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?)
    }
}

impl Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?)
    }
}
