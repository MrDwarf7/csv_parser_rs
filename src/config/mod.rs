mod core;
mod file_path_finds;

pub use core::Config;
use std::borrow::Cow;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use config::Value;
use regex::Regex;

use crate::error::Error;

#[derive(Debug, Clone)]
struct UserDefinedRegex<'b> {
    regex: Regex,
    _phantom: std::marker::PhantomData<&'b ()>,
}

#[derive(Debug, Clone)]
struct UserDefinedParts<'a, P>
where
    P: Into<PathBuf>,
    P: AsRef<Path>,
{
    base_path: P,
    before_regex: &'a str,
    user_regex: UserDefinedRegex<'a>,
    suffix_ext: Option<&'a str>,
}

#[allow(clippy::unnecessary_wraps)] // TODO: will need to change it over at some point
fn is_relative(prov_path_str: impl AsRef<str>) -> crate::prelude::Result<PathBuf> {
    if PathBuf::from(&prov_path_str.as_ref()).is_absolute() {
        Ok(PathBuf::from(prov_path_str.as_ref()))
    } else {
        let prov_path_str = resolve_if_relative(Path::new(prov_path_str.as_ref()));
        Ok(PathBuf::from(prov_path_str.as_ref()))
    }
}

pub(crate) fn resolve_if_relative(path: &'_ Path) -> Cow<'_, Path> {
    let mut current_dir_str = current_dir().unwrap().to_str().unwrap().to_string();
    let path_str = path.to_str().unwrap().to_string();

    if path.eq(Path::new(".")) {
        return Cow::Owned(PathBuf::from(current_dir_str));
    }

    if path.as_os_str().eq(current_dir_str.as_str()) {
        current_dir_str.clone().push_str(&path_str);
        return Cow::Owned(PathBuf::from(current_dir_str));
    }

    // For if user provides \\data\\required_name.csv vs data\\required_name.csv in the config file at runtime
    if !path_str.starts_with('\\') {
        current_dir_str.push('\\');
    }
    current_dir_str.push_str(&path_str);

    Cow::Owned(PathBuf::from(current_dir_str))
}

pub fn extract_cached_config_value(config: &config::Config, find_key_for: &str) -> crate::prelude::Result<String> {
    let (_key, prov_path) = config
        .cache
        .clone()
        .into_table()?
        .into_iter()
        .find(|(key, val)| *key == find_key_for && *val != Value::default())
        .ok_or(Error::ConfigSource)?;

    Ok(prov_path.to_string())
}

// Would be easier to write this as a Fn() -> () closure, but---
fn compare_criteria(first: &DirEntry, second: &DirEntry, criteria: &str) -> std::cmp::Ordering {
    match criteria {
        "date" => compare_files_by_date(first, second),
        "name" => compare_files_by_name(first, second),
        "size" => compare_files_by_sizelen(first, second),
        _ => std::cmp::Ordering::Equal,
    }
}

fn compare_files_by_date(first: &DirEntry, second: &DirEntry) -> std::cmp::Ordering {
    let first_meta = first.metadata().unwrap();
    let second_meta = second.metadata().unwrap();

    let first_date = first_meta.modified().unwrap();
    let second_date = second_meta.modified().unwrap();

    first_date.cmp(&second_date)
}

fn compare_files_by_name(first: &DirEntry, second: &DirEntry) -> std::cmp::Ordering {
    let first_name = first.file_name();
    let second_name = second.file_name();

    first_name.cmp(&second_name)
}

fn compare_files_by_sizelen(first: &DirEntry, second: &DirEntry) -> std::cmp::Ordering {
    let first_meta = first.metadata().unwrap();
    let second_meta = second.metadata().unwrap();

    let first_size = first_meta.len();
    let second_size = second_meta.len();

    first_size.cmp(&second_size)
}

/// Returns the current directory based on the build configuration.
///
/// In debug mode, this function returns the current working directory.
/// In release mode, it returns the directory of the executable.
///
/// # Returns
///
/// * `crate::Result<PathBuf>` - Returns the path to the current directory on success, or an `Error` on failure.
///
/// # Example
///
/// ```rust
/// let dir = current_dir().expect("Failed to get current directory");
/// println!("Current directory: {:?}", dir);
/// ```
#[cfg(debug_assertions)]
pub fn current_dir() -> crate::Result<PathBuf> {
    let dir = std::env::current_dir().map_err(Error::Io)?;
    Ok(dir.clone())
}

/// Returns the current directory based on the build configuration.
///
/// In debug mode, this function returns the current working directory.
/// In release mode, it returns the directory of the executable.
///
/// # Returns
///
/// * `crate::Result<PathBuf>` - Returns the path to the current directory on success, or an `Error` on failure.
///
/// # Example
///
/// ```rust
/// let dir = current_dir().expect("Failed to get current directory");
/// println!("Current directory: {:?}", dir);
/// ```
#[cfg(not(debug_assertions))]
pub fn current_dir() -> crate::Result<PathBuf> {
    let dir = std::env::current_exe().map_err(Error::Io)?;
    let dir = dir
        .parent()
        .ok_or(Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "No parent directory")))?;

    Ok(dir.to_path_buf())
}
