use std::iter::Iterator;

#[derive(Debug)]
pub struct ExpandableStringSplit<'a> {
    src: &'a str,
    chars_iter: std::str::CharIndices<'a>,
    token_start: usize,
    reading_var: bool,
}

impl<'a> ExpandableStringSplit<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            chars_iter: src.char_indices(),
            src,
            token_start: 0,
            reading_var: false,
        }
    }
}

#[derive(Debug)]
pub enum ExpandableStrEntry<'a> {
    Substr(&'a str),
    Var(&'a str),
}

#[derive(Debug)]
pub enum ExpandableStrSplitError {
    InvalidFormat
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
                    let token_slice = &self.src[self.token_start .. n];
                    self.token_start = n + 1;
                    if reading_var {
                        return Some(Ok(ExpandableStrEntry::Var(token_slice)));
                    } else {
                        return Some(Ok(ExpandableStrEntry::Substr(token_slice)));
                    }
                } else {
                    self.token_start = 1;
                }
            }
        }

        if !self.reading_var {
            let token_slice = &self.src[self.token_start .. ];
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_string() {
        let src = "%foo%bar";
        let split = ExpandableStringSplit::new(src);
        for x in split {
            let _ = dbg!(x);
        }
    }
}
