use super::{ExpandableStrEntry::*, *};
use std::collections::HashMap;

#[test]
fn splits_string() {
    let src = "foo%bar%";
    let x: Vec<_> = split_expandable_string(src)
        .filter_map(Result::ok)
        .collect();
    assert_eq!(x, vec![Substr("foo"), Var("bar")]);
}

#[test]
fn splits_string_starting_with_var() {
    let src = "%foo%bar";
    let x: Vec<_> = split_expandable_string(src)
        .filter_map(Result::ok)
        .collect();
    assert_eq!(x, vec![Var("foo"), Substr("bar")]);
}

#[test]
fn splits_string_with_two_adjacent_vars() {
    let src = "%foo%%bar%";
    let x: Vec<_> = split_expandable_string(src)
        .filter_map(Result::ok)
        .collect();
    assert_eq!(x, vec![Var("foo"), Var("bar")]);
}

#[test]
fn fails_to_parse_malformed_string() {
    let src = "%";
    let x: Vec<_> = split_expandable_string(src).collect();
    assert_eq!(x, vec![Err(ExpandableStrSplitError::InvalidFormat)]);
}

#[test]
fn fails_to_parse_invalid_var_name() {
    let src = "Some %FOO BAR% here";
    let x: Vec<_> = split_expandable_string(src).collect();
    assert_eq!(x[1], Err(ExpandableStrSplitError::InvalidVariableName));

    let src = "Some %FOO=BAR% here";
    let x: Vec<_> = split_expandable_string(src).collect();
    assert_eq!(x[1], Err(ExpandableStrSplitError::InvalidVariableName));    
}

#[test]
fn expands_string_with_values() {
    let values = {
        let mut values = HashMap::new();
        values.insert("DRINK", "a cup of tea");
        values.insert("FOOD", "cookies");
        values
    };

    let src = "This is a string with a %DRINK% and some %FOOD%.";
    let x = expand_string_with_values(src, |id| values.get(id)).unwrap();
    assert_eq!(x, "This is a string with a a cup of tea and some cookies.");
}

#[test]
fn reports_missing_variable() {
    let src = "Some %FOO%";
    let x = expand_string_with_values(src, |_| -> Option<&str> { None });
    assert_eq!(x, Err(ExpandStringError::MissingVariable("FOO")));
}
