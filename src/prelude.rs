#[allow(unused_imports)]
pub(crate) use log::{debug, error, info, trace, warn};
use self_update::Status;
// pub use tracing::{debug, error, info, warn};
pub use serde::{Deserialize, Serialize};

// in-crate Error type
pub use crate::error::Error;
use crate::{crate_authors, crate_name};

// in-crate result type
pub type Result<T> = std::result::Result<T, Error>;

// Wrapper struct
#[allow(dead_code)]
pub struct W<T>(pub T);

pub const CLI_ENV_PREFIX: &str = "CSV_CLI";
pub const DEFAULT_CONFIG_DIR: &str = "config";
pub const DEFAULT_CONFIG_FILE: &str = "config.json";
pub const DEFAULT_FILLER: &str = r#"
{
  "source": "some\\winodws\\path\\to\\file.csv",
  "output_type": "stdout",
  "output_path": "some\\windows\\path\\to\\output.csv",
  "has_headers": true,
  "fields": [
    "__fields_to_retain_always",
    "__fields_to_retain_always2",
    "__fields_to_retain_always3",
    "__fields_to_retain_always4"
  ],
  "unique_fields": [
  ],
  "include_cols_with": {
    "__fields_that_need_filtering_for_values": [
      "__value_of_field_to_filter_for",
      "__value_of_field_to_filter_for2",
      "__value_of_field_to_filter_for3"
    ],
    "__fields_that_need_filtering_for_values_two": [
      "__value_of_field_to_filter_for",
      "__value_of_field_to_filter_for2",
      "__value_of_field_to_filter_for3"
    ]
  }
}
"#;

pub fn update() -> Result<String> {
    let author = first_author().to_lowercase();
    info!("Checking for updates...");

    let mut status_builder = self_update::backends::github::Update::configure();
    status_builder
        .repo_owner(&author)
        .repo_name(crate_name!())
        .bin_name(crate_name!())
        .current_version(self_update::cargo_crate_version!())
        .show_output(true)
        .show_download_progress(true);

    trace!("stauts_build: {:#?}", status_builder);

    let stauts_cls = move || -> Result<Status> { Ok(status_builder.build()?.update()?) };
    let status = std::thread::spawn(move || stauts_cls()).join().unwrap();
    println!(); // self_update crate maintainer decided to use print! instead of println! or something....

    match status {
        Ok(v) => {
            info!("Update successful. Restarting with new version");
            Ok(v.version().to_string())
        }
        Err(_) => {
            error!("Error updating.");
            warn!("Update not completed. Continuing with current version");
            Ok(self_update::cargo_crate_version!().to_string())
        }
    }

    // let status = match std::thread::spawn(stauts_cls).join().unwrap() {
    //     Ok(v) => {
    //         info!("Update successful. Restarting with new version");
    //         v
    //     }
    //     Err(_) => {
    //         error!("Error updating.");
    //         warn!("Update not completed. Continuing with current version");
    //         return Ok(self_update::cargo_crate_version!().to_string());
    //     }
    // };

    // Ok(status.version().to_string())
}

fn first_author() -> String {
    let authors = crate_authors!();
    let authors = authors.split(":").collect::<Vec<&str>>();
    let author = authors[0].split(" ").collect::<Vec<&str>>();
    author[0].to_string()
}

pub const NO_CONFIG_FILE_MSG: &str = r#"
    Config file either doesn't exist,
    is empty, or there was an error parsing it.
    Please check the config.json file.
    "#;

// "Config file either doesn't exist,\n
//              is empty,\n
//               or there was an error parsing it,\n
//                please check the config.json file",
//
//
//
//
