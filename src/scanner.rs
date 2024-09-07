use std::{cell::RefCell, fmt::Debug, path::Path, rc::Rc};

use crate::{FindMatches, Result, ScannerImpl, ScannerMode};

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
    pub(crate) inner: Rc<RefCell<ScannerImpl>>,
}

impl Scanner {
    /// Returns an iterator over all non-overlapping matches.
    /// The iterator yields a [`Match`] value until no more matches could be found.
    pub fn find_iter<'h>(&self, input: &'h str) -> FindMatches<'h> {
        ScannerImpl::find_iter(self.inner.clone(), input)
    }

    /// Returns the current scanner mode.
    pub fn current_mode(&self) -> usize {
        self.inner.borrow().current_mode()
    }

    /// Sets the current scanner mode.
    ///
    /// A parser can explicitly set the scanner mode to switch to a different set of DFAs.
    /// Usually, the scanner mode is changed by the scanner itself based on the transitions defined
    /// in the active scanner mode.
    pub fn set_mode(&mut self, mode: usize) {
        self.inner.borrow_mut().set_mode(mode);
    }

    /// Returns the name of the scanner mode with the given index.
    /// If the index is out of bounds, None is returned.
    pub fn mode_name(&self, index: usize) -> Option<String> {
        self.inner.borrow().mode_name(index).map(|s| s.to_owned())
    }

    /// Logs the compiled DFAs as a Graphviz DOT file with the help of the `log` crate.
    /// To enable debug output compliled DFA as dot file set the environment variable `RUST_LOG` to
    /// `scnr::internal::scanner_impl=debug`.
    pub fn log_compiled_dfas_as_dot(&self, modes: &[ScannerMode]) -> Result<()> {
        self.inner.borrow().log_compiled_dfas_as_dot(modes)
    }

    /// Generates the compiled DFAs as a Graphviz DOT files.
    /// The DOT files are written to the target folder.
    /// The file names are derived from the scanner mode names and the index of the DFA.
    pub fn generate_compiled_dfas_as_dot<T>(
        &self,
        modes: &[ScannerMode],
        target_folder: T,
    ) -> Result<()>
    where
        T: AsRef<Path>,
    {
        self.inner
            .borrow()
            .generate_compiled_dfas_as_dot(modes, target_folder)
    }
}

impl Debug for Scanner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Scanner")
    }
}
