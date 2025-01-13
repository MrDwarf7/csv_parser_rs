pub(crate) mod cli;
pub(crate) mod config;
pub(crate) mod error;
pub(crate) mod prelude;
pub(crate) mod processor;
pub(crate) mod retained;

use std::fs::File;
use std::marker::PhantomData;

use crate::cli::{Cli, OutputType};
use crate::config::Config;
pub(crate) use crate::prelude::*;
use crate::processor::{CsvHandler, Processor};
use crate::retained::RetainedData;

/// The main entry point of the application.
///
/// This function performs the following steps:
/// 1. Initializes the `Cli` instance to parse command-line arguments.
/// 2. Creates a new `State` instance based on the `Cli` input.
/// 3. Processes the CSV data using the `State` instance.
/// 4. Deduplicates the retained data if unique fields are specified in the configuration.
/// 5. Outputs the retained data based on the configured output type.
///
/// # Returns
///
/// * `Result<()>` - Returns `Ok(())` on success, or an `Error` on failure.
///
/// # Errors
///
/// This function can return errors in the following cases:
/// * If the `State` instance cannot be created.
/// * If processing the CSV data fails.
/// * If outputting the retained data fails.
///
/// # Example
///
/// ```rust
/// fn main() {
///     if let Err(e) = main() {
///         eprintln!("Application error: {e}");
///     }
/// }
/// ```
pub fn main() -> Result<()> {
    let cli = Cli::new();

    let mut state = State::new(cli)?;
    println!("MAIN:: Config: {:#?}", &state.config);

    if let Err(proc_err) = state.process(&mut csv::Reader::from_path(&state.config.source)?) {
        eprintln!("Error processing: {proc_err}");
    }

    if !state.config.unique_fields.is_empty() || state.config.unique_fields.len().gt(&1) {
        state.deduplicate();
    } else {
        eprintln!("No unique fields provided, skipping deduplication");
    }

    match state.output() {
        Ok(_) => {
            println!("Output successful");
            Ok(())
        }
        Err(e) => {
            eprintln!("Error outputting: {e}");
            Err(e)
        }
    }
}

/// Represents the state of the application, encapsulating configuration, data, and processing components.
///
/// This struct holds the configuration settings, retained data, CSV handler, processor, and output data.
/// It also includes a phantom data marker to manage lifetimes.
///
/// # Fields
///
/// * `config` - The configuration settings for the application.
/// * `retained_data` - The data retained after processing the CSV file.
/// * `handler` - The handler for managing CSV processing.
/// * `processor` - The processor for performing data processing tasks.
/// * `output_data` - The data related to output configuration.
/// * `phantom_data` - A phantom data marker to manage lifetimes.
///
/// # Example
///
/// ```rust
/// let cli = Cli::new();
/// let state = State::new(cli).expect("Failed to create state");
/// ```
pub struct State<'a> {
    pub config: Config,
    pub retained_data: RetainedData,
    pub handler: CsvHandler,
    pub processor: Processor,
    output_data: OutputData,
    phantom_data: PhantomData<&'a ()>,
}

impl State<'_> {
    /// Creates a new `State` instance by initializing its components based on the provided `Cli` input.
    ///
    /// This function performs the following steps:
    /// 1. Parses the configuration from the `Cli` input.
    /// 2. Initializes a CSV reader with the configuration settings.
    /// 3. Creates an `OutputData` instance based on the configuration.
    /// 4. Initializes `RetainedData` with the appropriate capacity.
    /// 5. Creates a `CsvHandler` to manage CSV processing, using the configuration and headers from the CSV reader.
    /// 6. Initializes a `Processor` for data processing tasks.
    /// 7. Constructs and returns a `State` instance containing all the initialized components.
    ///
    /// # Arguments
    ///
    /// * `cli` - A `Cli` instance containing command-line arguments and options.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Returns a `State` instance on success, or an `Error` on failure.
    ///
    /// # Errors
    ///
    /// This function can return errors in the following cases:
    /// * If the configuration cannot be parsed.
    /// * If the CSV file cannot be read from the specified path.
    /// * If the CSV headers cannot be parsed.
    ///
    /// # Example
    ///
    /// ```rust
    /// let cli = Cli::new();
    /// let state = State::new(cli).expect("Failed to create state");
    /// ```
    pub fn new(cli: Cli) -> Result<Self> {
        let config = Config::new(cli)?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(config.has_headers)
            .from_path(&config.source)
            .map_err(|e| Error::CsvRead(format!("Failed to read CSV file from source provided: {e}")))?;

        let output_data = OutputData::new(config.output_type, config.output_path.clone());

        let mut retained_data = RetainedData::new(config.fields.len());

        #[rustfmt::skip]
        let handler = CsvHandler::new(
            &config,
            &mut retained_data,
            rdr.headers().map_err(|e| {
                Error::CsvHeaders(e.to_string())
            })?,
        );

        let processor = Processor::new(&config);

        Ok(Self {
            config,
            retained_data,
            handler,
            processor,
            output_data,
            phantom_data: PhantomData,
        })
    }
}

impl State<'_> {
    /// Processes the CSV data using the `Processor` and updates the retained data.
    ///
    /// This function delegates the processing of CSV data to the `Processor`
    /// and updates the `retained_data` with the results.
    ///
    /// # Arguments
    ///
    /// * `rdr` - A mutable reference to a `csv::Reader` instance for reading the CSV data.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns `Ok(())` on success, or an `Error` on failure.
    ///
    /// # Errors
    ///
    /// This function can return errors if the `Processor` encounters issues during processing.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut rdr = csv::Reader::from_path("data.csv").expect("Failed to open CSV file");
    /// state.process(&mut rdr).expect("Failed to process CSV data");
    /// ```
    pub fn process(&mut self, rdr: &mut csv::Reader<File>) -> Result<()> {
        Processor::process(&mut self.retained_data, &self.handler, rdr)
    }

    /// Deduplicates the retained data using the `Processor`.
    ///
    /// This function calls the `deduplicate` method of the `Processor`
    /// to remove duplicate entries from the `retained_data`.
    ///
    /// # Example
    ///
    /// ```rust
    /// state.deduplicate();
    /// ```
    pub fn deduplicate(&mut self) {
        self.processor.deduplicate(&mut self.retained_data);
    }

    /// Outputs the retained data based on the configured output type.
    ///
    /// This function writes the retained data to either stdout or a CSV file,
    /// depending on the `output_type` specified in the configuration.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns `Ok(())` on success, or an `Error` on failure.
    ///
    /// # Errors
    ///
    /// This function can return errors if writing to stdout or the CSV file fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// state.output().expect("Failed to output data");
    /// ```
    pub fn output(&self) -> Result<()> {
        match self.output_data.output_type {
            OutputType::Stdout => {
                self.retained_data.to_stdout()?;
            }
            OutputType::Csv => {
                self.retained_data.to_csv(self.output_data.output_path.clone())?;
            }
        }

        Ok(())
    }
}

/// Represents the output data configuration for the application.
///
/// This struct holds the output type and the path where the output data will be written.
///
/// # Fields
///
/// * `output_type` - The type of output (e.g., stdout, CSV file).
/// * `output_path` - The path to the output file.
///
/// # Example
///
/// ```rust
/// let output_data = OutputData::new(OutputType::Csv, PathBuf::from("output.csv"));
/// ```
#[derive(Debug)]
struct OutputData {
    output_type: OutputType,
    output_path: PathBuf,
}

impl OutputData {
    /// Creates a new `OutputData` instance with the specified output type and path.
    ///
    /// # Arguments
    ///
    /// * `output_type` - The type of output (e.g., stdout, CSV file).
    /// * `output_path` - The path to the output file.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns a new `OutputData` instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// let output_data = OutputData::new(OutputType::Csv, PathBuf::from("output.csv"));
    /// ```
    fn new(output_type: OutputType, output_path: PathBuf) -> Self {
        Self {
            output_type,
            output_path,
        }
    }
}
