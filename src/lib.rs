#[derive(Debug)]
pub struct ExpandableStringSplit<'a> {
    src: &'a str,
    chars_iter: std::str::CharIndices<'a>,
    token_start: usize,
}

impl<'a> ExpandableStringSplit<'a> {
    pub fn new(src: &'a str) -> Self {
        Self {
            chars_iter: src.char_indices(),
            src,
            token_start: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
