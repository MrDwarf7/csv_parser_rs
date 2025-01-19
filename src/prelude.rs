pub use std::path::PathBuf;

#[allow(unused_imports)]
pub(crate) use log::{debug, error, info, trace, warn};
// pub use tracing::{debug, error, info, warn};
pub use serde::{Deserialize, Serialize};

// in-crate Error type
pub use crate::error::Error;

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
