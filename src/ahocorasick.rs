use std::collections::HashMap;
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
    word_char_idx_map: HashMap<u32, u32>,
    haystack: Vec<&'b str>,
    pos: usize,
    state_id: S,
    match_index: usize,
}

impl<'a, 'b, S: StateID> FindOverlappingIter<'a, 'b, S> {
    fn new(
        ac: &'a AhoCorasick<S>,
        haystack_str: &'b str,
    ) -> FindOverlappingIter<'a, 'b, S> {
        use crate::word_split_trait::WordBoundarySplitter;

        let input_len = haystack_str.chars().count() + 1;

        let mut word_char_idx_map = HashMap::new();
        let mut haystack = Vec::new();

        for (word_idx, (char_idx, word)) in haystack_str
            .unicode_words_and_syms_indices()
            .enumerate() {
                word_char_idx_map.insert(word_idx as u32, char_idx);
                haystack.push(word);
            }

        word_char_idx_map.insert(haystack.len() as u32, input_len as u32);

        FindOverlappingIter {
            fsm: &ac.imp,
            word_char_idx_map,
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
            Some(mut m) => {
                self.pos = m.end();

                let start_idx = self.word_char_idx_map.get(&((m.end - m.len) as u32))?;
                let end_idx   = self.word_char_idx_map.get(&(m.end as u32))? - 1;

                let len = end_idx - start_idx;
                m.len = len as usize;
                m.end = end_idx as usize;
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
