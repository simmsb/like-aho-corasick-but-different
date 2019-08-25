use std::iter::{Filter, Map};

// use unicode_tables::perl_word::PERL_WORD;

struct UnicodeWordBoundaries<'a> {
    s: &'a str,
}

impl<'a> Iterator for UnicodeWordBoundaries<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.s.is_empty() {
            return None;
        }

        let mut c_it = self.s.char_indices().peekable();

        while let (Some((_, l)), Some((r_idx, r))) = (c_it.next(), c_it.peek().cloned()) {
            if is_word_character(l) != is_word_character(r) {
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
    fn next(&mut self) -> Option<Self::Item> { self.inner.next() }
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
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_boundary_splits() {
        assert_eq!("aaa.bbb,ccc'ddd@eee".unicode_words_and_syms().collect::<Vec<_>>(),
                   vec!["aaa", ".", "bbb", ",", "ccc", "\'", "ddd", "@", "eee"]);
    }
}
