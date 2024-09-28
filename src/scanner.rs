use std::{fmt::Debug, path::Path};

use log::trace;

use crate::{FindMatches, Result, ScannerImpl, ScannerMode};

/// A trait to switch between scanner modes.
pub trait ScannerModeSwitcher {
    /// Sets the current scanner mode.
    fn set_mode(&mut self, mode: usize);
    /// Returns the current scanner mode.
    fn current_mode(&self) -> usize;
    /// Returns the name of the scanner mode with the given index.
    fn mode_name(&self, index: usize) -> Option<&str>;
}

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
    /// Creates a new scanner.
    /// The scanner is created with the given scanner modes.
    /// The ScannerImpl is created from the scanner modes and the current mode is shared between
    /// the scanner and the scanner impl.
    pub fn try_new(scanner_modes: Vec<ScannerMode>) -> Result<Self> {
        // Share the current mode between the scanner and the scanner impl.
        Ok(Scanner {
            inner: ScannerImpl::try_from(scanner_modes)?,
        })
    }

    /// Returns an iterator over all non-overlapping matches.
    /// The iterator yields a [`Match`] value until no more matches could be found.
    pub fn find_iter<'h>(&self, input: &'h str) -> FindMatches<'h> {
        ScannerImpl::find_iter(self.inner.clone(), input)
    }

    /// Logs the compiled DFAs as a Graphviz DOT file with the help of the `log` crate.
    /// To enable debug output compliled DFA as dot file set the environment variable `RUST_LOG` to
    /// `scnr::internal::scanner_impl=debug`.
    pub fn log_compiled_dfas_as_dot(&self, modes: &[ScannerMode]) -> Result<()> {
        self.inner.log_compiled_dfas_as_dot(modes)
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
            .generate_compiled_dfas_as_dot(modes, target_folder)
    }
}

impl ScannerModeSwitcher for Scanner {
    /// Returns the current scanner mode.
    fn current_mode(&self) -> usize {
        self.inner.current_mode()
    }

    /// Sets the current scanner mode.
    ///
    /// A parser can explicitly set the scanner mode to switch to a different set of DFAs.
    /// Usually, the scanner mode is changed by the scanner itself based on the transitions defined
    /// in the active scanner mode.
    fn set_mode(&mut self, mode: usize) {
        trace!("Set scanner mode to {}", mode);
        self.inner.set_mode(mode);
    }

    /// Returns the name of the scanner mode with the given index.
    /// If the index is out of bounds, None is returned.
    fn mode_name(&self, index: usize) -> Option<&str> {
        self.inner.mode_name(index)
    }
}

impl Debug for Scanner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Scanner")
    }
}

#[cfg(test)]
mod tests {
    use crate::ScannerBuilder;

    use super::*;

    /// A macro that simplifies the rendering of a dot file for a DFA.
    macro_rules! compiled_dfa_render_to {
        ($dfa:expr, $label:expr, $reg:expr) => {
            let label = format!("{} Compiled DFA", $label);
            let mut f =
                std::fs::File::create(format!("target/{}CompiledDfaFromScannerMode.dot", $label))
                    .unwrap();
            $crate::internal::dot::compiled_dfa_render($dfa, &label, &$reg, &mut f);
        };
    }

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_scanner_builder_with_single_mode() {
        init();
        let scanner_mode = ScannerMode::new(
            "INITIAL",
            vec![(r"\r\n|\r|\n", 1), (r"(//.*(\r\n|\r|\n))", 3)],
            vec![(1, 1), (3, 1)],
        );
        let scanner = ScannerBuilder::new()
            .add_scanner_mode(scanner_mode)
            .build()
            .unwrap();
        assert_eq!("INITIAL", scanner.inner.scanner_modes[0].name);
        let compiled_dfa = &scanner.inner.scanner_modes[0].patterns[1].0;

        compiled_dfa_render_to!(
            &compiled_dfa,
            "LineComment",
            scanner.inner.character_classes
        );
    }

    #[test]
    // Test the correct sharing of the current mode between the scanner and the scanner impl.
    fn test_scanner_current_mode() {
        init();
        let scanner_mode = ScannerMode::new(
            "INITIAL",
            vec![(r"\r\n|\r|\n", 1), (r"(//.*(\r\n|\r|\n))", 3)],
            vec![(1, 1), (3, 1)],
        );
        let mut scanner = ScannerBuilder::new()
            .add_scanner_mode(scanner_mode)
            .build()
            .unwrap();
        // At the beginning, the scanner mode is 0.
        assert_eq!(0, scanner.current_mode());
        assert_eq!(0, scanner.inner.current_mode());

        scanner.set_mode(1);
        assert_eq!(1, scanner.current_mode());
        assert_eq!(1, scanner.inner.current_mode());

        let mut find_iter = scanner.find_iter("Hello\nWorld");
        // The creation of a find_iter sets its own scanner mode to 0.
        assert_eq!(0, find_iter.current_mode());

        assert_eq!(1, scanner.current_mode());
        assert_eq!(1, scanner.inner.current_mode());
        assert_eq!(1, scanner.inner.clone().current_mode());

        find_iter.set_mode(1);
        assert_eq!(1, find_iter.current_mode());
        scanner.set_mode(0);
        assert_eq!(0, scanner.current_mode());
        assert_eq!(0, scanner.inner.current_mode());
        assert_eq!(0, scanner.inner.clone().current_mode());
    }
}
