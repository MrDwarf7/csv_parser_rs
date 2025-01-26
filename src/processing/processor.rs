use std::collections::HashSet;
use std::pin::Pin;

use crate::config::Config;
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
pub struct CsvProcessor {
    pub config: Pin<Box<Config>>,
}

impl CsvProcessor {
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
        // use rayon::prelude::*;
        let mut seen = HashSet::new();

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
                seen.insert(val.clone())
            });
        }
    }
}
