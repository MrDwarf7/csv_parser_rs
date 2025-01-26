use std::fs::File;

use csv::Reader;

use crate::config::Config;
pub(crate) use crate::prelude::*;
use crate::processing::{CsvHandler, CsvProcessor};
use crate::retained::RetainedData;

pub struct CsvPipeline {
    reader: Reader<File>,
    handler: CsvHandler,
    processor: CsvProcessor,
}

impl CsvPipeline {
    pub fn new(config: &Config, retained_data: &mut RetainedData) -> Result<Self> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(config.has_headers)
            .from_path(&config.source)
            .map_err(|e| Error::CsvRead(format!("Failed to read CSV file from source provided: {e}")))?;

        #[rustfmt::skip]
        let handler = CsvHandler::new(
            config,
            retained_data,
            reader.headers().map_err(|e| {
                Error::CsvHeaders(e.to_string())
            })?,
        );

        let processor = CsvProcessor::new(config);

        Ok(Self {
            reader,
            handler,
            processor,
        })
    }

    /// Processes the CSV data and updates the retained data.
    ///
    /// This function iterates over the records in the CSV reader, applies filters using the `CsvHandler`,
    /// and retains the specified columns in the `retained_data`.
    ///
    /// # Arguments
    ///
    /// * `retained_data` - A mutable reference to `RetainedData` to store the processed data.
    /// * `handler` - A reference to a `CsvHandler` instance for handling CSV processing.
    /// * `rdr` - A mutable reference to a `csv::Reader` instance for reading the CSV data.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns `Ok(())` on success, or an `Error` on failure.
    ///
    /// # Errors
    ///
    /// This function can return errors if reading the CSV records fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut rdr = csv::Reader::from_path("data.csv").expect("Failed to open CSV file");
    /// processor.process(&mut retained_data, &handler, &mut rdr).expect("Failed to process CSV data");
    /// ```
    pub fn process(&mut self, retained_data: &mut RetainedData) -> Result<()> {
        for record_result in self.reader.records() {
            let record = record_result?;

            if self.handler.row_passes_filters(&record) {
                let retained = self.handler.keep_columns(&record);
                retained_data.data.push(retained);
            }
        }

        Ok(())
    }

    pub fn deduplicate(&mut self, retained_data: &mut RetainedData) {
        self.processor.deduplicate(retained_data);
    }
}
