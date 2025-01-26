#![allow(clippy::needless_doctest_main)]

use log::{error, info, warn};
use state::State;

pub(crate) mod cli;
pub(crate) mod config;
pub(crate) mod csv_pipeline;
pub(crate) mod error;
pub(crate) mod prelude;
pub(crate) mod processing;
pub(crate) mod retained;
pub(crate) mod state;

use crate::cli::{Cli, VerbosityLevel};
pub(crate) use crate::prelude::*;

/// The main entry point of the application.
///
/// This function performs the following steps:
/// 1. Initializes the `Cli` instance to parse command-line arguments.
/// 2. Creates a new `State` instance based on the `Cli` input.
/// 3. Processes the CSV data using the `State` instance.
/// 4. Deduplicates the retained data if unique fields are specified in the configuration.
/// 5. Outputs the retained data based on the configured output type.
///
/// # Returns
///
/// * `Result<()>` - Returns `Ok(())` on success, or an `Error` on failure.
///
/// # Errors
///
/// This function can return errors in the following cases:
/// * If the `State` instance cannot be created.
/// * If processing the CSV data fails.
/// * If outputting the retained data fails.
///
/// # Example
///
/// ```rust
/// fn main() {
///     if let Err(e) = main() {
///         eprintln!("Application error: {e}");
///     }
/// }
/// ```
pub fn main() -> Result<()> {
    let cli = Cli::new();
    let _ = stderrlog::new()
        .color(stderrlog::ColorChoice::Always)
        .verbosity(cli.verbosity_level.unwrap_or(VerbosityLevel::Info))
        .show_level(true)
        .show_module_names(true)
        .init();

    let mut state = State::new(cli)?;
    info!("MAIN:: Config: {:#?}", &state.config);

    if let Err(proc_err) = state.process() {
        error!("Error processing: {proc_err}");
    }

    info!("Config before finishing: {:#?}", &state.config);

    if !state.config.unique_fields.is_empty() || state.config.unique_fields.len().gt(&1) {
        state.deduplicate();
    } else {
        warn!("No unique fields provided, skipping deduplication");
    }

    match state.output() {
        Ok(()) => {
            info!("Output successful");
            Ok(())
        }
        Err(e) => {
            error!("Error outputting: {e}");
            Err(e)
        }
    }
}
