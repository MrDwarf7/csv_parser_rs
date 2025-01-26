use std::fs::File;
use std::path::Path;

use crate::prelude::*;

/// Represents the retained data after processing the CSV file.
///
/// This struct holds all headers, retained headers, and the processed data.
///
/// # Fields
///
/// * `all_headers` - A vector of all headers from the CSV file.
/// * `retained_headers` - A vector of headers that are retained after processing.
/// * `data` - A vector of vectors containing the retained data.
#[derive(Debug, Default, Clone)]
pub struct RetainedData {
    pub all_headers: Vec<String>,
    pub retained_headers: Vec<String>,
    pub data: Vec<Vec<String>>,
}

impl RetainedData {
    /// Creates a new `RetainedData` instance with the specified capacity for retained headers.
    ///
    /// # Arguments
    ///
    /// * `fields_len` - The capacity for the retained headers vector.
    ///
    /// # Returns
    ///
    /// * `Self` - Returns a new `RetainedData` instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// let retained_data = RetainedData::new(10);
    /// ```
    pub fn new(fields_len: usize) -> Self {
        let all_headers = Vec::new();
        let retained_headers = Vec::with_capacity(fields_len);
        let data = Vec::new();
        Self {
            all_headers,
            retained_headers,
            data,
        }
    }

    /// Writes the retained data to the provided CSV writer.
    ///
    /// # Arguments
    ///
    /// * `wtr` - A mutable reference to a CSV writer.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns `Ok(())` on success, or an `Error` on failure.
    fn write<W>(&self, wtr: &mut csv::Writer<W>) -> Result<()>
    where
        W: std::io::Write,
    {
        wtr.write_record(&self.retained_headers)?;
        for row in &self.data {
            wtr.write_record(row)?;
        }
        wtr.flush()?;
        Ok(())
    }

    /// Writes the retained data to a CSV file at the specified output path.
    ///
    /// This function handles the case where the output directory does not exist
    /// and creates it if necessary.
    ///
    /// # Arguments
    ///
    /// * `output_path` - The path to the output CSV file.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns `Ok(())` on success, or an `Error` on failure.
    ///
    /// # Example
    ///
    /// ```rust
    /// retained_data.to_csv("output.csv").expect("Failed to write to CSV");
    /// ```
    #[allow(dead_code)]
    pub fn to_csv(&self, output_path: impl AsRef<Path>) -> Result<()> {
        let printable = output_path.as_ref().display();
        let output_path = output_path.as_ref();

        // Handle the case where user has provided a directory
        // but the directory doesn't exist yet
        if !output_path.exists() {
            std::fs::create_dir_all(output_path.parent().unwrap())?;
            let mut file = File::create(output_path)?;
            std::io::Write::write_all(&mut file, b"")?;
        }

        let mut wtr = csv::Writer::from_path(output_path)?;

        self.write(&mut wtr)?;
        wtr.flush()?;

        info!("Output written to: {printable}");

        Ok(())
    }

    /// Writes the retained data to the standard output.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Returns `Ok(())` on success, or an `Error` on failure.
    ///
    /// # Example
    ///
    /// ```rust
    /// retained_data.to_stdout().expect("Failed to write to stdout");
    /// ```
    pub fn to_stdout(&self) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(std::io::stderr());

        self.write(&mut wtr)?;
        Ok(())
    }
}

// #[cfg(debug_assertions)]
#[cfg(test)]
mod output_retained_tests {
    use tempdir::TempDir;

    use super::*;

    fn gen_default_retained_data() -> RetainedData {
        RetainedData {
            all_headers: vec!["Header1".to_string(), "Header2".to_string()],
            retained_headers: vec!["Header1".to_string(), "Header2".to_string()],
            data: vec![vec!["Value1".to_string(), "Value2".to_string()]],
        }
    }

    #[test]
    fn test_retained_data_to_csv() {
        let temp_dir = TempDir::new("test").unwrap();
        let output_path = temp_dir.path().join("output.csv");

        let data = gen_default_retained_data();

        data.to_csv(output_path.clone()).unwrap();

        let mut rdr = csv::Reader::from_path(output_path).unwrap();
        let mut records = rdr.records();
        let first_record = records.next().unwrap().unwrap();
        assert_eq!(&first_record[0], "Value1");
        assert_eq!(&first_record[1], "Value2");
    }

    #[test]
    fn test_retained_data_to_csv_nested_dir() {
        let temp_dir = TempDir::new("test").unwrap();
        let output_path = temp_dir.path().join("nested").join("output.csv");

        let data = gen_default_retained_data();

        data.to_csv(output_path.clone()).unwrap();

        let mut rdr = csv::Reader::from_path(output_path).unwrap();
        let mut records = rdr.records();
        let first_record = records.next().unwrap().unwrap();
        assert_eq!(&first_record[0], "Value1");
        assert_eq!(&first_record[1], "Value2");
    }

    #[test]
    fn test_retained_data_to_stdout() {
        let data = RetainedData {
            retained_headers: vec!["Header1".to_string(), "Header2".to_string()],
            data: vec![vec!["Value1".to_string(), "Value2".to_string()]],
            ..Default::default()
        };

        // let stdout = std::io::stdout();
        // let mut handle = stdout.lock();

        data.to_stdout().unwrap();

        // This is difficult to test directly,  but we know it doesn't crash.
        // More robust testing would require capturing stdout.
    }

    #[test]
    fn test_retained_data_empty() {
        let temp_dir = TempDir::new("test").unwrap();
        let output_path = temp_dir.path().join("empty.csv");

        let data = RetainedData::default(); // Empty data

        data.to_csv(output_path.clone()).unwrap();

        // Check that the file exists and is empty (or contains only header row if headers present)
        let file_size = std::fs::metadata(&output_path).unwrap().len();
        assert_eq!(file_size, 3);
    }
}
