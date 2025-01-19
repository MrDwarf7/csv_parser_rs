use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::SystemTime;

use regex::Regex;

use crate::config::{UserDefinedParts, UserDefinedRegex, compare_criteria, is_relative};
use crate::prelude::*;

/// Regex tests at bottom of the file - see `#[cfg(test)] mod regex_filename`
/// This Regex is designed to allow the user to pass through a variable input from the config file or CLI.
///
/// We're able to accept `\\data\\required_name.csv`, \\data\\required_name 123.csv`, or C:\\some\\path\\to\\data\\required_name 2025-01-15.csv
/// And still remain compatible with the rest of the codebase.
///
/// This feature applies to the output pathing as well.
///
/// EG:
/// Provided an output paht that looks like this:
/// `\\data\\required_name.*.csv`
/// then the regex will capture the `required_name` part of the string - ie: only the actual filename
///
/// This allows the user to setup the config file as:
/// ```json
/// "source": "\\data\\required_name.*.csv",
/// "output_type": "csv",
/// "output_path": "\\data\\required_name.*.csv",
/// ```
/// and be able to drop in any file that matches the pattern `required_name.*.csv`; such as `required_name 2025-01-15.csv`
/// and have the output file be named `required_name 2025-01-15.csv` as well.
///
pub static USER_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{(.+?)\}").ok().unwrap());

/// Sized used for the default sort_by_modification_time function
/// Handles the const generic for the sort_by_modification_time function
///
/// Reason being:
/// It's marginally faster to use an Array/slice over generic size, than allocating to the heap via Vec::new();
const _S: usize = 1;

pub fn parse_user_variable_path(path_str: &str) -> Result<PathBuf> {
    // If the user has not provided a user defined regex, then we can just fix the path and return it
    let user_defined_parts = extract_user_regex(path_str);

    // Serpating the calls here for refactoring purposes later.
    // Can just call match on the extracted user_regex fnc call above to simplify if wanted.
    let user_defined_parts = match user_defined_parts {
        Some(mut parts) => {
            parts.base_path = if is_relative(parts.base_path.to_str().unwrap()).is_ok() {
                is_relative(parts.base_path.to_str().unwrap())?
            } else {
                parts.base_path
            };
            parts
        }
        None => return is_relative(path_str),
    };

    let base_path_parent = user_defined_parts
        .base_path
        .parent()
        .ok_or_else(|| Error::NoParentPath(user_defined_parts.base_path.clone()))?;

    let before_reg_filename =
        &user_defined_parts.before_regex[user_defined_parts.before_regex.rfind('\\').unwrap_or_default() + 1..];

    let mut matching_files = Box::new(
        find_match_files_from_regex_path(base_path_parent, &user_defined_parts, before_reg_filename)
            .unwrap_or_default(),
    );

    // let stored = Box::new(matching_files.iter().map(|f| f.path()).collect::<Vec<_>>());
    let stored = &matching_files.iter().map(|f| f.path()).collect::<Vec<_>>();

    let sorted_matching_files = sort_by_modification_time::<_S>(matching_files.as_mut_slice())?;

    let first_match = sorted_matching_files.first().ok_or_else(|| {
        error!("We found these files: {:?}", *stored);
        Error::NoMatchingFiles
    })?;
    let second_match = sorted_matching_files.get(1); // Keep the Option to safe match on Some(_)

    // This is... Not a great way to do this, should probably be something like fold() or reduce()
    if let Some(second_match) = second_match {
        let d = compare_criteria(first_match, second_match, "date");
        let n = compare_criteria(first_match, second_match, "name");
        let s = compare_criteria(first_match, second_match, "size");
        if d == std::cmp::Ordering::Equal && n == std::cmp::Ordering::Equal && s == std::cmp::Ordering::Equal {
            return Err(Error::AmbiguousFileMatch);
        }
    }
    Ok(first_match.path())
}

fn extract_user_regex(base_path: &str) -> Option<UserDefinedParts<'_, PathBuf>> {
    let re = &USER_PATH_REGEX;

    if let Some(captures) = re.captures(base_path) {
        let var = &captures[1];
        let user_defined_regex = Regex::new(var).ok()?;

        // Everything BEFORE the user defined regex (ie : before { __ } )
        let start = &base_path[0..captures.get(0).unwrap().start()];

        // Everything AFTER the user defined regex (ie : after { __ } ) could be ext, or more filename
        let end = &base_path[captures.get(0).unwrap().end()..];

        return Some(UserDefinedParts {
            base_path: PathBuf::from(base_path),
            before_regex: start,
            user_regex: UserDefinedRegex {
                regex: user_defined_regex,
                _phantom: std::marker::PhantomData,
            },
            suffix_ext: Some(end),
        });
    }

    None
}

#[rustfmt::skip]
fn sort_by_modification_time<const S: usize>(files: &mut [DirEntry]) -> Result<&mut [DirEntry]>
where
    [DirEntry; S]: AsMut<[DirEntry]>,
    [DirEntry; S]: AsMut<[DirEntry]>,
{
    files.as_mut().sort_by(|a, b| {
        let a = a.metadata().and_then(|meta| meta.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
        let b = b.metadata().and_then(|meta| meta.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
        b.cmp(&a) // Rev -- Most recent first
    });
    Ok(files)
}

fn find_match_files_from_regex_path(
    base_directory: &Path,
    parts: &UserDefinedParts<'_, PathBuf>,
    before_reg_filename: &str,
) -> Result<Vec<DirEntry>> {
    let mut matches: Vec<DirEntry> = Vec::new();

    for entry in std::fs::read_dir(base_directory).map_err(Error::Io)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let filename = entry.file_name().into_string().unwrap_or_default();

        if metadata.is_dir() {
            continue;
        }

        if filename.starts_with(before_reg_filename)
            && filename.ends_with(parts.suffix_ext.unwrap_or_default())
            && parts
                .user_regex
                .regex
                .is_match(&filename[before_reg_filename.len()..filename.len() - parts.suffix_ext.unwrap().len()])
        {
            matches.push(entry);
        }
    }
    Ok(matches)
}

#[cfg(test)]
mod regex_filename {
    use std::fs::File;

    use tempfile::tempdir;

    use super::*;
    #[test]
    fn test_extract_user_regex() {
        let base_path = r"C:\data\file_{.*}.csv";

        let parts = extract_user_regex(base_path).expect("Failed to extract user regex");
        assert_eq!(parts.before_regex, r"C:\data\file_");
        assert_eq!(parts.suffix_ext, Some(".csv"));
        assert!(parts.user_regex.regex.is_match("look Ma regex in production"));
    }

    #[test]
    fn test_extract_user_regex_no_regex() {
        let base_path = r"C:\data\file.csv";

        let parts = extract_user_regex(base_path);
        assert!(parts.is_none());
    }

    #[test]
    fn test_find_match_files_from_regex_path() {
        let dir = tempdir().expect("Failed to create temp directory");

        // Create files that match and don't match the regex
        let file1_path = dir.path().join("file_123.csv");
        let file2_path = dir.path().join("file_456.csv");
        let file3_path = dir.path().join("not_a_match.txt");
        File::create(&file1_path).expect("Failed to create file1");
        File::create(&file2_path).expect("Failed to create file2");
        File::create(&file3_path).expect("Failed to create file3");

        let parts = UserDefinedParts {
            base_path: dir.path().to_path_buf(),
            before_regex: "file_",
            user_regex: UserDefinedRegex {
                regex: Regex::new(r"\d+").expect("Invalid regex"),
                _phantom: std::marker::PhantomData,
            },
            suffix_ext: Some(".csv"),
        };

        let matches =
            find_match_files_from_regex_path(dir.path(), &parts, "file_").expect("Failed to find matching files");

        let matched_filenames: Vec<_> = matches
            .iter()
            .map(|entry| entry.file_name().to_str().unwrap().to_string())
            .collect();

        assert_eq!(matched_filenames.len(), 2);
        assert!(matched_filenames.contains(&"file_123.csv".to_string()));
        assert!(matched_filenames.contains(&"file_456.csv".to_string()));
        assert!(!matched_filenames.contains(&"not_a_match.txt".to_string()));
    }

    #[ignore]
    #[test]
    fn test_parse_user_variable_path_with_regex() {
        let dir = tempdir().expect("Failed to create temp directory");
        let file1_path = dir.path().join("file_123.csv");
        let file2_path = dir.path().join("file_456.csv");
        File::create(&file1_path).expect("Failed to create file1");
        File::create(&file2_path).expect("Failed to create file2");

        let binding = dir.path().join("file_{.*}.csv");
        let path_str = binding.to_str().unwrap();
        let resolved_path = parse_user_variable_path(path_str).expect("Failed to parse user variable path");

        // Ensure the most recent file is chosen
        assert_eq!(resolved_path.file_name().unwrap(), "file_123.csv");
    }

    #[test]
    fn test_parse_user_variable_path_without_regex() {
        let dir = tempdir().expect("Failed to create temp directory");
        let file_path = dir.path().join("file.csv");
        File::create(&file_path).expect("Failed to create file");

        let path_str = file_path.to_str().unwrap();
        let resolved_path = parse_user_variable_path(path_str).expect("Failed to parse user variable path");

        assert_eq!(resolved_path, file_path);
    }
}
