//! An internal representation of email filters.
//!
//! Supports (de)serialization.

use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use serde::Deserialize;
use std::collections::HashMap;

/// Immediate representation of a deserialized filter config. Not intended for
/// public use, just as an intermediate representation of the input file.
#[derive(Deserialize, Debug)]
pub(super) struct FiltersFile {
    /// A mapping of variables to search for and replace in predicate values.
    #[serde(alias = "variables", default)]
    vars: Option<HashMap<String, String>>,

    /// The raw filters read directly from the config file.
    filters: Vec<FilterRaw>,
}

impl FiltersFile {
    /// Retrieve filters from the file, substituting in variables and creating
    /// the dependency graph.
    pub(super) fn get_filters(mut self) -> Vec<Filter> {
        // Substitute variables
        if let Some(vars) = self.vars {
            let keys: Vec<String> = vars.keys().map(|key| format!("{{{}}}", key)).collect();
            let values: Vec<&str> = vars.values().map(|v| v.as_str()).collect();

            let ac = AhoCorasickBuilder::new()
                .ascii_case_insensitive(false)
                .build(keys)
                .unwrap();

            Self::transform(&mut self.filters, &ac, values.as_slice());
        }

        // Break down into dependency graph
        self.filters.into_iter().map(FilterRaw::cook).collect()
    }

    /// Recursively substitutes in variables.
    fn transform(filters: &mut Vec<FilterRaw>, ac: &AhoCorasick, values: &[&str]) {
        // Closure to avoid code repetition. Checks if any predicates are
        // present, then injects the variables.
        let maybe_inject = |maybe_predicates: &mut Option<Vec<Predicate>>| {
            if let Some(ref mut predicates) = maybe_predicates {
                for predicate in predicates {
                    predicate.inject_variables(ac, values);
                }
            }
        };

        for filter in filters {
            maybe_inject(&mut filter.mailing_lists);
            maybe_inject(&mut filter.tos);

            // Recursively perform substitutions on children
            if let Some(ref mut children) = filter.children {
                Self::transform(children, ac, values)
            }
        }
    }
}

/// Internal representation of a single filter along with its children. Only
/// used during the pre-processing step. Necessary for easier, direct
/// deserialization from config file.
#[derive(Deserialize, Debug)]
struct FilterRaw {
    /// The name of a given filter. Equivalent to label name in email clients.
    name: String,

    /// Mailing lists to filter on.
    #[serde(default)]
    mailing_lists: Option<Vec<Predicate>>,

    /// "To" addresses to filter on.
    #[serde(default)]
    tos: Option<Vec<Predicate>>,

    /// Any filters that depend on the parent filter being true.
    #[serde(default)]
    children: Option<Vec<Self>>,
}

impl FilterRaw {
    /// Get a "cooked" filter, baking a raw one down into a more
    /// memory-efficient representation with children separated out.
    pub(self) fn cook(self) -> Filter {
        let filter = FilterInner {
            name: self.name,
            mailing_lists: self.mailing_lists,
            tos: self.tos,
        };

        let children = self
            .children
            .map(|children| children.into_iter().map(FilterRaw::cook).collect());

        Filter { filter, children }
    }
}

/// A "true" filter, only containing the filter rules themselves.
#[derive(Debug)]
pub struct FilterInner {
    /// The name of a given filter. Equivalent to label name in email clients.
    pub name: String,

    /// Mailing lists to filter on.
    pub mailing_lists: Option<Vec<Predicate>>,

    /// "To" addresses to filter on.
    pub tos: Option<Vec<Predicate>>,
}

/// Public filter interface, containing a "true" filter and any of its children.
#[derive(Debug)]
pub struct Filter {
    /// Internal filter rules
    pub filter: FilterInner,

    /// Dependent filters
    pub children: Option<Vec<Self>>,
}

/// A boolean predicate representing a single filter rule.
///
/// For example, a filter in gmail for only selecting emails
/// "from:*@amazon.com" would make a predicate with the value "*@amazon.com",
/// and "-from:*@amazon.com" would be the same with negate set to true.
#[derive(Deserialize, Debug)]
pub struct Predicate {
    /// Actual predicate to match on. Should have an alias to a more
    /// user-friendly name for deserialization.
    #[serde(alias = "url", alias = "addr")]
    pub rule: String,

    /// Whether or not to negate a rule. False if not specified.
    #[serde(default)]
    pub negate: bool,
}

impl Predicate {
    /// Find and replace templates with the provided variables. Do nothing
    /// with unrecognized variables.
    pub(self) fn inject_variables(&mut self, ac: &AhoCorasick, values: &[&str]) {
        self.rule = ac.replace_all(&self.rule, values)
    }
}
