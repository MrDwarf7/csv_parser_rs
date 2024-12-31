use std::collections::{HashMap, HashSet};

use crate::config::Config;
use crate::prelude::*;
use crate::retained::RetainedData;

pub struct Processor {
    config: Config,
    handler: CsvHandler,
}

pub struct CsvHandler {
    field_idxs: Vec<usize>,
    filter_idxs: HashMap<usize, Vec<String>>,
}

impl Processor {
    pub fn new(config: Config, retained_data: &mut RetainedData) -> Result<Self> {
        let handler = CsvHandler::new(&config, retained_data)?;
        Ok(Processor { config, handler })
    }

    pub fn process(&self, retained_data: &mut RetainedData) -> Result<()> {
        println!("Processing...");

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(self.config.source.as_ref().unwrap())?;

        for record_result in rdr.records() {
            let record = record_result?;

            if self.row_passes_filters(&record) {
                let retained = self.keep_columns(&record);
                retained_data.data.push(retained);
            }
        }

        Ok(())
    }

    fn row_passes_filters(&self, record: &csv::StringRecord) -> bool {
        for (col_idx, valid_values) in &self.handler.filter_idxs {
            let val = record.get(*col_idx).unwrap_or("");
            if !valid_values.contains(&val.to_string()) {
                return false;
            }
        }
        true
    }

    fn keep_columns(&self, record: &csv::StringRecord) -> Vec<String> {
        let mut row_subset = Vec::with_capacity(self.handler.field_idxs.len());
        for idx in &self.handler.field_idxs {
            let val = record.get(*idx).unwrap_or("").to_string();
            row_subset.push(val);
        }
        row_subset
    }
}

impl CsvHandler {
    fn new(config: &Config, retained_data: &mut RetainedData) -> Result<Self> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(config.source.as_ref().unwrap())
            .map_err(Error::CsvParse)?;

        let headers = rdr.headers()?;
        retained_data.all_headers = headers.iter().map(|s| s.to_string()).collect();

        let fields_set: HashSet<&String> = config.fields.iter().collect();
        let mut field_idxs = Vec::new();

        let mut filter_idxs = HashMap::new();

        for (idx, col_name) in headers.iter().enumerate() {
            if fields_set.contains(&col_name.to_string()) {
                field_idxs.push(idx);
            }

            if let Some(valid_values) = config.filter_by.get(col_name) {
                filter_idxs.insert(idx, valid_values.clone());
            }
        }

        retained_data.retained_headers = field_idxs.iter().map(|&idx| headers[idx].to_string()).collect();

        Ok(Self {
            field_idxs,
            filter_idxs,
        })
    }
}
