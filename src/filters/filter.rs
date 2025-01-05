//! An internal representation of email filters.
//!
//! Supports (de)serialization.

use crate::{prelude::*, write_item};
use ::std::{collections::HashMap, fmt::Write};
use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use serde::Deserialize;
use std::fmt::Display;

/// Immediate representation of a deserialized filter config. Not intended for
/// public use, just as an intermediate representation of the input file.
#[derive(Deserialize, Debug)]
pub struct FiltersFile<FType> {
    /// A mapping of variables to search for and replace in predicate values.
    #[serde(alias = "variables", default)]
    vars: Option<HashMap<String, String>>,

    author: Author,

    /// The raw filters read directly from the config file.
    filters: Vec<FType>,
}

impl FiltersFile<FilterRaw> {
    /// Retrieve filters from the file, substituting in variables and creating
    /// the dependency graph.
    fn get_filters(&mut self) -> Vec<Filter> {
        // Substitute variables
        if let Some(vars) = &self.vars {
            let keys: Vec<String> = vars.keys().map(|key| format!("{{{}}}", key)).collect();
            let values: Vec<&str> = vars.values().map(|v| v.as_str()).collect();

            let ac = AhoCorasickBuilder::new()
                .ascii_case_insensitive(false)
                .build(keys)
                .unwrap();

            Self::transform(&mut self.filters, &ac, values.as_slice());
        }

        Self::normalize_labels(&mut self.filters);

        // Break down into dependency graph
        self.filters.clone().into_iter().map(FilterRaw::cook).collect()
    }

    pub(super) fn parse(mut self) -> FiltersFile<Filter> {
        let filters = self.get_filters();
        FiltersFile {
            vars: self.vars,
            author: self.author,
            filters,
        }
    }

    /// Recursively substitutes in variables.
    fn transform(filters: &mut Vec<FilterRaw>, ac: &AhoCorasick, values: &[&str]) {
        // Closure to avoid code repetition. Just injects variables into a predicate.
        let inject = |predicates: &mut Vec<Predicate>| {
            for predicate in predicates {
                predicate.inject_variables(ac, values);
            }
        };

        for filter in filters {
            inject(&mut filter.mailing_lists);
            inject(&mut filter.tos);
            // Recursively perform substitutions on children
            Self::transform(&mut filter.children, ac, values);
        }
    }

    /// For each parent "P", take a given child "C" and rename the child to "P/C".
    /// Repeat for each parent, for each child.
    fn normalize_labels(filters: &mut [FilterRaw]) {
        for filter in filters.iter_mut() {
            for child in filter.children.iter_mut() {
                child.name = format!("{}/{}", filter.name, child.name);
            }
            Self::normalize_labels(&mut filter.children);
        }
    }
}

impl IntoIterator for FiltersFile<Filter> {
    type Item = Filter;
    type IntoIter = std::vec::IntoIter<Filter>;

    fn into_iter(self) -> Self::IntoIter {
        self.filters.into_iter()
    }
}

#[derive(Deserialize, Debug)]
pub struct Author {
    pub name: String,
    pub email: String,
}

/// Internal representation of a single filter along with its children. Only
/// used during the pre-processing step. Necessary for easier, direct
/// deserialization from config file.
#[derive(Deserialize, Debug, Clone)]
pub(super) struct FilterRaw {
    /// The name of a given filter. Equivalent to label name in email clients.
    name: String,

    /// Generated unique ID of a filter.
    #[serde(default = "uuid::Uuid::new_v4", skip)]
    id: uuid::Uuid,

    /// Mailing lists to filter on.
    #[serde(default)]
    mailing_lists: Vec<Predicate>,

    /// "To" addresses to filter on.
    #[serde(default)]
    tos: Vec<Predicate>,

    /// Any filters that depend on the parent filter being true.
    #[serde(default)]
    children: Vec<Self>,
}

impl FilterRaw {
    /// Get a "cooked" filter, baking a raw one down into a more
    /// memory-efficient representation with children separated out.
    pub(self) fn cook(self) -> Filter {
        let filter = FilterInner {
            name: self.name,
            mailing_lists: self.mailing_lists,
            tos: self.tos,
            id: self.id,
        };

        let children = self.children.into_iter().map(FilterRaw::cook).collect();

        Filter { filter, children }
    }
}

/// A "true" filter, only containing the filter rules themselves.
#[derive(Debug)]
pub struct FilterInner {
    /// The name of a given filter. Equivalent to label name in email clients.
    pub name: String,

    /// The unique ID of this filter.
    pub id: uuid::Uuid,

    /// Mailing lists to filter on.
    pub mailing_lists: Vec<Predicate>,

    /// "To" addresses to filter on.
    pub tos: Vec<Predicate>,
}

impl FilterInner {
    /// Convert an inner filter into XML elements for printing.
    pub fn into_xml_string(self) -> Result<String> {
        /// Helper for dynamically building XML string
        #[derive(Default)]
        #[allow(clippy::missing_docs_in_private_items)]
        struct FilterTagBuilder {
            tag: String,
            attrs: Vec<(String, String)>,
            content: Option<String>,
            short: bool,
        }

        #[allow(clippy::missing_docs_in_private_items)]
        impl FilterTagBuilder {
            fn new(tag: &str) -> Self {
                Self {
                    tag: tag.into(),
                    ..Default::default()
                }
            }

            fn attr(mut self, key: &str, value: &str) -> Self {
                self.attrs.push((key.into(), value.into()));
                self
            }

            fn content(mut self, content: &str) -> Self {
                self.content = Some(content.into());
                self
            }

            fn short(mut self) -> Self {
                self.short = true;
                self
            }

            fn build(self) -> Result<String> {
                let attrs = self
                    .attrs
                    .into_iter()
                    .map(|pair| format!(" {}=\"{}\"", pair.0, pair.1))
                    .collect::<Vec<String>>()
                    .join("");

                if self.content.is_some() && self.short {
                    return Err("Cannot use XML shorthand with text content".into());
                }

                match self.short {
                    false => Ok(format!(
                        "<{}{}>{}</{}>",
                        self.tag,
                        attrs,
                        self.content.unwrap_or_default(),
                        self.tag
                    )),
                    true => Ok(format!("<{}{}/>", self.tag, attrs,)),
                }
            }
        }

        let mut buf = String::new();

        write_item!(
            &mut buf,
            FilterTagBuilder::new("category")
                .attr("term", "filter")
                .build()?
        );
        write_item!(
            &mut buf,
            FilterTagBuilder::new("title")
                .content("Mail Filter")
                .build()?
        );
        write_item!(
            &mut buf,
            FilterTagBuilder::new("id")
                .content(&format!("tag:mail.google.com,2008:filter:{}", self.id))
                .build()?
        );
        write_item!(
            &mut buf,
            FilterTagBuilder::new("updated")
                .content(&chrono::Utc::now().to_rfc3339())
                .build()?
        );
        write_item!(&mut buf, FilterTagBuilder::new("content").build()?);

        let tos_str = Self::join_predicates(self.tos, None, None);

        if !tos_str.is_empty() {
            write_item!(
                &mut buf,
                FilterTagBuilder::new("apps:property")
                    .attr("name", "to")
                    .attr("value", &tos_str)
                    .short()
                    .build()?
            )
        }

        let mailing_list_str = Self::join_predicates(self.mailing_lists, Some("list:("), Some(")"));

        if !mailing_list_str.is_empty() {
            write_item!(
                &mut buf,
                FilterTagBuilder::new("apps:property")
                    .attr("name", "hasTheWord")
                    .attr("value", &mailing_list_str)
                    .short()
                    .build()?
            )
        }

        write_item!(
            &mut buf,
            FilterTagBuilder::new("apps:property")
                .attr("name", "sizeOperator")
                .attr("value", "s_sl")
                .short()
                .build()?
        );
        write_item!(
            &mut buf,
            FilterTagBuilder::new("apps:property")
                .attr("name", "sizeUnit")
                .attr("value", "s_smb")
                .short()
                .build()?
        );

        Ok(buf)
    }

    /// Helper function to combine predicates
    fn join_predicates(
        predicates: Vec<Predicate>,
        prefix: Option<&str>,
        suffix: Option<&str>,
    ) -> String {
        predicates
            .into_iter()
            .map(|rule| {
                format!(
                    "{}{}{}",
                    prefix.unwrap_or_default(),
                    rule,
                    suffix.unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    }
}

/// Public filter interface, containing a "true" filter and any of its children.
#[derive(Debug)]
pub struct Filter {
    /// Internal filter rules
    pub filter: FilterInner,

    /// Dependent filters
    pub children: Vec<Self>,
}

/// A boolean predicate representing a single filter rule.
///
/// For example, a filter in gmail for only selecting emails
/// "from:*@amazon.com" would make a predicate with the value "*@amazon.com",
/// and "-from:*@amazon.com" would be the same with negate set to true.
#[derive(Deserialize, Debug, Clone)]
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

impl Display for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let maybe_negate = match self.negate {
            true => "-",
            false => "",
        };

        write!(f, "{}{}", maybe_negate, self.rule)
    }
}
