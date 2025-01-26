use std::path::PathBuf;

use crate::cli::OutputType;

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
pub struct OutputData {
    pub output_type: OutputType,
    pub output_path: PathBuf,
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
    pub fn new(output_type: OutputType, output_path: PathBuf) -> Self {
        Self {
            output_type,
            output_path,
        }
    }
}
