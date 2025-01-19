use std::ffi::OsStr;
use std::fmt::{Debug, Display};
use std::ops::Not;
use std::path::PathBuf;

use clap::{Parser, ValueEnum, command};
use stderrlog::LogLevelNum;

use crate::prelude::{Deserialize, Serialize, *};

/// Command Line Interface (CLI) structure for the `parse_csv_rs` tool.
///
/// This structure defines the CLI arguments and options for the `parse_csv_rs` tool, which is designed
/// to parse a CSV file and filter out rows based on a set of criteria. The criteria can be defined via
/// a configuration file or overridden via CLI arguments. If CLI arguments are provided, they will always
/// override the values in the configuration file.
///
/// # Fields
///
/// * `source` - The source CSV file to parse. This argument is optional and overrides the source file in the configuration file if provided.
/// * `config_file` - The configuration file to use. This option is optional and overrides the default configuration file.
/// * `output_type` - The output type to use. This option is optional and specifies the format of the output.
/// * `output_path` - The output file path to use. This option is optional and specifies the path where the output file will be saved.
///
/// # Example
///
/// ```rust
/// let cli = Cli::parse();
/// println!("{:?}", cli);
/// ```
#[derive(Debug, Parser, Clone)]
#[command(
    name = "parse_csv_rs",
    about = "A CLI tool to parse a CSV file and filter out rows based on a set of criteria.",
    author = "Blake B.",
    long_about = "\n
    This CLI tool is designed to parse a CSV file and filter out columns & rows based on a set of criteria.
    The criteria are defined via a configuration file, or overrideable via CLI arguments.\n
    If Command Line Interface (CLI) arguments are provided, they will always override the values in the configuration file.",
    version = clap::crate_version!(),
    arg_required_else_help = false,
    styles=get_styles()
)]
#[rustfmt::skip]
pub struct Cli {
    /// The source CSV file to parse.
    #[arg(name = "source", index = 1, help = "The source CSV file to parse - overrides the source file in the config file file if provided.", required = false, value_hint = clap::ValueHint::FilePath)]
    pub source: Option<PathBuf>,

    /// The configuration file to use - overrides the default configuration file.
    #[arg(name = "config_file", short = 'c', long = "config", help = "The configuration file to use - overrides the default configuration file.", required = false, value_hint = clap::ValueHint::FilePath)]
    pub config_file: Option<PathBuf>,
    
    /// The output type to use.
    #[arg(name = "output_type", short = 't', long = "output_type", help = "The output type to use.", required = false, value_hint = clap::ValueHint::Other, value_enum,)]
    pub output_type: Option<OutputType>,

    #[arg(name = "output_path", short = 'o', long = "output_path", help = "The output file path to use.", required = false, value_hint = clap::ValueHint::FilePath)]
    pub output_path: Option<PathBuf>,
    
    /// Optional verbosity level of the logger.
    /// You may provide this as either a string or a number.
    ///
    /// The least verbose as 0 (Error -> Error Only)
    /// Most verbose as 4 (Trace -> Trace Everything
    /// If not provided, the default value is "INFO".
    #[arg(value_enum, name = "verbosity", short = 'v', long = "verbosity", help = "The verbosity level of the logger.", required = false, default_value = "INFO", value_hint = clap::ValueHint::Other)]
    pub verbosity_level: Option<VerbosityLevel>,
    
}

/// The verbosity level of the logger.
///
/// The least verbose as 0 (Error -> Error Only)
/// Most verbose as 4 (Trace -> Trace Everything).
#[derive(Debug, ValueEnum, Clone, Copy, PartialEq, Eq)]
#[clap(name = "VerbosityLevel", rename_all = "upper")]
pub enum VerbosityLevel {
    #[value(name = "ERROR", alias = "error", alias = "Error", alias = "0")]
    Error,
    #[value(name = "WARN", alias = "warn", alias = "Warn", alias = "1")]
    Warn,
    #[value(name = "INFO", alias = "info", alias = "Info", alias = "2")]
    Info,
    #[value(name = "DEBUG", alias = "debug", alias = "Debug", alias = "3")]
    Debug,
    #[value(name = "TRACE", alias = "trace", alias = "Trace", alias = "4")]
    Trace,
}

impl From<VerbosityLevel> for LogLevelNum {
    fn from(value: VerbosityLevel) -> Self {
        match value {
            VerbosityLevel::Error => LogLevelNum::Error,
            VerbosityLevel::Warn => LogLevelNum::Warn,
            VerbosityLevel::Info => LogLevelNum::Info,
            VerbosityLevel::Debug => LogLevelNum::Debug,
            VerbosityLevel::Trace => LogLevelNum::Trace,
        }
    }
}

/// Represents the output type for the `parse_csv_rs` tool.
///
/// This enum defines the possible output types for the tool, which can be either `Stdout` or `Csv`.
/// It supports serialization and deserialization using `serde`, and can be used as a value enum in CLI arguments.
///
/// # Variants
///
/// * `Stdout` - Represents output to the standard output.
/// * `Csv` - Represents output to a CSV file.
///
/// # Example
///
/// ```rust
/// use crate::OutputType;
///
/// let output_type = OutputType::Stdout;
/// println!("{}", output_type); // prints "stdout"
/// ```
#[derive(Serialize, Deserialize, Clone, Copy, ValueEnum)]
pub enum OutputType {
    #[value(name = "stdout", alias = "stdout", alias = "Stdout", alias = "0")]
    #[serde(rename = "stdout")]
    Stdout,

    #[value(name = "csv", alias = "csv", alias = "Csv", alias = "1")]
    #[serde(rename = "csv")]
    Csv,
}

impl Debug for OutputType {
    /// Formats the `OutputType` for debugging purposes.
    ///
    /// This implementation provides a human-readable representation of the `OutputType` variants.
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
    /// let output_type = OutputType::Stdout;
    /// println!("{:?}", output_type); // prints "OutputType::Stdout"
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputType::Stdout => write!(f, "OutputType::Stdout"),
            OutputType::Csv => write!(f, "OutputType::Csv"),
        }
    }
}

impl Display for OutputType {
    /// Formats the `OutputType` as a string.
    ///
    /// This implementation provides a human-readable representation of the `OutputType` variants.
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
    /// let output_type = OutputType::Stdout;
    /// println!("{}", output_type); // prints "stdout"
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputType::Stdout => write!(f, "stdout"),
            OutputType::Csv => write!(f, "csv"),
        }
    }
}

impl From<OutputType> for String {
    /// Converts the `OutputType` to a `String`.
    ///
    /// This implementation provides a string representation of the `OutputType` variants.
    ///
    /// # Arguments
    ///
    /// * `output_type` - The `OutputType` instance to convert.
    ///
    /// # Returns
    ///
    /// * `String` - Returns the string representation of the `OutputType`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let output_type = OutputType::Stdout;
    /// let output_type_str: String = output_type.into();
    /// println!("{}", output_type_str); // prints "stdout"
    /// ```
    fn from(output_type: OutputType) -> Self {
        match output_type {
            OutputType::Stdout => "stdout".to_string(),
            OutputType::Csv => "csv".to_string(),
        }
    }
}

impl AsRef<OsStr> for OutputType {
    /// Converts the `OutputType` to an `OsStr`.
    ///
    /// This implementation provides an OS string representation of the `OutputType` variants.
    ///
    /// # Arguments
    ///
    /// * `self` - The `OutputType` instance to convert.
    ///
    /// # Returns
    ///
    /// * `&OsStr` - Returns the OS string representation of the `OutputType`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let output_type = OutputType::Stdout;
    /// let output_type_os_str: &OsStr = output_type.as_ref();
    /// println!("{:?}", output_type_os_str); // prints "stdout"
    /// ```
    fn as_ref(&self) -> &OsStr {
        match self {
            OutputType::Stdout => OsStr::new("stdout"),
            OutputType::Csv => OsStr::new("csv"),
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for OutputType {
    /// Provides a default `OutputType` instance.
    ///
    /// This implementation returns `OutputType::Stdout` as the default value.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns `OutputType::Stdout` as the default value.
    ///
    /// # Example
    ///
    /// ```rust
    /// let default_output_type = OutputType::default();
    /// asser
    fn default() -> Self {
        OutputType::Stdout
    }
}

impl PartialEq for OutputType {
    /// Compares two `OutputType` instances for equality.
    ///
    /// This implementation checks if both instances are either `OutputType::Stdout` or `OutputType::Csv`.
    ///
    /// # Arguments
    ///
    /// * `self` - The first `OutputType` instance.
    /// * `other` - The second `OutputType` instance to compare with.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns `true` if both instances are equal, otherwise `false`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let output_type1 = OutputType::Stdout;
    /// let output_type2 = OutputType::Stdout;
    /// assert_eq!(output_type1, output_type2);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        matches!((self, other), (OutputType::Stdout, OutputType::Stdout) | (OutputType::Csv, OutputType::Csv))
    }
}

impl Eq for OutputType {}

impl Not for OutputType {
    type Output = OutputType;

    /// Implements the logical NOT operator for `OutputType`.
    ///
    /// This implementation toggles between `OutputType::Stdout` and `OutputType::Csv`.
    ///
    /// # Returns
    ///
    /// * `OutputType` - Returns `OutputType::Csv` if the current instance is `OutputType::Stdout`, and vice versa.
    ///
    /// # Example
    ///
    /// ```rust
    /// let output_type = OutputType::Stdout;
    /// let toggled_output_type = !output_type;
    /// assert_eq!(toggled_output_type, OutputType::Csv);
    /// ```
    fn not(self) -> Self::Output {
        match self {
            OutputType::Stdout => OutputType::Csv,
            OutputType::Csv => OutputType::Stdout,
        }
    }
}

impl Cli {
    /// Creates a new `Cli` instance by parsing the command-line arguments and setting environment variables.
    ///
    /// This implementation parses the CLI arguments and attempts to set the corresponding environment variables.
    /// If setting the environment variables fails, an error message is printed.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns a new `Cli` instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// let cli = Cli::new();
    /// println!("{:?}", cli);
    /// ```
    pub fn new() -> Self {
        let s = Self::parse();
        s.to_env()
            .unwrap_or_else(|e| error!("Error setting environment variables: {e}"));
        s
    }
}

impl Default for Cli {
    /// Provides a default `Cli` instance.
    ///
    /// This implementation creates a new `Cli` instance by calling the `new` method.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns a default `Cli` instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// let default_cli = Cli::default();
    /// println!("{:?}", default_cli);
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

pub trait ToEnv {
    /// Trait to set environment variables based on the CLI arguments.
    ///
    /// This trait defines a method to set environment variables using the values provided in the CLI arguments.
    ///
    /// # Methods
    ///
    /// * `to_env` - Sets the environment variables based on the CLI arguments.
    ///
    /// # Example
    ///
    /// ```rust
    /// let cli = Cli::parse();
    /// cli.to_env().expect("Failed to set environment variables");
    /// ```
    fn to_env(&self) -> Result<()>;
}

impl ToEnv for Cli {
    /// Implementation of the `ToEnv` trait for the `Cli` struct.
    ///
    /// This implementation sets environment variables using the values provided in the `Cli` struct.
    /// It sets the environment variables for the source file, configuration file, output type, and output path.
    ///
    /// # Example
    ///
    /// ```rust
    /// let cli = Cli::parse();
    /// cli.to_env().expect("Failed to set environment variables");
    /// ```
    fn to_env(&self) -> Result<()> {
        let (source, config_file, output_type, output_path) =
            (self.source.as_ref(), self.config_file.as_ref(), self.output_type.as_ref(), self.output_path.as_ref());

        let prefix = &CLI_ENV_PREFIX;

        if let Some(source) = source {
            let source_name = format!("{}_{}", prefix, "SOURCE");
            // Safety:
            // It's safe to use set_var here as we're running in a single-threaded environment.
            unsafe {
                std::env::set_var(&source_name, source);
            }
        }

        if let Some(config_file) = config_file {
            let config_file_name = format!("{}_{}", prefix, "CONFIG_FILE");

            // Safety:
            // It's safe to use set_var here as we're running in a single-threaded environment.
            unsafe {
                std::env::set_var(&config_file_name, config_file);
            }
        }

        if let Some(output_type) = output_type {
            let output_type_name = format!("{}_{}", prefix, "OUTPUT_TYPE");

            // Safety:
            // It's safe to use set_var here as we're running in a single-threaded environment.
            unsafe {
                std::env::set_var(&output_type_name, output_type);
            }
            std::env::var(&output_type_name).unwrap();
        }

        if let Some(output_path) = output_path {
            let output_path_name = format!("{}_{}", prefix, "OUTPUT_PATH");

            // Safety:
            // It's safe to use set_var here as we're running in a single-threaded environment.
            unsafe {
                std::env::set_var(&output_path_name, output_path);
            }
        }

        Ok(())
    }
}

/// Returns a set of custom styles for the CLI tool.
///
/// This function defines and returns a set of styles to be used in the CLI tool's help and error messages.
/// The styles include formatting for usage, headers, literals, invalid inputs, errors, valid inputs, and placeholders.
///
/// # Returns
///
/// * `clap::builder::Styles` - Returns a `Styles` instance with the defined custom styles.
///
/// # Example
///
/// ```rust
/// let styles = get_styles();
/// ```
pub fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))), // When a command is inc. This is the tag collor for 'Usage:'
        )
        .header(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Blue))), // Main headers in the help menu (e.g. Arguments, Options)
        )
        .literal(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::BrightWhite))), // Strings for args etc. { -t, --total }
        )
        .invalid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .error(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red)))
                .effects(anstyle::Effects::ITALIC),
        )
        .valid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Cyan))),
        )
        .placeholder(anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::White))))
}
