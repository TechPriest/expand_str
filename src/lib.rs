use std::iter::Iterator;

#[derive(Debug)]
pub struct ExpandableStringSplit<'a> {
    src: &'a str,
    chars_iter: std::str::CharIndices<'a>,
    token_start: usize,
    reading_var: bool,
    done: bool,
}

pub fn split_expandable_string(s: &str) -> ExpandableStringSplit {
    ExpandableStringSplit {
        chars_iter: s.char_indices(),
        src: s,
        token_start: 0,
        reading_var: false,
        done: false,
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
        if self.done {
            return None;
        }

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

        self.done = true;

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
pub enum ExpandStringError<'a> {
    InvalidFormat,
    MissingVariable(&'a str),
}

impl<'a> std::convert::From<ExpandableStrSplitError> for ExpandStringError<'a> {
    fn from(src: ExpandableStrSplitError) -> Self {
        match src {
            ExpandableStrSplitError::InvalidFormat => Self::InvalidFormat,
        }
    }
}

pub fn expand_string_with_values<F, S>(s: &str, get_value: F) -> Result<String, ExpandStringError>
where
    F: Fn(&str) -> Option<S>,
    S: AsRef<str>,
{
    let mut expanded_str = String::with_capacity(s.len());

    for entry in split_expandable_string(s) {
        match entry? {
            ExpandableStrEntry::Substr(s) => {
                expanded_str += s;
            }
            ExpandableStrEntry::Var(id) => {
                let val = get_value(id).ok_or(ExpandStringError::MissingVariable(id))?;
                expanded_str += val.as_ref();
            }
        }
    }

    Ok(expanded_str)
}

#[cfg(feature = "env")]
pub fn expand_string_with_env(s: &str) -> Result<String, ExpandStringError> {
    fn get_var_value(key: &str) -> Option<String> {
        use std::ffi::{OsString, OsStr};

        std::env::var_os(key)
            .as_ref()
            .map(OsString::as_os_str)
            .map(OsStr::to_string_lossy)
            .map(Into::into)
    }

    expand_string_with_values(s, get_var_value)
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
    fn fails_to_parse_malformed_string() {
        let src = "%";
        let x: Vec<_> = split_expandable_string(src).collect();
        assert_eq!(x, vec![Err(ExpandableStrSplitError::InvalidFormat)]);
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
}
