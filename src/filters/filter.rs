use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub(super) struct FiltersFile {
    vars: HashMap<String, String>,
    filters: Vec<FilterRaw>,
}

impl FiltersFile {
    pub fn get_filters(mut self) -> Vec<Filter> {
        let keys: Vec<String> = self.vars.keys().map(|key| format!("{{{}}}", key)).collect();
        let values: Vec<&str> = self.vars.values().map(|v| v.as_str()).collect();

        let ac = AhoCorasickBuilder::new()
            .ascii_case_insensitive(false)
            .build(keys)
            .unwrap();

        Self::transform(&mut self.filters, &ac, values.as_slice());

        self.filters.into_iter().map(FilterRaw::cook).collect()
    }

    fn transform(filters: &mut Vec<FilterRaw>, ac: &AhoCorasick, values: &[&str]) {
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

            if let Some(ref mut children) = filter.children {
                Self::transform(children, ac, values)
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct FilterRaw {
    name: String,

    #[serde(default)]
    mailing_lists: Option<Vec<Predicate>>,

    #[serde(default)]
    tos: Option<Vec<Predicate>>,

    #[serde(default)]
    children: Option<Vec<Self>>,
}

impl FilterRaw {
    pub(self) fn cook(self) -> Filter {
        let filter = FilterInternal {
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

#[derive(Debug)]
struct FilterInternal {
    name: String,
    mailing_lists: Option<Vec<Predicate>>,
    tos: Option<Vec<Predicate>>,
}

#[derive(Debug)]
pub struct Filter {
    filter: FilterInternal,
    children: Option<Vec<Self>>,
}

#[derive(Deserialize, Debug)]
struct Predicate {
    #[serde(alias = "url", alias = "addr")]
    val: String,

    #[serde(default)]
    negate: bool,
}

impl Predicate {
    pub(self) fn inject_variables(&mut self, ac: &AhoCorasick, values: &[&str]) {
        self.val = ac.replace_all(&self.val, values)
    }
}
