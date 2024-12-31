pub use std::path::PathBuf;

// pub use tracing::{debug, error, info, warn};
pub use serde::{Deserialize, Serialize};

// in-crate Error type
pub use crate::error::Error;

// in-crate result type
pub type Result<T> = std::result::Result<T, Error>;

// Wrapper struct
#[allow(dead_code)]
pub struct W<T>(pub T);

pub const DEFAULT_CONFIG_DIR: &str = "config";
pub const DEFAULT_CONFIG_FILE: &str = "config.json";
pub const DEFAULT_FILLER: &str = r#"
{
  "source": "some\\winodws\\path\\to\\file.csv",
  "fields": [
    "fields_to_retain_always",
    "fields_to_retain_always2",
    "fields_to_retain_always3",
    "fields_to_retain_always4"
  ],
  "filter_by": {
    "fields_that_need_filtering_for_values": [
      "value_of_field_to_filter_for",
      "value_of_field_to_filter_for2",
      "value_of_field_to_filter_for3"
    ],
    "fields_that_need_filtering_for_values_two": [
      "value_of_field_to_filter_for",
      "value_of_field_to_filter_for2",
      "value_of_field_to_filter_for3"
    ]
  }
}
"#;
