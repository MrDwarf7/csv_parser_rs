use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::pin::Pin;

use csv::{Reader, StringRecord};
use rayon::prelude::*;

use crate::config::Config;
use crate::prelude::*;
use crate::retained::RetainedData;

/// Represents the processor responsible for handling CSV data processing.
///
/// This struct holds a pinned configuration and provides methods for processing
/// and deduplicating CSV data.
///
/// # Fields
///
/// * `config` - A pinned `Config` instance containing the configuration settings.
///
/// # Example
///
/// ```rust
/// let config = Config::new(cli).expect("Failed to create config");
/// let processor = Processor::new(&config);
/// ```
pub struct Processor {
    pub config: Pin<Box<Config>>,
}

impl Processor {
    /// Creates a new `Processor` instance by pinning the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - A reference to a `Config` instance containing the configuration settings.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns a new `Processor` instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// let config = Config::new(cli).expect("Failed to create config");
    /// let processor = Processor::new(&config);
    /// ```
    pub fn new(config: &Config) -> Self {
        Self {
            config: Box::pin(config.clone()),
        }
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
    pub fn process(retained_data: &mut RetainedData, handler: &CsvHandler, rdr: &mut Reader<File>) -> Result<()> {
        for record_result in rdr.records() {
            let record = record_result?;

            if handler.row_passes_filters(&record) {
                let retained = handler.keep_columns(&record);
                retained_data.data.push(retained);
            }
        }

        Ok(())
    }

    /// Deduplicates the retained data based on the unique fields specified in the configuration.
    ///
    /// This function removes duplicate entries from the `retained_data` by retaining only unique values
    /// for the specified fields.
    ///
    /// # Arguments
    ///
    /// * `retained_data` - A mutable reference to `RetainedData` to deduplicate the data.
    ///
    /// # Example
    ///
    /// ```rust
    /// processor.deduplicate(&mut retained_data);
    /// ```
    pub(crate) fn deduplicate(&mut self, retained_data: &mut RetainedData) {
        let mut unique_values = HashSet::new();
        for field in &self.config.as_ref().unique_fields {
            let field_idx_in_existing = retained_data
                .retained_headers
                .iter()
                .position(|x| x == field)
                .unwrap_or_else(|| {
                    panic!(
                        "{}",
                        format!("Csv file headers are missing fields or are unevenly distributed. Failed on: {field}")
                    );
                });

            retained_data.data.retain(|row| {
                let val = row[field_idx_in_existing].clone();
                unique_values.insert(val)
            });
        }
    }
}

/// Represents the handler for managing CSV processing.
///
/// This struct holds the indices of the fields to be retained and the indices of the fields
/// to be filtered based on the configuration settings.
///
/// # Fields
///
/// * `field_idxs` - A vector of indices representing the columns to be retained.
/// * `filter_idxs` - A hashmap where the key is the column index and the value is a vector of valid values for filtering.
///
/// # Example
///
/// ```rust
/// let handler = CsvHandler::new(&config, &mut retained_data, &headers);
/// ```
pub struct CsvHandler {
    field_idxs: Vec<usize>,
    filter_idxs: HashMap<usize, Vec<String>>,
}

impl CsvHandler {
    /// Creates a new `CsvHandler` instance for managing CSV processing.
    ///
    /// This function initializes the handler by setting up the field indices and filter indices
    /// based on the provided configuration and CSV headers.
    ///
    /// # Arguments
    ///
    /// * `config` - A reference to a `Config` instance containing the configuration settings.
    /// * `retained_data` - A mutable reference to `RetainedData` to store the processed data.
    /// * `headers` - A reference to a `StringRecord` instance containing the CSV headers.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns a new `CsvHandler` instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// let handler = CsvHandler::new(&config, &mut retained_data, &headers);
    /// ```
    #[allow(clippy::unnecessary_to_owned)] // for (idx, col_name) loop -- contains(&col_name.to_string()) loop
    pub(crate) fn new(config: &Config, retained_data: &mut RetainedData, headers: &StringRecord) -> Self {
        retained_data.all_headers = headers.iter().map(ToString::to_string).collect();

        let fields_set: HashSet<&String> = config.fields.iter().collect();

        let mut field_idxs = Vec::with_capacity(fields_set.len());
        let mut filter_idxs = HashMap::with_capacity(config.include_cols_with.len());

        for (idx, col_name) in headers.iter().enumerate() {
            if fields_set.contains(&col_name.to_string()) {
                field_idxs.push(idx);
            }

            if let Some(valid_values) = config.include_cols_with.get(col_name) {
                filter_idxs.insert(idx, valid_values.clone());
            }
        }

        retained_data.retained_headers = field_idxs.iter().map(|&idx| headers[idx].to_string()).collect();

        Self {
            field_idxs,
            filter_idxs,
        }
    }

    /// Checks if a CSV record passes the configured filters.
    ///
    /// This function iterates over the filter indices and checks if the values in the record
    /// match the valid values specified in the configuration.
    ///
    /// # Arguments
    ///
    /// * `record` - A reference to a `StringRecord` instance containing the CSV record.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns `true` if the record passes the filters, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// let passes = handler.row_passes_filters(&record);
    /// ```
    fn row_passes_filters(&self, record: &StringRecord) -> bool {
        self.filter_idxs.par_iter().all(|(col_idx, valid_values)| {
            let val = record.get(*col_idx).unwrap_or("");
            valid_values.contains(&val.to_string())
        })
    }

    /// Retains the specified columns from a CSV record.
    ///
    /// This function creates a subset of the record containing only the columns specified
    /// in the field indices.
    ///
    /// # Arguments
    ///
    /// * `record` - A reference to a `StringRecord` instance containing the CSV record.
    ///
    /// # Returns
    ///
    /// * `Vec<String>` - Returns a vector containing the retained columns.
    ///
    /// # Example
    ///
    /// ```rust
    /// let columns = handler.keep_columns(&record);
    /// ```
    fn keep_columns(&self, record: &StringRecord) -> Vec<String> {
        let mut row_subset = Vec::with_capacity(self.field_idxs.len());
        for idx in &self.field_idxs {
            let val = record.get(*idx).unwrap_or("").to_string();
            row_subset.push(val);
        }
        row_subset
    }
}
