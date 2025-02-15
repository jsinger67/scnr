use log::trace;

use crate::{
    internal::{find_matches_impl::FindMatchesImpl, ScannerImpl},
    Match, Position, PositionProvider, ScannerModeSwitcher,
};

/// The result of a peek operation.
#[derive(Debug, PartialEq)]
pub enum PeekResult {
    /// The peek operation found n matches.
    Matches(Vec<Match>),
    /// The peek operation found less than n matches because the end of the haystack was reached.
    MatchesReachedEnd(Vec<Match>),
    /// The peek operation found less than n matches because the last token type would have
    /// triggered a mode switch. The matches are returned along with the index of the new mode that
    /// would be switched to on the last match.
    MatchesReachedModeSwitch((Vec<Match>, usize)),
    /// The peek operation found no matches.
    NotFound,
}

/// An iterator over all non-overlapping matches.
///
/// The iterator yields [`Match`] values until no more matches could be found.
///
/// * `'h` represents the lifetime of the haystack being searched.
///
/// This iterator can be created with the [`crate::Scanner::find_iter`] method.
#[derive(Debug)]
pub struct FindMatches<'h> {
    inner: FindMatchesImpl<'h>,
}

impl<'h> FindMatches<'h> {
    /// Creates a new `FindMatches` iterator.
    pub(crate) fn new(scanner_impl: ScannerImpl, input: &'h str) -> Self {
        Self {
            inner: FindMatchesImpl::new(scanner_impl, input),
        }
    }

    /// Set the offset in the haystack to the given position relative to the start of the haystack.
    /// If a parser resets the scanner to a certain position, it can use this method.
    /// A use case is a parser that backtracks to a previous position in the input or a parser that
    /// switches between different scanner modes on its own.
    /// If the offset is greater than the length of the haystack, the offset is set to the length of
    /// the haystack.
    pub fn with_offset(self, offset: usize) -> Self {
        Self {
            inner: self.inner.with_offset(offset),
        }
    }

    /// Set the offset in the haystack to the given position relative to the start of the haystack.
    /// The function is used to set the position in the haystack to the given position.
    /// It provides the same functionality as the `with_offset` method, but it mutates the object
    /// in place.
    pub fn set_offset(&mut self, position: usize) {
        self.inner.set_offset(position);
    }

    /// Retrieve the current byte offset from the start of the haystack.
    /// This is the end offset of the last match found by the iterator.
    #[inline]
    pub fn offset(&self) -> usize {
        self.inner.offset()
    }

    /// Returns the next match in the haystack.
    ///
    /// If a match is found, the function advances the iterator to the end of the match and returns
    /// the match.
    ///
    /// If no match is found, the function repeatedly advances the haystack by one and tries again
    /// until a match is found or the iterator is exhausted.
    ///
    /// If the iterator is exhausted and no match is found, `None` is returned.
    ///
    /// This method is also used in the implementation of the `Iterator` trait for the `FindMatches`.
    #[inline]
    pub fn next_match(&mut self) -> Option<Match> {
        self.inner.next_match()
    }

    /// Peeks n matches ahead without consuming the matches.
    /// The function returns [PeekResult].
    ///
    /// The peek operation always stops at the end of the haystack or when a mode switch is
    /// triggered by the last match. The mode switch is not conducted by the peek operation to not
    /// change the state of the scanner as well as to avoid a mix of tokens from different modes
    /// being returned.
    pub fn peek_n(&mut self, n: usize) -> PeekResult {
        self.inner.peek_n(n)
    }

    /// Advance the haystack to the given position.
    /// The function is used to skip a given number of characters in the haystack.
    /// It can be used after a peek operation to skip the characters of the peeked matches.
    /// The function returns the new position in the haystack.
    /// If the new position is greater than the length of the haystack, the function returns the
    /// length of the haystack.
    /// If the new position is less than the current position in the haystack, the
    /// function returns the current position in the haystack, i.e. it does not allow to move
    /// backwards in the haystack.
    /// The current position in the haystack is the end index of the last match found
    /// by the iterator, such that the next call to `next_match` will start searching for matches
    /// at the following position.
    pub fn advance_to(&mut self, position: usize) -> usize {
        self.inner.advance_to(position)
    }
}

impl Iterator for FindMatches<'_> {
    type Item = Match;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_match()
    }
}

impl PositionProvider for FindMatches<'_> {
    /// Returns the line and column numbers of the given offset.
    /// The line number is the index of the line offset in the vector plus one.
    /// The column number is the offset minus the line offset.
    /// If the offset is greater than the length of the haystack, the function returns the last
    /// recorded line and the column number is calculated from the last recorded position.
    fn position(&self, offset: usize) -> Position {
        self.inner.position(offset)
    }

    /// Sets the offset of the haystack to the given position.
    fn set_offset(&mut self, offset: usize) {
        self.inner.set_offset(offset);
    }
}

impl ScannerModeSwitcher for FindMatches<'_> {
    /// Sets the current scanner mode of the scanner implementation.
    ///
    /// A parser can explicitly set the scanner mode to switch to a different set of DFAs.
    /// Usually, the scanner mode is changed by the scanner itself based on the transitions defined
    /// in the active scanner mode.
    fn set_mode(&mut self, mode: usize) {
        trace!("Set scanner mode to {}", mode);
        self.inner.set_mode(mode);
    }

    /// Returns the current scanner mode. Used for tests and debugging purposes.
    #[allow(dead_code)]
    #[inline]
    fn current_mode(&self) -> usize {
        self.inner.current_mode()
    }

    fn mode_name(&self, index: usize) -> Option<&str> {
        self.inner.mode_name(index)
    }
}
