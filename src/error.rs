#[derive(thiserror::Error, Debug)]
pub enum Error {
    // #[error("Generic error handler: {0}")]
    // Generic(String),
    #[error("Failed due to IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse config.json file: {0}")]
    ConfigParse(#[from] config::ConfigError),

    #[error("Failed to parse CSV file: {0}")]
    CsvParse(#[from] csv::Error),
    // #[error("Failed to parse the Config file and no CLI argumetns were passed")]
    // FailedConfigFileParseWithNoCliArgs,
    //
    // #[error("Output filepath already exists: {0}")]
    // OutputPathExists(PathBuf),
}
