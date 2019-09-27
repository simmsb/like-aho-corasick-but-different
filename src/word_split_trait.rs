use std::iter::{Filter, Map};

// pub fn split_unicode_word_and_syms(s: &str) -> Vec<(usize, &str)> {
//     let chars_and_stuff: Vec<_> = s.char_indices()
//                                    .map(|(idx, c)| (idx, c, is_word_character(c), c.is_whitespace()))
//                                    .collect();

//     let mut results = Vec::new();

//     let mut left_idx = 0;
//     let mut last_c_idx = 0;

//     for (c_idx, ((_, _, l_isw, l_iss), (r_idx, _, r_isw, r_iss))) in
//         chars_and_stuff.iter().zip(chars_and_stuff.iter().skip(1)).enumerate() {
//             last_c_idx = c_idx;

//             if !((l_isw != r_isw) || *l_iss || *r_iss) {
//                 continue;
//             }

//             let n_s = &s[left_idx..*r_idx];

//             left_idx = *r_idx;

//             if !n_s.trim().is_empty() {
//                 results.push((c_idx - 2, n_s));
//             }
//         }

//     if !(&s[left_idx..]).is_empty() {
//         results.push((last_c_idx.saturating_sub(2), &s[left_idx..]));
//     }

//     results
// }

struct UnicodeWordBoundaries<'a> {
    s: &'a str,
}

impl<'a> Iterator for UnicodeWordBoundaries<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.s.is_empty() {
            return None;
        }

        let mut c_it = self.s.char_indices()
                             .map(|(idx, c)| ((is_word_character(c), c.is_whitespace()), idx))
                             .peekable();

        while let (Some(((lhs_is_word, lhs_is_ws), _)),
                   Some(((rhs_is_word, rhs_is_ws), r_idx))) = (c_it.next(), c_it.peek().cloned()) {
            if (lhs_is_word != rhs_is_word) || (lhs_is_ws != rhs_is_ws) {
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
