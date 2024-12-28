//! Parsing for input filter config files.
use toml;

use super::filter::{Filter, FiltersFile};
use crate::prelude::*;

/// Read in and parse `path` into filters ready for serialization.
pub fn get_config(config: String) -> Result<Vec<Filter>> {
    let filters_raw: FiltersFile = match toml::from_str(&config) {
        Ok(fr) => fr,
        Err(err) => return Err(format!("Error parsing TOML: {}", err.message()).into()),
    };

    Ok(filters_raw.get_filters())
}
