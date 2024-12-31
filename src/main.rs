mod cli;
mod config;
mod error;
mod prelude;
mod processor;
mod retained;

pub(crate) use crate::prelude::*;
use crate::processor::Processor;
use crate::retained::RetainedData;

fn main() -> Result<()> {
    let cli = cli::Cli::new();
    let config = config::Config::new(&cli)?;
    let output = config.output.unwrap_or(config::Output::Stdout);

    let mut retained_data = RetainedData::default();
    let processor = Processor::new(config, &mut retained_data)?;

    if let Err(e) = processor.process(&mut retained_data) {
        eprintln!("Error: {}", e);
    }

    match output {
        config::Output::Stdout => {
            retained_data.to_stdout()?;
        }
        config::Output::Csv => {
            retained_data.to_csv(std::env::current_dir()?.join(PathBuf::from("output.csv")))?;
        }
    }

    Ok(())
}
