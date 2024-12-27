//! Parsing for input filter config files.
use std::{fs, path::PathBuf};
use toml;

use super::filter::{Filter, FiltersFile};

/// Read in and parse `path` into filters ready for serialization.
pub fn get_config(path: PathBuf) -> Vec<Filter> {
    let filters_raw: FiltersFile = {
        let filters_file_content = fs::read_to_string(path).unwrap_or_else(|err| {
            eprintln!("Error: {err}");
            std::process::exit(1);
        });

        toml::from_str(&filters_file_content).unwrap_or_else(|err| {
            eprintln!("Error parsing file: {err}");
            std::process::exit(1);
        })
    };

    filters_raw.get_filters()
}
