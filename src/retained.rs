use std::fs::File;

use crate::prelude::*;

#[derive(Debug, Default, Clone)]
pub struct RetainedData {
    pub all_headers: Vec<String>,
    pub retained_headers: Vec<String>,
    pub data: Vec<Vec<String>>,
}

impl RetainedData {
    #[allow(dead_code)]
    pub fn to_csv(&self, output_path: PathBuf) -> Result<()> {
        // Handle the case where user has provided a directory
        // but the directory doesn't exist yet
        if !output_path.exists() {
            std::fs::create_dir_all(output_path.parent().unwrap())?;
            let mut file = File::create(&output_path)?;
            std::io::Write::write_all(&mut file, b"")?;
        }

        let mut wtr = csv::Writer::from_path(output_path)?;

        self.write(&mut wtr)?;
        wtr.flush()?;

        Ok(())
    }

    fn write<W>(&self, wtr: &mut csv::Writer<W>) -> Result<()>
    where
        W: std::io::Write,
    {
        wtr.write_record(&self.retained_headers)?;
        for row in self.data.iter() {
            wtr.write_record(row)?;
        }
        wtr.flush()?;
        Ok(())
    }

    pub fn to_stdout(&self) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(std::io::stderr());

        self.write(&mut wtr)?;
        // wtr.write_record(&self.retained_headers)?;
        // for row in self.data.iter() {
        //     wtr.write_record(row)?;
        // }
        // wtr.flush()?;
        Ok(())
    }
}

// #[cfg(debug_assertions)]
#[cfg(test)]
mod output_retained_tests {
    use tempdir::TempDir;

    use super::*;

    #[test]
    fn test_retained_data_to_csv() {
        let temp_dir = TempDir::new("test").unwrap();
        let output_path = temp_dir.path().join("output.csv");

        let data = RetainedData {
            retained_headers: vec!["Header1".to_string(), "Header2".to_string()],
            data: vec![vec!["Value1".to_string(), "Value2".to_string()]],
            ..Default::default()
        };

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

        let data = RetainedData {
            retained_headers: vec!["Header1".to_string(), "Header2".to_string()],
            data: vec![vec!["Value1".to_string(), "Value2".to_string()]],
            ..Default::default()
        };

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
