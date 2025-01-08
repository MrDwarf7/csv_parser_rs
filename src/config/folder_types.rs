use std::ffi::OsStr;
use std::io::Bytes;
use std::path::{Path, PathBuf};

use crate::error::Error;

trait PathBufExt {
    fn pop_if_dir(&mut self);

    fn pop_if_extension(&mut self) -> Self;
}

impl PathBufExt for PathBuf {
    fn pop_if_dir(&mut self) {
        if self.ends_with("\\") || self.ends_with("/") {
            self.pop();
        }
    }

    fn pop_if_extension(&mut self) -> Self {
        if self.extension().is_some() {
            self.set_extension("");
        }
        self.clone()
    }
}


#[derive(Debug)]
pub enum SimpleFilename {
    SimpleFileName,        // output
    SimpleFileNameWithExt, // output.csv
}

impl From<&PathBuf> for SimpleFilename {
    fn from(value: &PathBuf) -> Self {
        let ext = value.extension().and_then(OsStr::to_str);
        let maybe_subdir = value.ends_with("\\") || value.ends_with("/") || value.ends_with(r#"\"#);

        match (ext, maybe_subdir) {
            (_, true) => {
                match ext {
                    Some(_) => SimpleFilename::SimpleFileNameWithExt,
                    None => SimpleFilename::SimpleFileName,
                }
            }
            (_, false) => {
                match ext {
                    Some(_) => SimpleFilename::SimpleFileNameWithExt,
                    None => SimpleFilename::SimpleFileName,
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum AbsolouteFilePath {
    AbsoloutePathNoFile,        // C:\some\path\
    AbsoloutePathWithFileNoExt, // C:\some\path\file
    AbsoloutePathWithFile,      // C:\some\path\file.csv
}

impl From<&PathBuf> for AbsolouteFilePath {
    fn from(value: &PathBuf) -> Self {
        let value_with_slash = value.ends_with("\\") || value.ends_with("/") || value.ends_with(r#"\"#);
        let ext = value.extension().and_then(OsStr::to_str);
        let is_file = value.extension().is_some();

        match (value_with_slash, ext, is_file) {
            (true, _, _) => AbsolouteFilePath::AbsoloutePathNoFile,
            (false, Some(_), true) => AbsolouteFilePath::AbsoloutePathWithFile,
            (false, Some(_), false) => AbsolouteFilePath::AbsoloutePathWithFileNoExt,
            _ => AbsolouteFilePath::AbsoloutePathNoFile,
        }
    }
}

#[derive(Debug)]
pub enum FolderPath {
    FolderNoFile,        // some\path\
    FolderWithFileNoExt, // some\path\file
    FolderWithFile,      // some\path\file.csv
}

impl From<&PathBuf> for FolderPath {
    fn from(value: &PathBuf) -> Self {
        let no_starting_root = value.starts_with("\\") || value.starts_with("/") || value.starts_with(r#"\"#);
        let value_with_slash = value.ends_with("\\") || value.ends_with("/") || value.ends_with(r#"\"#);
        let ext = value.extension().and_then(OsStr::to_str);
        let is_file = value.extension().is_some();

        match (no_starting_root, value_with_slash, ext, is_file) {
            (true, _, _, _) => FolderPath::FolderNoFile,
            (false, true, Some(_), true) => FolderPath::FolderWithFile,
            (false, true, Some(_), false) => FolderPath::FolderWithFileNoExt,
            _ => FolderPath::FolderNoFile,
        }
    }
}

#[derive(Debug)]
pub enum OutPathShape {
    SimpleFile(SimpleFilename),
    AbsolouteFile(AbsolouteFilePath),
    FolderFile(FolderPath),
}

impl From<&PathBuf> for OutPathShape {
    fn from(value: &PathBuf) -> Self {
        let simple = SimpleFilename::from(value);
        let abs = AbsolouteFilePath::from(value);
        let folder = FolderPath::from(value);

        let simple_parse = match simple {
            SimpleFilename::SimpleFileName => Some(OutPathShape::SimpleFile(SimpleFilename::SimpleFileName)),
            SimpleFilename::SimpleFileNameWithExt => {
                Some(OutPathShape::SimpleFile(SimpleFilename::SimpleFileNameWithExt))
            }
        };

        let abs_parse = match abs {
            AbsolouteFilePath::AbsoloutePathNoFile => {
                Some(OutPathShape::AbsolouteFile(AbsolouteFilePath::AbsoloutePathNoFile))
            }
            AbsolouteFilePath::AbsoloutePathWithFile => {
                Some(OutPathShape::AbsolouteFile(AbsolouteFilePath::AbsoloutePathWithFile))
            }
            AbsolouteFilePath::AbsoloutePathWithFileNoExt => {
                Some(OutPathShape::AbsolouteFile(AbsolouteFilePath::AbsoloutePathWithFileNoExt))
            }
        };

        let folder_parse = match folder {
            FolderPath::FolderNoFile => Some(OutPathShape::FolderFile(FolderPath::FolderNoFile)),
            FolderPath::FolderWithFile => Some(OutPathShape::FolderFile(FolderPath::FolderWithFile)),
            FolderPath::FolderWithFileNoExt => Some(OutPathShape::FolderFile(FolderPath::FolderWithFileNoExt)),
        };

        match (simple_parse, abs_parse, folder_parse) {
            (Some(s), _, _) => s,
            (_, Some(a), _) => a,
            (_, _, Some(f)) => f,
            _ => OutPathShape::SimpleFile(SimpleFilename::SimpleFileName),
        }
    }
}
