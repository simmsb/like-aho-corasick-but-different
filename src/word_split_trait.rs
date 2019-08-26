use std::iter::{Filter, Map};

// use unicode_tables::perl_word::PERL_WORD;

struct UnicodeWordBoundaries<'a> {
    s: &'a str,
}

impl<'a> Iterator for UnicodeWordBoundaries<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        fn should_split(lhs: char, rhs: char) -> bool {
            if is_word_character(lhs) != is_word_character(rhs) {
                return true;
            }

            if lhs.is_whitespace() != rhs.is_whitespace() {
                return true;
            }

            false
        }

        if self.s.is_empty() {
            return None;
        }

        let mut c_it = self.s.char_indices().peekable();

        while let (Some((_, l)), Some((r_idx, r))) = (c_it.next(), c_it.peek().cloned()) {
            if should_split(l, r) {
                let (lhs, rhs) = self.s.split_at(r_idx);
                self.s = rhs;

                return Some(lhs);
            }
        }

        // if we got here, it means that the string had no word boundaries in it
        let empty_s = &self.s[0..0];
        let s = std::mem::replace(&mut self.s, empty_s);

        Some(s)
    }
}

pub struct UnicodeWordsAndSyms<'a> {
    inner: Filter<Map<UnicodeWordBoundaries<'a>, fn(&str) -> &str>, fn(&&str) -> bool>,
}

impl<'a> Iterator for UnicodeWordsAndSyms<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

struct UnicodeWordsAndSymsIndicesInner<'a> {
    current_offset: u32,
    inner: UnicodeWordBoundaries<'a>,
}

impl<'a> UnicodeWordsAndSymsIndicesInner<'a> {
    fn new(init: &'a str) -> Self {
        UnicodeWordsAndSymsIndicesInner {
            current_offset: 0,
            inner: UnicodeWordBoundaries { s: init },
        }
    }
}

impl<'a> Iterator for UnicodeWordsAndSymsIndicesInner<'a> {
    type Item = (u32, &'a str);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next()?;

        let offset = self.current_offset;
        self.current_offset += next.chars().count() as u32;

        Some((offset, next))
    }
}

pub struct UnicodeWordsAndSymsIndices<'a> {
    inner: Filter<
        Map<UnicodeWordsAndSymsIndicesInner<'a>, fn((u32, &str)) -> (u32, &str)>,
        fn(&(u32, &str)) -> bool,
    >,
}

impl<'a> Iterator for UnicodeWordsAndSymsIndices<'a> {
    type Item = (u32, &'a str);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

fn is_word_byte(c: u8) -> bool {
    match c {
        b'_' | b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' => true,
        _ => false,
    }
}

fn is_word_character(c: char) -> bool {
    use crate::unicode_tables::perl_word::PERL_WORD;
    use std::cmp::Ordering;

    if c <= 0x7F as char && is_word_byte(c as u8) {
        return true;
    }
    PERL_WORD
        .binary_search_by(|&(start, end)| {
            if start <= c && c <= end {
                Ordering::Equal
            } else if start > c {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        })
        .is_ok()
}

pub trait WordBoundarySplitter {
    fn unicode_words_and_syms(&self) -> UnicodeWordsAndSyms;

    fn unicode_words_and_syms_indices(&self) -> UnicodeWordsAndSymsIndices;
}

impl WordBoundarySplitter for str {
    fn unicode_words_and_syms(&self) -> UnicodeWordsAndSyms {
        fn is_not_empty(s: &&str) -> bool {
            !s.is_empty()
        }

        UnicodeWordsAndSyms {
            inner: UnicodeWordBoundaries { s: self }
                .map(str::trim as fn(&str) -> &str)
                .filter(is_not_empty),
        }
    }

    fn unicode_words_and_syms_indices(&self) -> UnicodeWordsAndSymsIndices {
        fn trim((idx, s): (u32, &str)) -> (u32, &str) {
            // keep idx correct

            let new_s = s.trim_start();

            let removed_chars = s.chars().count() - new_s.chars().count();
            let s = new_s.trim_end();

            (idx + removed_chars as u32, s)
        }

        fn is_not_empty((_, s): &(u32, &str)) -> bool {
            !s.is_empty()
        }

        UnicodeWordsAndSymsIndices {
            inner: UnicodeWordsAndSymsIndicesInner::new(self)
                .map(trim as fn ((u32, &str)) -> (u32, &str))
                .filter(is_not_empty as fn(&(u32, &str)) -> bool),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_boundary_splits() {
        assert_eq!(
            "aaa.bbb,ccc'ddd@eee"
                .unicode_words_and_syms()
                .collect::<Vec<_>>(),
            vec!["aaa", ".", "bbb", ",", "ccc", "\'", "ddd", "@", "eee"]
        );
    }
}
