use std::path::PathBuf;

/// Represents the various errors that can occur in the application.
///
/// This enum defines different types of errors that can be encountered during the execution
/// of the application, including IO errors, configuration parsing errors, CSV parsing errors,
/// and custom errors related to CSV headers and reading CSV files.
///
/// # Variants
///
/// * `Io` - Represents an IO error that occurred during file operations.
/// * `ConfigParse` - Represents an error that occurred while parsing the `config.json` file.
/// * `CsvParse` - Represents an error that occurred while parsing a CSV file.
/// * `CsvHeaders` - Represents an error related to parsing CSV headers.
/// * `CsvRead` - Represents an error that occurred while reading a CSV file from the provided source.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Represents an IO error that occurred during file operations.
    ///
    /// This variant wraps a `std::io::Error` and provides a detailed error message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::fs::File;
    /// use std::io::Error as IoError;
    /// use crate::Error;
    ///
    /// fn open_file(path: &str) -> Result<File, Error> {
    ///     File::open(path).map_err(Error::from)
    /// }
    /// ```
    #[error("Failed due to IO Error: {0}")]
    Io(#[from] std::io::Error),

    /// Represents an error that occurred while parsing the `config.json` file.
    ///
    /// This variant wraps a `config::ConfigError` and provides a detailed error message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use config::Config;
    /// use crate::Error;
    ///
    /// fn load_config() -> Result<Config, Error> {
    ///     Config::builder().add_source(config::File::with_name("config.json")).build().map_err(Error::from)
    /// }
    /// ```
    #[error("Failed to parse config.json file: {0}")]
    ConfigParse(#[from] config::ConfigError),

    /// Represents an error that occurred while parsing a CSV file.
    ///
    /// This variant wraps a `csv::Error` and provides a detailed error message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use csv::Reader;
    /// use crate::Error;
    ///
    /// fn read_csv(path: &str) -> Result<Reader<File>, Error> {
    ///     Reader::from_path(path).map_err(Error::from)
    /// }
    /// ```
    #[error("Failed to parse CSV file: {0}")]
    CsvParse(#[from] csv::Error),

    /// Represents an error related to parsing CSV headers.
    ///
    /// This variant provides a detailed error message indicating the issue with the CSV headers.
    ///
    /// # Example
    ///
    /// ```rust
    /// use crate::Error;
    ///
    /// fn check_headers(headers: &str) -> Result<(), Error> {
    ///     if headers.is_empty() {
    ///         return Err(Error::CsvHeaders("Headers are empty".to_string()));
    ///     }
    ///     Ok(())
    /// }
    /// ```
    #[error("CSV Headers could not be parsed: {0}")]
    CsvHeaders(String),

    /// Represents an error that occurred while reading a CSV file from the provided source.
    ///
    /// This variant provides a detailed error message indicating the issue with reading the CSV file.
    ///
    /// # Example
    ///
    /// ```rust
    /// use crate::Error;
    /// use std::fs::File;
    ///
    /// fn read_csv_file(path: &str) -> Result<File, Error> {
    ///     File::open(path).map_err(|_| Error::CsvRead(format!("Failed to read CSV file from path: {}", path)))
    /// }
    /// ```
    #[error("Failed to read CSV file from source provided: {0}")]
    CsvRead(String),

    #[error("Failed to capture or parsee regex: {0}")]
    RegexCapture(String),

    #[error("Failed to parse config 'source'")]
    ConfigSource,

    #[error(
        "Ambiguous file match - Two files have the exact same name, modified timestamp and size - Congratulations, you've found a bug in Windows!"
    )]
    AmbiguousFileMatch,

    #[error("No matching files found")]
    NoMatchingFiles,

    #[error("Failed to find a parent path for the provided path: {0}. Please ensure the path is valid.")]
    NoParentPath(PathBuf),

    #[error("Failed to parse path: {0}")]
    ParsingPath(String),

    #[error("Failed to update the application: {0}")]
    SelfUpdateFailed(#[from] self_update::errors::Error),
}
