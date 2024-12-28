use std::sync::LazyLock;

use filter::Filter;

use super::*;

static EXPECT_TEST_CASE: &str = "Hard-coded by test-case";

static TEST_CONFIG: LazyLock<Vec<Filter>> = LazyLock::new(|| {
    static TEST_CONFIG_RAW: &str = r#"
    [vars]
    a = "aaa"
    b = "bbb"

    [[filters]]
    name = "F1"
    
        [[filters.mailing_lists]]
        url = "{a}"
    
        [[filters.mailing_lists]]
        url = "{b}"

    [[filters]]
    name = "F2"

        [[filters.children]]
        name = "F2C1"

        [[filters.children]]
        name = "F2C2"
    "#;

    parsing::get_config(TEST_CONFIG_RAW.into())
});

#[test]
fn test_inject_variables() {
    let filter = &TEST_CONFIG.first().expect(EXPECT_TEST_CASE).filter;
    let mailing_lists = filter.mailing_lists.as_ref().expect(EXPECT_TEST_CASE);

    assert_eq!(mailing_lists.first().expect(EXPECT_TEST_CASE).rule, "aaa");
    assert_eq!(mailing_lists.get(1).expect(EXPECT_TEST_CASE).rule, "bbb");
}

#[test]
fn test_cook_filters() {
    let Some(children) = &TEST_CONFIG.get(1).expect(EXPECT_TEST_CASE).children else {
        panic!("No children were found.");
    };

    assert_eq!(children.len(), 2);
}
