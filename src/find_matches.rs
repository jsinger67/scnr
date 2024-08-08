use crate::{FindMatchesImpl, Match, Result, ScannerImpl};

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
    pub(crate) fn new(scanner_impl: &ScannerImpl, input: &'h str) -> Result<Self> {
        Ok(Self {
            inner: FindMatchesImpl::try_new(scanner_impl, input)?,
        })
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
}

impl Iterator for FindMatches<'_> {
    type Item = Match;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_match()
    }
}
