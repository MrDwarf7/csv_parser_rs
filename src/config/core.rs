// use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
// use std::hash::Hash;
use std::path::PathBuf;

use config::builder::DefaultState;

use crate::cli::{Cli, OutputType};
use crate::config::extract_cached_config_value;
use crate::config::file_path_finds::parse_user_variable_path;
use crate::prelude::{Deserialize, Serialize, *};

/// Represents the configuration settings for the application.
///
/// This struct is used to deserialize the configuration from a JSON file and holds various
/// settings required for processing CSV files.
///
/// # Fields
///
/// * `source` - The path to the source CSV file.
/// * `output_type` - The type of output (e.g., stdout, CSV file).
/// * `output_path` - The path to the output file.
/// * `has_headers` - A boolean indicating whether the CSV file has headers.
/// * `fields` - A vector of field names to be retained from the CSV file.
/// * `unique_fields` - A vector of field names to be used for deduplication.
/// * `include_cols_with` - A hashmap where the key is a column name and the value is a vector of valid values for filtering.
///
/// # Example
///
/// ```json
/// {
///   "source": "some\\winodws\\path\\to\\file.csv",
///   "output_type": "stdout",
///   "output_path": "some\\windows\\path\\to\\output.csv",
///   "has_headers": true,
///   "fields": [
///     "__fields_to_retain_always",
///     "__fields_to_retain_always2",
///     "__fields_to_retain_always3",
///     "__fields_to_retain_always4"
///   ],
///   "unique_fields": [
///   ],
///   "include_cols_with": {
///     "__fields_that_need_filtering_for_values": [
///       "__value_of_field_to_filter_for",
///       "__value_of_field_to_filter_for2",
///       "__value_of_field_to_filter_for3"
///     ],
///     "__fields_that_need_filtering_for_values_two": [
///       "__value_of_field_to_filter_for",
///       "__value_of_field_to_filter_for2",
///       "__value_of_field_to_filter_for3"
///     ]
///   }
/// }
/// ```
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

    pub include_cols_with: HashMap<String, Vec<String>>,
}

impl Config {
    /// Creates a new `Config` instance from the provided CLI arguments.
    ///
    /// This function creates a new `Config` instance - if CLI Arguments are provided they're used to override the configuration file.
    /// If no CLI arguments are provided, the function will search in the default location for one (or create one if it doesn't exist).
    /// If the conversion is successful, it checks if the `output_path` ends with a `.csv` extension.
    /// If not, it sets the extension to `.csv`.
    ///
    /// # Arguments
    ///
    /// * `cli` - A `Cli` instance containing the command-line arguments.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns a `Config` instance on success, or an `Error` on failure.
    ///
    /// # Example
    ///
    /// ```rust
    /// let cli = Cli::parse();
    /// let config = Config::new(cli).expect("Failed to create config");
    /// ```
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
    }
}

/// Ensures the existence of a configuration file in the specified directory.
///
/// This function checks if the configuration file exists in the given directory. If the file
/// does not exist or is empty, it creates the necessary directories and writes a default
/// configuration file. If the file already exists and is not empty, it simply returns the path
/// to the configuration file.
///
/// # Arguments
///
/// * `current_dir` - A `PathBuf` representing the current directory where the configuration file should be located.
///
/// # Returns
///
/// * `Result<PathBuf>` - Returns the path to the configuration file on success, or an `Error` on failure.
///
/// # Example
///
/// ```rust
/// let current_dir = std::env::current_dir().unwrap();
/// let config_path = config_file(current_dir).expect("Failed to ensure config file");
/// println!("Config file is located at: {:?}", config_path);
/// ```
pub(crate) fn config_file(current_dir: PathBuf) -> Result<PathBuf> {
    let def_config = Config::default();
    let config_folder = current_dir.join(DEFAULT_CONFIG_DIR);
    if !config_folder.exists() {
        std::fs::create_dir_all(&config_folder)?;
    }
    let config_file = config_folder.join(DEFAULT_CONFIG_FILE);
    if !config_file.exists() || config_file.metadata()?.len() == 0 {
        std::fs::write(&config_file, def_config.to_string())?;
        let msg = "Config file could not be found or had no content, one has been generated for you at:";
        error!("{}\n{:?}", msg, config_file.display());
        return Ok(current_dir);
    }

    Ok(config_file)
}

impl TryFrom<PathBuf> for Config {
    type Error = Error;

    /// Attempts to create a `Config` instance from a given `PathBuf`.
    ///
    /// This function reads the configuration file from the specified path and deserializes it
    /// into a `Config` instance. If the file cannot be read or deserialized, an error is returned.
    ///
    /// # Arguments
    ///
    /// * `path` - A `PathBuf` representing the path to the configuration file.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns a `Config` instance on success, or an `Error` on failure.
    ///
    /// # Example
    ///
    /// ```rust
    /// let config_path = PathBuf::from("config.json");
    /// let config = Config::try_from(config_path).expect("Failed to load config");
    /// ```
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

    /// Attempts to create a `Config` instance from the provided CLI arguments.
    ///
    /// This function first creates a default `Config` instance and then overrides its values
    /// with the CLI arguments. It also ensures that the configuration file exists and is valid.
    ///
    /// # Arguments
    ///
    /// * `cli` - A `Cli` instance containing the command-line arguments.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns a `Config` instance on success, or an `Error` on failure.
    ///
    /// # Example
    ///
    /// ```rust
    /// let cli = Cli::parse();
    /// let config = Config::try_from(cli).expect("Failed to create config from CLI");
    /// ```
    fn try_from(cli: Cli) -> Result<Self> {
        let default_config_base = Config::default();

        let builder = config::Config::builder()
            .add_source(config::Config::try_from(&default_config_base).map_err(Error::ConfigParse)?);

        let mut builder = cli_valid(builder, &cli)?;

        let config_file = config_file(crate::config::current_dir()?)?;
        // and finally - we attempt to parse the config file

        if let Some(cli_config_file) = &cli.config_file {
            builder = builder.set_override("config_file", cli_config_file.to_str().unwrap())?; // Set the new one the user provided
            builder = builder.add_source(config::File::from(cli_config_file.clone()));
        // Use the new one as the base
        } else {
            builder = builder.set_override("config_file", config_file.to_str().unwrap())?;
            builder = builder.add_source(config::File::from(config_file));
        }

        let config = builder.build().map_err(Error::ConfigParse)?;

        let fixed = fix_multiple_path_subs(&config, vec!["source", "output_path"]).unwrap_or_else(|e| {
            match e {
                Error::ParsingPathError(e) => warn!("Failed to fix multiple path substitutions: {}", e),
                _ => panic!("Failed to fix multiple path substitutions"),
            }
            vec![]
        });

        let fixed_source = fixed.first().unwrap();
        let fixed_output_path = fixed.get(1).unwrap();

        let mut config: Config = config.try_deserialize().expect("Failed to deserialize config");

        config.source = fixed_source.clone();
        config.output_path = fixed_output_path.clone();

        config = clear_placeholder_keys(config)?;

        Ok(config)
    }
}

/// remove any keys & values that start with __ as these are the 'default' filler keys
fn clear_placeholder_keys(mut config: Config) -> Result<Config> {
    config.fields.retain(|f| !f.starts_with("__"));
    config.include_cols_with.retain(|k, _| !k.starts_with("__"));
    Ok(config)
}

fn fix_multiple_path_subs(config: &config::Config, paths: Vec<&str>) -> Result<Vec<PathBuf>> {
    let mut extracted = vec![];

    #[allow(unused_assignments)]
    let mut last_path: Box<&str> = Box::default();
    for path in paths {
        last_path = Box::new(path);
        debug!("Attempting to extract path: {}", path);

        let extracted_path = extract_cached_config_value(config, path)?;
        let fixed_path = match parse_user_variable_path(&extracted_path) {
            Ok(f) => f,
            Err(_) => {
                if *last_path == path {
                    warn!("Failed to extract path: {}", path);
                    let extension_idx = extracted_path.rfind('.');
                    // we want to append "out" after the filename, but before the .extension
                    let fixed_path = match extension_idx {
                        Some(idx) => {
                            let (before, after) = extracted_path.split_at(idx);
                            format!("{}out{}", before, after)
                        }
                        None => format!("{}out", extracted_path),
                    };
                    extracted.push(PathBuf::from(fixed_path));
                    continue;
                } else {
                    unreachable!("Failed to extract path: {}", path);
                    // return Err(Error::ParsingPathError(e.to_string()));
                }
            }
        };
        extracted.push(fixed_path);
    }
    Ok(extracted)
}

/// Validates and overrides configuration settings with CLI arguments.
///
/// This function takes a `ConfigBuilder` and a `Cli` instance, and overrides the configuration
/// settings with the values provided via the CLI arguments. It ensures that the source path,
/// output type, and output path are correctly set in the configuration builder.
///
/// # Arguments
///
/// * `builder` - A `ConfigBuilder<DefaultState>` instance used to build the configuration.
/// * `cli` - A `Cli` instance containing the command-line arguments.
///
/// # Returns
///
/// * `Result<config::ConfigBuilder<DefaultState>>` - Returns the updated `ConfigBuilder` on success, or an `Error` on failure.
///
/// # Example
///
/// ```rust
/// let cli = Cli::parse();
/// let builder = config::Config::builder();
/// let builder = cli_valid(builder, &cli).expect("Failed to validate CLI arguments");
/// ```
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
        builder = builder.set_override("output_type", output_type.to_string().as_str())?;
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
    /// Provides a default `Config` instance.
    ///
    /// This implementation creates a `Config` instance using a predefined JSON string (`DEFAULT_FILLER`).
    /// It attempts to deserialize the JSON string into a `Config` instance and unwraps the result.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns a default `Config` instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// let default_config = Config::default();
    /// ```
    fn default() -> Self {
        Self::try_from(DEFAULT_FILLER).unwrap()
    }
}

impl TryFrom<&str> for Config {
    type Error = Error;

    /// Attempts to create a `Config` instance from a JSON string.
    ///
    /// This implementation reads the configuration from the provided JSON string and deserializes it
    /// into a `Config` instance. If the string cannot be deserialized, an error is returned.
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice containing the JSON configuration.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns a `Config` instance on success, or an `Error` on failure.
    ///
    /// # Example
    ///
    /// ```rust
    /// let json_str = r#"{"source": "data.csv", "output_type": "Csv", "output_path": "output.csv", "has_headers": true, "fields": ["field1", "field2"], "unique_fields": ["field1"], "include_cols_with": {"field1": ["value1", "value2"]}}"#;
    /// let config = Config::try_from(json_str).expect("Failed to create config from JSON string");
    /// ```
    fn try_from(s: &str) -> Result<Self> {
        let builder = config::Config::builder().add_source(config::File::from_str(s, config::FileFormat::Json));
        let config = builder.build().map_err(Error::ConfigParse)?;
        let config: Config = config.try_deserialize().map_err(Error::ConfigParse)?;

        Ok(config)
    }
}

impl Display for Config {
    /// Formats the `Config` instance as a pretty-printed JSON string.
    ///
    /// This implementation uses `serde_json` to serialize the `Config` instance into a pretty-printed JSON string.
    ///
    /// # Arguments
    ///
    /// * `f` - A mutable reference to a `Formatter`.
    ///
    /// # Returns
    ///
    /// * `std::fmt::Result` - Returns the result of the write operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// let config = Config::default();
    /// println!("{}", config);
    /// ```
    #[allow(clippy::write_with_newline)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?)
    }
}

impl Debug for Config {
    /// Formats the `Config` instance as a pretty-printed JSON string for debugging purposes.
    ///
    /// This implementation uses `serde_json` to serialize the `Config` instance into a pretty-printed JSON string.
    ///
    /// # Arguments
    ///
    /// * `f` - A mutable reference to a `Formatter`.
    ///
    /// # Returns
    ///
    /// * `std::fmt::Result` - Returns the result of the write operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// let config = Config::default();
    /// println!("{:?}", config);
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).map_err(|_| std::fmt::Error)?)
    }
}
