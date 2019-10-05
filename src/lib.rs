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

impl<'a> Iterator for ExpandableStringSplit<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let mut token_len = 0usize;
        while let Some((n, c)) = self.chars_iter.next() {
            if c == '%' {

                self.reading_var != self.reading_var;
            }

            token_len += 1;
        }

        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
