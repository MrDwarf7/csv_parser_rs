use crate::prelude::{*};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use chrono::{NaiveDate, NaiveTime};
use crate::levenshtein::levenshtein_distance_matrix;

pub static REGEX_FILENAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(.*?)(\{.*?\}).(csv)").unwrap());

pub fn all_files_in_given(root: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(root).map_err(Error::Io)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            continue;
        }
        files.push(entry.path());
    }
    Ok(files)
}

pub fn parse_user_variable_path(config: &config::Config) -> Result<PathBuf> {
    let (_key, prov_path) = config.cache.clone().into_table()?.into_iter().next().ok_or("No source path found").unwrap();
    let prov_path_str = prov_path.to_string();
    let re = &REGEX_FILENAME;
    let captures = match re.captures(&prov_path_str) {
        Some(c) => c,
        None => return Err(Error::RegexCapture("No captures found".to_string())),
    };

    let prefix = &captures[1];
    let var = &captures[2];
    let suffix_ext = &captures[3];

    let current_dir = crate::config::current_dir()?;
    dbg!(&current_dir);

    let base_path = if Path::new(prefix).is_relative() {
        current_dir.join(prefix)
    } else {
        PathBuf::from(prefix)
    };

    let mut best_match = None;
    let mut min_dist = usize::MAX;
    // let mut last_mod = chrono::NaiveDateTime::new(NaiveDate::default(), NaiveTime::default());
    let mut last_mod = 0;

    for entry in std::fs::read_dir(&base_path).map_err(Error::Io).expect("No such file or directory for fs readdir") {
        let entry = entry?;
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            continue;
        }

        let filename = entry.file_name().into_string().map_err(|_| "Invalid filename").unwrap();
        let fullname = base_path.join(&filename);
        if fullname.file_name().unwrap().to_str().unwrap().contains(var) {
            let dist = levenshtein_distance_matrix(var, fullname.file_stem().unwrap().to_str().unwrap()) as usize;

            if dist < min_dist {
                min_dist = dist;
                best_match = Some(fullname);
                last_mod = metadata.modified()?.elapsed().unwrap().as_secs();
            } else if dist == min_dist && metadata.modified()?.elapsed().unwrap().as_secs() > last_mod {
                best_match = Some(fullname);
                last_mod = metadata.modified()?.elapsed().unwrap().as_secs();
            }
        }
    }

    match best_match {
        Some(path) => Ok(path),
        None => Err(Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "No file found"))),
    }
}



















































