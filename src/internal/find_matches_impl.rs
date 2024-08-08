use super::{CharClassID, CompiledScannerMode, ScannerImpl};
use crate::{Match, PeekResult, Result};

/// An iterator over all non-overlapping matches.
pub(crate) struct FindMatchesImpl<'h> {
    // The scanner used to find matches.
    scanner_modes: Vec<CompiledScannerMode>,
    // The function used to match characters to character classes.
    match_char_class: Box<dyn Fn(CharClassID, char) -> bool + 'static>,
    // The input haystack.
    char_indices: std::str::CharIndices<'h>,
}

impl<'h> FindMatchesImpl<'h> {
    /// Creates a new `FindMatches` iterator.
    pub(crate) fn try_new(scanner_impl: &ScannerImpl, input: &'h str) -> Result<Self> {
        Ok(Self {
            scanner_modes: scanner_impl.scanner_modes.clone(),
            match_char_class: scanner_impl.create_match_char_class().unwrap(),
            char_indices: input.char_indices(),
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
    pub(crate) fn next_match(&mut self) -> Option<Match> {
        todo!()
    }

    /// Peeks n matches ahead without consuming the matches.
    /// The function returns [PeekResult].
    ///
    /// The peek operation always stops at the end of the haystack or when a mode switch is
    /// triggered by the last match. The mode switch is not conducted by the peek operation to not
    /// change the state of the scanner as well as to aviod a mix of tokens from different modes
    /// being returned.
    pub(crate) fn peek_n(&mut self, _n: usize) -> PeekResult {
        todo!()
    }
}

impl std::fmt::Debug for FindMatchesImpl<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FindMatchesImpl").finish()
    }
}
