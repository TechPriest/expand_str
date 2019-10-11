use std::convert::From;
use std::fmt::{Display, Error as FmtError, Write};
use std::iter::Iterator;

#[cfg(test)]
mod tests;

#[derive(Debug)]
struct ExpandableStringSplit<'a> {
    src: &'a str,
    chars_iter: std::str::CharIndices<'a>,
    token_start: usize,
    reading_var: bool,
    done: bool,
}

fn split_expandable_string(s: &str) -> ExpandableStringSplit {
    ExpandableStringSplit {
        chars_iter: s.char_indices(),
        src: s,
        token_start: 0,
        reading_var: false,
        done: false,
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ExpandableStrEntry<'a> {
    Substr(&'a str),
    Var(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExpandableStrSplitError {
    /// Invalid input string (basically, non-closed variable name)
    InvalidFormat,

    /// Bad variable name; names should not contain space or equality sign
    InvalidVariableName,
}

type ExpandableStrSplitResult<'a> = Result<ExpandableStrEntry<'a>, ExpandableStrSplitError>;

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
            } else if self.reading_var {
                match c {
                    '=' | ' ' => {
                        self.done = true;
                        return Some(Err(ExpandableStrSplitError::InvalidVariableName))
                    }
                    _ => (),
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

#[derive(Debug, PartialEq, Eq)]
pub enum ExpandStringError<'a> {
    Splitting(ExpandableStrSplitError),
    /// Variable specified in source string is missing in provided context (or process environment)
    MissingVariable(&'a str),
    Formatting(FmtError),
}

impl<'a> From<ExpandableStrSplitError> for ExpandStringError<'a> {
    fn from(e: ExpandableStrSplitError) -> Self {
        Self::Splitting(e)
    }
}

impl<'a> From<FmtError> for ExpandStringError<'a> {
    fn from(e: FmtError) -> Self {
        Self::Formatting(e)
    }
}

pub fn expand_string_with_values<F, S>(s: &str, get_value: F) -> Result<String, ExpandStringError>
where
    F: Fn(&str) -> Option<S>,
    S: Display,
{
    let mut expanded_str = String::with_capacity(s.len());

    for entry in split_expandable_string(s) {
        match entry? {
            ExpandableStrEntry::Substr(s) => {
                expanded_str += s;
            }
            ExpandableStrEntry::Var(id) => {
                let val = get_value(id).ok_or(ExpandStringError::MissingVariable(id))?;
                write!(&mut expanded_str, "{}", val)?;
            }
        }
    }

    Ok(expanded_str)
}

#[cfg(feature = "env")]
pub fn expand_string_with_env(s: &str) -> Result<String, ExpandStringError> {
    fn get_var_value(key: &str) -> Option<String> {
        use std::ffi::{OsStr, OsString};

        std::env::var_os(key)
            .as_ref()
            .map(OsString::as_os_str)
            .map(OsStr::to_string_lossy)
            .map(Into::into)
    }

    expand_string_with_values(s, get_var_value)
}
