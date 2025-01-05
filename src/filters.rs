//! Parsing and (de)serialization of email filters.

use filter::FiltersFile;
use uuid::Uuid;
use ::std::fmt::Write;

use self::filter::Filter;
use crate::prelude::*;

pub mod filter;
pub mod parsing;

#[cfg(test)]
pub mod tests;

#[macro_export]
/// Shortcut macro to use "{}" less
macro_rules! write_item {
    ($buf:expr, $item:expr) => {
        write!($buf, "{}", $item)?
    };
}

/// Build importable XML string of gmail filters
pub fn create_import(filters: FiltersFile<Filter>) -> Result<String> {
    let uuids = {
        /// Recursively retrieve UUIDs
        fn inner(uuids: &mut Vec<String>, filters: &[Filter]) -> Vec<String> {
            for filter in filters {
                inner(uuids, &filter.children);
                uuids.push(filter.filter.id.to_string());
            }
            uuids.to_vec()
        }
        inner(&mut Vec::new(), &filters)
    };

    let mut buf = String::new();

    write_item!(
        &mut buf,
        "<?xml version='1.0' encoding='UTF-8'?>".to_string()
    );
    write_item!(
        &mut buf,
        "<feed xmlns='http://www.w3.org/2005/Atom' xmlns:apps='http://schemas.google.com/apps/2006'>"
    );
    write_item!(
        &mut buf,
        "<title>Mail Filters</title>"
    );
    write_item!(
        &mut buf,
        format!("<id>tag:mail.google.com,2008:filters:{}</id>", uuids.join(","))
    );
    write_item!(
        &mut buf,
        format!("<updated>{}</updated>", chrono::Utc::now().to_rfc3339())
    );
    write_item!(
        &mut buf,
        format!("<author><name>{}</name><email>{}</email>", )
    );

    Ok(buf)
}
