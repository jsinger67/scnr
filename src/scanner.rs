use crate::{FindMatches, Result, ScannerImpl};

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
    pub(crate) inner: ScannerImpl,
}

impl Scanner {
    /// Returns an iterator over all non-overlapping matches.
    /// The iterator yields a [`Match`] value until no more matches could be found.
    pub fn find_iter<'h>(&self, input: &'h str) -> Result<FindMatches<'h>> {
        self.inner.find_iter(input)
    }

    /// Returns the current scanner mode.
    pub fn current_mode(&self) -> usize {
        self.inner.current_mode()
    }

    /// Sets the current scanner mode.
    ///
    /// A parser can explicitly set the scanner mode to switch to a different set of DFAs.
    /// Usually, the scanner mode is changed by the scanner itself based on the transitions defined
    /// in the active scanner mode.
    pub fn set_mode(&mut self, mode: usize) {
        self.inner.set_mode(mode);
    }

    /// Returns the name of the scanner mode with the given index.
    /// If the index is out of bounds, None is returned.
    pub fn mode_name(&self, index: usize) -> Option<&str> {
        self.inner.mode_name(index)
    }
}
