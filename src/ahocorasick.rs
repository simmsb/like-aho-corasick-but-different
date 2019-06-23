use crate::{
    automaton::Automaton,
    nfa::{self, NFA},
    state_id::StateID,
    Match,
};

#[derive(Clone)]
pub(crate) struct AhoCorasick<'p, S: StateID = usize> {
    imp: NFA<'p, S>,
}

impl<'p, S: StateID> AhoCorasick<'p, S> {
    pub(crate) fn find_overlapping_iter<'a, 'b: 'a>(
        &'a self,
        haystack: &'b str,
    ) -> FindOverlappingIter<'a, 'b, 'p, S> {
        FindOverlappingIter::new(self, haystack)
    }

    pub(crate) fn pattern_count(&self) -> usize {
        self.imp.pattern_count()
    }

    pub(crate) fn heap_bytes(&self) -> usize {
        self.imp.heap_bytes()
    }
}

pub(crate) struct FindOverlappingIter<'a, 'b, 'p: 'a, S: 'a + StateID> {
    fsm: &'a NFA<'p, S>,
    haystack: Vec<&'b str>,
    pos: usize,
    state_id: S,
    match_index: usize,
}

impl<'a, 'b, 'p, S: StateID> FindOverlappingIter<'a, 'b, 'p, S> {
    fn new(
        ac: &'a AhoCorasick<'p, S>,
        haystack: &'b str,
    ) -> FindOverlappingIter<'a, 'b, 'p, S> {
        use unicode_segmentation::UnicodeSegmentation;

        let haystack = haystack
            .split_word_bounds()
            .filter(|w| !w.trim().is_empty())
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

impl<'a, 'b, 'p, S: StateID> Iterator for FindOverlappingIter<'a, 'b, 'p, S> {
    type Item = Match;

    fn next(&mut self) -> Option<Match> {
        let result = self.fsm.overlapping_find_at(
            &self.haystack,
            self.pos,
            &mut self.state_id,
            &mut self.match_index,
        );
        match result {
            None => return None,
            Some(m) => {
                self.pos = m.end();
                Some(m)
            }
        }
    }
}

pub(crate) fn build_aho_corasick<'p, I>(patterns: I) -> AhoCorasick<'p>
where
    I: IntoIterator<Item = &'p str>,
{
    AhoCorasick {
        imp: nfa::build_nfa(patterns).unwrap(),
    }
}
