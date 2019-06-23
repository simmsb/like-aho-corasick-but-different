use std::hash::Hash;

// pretty much copied from
// https://github.com/BurntSushi/aho-corasick/blob/f166d2e63d0d7a41339b5e7f8c939dd4196f92f0/src/state_id.rs

/// Convert the given `usize` to the chosen state identifier
/// representation. If the given value cannot fit in the chosen
/// representation, then an error is returned.
pub(crate) fn usize_to_state_id<S: StateID>(value: usize) -> Option<S> {
    if value > S::max_id() {
        None
    } else {
        Some(S::from_usize(value))
    }
}

pub(crate) fn fail_id<S: StateID>() -> S {
    S::from_usize(0)
}

mod private {
    pub(crate) trait Sealed {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
    impl Sealed for usize {}
}

// for explanation of invariants, check:
// https://github.com/BurntSushi/aho-corasick/blob/f166d2e63d0d7a41339b5e7f8c939dd4196f92f0/src/state_id.rs#L60
pub(crate) unsafe trait StateID:
    private::Sealed + Clone + Copy + Eq + Hash + PartialEq + PartialOrd + Ord
{
    fn from_usize(n: usize) -> Self;

    fn to_usize(self) -> usize;

    fn max_id() -> usize;
}

unsafe impl StateID for usize {
    #[inline]
    fn from_usize(n: usize) -> usize {
        n
    }

    #[inline]
    fn to_usize(self) -> usize {
        self
    }

    #[inline]
    fn max_id() -> usize {
        ::std::usize::MAX
    }
}

unsafe impl StateID for u8 {
    #[inline]
    fn from_usize(n: usize) -> u8 {
        n as u8
    }

    #[inline]
    fn to_usize(self) -> usize {
        self as usize
    }

    #[inline]
    fn max_id() -> usize {
        ::std::u8::MAX as usize
    }
}

unsafe impl StateID for u16 {
    #[inline]
    fn from_usize(n: usize) -> u16 {
        n as u16
    }

    #[inline]
    fn to_usize(self) -> usize {
        self as usize
    }

    #[inline]
    fn max_id() -> usize {
        ::std::u16::MAX as usize
    }
}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
unsafe impl StateID for u32 {
    #[inline]
    fn from_usize(n: usize) -> u32 {
        n as u32
    }

    #[inline]
    fn to_usize(self) -> usize {
        self as usize
    }

    #[inline]
    fn max_id() -> usize {
        ::std::u32::MAX as usize
    }
}

#[cfg(target_pointer_width = "64")]
unsafe impl StateID for u64 {
    #[inline]
    fn from_usize(n: usize) -> u64 {
        n as u64
    }

    #[inline]
    fn to_usize(self) -> usize {
        self as usize
    }

    #[inline]
    fn max_id() -> usize {
        ::std::u64::MAX as usize
    }
}
