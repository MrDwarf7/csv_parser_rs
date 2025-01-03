mod core;

pub use core::Config;
use std::path::PathBuf;

// use crate::cli::OutputType;
use crate::error::Error;

#[cfg(debug_assertions)]
pub fn current_dir() -> crate::Result<PathBuf> {
    let dir = std::env::current_dir().map_err(Error::Io)?;

    Ok(dir.to_path_buf())
}

#[allow(inactive_code)]
#[cfg(not(debug_assertions))]
pub fn current_dir() -> crate::Result<PathBuf> {
    let dir = std::env::current_exe().map_err(Error::Io)?;
    let dir = dir
        .parent()
        .ok_or(Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "No parent directory")))?;

    Ok(dir.to_path_buf())
}
