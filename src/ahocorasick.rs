use crate::{
    automaton::Automaton,
    nfa::{self, NFA},
    state_id::StateID,
    Match,
};

#[derive(Clone)]
pub(crate) struct AhoCorasick<S: StateID = usize> {
    imp: NFA<S>,
}

impl<S: StateID> AhoCorasick<S> {
    pub(crate) fn find_overlapping_iter<'a: 'b, 'b>(
        &'a self,
        haystack: &'b str,
    ) -> FindOverlappingIter<'a, 'b, S> {
        FindOverlappingIter::new(self, haystack)
    }

    pub(crate) fn pattern_count(&self) -> usize {
        self.imp.pattern_count()
    }

    pub(crate) fn heap_bytes(&self) -> usize {
        self.imp.heap_bytes()
    }
}

pub(crate) struct FindOverlappingIter<'a, 'b, S: 'a + StateID> {
    fsm: &'a NFA<S>,
    haystack: Vec<&'b str>,
    pos: usize,
    state_id: S,
    match_index: usize,
}

impl<'a, 'b, S: StateID> FindOverlappingIter<'a, 'b, S> {
    fn new(
        ac: &'a AhoCorasick<S>,
        haystack: &'b str,
    ) -> FindOverlappingIter<'a, 'b, S> {
        use crate::word_split_trait::WordBoundarySplitter;

        let haystack = haystack
            .unicode_words_and_syms()
            .collect();

        FindOverlappingIter {
            fsm: &ac.imp,
            haystack,
            pos: 0,
            state_id: ac.imp.start_state(),
            match_index: 0,
        }
    }
}

impl<'a, 'b, S: StateID> Iterator for FindOverlappingIter<'a, 'b, S> {
    type Item = Match;

    fn next(&mut self) -> Option<Match> {
        let result = self.fsm.overlapping_find_at(
            &self.haystack,
            self.pos,
            &mut self.state_id,
            &mut self.match_index,
        );
        match result {
            None => None,
            Some(m) => {
                self.pos = m.end();
                Some(m)
            }
        }
    }
}

pub(crate) fn build_aho_corasick<'p, I>(patterns: I) -> AhoCorasick
where
    I: IntoIterator<Item = &'p str>,
{
    AhoCorasick {
        imp: nfa::build_nfa(patterns).unwrap(),
    }
}
