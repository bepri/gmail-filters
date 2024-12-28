//! Parsing for input filter config files.
use toml;

use super::filter::{Filter, FiltersFile};

/// Read in and parse `path` into filters ready for serialization.
pub fn get_config(config: String) -> Vec<Filter> {
    let filters_raw: FiltersFile = toml::from_str(&config).unwrap_or_else(|err| {
        eprintln!("Error parsing file: {err}");
        std::process::exit(1);
    });

    filters_raw.get_filters()
}
