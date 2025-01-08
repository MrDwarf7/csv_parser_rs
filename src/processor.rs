use std::collections::{HashMap, HashSet};

use rayon::prelude::*;

use crate::config::Config;
use crate::prelude::*;
use crate::retained::RetainedData;

pub struct Processor {
    pub config: Config,
    handler: CsvHandler,
}

impl Processor {
    pub(crate) fn deduplicate(&self, retained_data: &mut RetainedData) {
        let mut unique_values = HashSet::new();
        for field in &self.config.unique_fields {
            let field_idx_in_existing = retained_data.retained_headers.iter().position(|x| x == field).unwrap();
            retained_data.data.retain(|row| {
                let val = row[field_idx_in_existing].clone();
                unique_values.insert(val)
            });
        }
    }
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
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(self.config.has_headers)
            .from_path(&self.config.source)?;

        let handler = &self.handler;

        for record_result in rdr.records() {
            let record = record_result?;

            if handler.row_passes_filters(&record) {
                let retained = handler.keep_columns(&record);
                retained_data.data.push(retained);
            }
        }

        Ok(())
    }
}

impl CsvHandler {
    #[allow(clippy::unnecessary_to_owned)] // for (idx, col_name) loop -- contains(&col_name.to_string()) loop
    fn new(config: &Config, retained_data: &mut RetainedData) -> Result<Self> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(config.has_headers)
            .from_path(&config.source)
            .expect("Failed to read CSV file from source provided");
        // .map_err(|e| Err(Error::CsvReader(format!("Failed to read CSV file: {}", e))))
        // .expect("Failed to read CSV file");

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

    fn row_passes_filters(&self, record: &csv::StringRecord) -> bool {
        self.filter_idxs.par_iter().all(|(col_idx, valid_values)| {
            let val = record.get(*col_idx).unwrap_or("");
            valid_values.contains(&val.to_string())
        })
    }

    fn keep_columns(&self, record: &csv::StringRecord) -> Vec<String> {
        let mut row_subset = Vec::with_capacity(self.field_idxs.len());
        for idx in &self.field_idxs {
            let val = record.get(*idx).unwrap_or("").to_string();
            row_subset.push(val);
        }
        row_subset
    }
}
