use crate::{Match, state_id::{StateID, fail_id}};

pub(crate) trait Automaton {
    /// The representation used for state identifiers in this automaton.
    ///
    /// Typically, this is one of `u8`, `u16`, `u32`, `u64` or `usize`.
    type ID: StateID;

    /// Return the identifier of this automaton's start state.
    fn start_state(&self) -> Self::ID;

    /// Returns true if and only if the given state identifier refers to a
    /// valid state.
    fn is_valid(&self, id: Self::ID) -> bool;

    /// Returns true if and only if the given identifier corresponds to a match
    /// state.
    ///
    /// The state ID given must be valid, or else implementors may panic.
    fn is_match_state(&self, id: Self::ID) -> bool;

    /// If the given state is a match state, return the match corresponding
    /// to the given match index. `end` must be the ending position of the
    /// detected match. If no match exists or if `match_index` exceeds the
    /// number of matches in this state, then `None` is returned.
    ///
    /// The state ID given must be valid, or else implementors may panic.
    ///
    /// If the given state ID is correct and if the `match_index` is less than
    /// the number of matches for that state, then this is guaranteed to return
    /// a match.
    fn get_match(
        &self,
        id: Self::ID,
        match_index: usize,
        end: usize,
    ) -> Option<Match>;

    /// Returns the number of matches for the given state. If the given state
    /// is not a match state, then this returns 0.
    ///
    /// The state ID given must be valid, or else implementors must panic.
    fn match_count(&self, id: Self::ID) -> usize;

    /// Given the current state that this automaton is in and the next input
    /// byte, this method returns the identifier of the next state. The
    /// identifier returned must always be valid and may never correspond to
    /// the fail state. The returned identifier may, however, point to the
    /// dead state.
    ///
    /// This is not safe so that implementors may look up the next state
    /// without memory safety checks such as bounds checks. As such, callers
    /// must ensure that the given identifier corresponds to a valid automaton
    /// state. Implementors must, in turn, ensure that this routine is safe for
    /// all valid state identifiers and for all possible `u8` values.
    unsafe fn next_state_unchecked(
        &self,
        current: Self::ID,
        input: &str,
    ) -> Self::ID;

    /// Like next_state_unchecked, but debug_asserts that the underlying
    /// implementation never returns a `fail_id()` for the next state.
    unsafe fn next_state_unchecked_no_fail(
        &self,
        current: Self::ID,
        input: &str,
    ) -> Self::ID {
        let next = self.next_state_unchecked(current, input);
        // We should never see a transition to the failure state.
        debug_assert!(
            next != fail_id(),
            "automaton should never return fail_id for next state"
        );
        next
    }

    // It's important for this to always be inlined. Namely, it's only caller
    // is standard_find_at, and the inlining should remove the case analysis
    // for prefilter scanning when there is no prefilter available.
    #[inline(always)]
    fn standard_find_at(
        &self,
        haystack: &[&str],
        at: usize,
        state_id: &mut Self::ID,
    ) -> Option<Match> {
        // This is necessary for guaranteeing a safe API, since we use the
        // state ID below in a function that exhibits UB if called with an
        // invalid state ID.
        assert!(
            self.is_valid(*state_id),
            "{} is not a valid state ID",
            state_id.to_usize()
        );

        // println!("outer: state: {}, at: {}", state_id.to_usize(), at);

        for (idx, elem) in haystack[at..].iter().enumerate() {
            *state_id = unsafe { self.next_state_unchecked_no_fail(*state_id, elem) };
            // println!("inner: state: {}", state_id.to_usize());

            if let Some(m) = self.get_match(*state_id, 0, idx + at + 1) {
                return Some(m);
            }
        }
        None

        // unsafe {
        //     let start = haystack.as_ptr();
        //     let end = haystack[haystack.len()..].as_ptr();
        //     let mut ptr = haystack[at..].as_ptr();
        //     while ptr < end {
        //         // SAFETY: next_state is safe for all possible u8 values,
        //         // so the only thing we're concerned about is the validity
        //         // of `state_id`. `state_id` either comes from the caller
        //         // (in which case, we assert above that it is valid), or it
        //         // comes from the return value of next_state, which is also
        //         // guaranteed to be valid.
        //         *state_id = self.next_state_unchecked_no_fail(*state_id, *ptr);
        //         ptr = ptr.offset(1);

        //         let end = ptr as usize - start as usize;
        //         eprintln!("at: {}, start: {:?}, end: {}, ptr: {:?}", at, start, end, ptr);
        //         if let Some(m) = self.get_match(*state_id, 0, end) {
        //             return Some(m);
        //         }
        //     }
        //     None
        // }
    }

    /// Execute an overlapping search.
    ///
    /// When executing an overlapping match, the previous state ID in addition
    /// to the previous match index should be given. If there are more matches
    /// at the given state, then the match is reported and the given index is
    /// incremented.
    #[inline(always)]
    fn overlapping_find_at(
        &self,
        haystack: &[&str],
        at: usize,
        state_id: &mut Self::ID,
        match_index: &mut usize,
    ) -> Option<Match> {
        println!("entering overlapping_find_at");
        let match_count = self.match_count(*state_id);
        if *match_index < match_count {
            // This is guaranteed to return a match since
            // match_index < match_count.
            let result = self.get_match(
                *state_id,
                *match_index,
                at,
            );
            debug_assert!(result.is_some(), "must be a match");
            *match_index += 1;
            return result;
        }

        *match_index = 0;
        match self.standard_find_at(haystack, at, state_id) {
            None => None,
            Some(m) => {
                *match_index = 1;
                Some(m)
            }
        }
    }
}
