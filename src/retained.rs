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
        println!("Writing to file: {:?}", output_path);
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
        let mut wtr = csv::Writer::from_writer(std::io::stdout());
        wtr.write_record(&self.retained_headers)?;
        for row in self.data.iter() {
            wtr.write_record(row)?;
        }
        wtr.flush()?;
        Ok(())
    }
}
