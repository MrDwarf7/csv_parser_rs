#![allow(clippy::needless_doctest_main)]

use std::marker::PhantomData;

use crate::cli::{Cli, OutputType};
use crate::config::Config;
use crate::csv_pipeline::CsvPipeline;
pub(crate) use crate::prelude::*;
use crate::processing::OutputData;
use crate::retained::RetainedData;

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
    pub csv_pipeline: CsvPipeline,
    // pub handler: CsvHandler,
    // pub processor: Processor,
    pub output_data: OutputData,
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
        let output_data = OutputData::new(config.output_type, config.output_path.clone());
        let mut retained_data = RetainedData::new(config.fields.len());

        let csv_pipeline = CsvPipeline::new(&config, &mut retained_data)?;

        Ok(Self {
            config,
            retained_data,
            csv_pipeline,
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
    pub fn process(&mut self) -> Result<()> {
        // Processor::process(&mut self.retained_data, &self.handler, rdr)
        self.csv_pipeline.process(&mut self.retained_data)
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
        self.csv_pipeline.deduplicate(&mut self.retained_data);
        // self.processor.deduplicate(&mut self.retained_data);
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
