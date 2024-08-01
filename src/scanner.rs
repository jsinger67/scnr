use crate::{FindMatches, Match, ScannerImpl};

/// A Scanner.
/// It consists of multiple DFAs that are used to search for matches.
///
/// Each DFA corresponds to a terminal symbol (token type) the lexer/scanner can recognize.
/// The DFAs are advanced in parallel to search for matches.
/// It further constists of at least one scanner mode. Scanners support multiple scanner modes.
/// This feature is known from Flex as *Start conditions* and provides more
/// flexibility by defining several scanners for several parts of your grammar.
/// See <https://www.cs.princeton.edu/~appel/modern/c/software/flex/flex.html#SEC11>
/// for more information.
///
/// To create a scanner, you can use the `ScannerBuilder` to add scanner mode data.
/// At least one scanner mode must be added to the scanner. This is usually the mode named `INITIAL`.
pub struct Scanner {
    _inner: ScannerImpl,
}

impl Scanner {
    /// Returns an iterator over all non-overlapping matches.
    /// The iterator yields a [`Match`] value until no more matches could be found.
    pub fn find_iter<'h>(&self, _input: &'h str) -> FindMatches<'h> {
        todo!()
    }

    /// Executes a leftmost search and returns the first match that is found, if one exists.
    /// It starts the search at the position of the given CharIndices iterator.
    /// During the search, all DFAs are advanced in parallel by one character at a time.
    pub fn find_from(&mut self, _char_indices: std::str::CharIndices) -> Option<Match> {
        todo!()
    }
}
