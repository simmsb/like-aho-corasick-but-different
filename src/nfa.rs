use std::{cmp, collections::VecDeque, mem::size_of};

use crate::{
    automaton::Automaton,
    state_id::{fail_id, usize_to_state_id, StateID},
    Match,
};

type PatternID = usize;
type PatternLength = usize;

#[derive(Clone)]
pub(crate) struct NFA<'a, S> {
    start_id: S,
    max_pattern_len: usize,
    pattern_count: usize,
    heap_bytes: usize,
    states: Vec<State<'a, S>>,
}

impl<'a, S: StateID> NFA<'a, S> {
    pub(crate) fn heap_bytes(&self) -> usize {
        self.heap_bytes
    }

    pub(crate) fn pattern_count(&self) -> usize {
        self.pattern_count
    }

    fn state(&self, id: S) -> &State<'a, S> {
        &self.states[id.to_usize()]
    }

    fn state_mut(&mut self, id: S) -> &mut State<'a, S> {
        &mut self.states[id.to_usize()]
    }

    fn start(&self) -> &State<'a, S> {
        self.state(self.start_id)
    }

    // fn start_mut(&mut self) -> &mut State<'a, S> {
    //     let id = self.start_id;
    //     self.state_mut(id)
    // }

    fn iter_transitions_mut(&mut self, id: S) -> IterTransitionsMut<'_, 'a, S> {
        IterTransitionsMut::new(self, id)
    }

    fn copy_matches(&mut self, src: S, dst: S) {
        let (src, dst) = get_two_mut(&mut self.states, src.to_usize(), dst.to_usize());
        dst.matches.extend_from_slice(&src.matches);
    }

    fn copy_empty_matches(&mut self, dst: S) {
        let start_id = self.start_id;
        self.copy_matches(start_id, dst);
    }

    fn add_sparse_state(&mut self) -> Option<S> {
        let trans = Transitions(vec![]);
        let id = usize_to_state_id(self.states.len())?;
        self.states.push(State {
            trans,
            fail: self.start_id,
            matches: vec![],
        });
        Some(id)
    }
}

impl<'a, S: StateID> Automaton for NFA<'a, S> {
    type ID = S;

    fn start_state(&self) -> S {
        self.start_id
    }

    fn is_valid(&self, id: S) -> bool {
        id.to_usize() < self.states.len()
    }

    fn is_match_state(&self, id: S) -> bool {
        self.states[id.to_usize()].is_match()
    }

    fn get_match(&self, id: S, match_index: usize, end: usize) -> Option<Match> {
        let state = match self.states.get(id.to_usize()) {
            None => return None,
            Some(state) => state,
        };
        state.matches.get(match_index).map(|&(id, len)| Match {
            pattern: id,
            len,
            end,
        })
    }

    fn match_count(&self, id: S) -> usize {
        self.states[id.to_usize()].matches.len()
    }

    unsafe fn next_state_unchecked(&self, mut current: S, input: &str) -> S {
        loop {
            let state = self.states.get_unchecked(current.to_usize());
            let next = state.next_state(input);
            if next != fail_id() {
                return next;
            }

            // if next is fail_id and the current state is the start state, we
            // loop back to the start
            if current == self.start_state() {
                return self.start_state();
            }
            current = state.fail;
        }
    }
}

#[derive(Clone)]
pub(crate) struct State<'a, S> {
    trans: Transitions<'a, S>,
    fail: S,
    matches: Vec<(PatternID, PatternLength)>,
}

impl<'a, S: StateID> State<'a, S> {
    fn heap_bytes(&self) -> usize {
        self.trans.heap_bytes() + (self.matches.len() * size_of::<(PatternID, PatternLength)>())
    }

    fn add_match(&mut self, i: PatternID, len: PatternLength) {
        self.matches.push((i, len));
    }

    fn is_match(&self) -> bool {
        !self.matches.is_empty()
    }

    fn next_state(&self, input: &str) -> S {
        self.trans.next_state(input)
    }

    fn set_next_state<'b: 'a>(&mut self, input: &'b str, next: S) {
        self.trans.set_next_state(input, next);
    }
}

#[derive(Clone)]
struct Transitions<'a, S>(pub Vec<(&'a str, S)>);

impl<'a, S: StateID> Transitions<'a, S> {
    fn heap_bytes(&self) -> usize {
        self.0.len() * size_of::<(&'a str, S)>()
    }

    fn next_state(&self, input: &str) -> S {
        // self.0.get(input).cloned().unwrap_or(fail_id())

        for &(b, id) in &self.0 {
            if b == input {
                return id;
            }
        }
        fail_id()
    }

    fn set_next_state<'b: 'a>(&mut self, input: &'b str, next: S) {
        // self.0.insert(input, next);
        match self.0.binary_search_by_key(&input, |&(b, _)| b) {
            Ok(i) => self.0[i] = (input, next),
            Err(i) => self.0.insert(i, (input, next)),
        }
    }
}

struct IterTransitionsMut<'a, 'b, S: StateID + 'a> {
    nfa: &'a mut NFA<'b, S>,
    state_id: S,
    cur: usize,
}

impl<'a, 'b, S: StateID> IterTransitionsMut<'a, 'b, S> {
    fn new(nfa: &'a mut NFA<'b, S>, state_id: S) -> IterTransitionsMut<'a, 'b, S> {
        IterTransitionsMut {
            nfa,
            state_id,
            cur: 0,
        }
    }

    fn nfa(&mut self) -> &mut NFA<'b, S> {
        self.nfa
    }
}

impl<'a, 'b, S: StateID> Iterator for IterTransitionsMut<'a, 'b, S> {
    type Item = (&'b str, S);

    fn next(&mut self) -> Option<(&'b str, S)> {
        let trans = &self.nfa.states[self.state_id.to_usize()].trans;
        if self.cur >= trans.0.len() {
            return None;
        }
        let i = self.cur;
        self.cur += 1;
        Some(trans.0[i])
    }
}

struct Compiler<'a, S: StateID> {
    nfa: NFA<'a, S>,
}

impl<'a, S: StateID> Compiler<'a, S> {
    fn new() -> Option<Compiler<'a, S>> {
        Some(Compiler {
            nfa: NFA {
                start_id: usize_to_state_id(1)?,
                max_pattern_len: 0,
                pattern_count: 0,
                heap_bytes: 0,
                states: vec![],
            },
        })
    }

    fn compile<I>(mut self, patterns: I) -> Option<NFA<'a, S>>
    where
        I: IntoIterator<Item = &'a str>,
    {
        use unicode_segmentation::UnicodeSegmentation;

        self.add_state()?; // the fail state, which is never entered
        self.add_state()?; // the start state
        let patterns: Vec<Vec<_>> = patterns
            .into_iter()
            .map(|p| {
                p.unicode_words()
                 .collect()
            })
            .collect();
        self.build_trie(&patterns)?;
        self.fill_failure_transitions_standard();
        self.calculate_size();
        Some(self.nfa)
    }

    /// This sets up the initial prefix trie that makes up the Aho-Corasick
    /// automaton. Effectively, it creates the basic structure of the
    /// automaton, where every pattern given has a path from the start state to
    /// the end of the pattern.
    fn build_trie<'b>(&mut self, patterns: &'b [Vec<&'a str>]) -> Option<()> {
        'PATTERNS: for (pati, pat) in patterns.into_iter().enumerate() {
            self.nfa.max_pattern_len = cmp::max(self.nfa.max_pattern_len, pat.len());
            self.nfa.pattern_count += 1;

            let mut prev = self.nfa.start_id;
            let mut saw_match = false;
            for &b in pat.iter() {
                saw_match = saw_match || self.nfa.state(prev).is_match();
                // If the transition from prev using the current byte already
                // exists, then just move through it. Otherwise, add a new
                // state. We track the depth here so that we can determine
                // how to represent transitions. States near the start state
                // use a dense representation that uses more memory but is
                // faster. Other states use a sparse representation that uses
                // less memory but is slower.
                let next = self.nfa.state(prev).next_state(b);
                if next != fail_id() {
                    prev = next;
                } else {
                    let next = self.add_state()?;
                    self.nfa.state_mut(prev).set_next_state(b, next);
                    prev = next;
                }
            }
            // Once the pattern has been added, log the match in the final
            // state that it reached.
            self.nfa.state_mut(prev).add_match(pati, pat.len());
        }
        Some(())
    }

    /// This routine creates failure transitions according to the standard
    /// textbook formulation of the Aho-Corasick algorithm.
    ///
    /// Building failure transitions is the most interesting part of building
    /// the Aho-Corasick automaton, because they are what allow searches to
    /// be performed in linear time. Specifically, a failure transition is
    /// a single transition associated with each state that points back to
    /// the longest proper suffix of the pattern being searched. The failure
    /// transition is followed whenever there exists no transition on the
    /// current state for the current input byte. If there is no other proper
    /// suffix, then the failure transition points back to the starting state.
    ///
    /// For example, let's say we built an Aho-Corasick automaton with the
    /// following patterns: 'abcd' and 'cef'. The trie looks like this:
    ///
    /// ```ignore
    ///          a - S1 - b - S2 - c - S3 - d - S4*
    ///         /
    ///     S0 - c - S5 - e - S6 - f - S7*
    /// ```
    ///
    /// At this point, it should be fairly straight-forward to see how this
    /// trie can be used in a simplistic way. At any given position in the
    /// text we're searching (called the "subject" string), all we need to do
    /// is follow the transitions in the trie by consuming one transition for
    /// each byte in the subject string. If we reach a match state, then we can
    /// report that location as a match.
    ///
    /// The trick comes when searching a subject string like 'abcef'. We'll
    /// initially follow the transition from S0 to S1 and wind up in S3 after
    /// observng the 'c' byte. At this point, the next byte is 'e' but state
    /// S3 has no transition for 'e', so the search fails. We then would need
    /// to restart the search at the next position in 'abcef', which
    /// corresponds to 'b'. The match would fail, but the next search starting
    /// at 'c' would finally succeed. The problem with this approach is that
    /// we wind up searching the subject string potentially many times. In
    /// effect, this makes the algorithm have worst case `O(n * m)` complexity,
    /// where `n ~ len(subject)` and `m ~ len(all patterns)`. We would instead
    /// like to achieve a `O(n + m)` worst case complexity.
    ///
    /// This is where failure transitions come in. Instead of dying at S3 in
    /// the first search, the automaton can instruct the search to move to
    /// another part of the automaton that corresponds to a suffix of what
    /// we've seen so far. Recall that we've seen 'abc' in the subject string,
    /// and the automaton does indeed have a non-empty suffix, 'c', that could
    /// potentially lead to another match. Thus, the actual Aho-Corasick
    /// automaton for our patterns in this case looks like this:
    ///
    /// ```ignore
    ///          a - S1 - b - S2 - c - S3 - d - S4*
    ///         /                      /
    ///        /       ----------------
    ///       /       /
    ///     S0 - c - S5 - e - S6 - f - S7*
    /// ```
    ///
    /// That is, we have a failure transition from S3 to S5, which is followed
    /// exactly in cases when we are in state S3 but see any byte other than
    /// 'd' (that is, we've "failed" to find a match in this portion of our
    /// trie). We know we can transition back to S5 because we've already seen
    /// a 'c' byte, so we don't need to re-scan it. We can then pick back up
    /// with the search starting at S5 and complete our match.
    ///
    /// Adding failure transitions to a trie is fairly simple, but subtle. The
    /// key issue is that you might have multiple failure transition that you
    /// need to follow. For example, look at the trie for the patterns
    /// 'abcd', 'b', 'bcd' and 'cd':
    ///
    /// ```ignore
    ///        - a - S1 - b - S2 - c - S3 - d - S4*
    ///       /
    ///     S0 - b - S5* - c - S6 - d - S7*
    ///       \
    ///        - c - S8 - d - S9*
    /// ```
    ///
    /// The failure transitions for this trie are defined from S2 to S5,
    /// S3 to S6 and S6 to S8. Moreover, state S2 needs to track that it
    /// corresponds to a match, since its failure transition to S5 is itself
    /// a match state.
    ///
    /// Perhaps simplest way to think about adding these failure transitions
    /// is recursively. That is, if you know the failure transitions for every
    /// possible previous state that could be visited (e.g., when computing the
    /// failure transition for S3, you already know the failure transitions
    /// for S0, S1 and S2), then you can simply follow the failure transition
    /// of the previous state and check whether the incoming transition is
    /// defined after following the failure transition.
    ///
    /// For example, when determining the failure state for S3, by our
    /// assumptions, we already know that there is a failure transition from
    /// S2 (the previous state) to S5. So we follow that transition and check
    /// whether the transition connecting S2 to S3 is defined. Indeed, it is,
    /// as there is a transition from S5 to S6 for the byte 'c'. If no such
    /// transition existed, we could keep following the failure transitions
    /// until we reach the start state, which is the failure transition for
    /// every state that has no corresponding proper suffix.
    ///
    /// We don't actually use recursion to implement this, but instead, use a
    /// breadth first search of the automaton. Our base case is the start
    /// state, whose failure transition is just a transition to itself.
    fn fill_failure_transitions_standard(&mut self) {
        // Initialize the queue for breadth first search with all transitions
        // out of the start state. We handle the start state specially because
        // we only want to follow non-self transitions. If we followed self
        // transitions, then this would never terminate.
        let mut queue = VecDeque::new();
        queue.extend(self.nfa.start().trans.0.iter().filter_map(|(_, id)| {
            if *id != self.nfa.start_id {
                Some(id)
            } else {
                None
            }
        }));
        queue.push_back(fail_id());
        while let Some(id) = queue.pop_front() {
            let mut it = self.nfa.iter_transitions_mut(id);
            while let Some((b, next)) = it.next() {
                queue.push_back(next);

                let mut fail = it.nfa().state(id).fail;
                while it.nfa().state(fail).next_state(b) == fail_id() {
                    fail = it.nfa().state(fail).fail;
                }
                fail = it.nfa().state(fail).next_state(b);
                it.nfa().state_mut(next).fail = fail;
                it.nfa().copy_matches(fail, next);
            }
            // If the start state is a match state, then this automaton can
            // match the empty string. This implies all states are match states
            // since every position matches the empty string, so copy the
            // matches from the start state to every state. Strictly speaking,
            // this is only necessary for overlapping matches since each
            // non-empty non-start match state needs to report empty matches
            // in addition to its own. For the non-overlapping case, such
            // states only report the first match, which is never empty since
            // it isn't a start state.
            it.nfa().copy_empty_matches(id);
        }
    }

    /// Computes the total amount of heap used by this NFA in bytes.
    fn calculate_size(&mut self) {
        let mut size = 0;
        for state in &self.nfa.states {
            size += state.heap_bytes();
        }
        self.nfa.heap_bytes = size;
    }

    /// Add a new state to the underlying NFA with the given depth. The depth
    /// is used to determine how to represent the transitions.
    ///
    /// If adding the new state would overflow the chosen state ID
    /// representation, then this returns an error.
    fn add_state(&mut self) -> Option<S> {
        self.nfa.add_sparse_state()
    }
}

pub(crate) fn build_nfa<'a, I, S: StateID>(patterns: I) -> Option<NFA<'a, S>>
where
    I: IntoIterator<Item = &'a str>,
{
    Compiler::new()?.compile(patterns)
}

/// Safely return two mutable borrows to two different locations in the given
/// slice.
///
/// This panics if i == j.
fn get_two_mut<T>(xs: &mut [T], i: usize, j: usize) -> (&mut T, &mut T) {
    assert!(i != j, "{} must not be equal to {}", i, j);
    if i < j {
        let (before, after) = xs.split_at_mut(j);
        (&mut before[i], &mut after[0])
    } else {
        let (before, after) = xs.split_at_mut(i);
        (&mut after[0], &mut before[j])
    }
}
