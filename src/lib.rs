use std::iter::Iterator;

#[derive(Debug)]
pub struct ExpandableStringSplit<'a> {
    src: &'a str,
    chars_iter: std::str::CharIndices<'a>,
    token_start: usize,
    reading_var: bool,
}

pub fn split_expandable_string(s: &str) -> ExpandableStringSplit {
    ExpandableStringSplit {
        chars_iter: s.char_indices(),
        src: s,
        token_start: 0,
        reading_var: false,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExpandableStrEntry<'a> {
    Substr(&'a str),
    Var(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExpandableStrSplitError {
    InvalidFormat,
}

pub type ExpandableStrSplitResult<'a> = Result<ExpandableStrEntry<'a>, ExpandableStrSplitError>;

impl<'a> Iterator for ExpandableStringSplit<'a> {
    type Item = ExpandableStrSplitResult<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((n, c)) = self.chars_iter.next() {
            if c == '%' {
                let reading_var = self.reading_var;
                self.reading_var = !reading_var;
                if n > 0 {
                    let token_slice = &self.src[self.token_start..n];
                    self.token_start = n + 1;
                    if !token_slice.is_empty() {
                        if reading_var {
                            return Some(Ok(ExpandableStrEntry::Var(token_slice)));
                        } else {
                            return Some(Ok(ExpandableStrEntry::Substr(token_slice)));
                        }
                    }
                } else {
                    self.token_start = 1;
                }
            }
        }

        if !self.reading_var {
            let token_slice = &self.src[self.token_start..];
            if !token_slice.is_empty() {
                self.token_start = self.src.len();
                Some(Ok(ExpandableStrEntry::Substr(token_slice)))
            } else {
                None
            }
        } else {
            Some(Err(ExpandableStrSplitError::InvalidFormat))
        }
    }
}

pub trait NamedValuesSource {
    fn get(&self, key: &str) -> Option<&str>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExpandStringError {
    InvalidFormat,
    MissingVariable,
}

impl std::convert::From<ExpandableStrSplitError> for ExpandStringError {
    fn from(src: ExpandableStrSplitError) -> Self {
        match src {
            ExpandableStrSplitError::InvalidFormat => Self::InvalidFormat,
        }
    }
}

pub fn expand_string_with_values<F>(s: &str, get_value: F) -> Result<String, ExpandStringError>
where
    F: Fn(&str) -> Option<String>,
{
    let mut expanded_str = String::with_capacity(s.len());

    for entry in split_expandable_string(s) {
        match entry? {
            ExpandableStrEntry::Substr(s) => {
                expanded_str += s;
            }
            ExpandableStrEntry::Var(id) => {
                expanded_str += &get_value(id).ok_or(ExpandStringError::MissingVariable)?;
            }
        }
    }

    Ok(expanded_str)
}

#[cfg(test)]
mod tests {
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
    fn expands_string_with_values() {
        let values = {
            let mut values = HashMap::new();
            values.insert("DRINK", "a cup of tea");
            values.insert("FOOD", "cookies");
            values
        };

        let src = "This is a string with a %DRINK% and some %FOOD%.";
        let x =
            expand_string_with_values(src, |id| values.get(id).copied().map(String::from)).unwrap();
        assert_eq!(x, "This is a string with a a cup of tea and some cookies.");
    }
}
