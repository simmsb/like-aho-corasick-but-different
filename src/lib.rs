use std::collections::{HashMap, HashSet};

mod ahocorasick;
mod automaton;
mod nfa;
mod state_id;
mod word_split_trait;
mod unicode_tables;


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

pub struct SimpleFinder<D> {
    aho: ahocorasick::AhoCorasick,
    data: HashMap<usize, D>,
}

pub struct SimpleFinderIter<'a, 'b, D> {
    finder: &'a SimpleFinder<D>,
    iter: ahocorasick::FindOverlappingIter<'a, 'b, usize>,
}

impl<'a, 'b, D> Iterator for SimpleFinderIter<'a, 'b, D> {
    type Item = (Match, &'a D);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next()?;
        let data = &self.finder.data.get(&next.pattern)?;

        Some((next, data))
    }
}

impl<D> SimpleFinder<D> {
    pub fn new<'p, I>(patterns: I) -> Self
    where
        I: IntoIterator<Item = (&'p str, D)>,
    {
        let (patterns, datas): (Vec<_>, Vec<_>) = patterns.into_iter().unzip();

        let aho = ahocorasick::build_aho_corasick(patterns);

        let data = (0..aho.pattern_count()).zip(datas.into_iter()).collect();

        SimpleFinder { aho, data }
    }

    pub fn find_all<'a: 'b, 'b>(&'a self, haystack: &'b str) -> SimpleFinderIter<'a, 'b, D> {
        SimpleFinderIter {
            finder: self,
            iter: self.aho.find_overlapping_iter(haystack),
        }
    }

    pub fn pattern_count(&self) -> usize {
        self.aho.pattern_count()
    }

    pub fn heap_bytes(&self) -> usize {
        self.aho.heap_bytes() + self.data.capacity() * (std::mem::size_of::<(usize, D)>())
    }

    pub fn data(&self) -> &HashMap<usize, D> {
        &self.data
    }
}

impl<'p, D: std::hash::Hash + std::cmp::Eq + Copy> SimpleFinder<D> {
    pub fn find_all_unique<'a, 'b>(&'a self, haystack: &'b str) -> HashSet<D> {
        self.find_all(haystack).map(|(_, d)| *d).collect()
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
            .find_all("foo bar baz foobar foo'bar foo,bar")
            .map(|(_, k)| k)
            .cloned()
            .collect();

        assert_eq!(
            results.len(),
            8,
            "Results {:?} was not 8 in length?",
            results
        );
        assert!(results.contains(&123));
        assert!(results.contains(&234));
        assert!(results.contains(&345));
        assert!(results.contains(&456));
    }

    #[test]
    fn test_unique() {
        let finder = SimpleFinder::new(vec![
            ("foo", 123),
            ("bar", 234),
            ("baz", 345),
            ("bar baz", 456),
        ]);

        let results = finder.find_all_unique("foo bar baz foobar foo'bar foo,bar");

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

    #[test]
    fn test_loops() {
        let finder = SimpleFinder::new(vec![
            ("lol lol_", 0),
            ("lol lol", 2),
        ]);

        let results: Vec<_> = finder.find_all("lol lol lol lol_").collect();
        assert_eq!(
            results,
            vec![
                (
                    Match {
                        pattern: 1,
                        len: 7,
                        end: 7
                    },
                    &2
                ),
                (
                    Match {
                        pattern: 1,
                        len: 7,
                        end: 11
                    },
                    &2
                ),
                (
                    Match {
                        pattern: 0,
                        len: 8,
                        end: 16
                    },
                    &0
                )
            ]
        );
    }
}
