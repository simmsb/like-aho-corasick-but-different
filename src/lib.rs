#![feature(box_syntax)]
use std::collections::HashMap;

mod ahocorasick;
mod automaton;
mod nfa;
mod state_id;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Match {
    /// The pattern id.
    pattern: usize,
    /// The length of this match, such that the starting position of the match
    /// is `end - len`.
    ///
    /// We use length here because, other than the pattern id, the only
    /// information about each pattern that the automaton stores is its length.
    /// So using the length here is just a bit more natural. But it isn't
    /// technically required.
    len: usize,
    /// The end offset of the match, exclusive.
    end: usize,
}

impl Match {
    /// Returns the identifier of the pattern that matched.
    ///
    /// The identifier of a pattern is derived from the position in which it
    /// was originally inserted into the corresponding automaton. The first
    /// pattern has identifier `0`, and each subsequent pattern is `1`, `2`
    /// and so on.
    #[inline]
    pub fn pattern(&self) -> usize {
        self.pattern
    }

    /// The starting position of the match.
    #[inline]
    pub fn start(&self) -> usize {
        self.end - self.len
    }

    /// The ending position of the match.
    #[inline]
    pub fn end(&self) -> usize {
        self.end
    }

    /// Returns true if and only if this match is empty. That is, when
    /// `start() == end()`.
    ///
    /// An empty match can only be returned when the empty string was among
    /// the patterns used to build the Aho-Corasick automaton.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

pub struct SimpleFinder<'p, D> {
    aho: ahocorasick::AhoCorasick<'p>,
    data: HashMap<usize, D>,
}

pub struct SimpleFinderIter<'a, 'b, 'p, D> {
    finder: &'a SimpleFinder<'p, D>,
    iter: ahocorasick::FindOverlappingIter<'a, 'b, 'p, usize>,
}

impl<'a, 'b, 'p, D> Iterator for SimpleFinderIter<'a, 'b, 'p, D> {
    type Item = (Match, &'a D);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next()?;
        let data = &self.finder.data.get(&next.pattern)?;

        Some((next, data))
    }
}

impl<'p, D> SimpleFinder<'p, D> {
    pub fn new<I>(patterns: I) -> Self
    where
        I: IntoIterator<Item = (&'p str, D)>,
    {
        let (patterns, datas): (Vec<_>, Vec<_>) = patterns.into_iter().unzip();

        let aho = ahocorasick::build_aho_corasick(patterns);

        let data = (0..aho.pattern_count()).zip(datas.into_iter()).collect();

        SimpleFinder { aho, data }
    }

    pub fn find_all<'a, 'b: 'a>(
        &'a self,
        haystack: &'b str,
    ) -> SimpleFinderIter<'a, 'b, 'p, D> {
        SimpleFinderIter {
            finder: self,
            iter: self.aho.find_overlapping_iter(haystack),
        }
    }

    pub fn pattern_count(&self) -> usize {
        self.aho.pattern_count()
    }

    pub fn heap_bytes(&self) -> usize {
        self.aho.heap_bytes()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_iguess() {
        let finder = SimpleFinder::new(vec![
            ("foo", 123),
            ("bar", 234),
            ("baz", 345),
            ("bar baz", 456),
        ]);

        let results: Vec<u64> = finder
            .find_all("foo bar baz foobar")
            .map(|(_, k)| k)
            .cloned()
            .collect();

        assert_eq!(
            results.len(),
            4,
            "Results {:?} was not 4 in length?",
            results
        );
        assert!(results.contains(&123));
        assert!(results.contains(&234));
        assert!(results.contains(&345));
        assert!(results.contains(&456));
    }
}
