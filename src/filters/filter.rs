use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub(super) struct FiltersFile {
    vars: HashMap<String, String>,
    filters: Vec<Filter>,
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
        self.filters
    }

    fn transform(filters: &mut Vec<Filter>, ac: &AhoCorasick, values: &[&str]) {
        for filter in filters {
            for mailing_list in &mut filter.mailing_lists {
                mailing_list.inject_variables(ac, values);
            }

            for to in &mut filter.tos {
                to.inject_variables(ac, values);
            }

            Self::transform(&mut filter.children, ac, values);
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Filter {
    name: String,

    #[serde(default)]
    mailing_lists: Vec<Predicate>,

    #[serde(default)]
    tos: Vec<Predicate>,

    #[serde(default)]
    children: Vec<Filter>,
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
