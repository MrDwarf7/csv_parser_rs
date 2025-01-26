use std::collections::{HashMap, HashSet};

use csv::StringRecord;
use rayon::prelude::*;

use crate::config::Config;
use crate::retained::RetainedData;

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
    pub(crate) fn row_passes_filters(&self, record: &StringRecord) -> bool {
        self.filter_idxs.par_iter().all(|(col_idx, valid_values)| {
            // let val =
            record
                .get(*col_idx)
                .is_some_and(|val| valid_values.contains(&val.to_string()))
            // unwrap_or("");
            // valid_values.contains(&val.to_string())
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
    pub(crate) fn keep_columns(&self, record: &StringRecord) -> Vec<String> {
        let mut row_subset = Vec::with_capacity(self.field_idxs.len());
        for idx in &self.field_idxs {
            let val = record.get(*idx).unwrap_or("").to_string();
            row_subset.push(val);
        }
        row_subset
    }
}
