use std::{cell::RefCell, rc::Rc};

use crate::{FindMatchesImpl, Match, ScannerImpl};

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
/// The iterator yields a [`Match`] value until no more matches could be found.
///
/// * `'h` represents the lifetime of the haystack being searched.
///
/// This iterator can be created with the [`Scanner::find_iter`] method.
#[derive(Debug)]
pub struct FindMatches<'h> {
    inner: FindMatchesImpl<'h>,
}

impl<'h> FindMatches<'h> {
    /// Creates a new `FindMatches` iterator.
    pub(crate) fn new(scanner_impl: Rc<RefCell<ScannerImpl>>, input: &'h str) -> Self {
        Self {
            inner: FindMatchesImpl::new(scanner_impl, input),
        }
    }

    /// Sets an offset to the current position of the scanner. This offset is added to the start
    /// position of each match.
    /// If a parser resets the scanner to a previous position, it can set the offset to the number
    /// of bytes where the scanner was reset.
    pub fn with_offset(self, offset: usize) -> Self {
        Self {
            inner: self.inner.with_offset(offset),
        }
    }

    /// Retrieve the byte offset of the char indices iterator from the start of the haystack.
    #[inline]
    pub fn offset(&self) -> usize {
        self.inner.offset()
    }

    /// Returns the next match in the haystack.
    ///
    /// If no match is found, `None` is returned.
    ///
    /// The function calls the `find_from` method of the scanner to find the next match.
    /// If a match is found, the function advances the char_indices iterator to the end of the match.
    /// If no match is found, the function repeatedly advances the char_indices iterator by one
    /// and tries again until a match is found or the iterator is exhausted.
    #[inline]
    pub fn next_match(&mut self) -> Option<Match> {
        self.inner.next_match()
    }

    /// Peeks n matches ahead without consuming the matches.
    /// The function returns [PeekResult].
    ///
    /// The peek operation always stops at the end of the haystack or when a mode switch is
    /// triggered by the last match. The mode switch is not conducted by the peek operation to not
    /// change the state of the scanner as well as to aviod a mix of tokens from different modes
    /// being returned.
    pub fn peek_n(&mut self, n: usize) -> PeekResult {
        self.inner.peek_n(n)
    }

    /// Advane the char_indices iterator to the given position.
    /// The function is used to skip a given number of characters in the haystack.
    /// It can be used after a peek operation to skip the characters of the peeked matches.
    /// The function returns the new position of the char_indices iterator.
    /// If the new position is greater than the length of the haystack, the function returns the
    /// length of the haystack.
    /// If the new position is less than the current position of the char_indices iterator, the
    /// function returns the current position of the char_indices iterator.
    /// The current position of the char_indices iterator is the end index of the last match found
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
