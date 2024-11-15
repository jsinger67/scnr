use std::{fmt::Debug, path::Path};

use log::trace;

use crate::{
    internal::{ScannerImpl, ScannerNfaImpl},
    FindMatches, Match, Result, ScannerMode,
};

/// A trait to switch between scanner modes.
///
/// This trait is used to switch between different scanner modes from a parser's perspective.
/// The parser can set the current scanner mode to switch to a different set of DFAs resp. NFAs, for
/// short called Finite State Machines, FSMs.
/// Usually, the scanner mode is changed by the scanner itself based on the transitions defined
/// in the active scanner mode.
///
/// It is discouraged to use a mix of scanner induced mode changes and parser induced mode changes.
/// This can lead to unexpected behavior and is not recommended.
///
/// Only several kinds of parsers are able to handle mode changes as part of the grammar.
/// Note that 'part of the grammar' means that the mode changes are part of the parser's state
/// machine and not anything implemented in semantic actions.
///
/// For example, an LL parser is able to handle scanner mode switches in the grammar because it
/// always 'knows' the next production to parse. If the production contains a scanner mode switch,
/// the parser can switch the scanner mode before parsing the next token.
///
/// A LR parser is not able to handle mode changes in the grammar because it does not know the next
/// production to parse. The parser has to decide which production to parse based on the lookahead
/// tokens. If the lookahead tokens contain a token that needed a scanner mode switch, the parser
/// is not able to switch the scanner mode before reading the next token. This can lead to
/// unexpected behavior.
///
/// An example of a parser induced mode change is the `parol` parser generator. It is able to handle
/// mode changes in the grammar because it generates LL parsers. The parser is able to switch the
/// scanner mode before parsing the next token.
/// `parol` is also able to handle scanner induced mode changes stored as transitions in the scanner
/// modes. The scanner mode changes are then no part of the grammar but instead part of the scanner.
///
/// Furthermore `parol` can also generate LR parsers. In this case, the scanner mode changes are not
/// part of the grammar but instead part of the scanner. `parol` will prevent the LR grammar from
/// containing scanner mode changes.
///
/// See https://github.com/jsinger67/parol for more informationon about the `parol` parser generator.
pub trait ScannerModeSwitcher {
    /// Sets the current scanner mode.
    fn set_mode(&mut self, mode: usize);
    /// Returns the current scanner mode.
    fn current_mode(&self) -> usize;
    /// Returns the name of the scanner mode with the given index.
    fn mode_name(&self, index: usize) -> Option<&str>;
}

/// A internal trait for scanner implemenations.
pub(crate) trait ScannerImplTrait: ScannerModeSwitcher + Debug + Send + Sync {
    /// Returns an iterator over all non-overlapping matches.
    /// The iterator yields a [`Match`] value until no more matches could be found.
    fn find_iter<'h>(&self, input: &'h str) -> FindMatches<'h>;

    /// Resets the scanner to the initial state.
    fn reset(&mut self);

    /// Executes a leftmost search and returns the first match that is found, if one exists.
    /// It starts the search at the position of the given CharIndices iterator.
    /// During the search, all DFAs are advanced in parallel by one character at a time.
    fn find_from(&mut self, char_indices: std::str::CharIndices) -> Option<Match>;

    /// This function is used by [super::find_matches_impl::FindMatchesImpl::peek_n].
    ///
    /// Executes a leftmost search and returns the first match that is found, if one exists.
    /// It starts the search at the position of the given CharIndices iterator.
    /// In contrast to `find_from`, this method does not execute a mode switch if a transition is
    /// defined for the token type found.
    ///
    /// The name `peek_from` is used to indicate that this method is used for peeking ahead.
    /// It is called by the `peek_n` method of the `FindMatches` iterator on a copy of the
    /// `CharIndices` iterator. Thus, the original `CharIndices` iterator is not advanced.
    fn peek_from(&mut self, char_indices: std::str::CharIndices) -> Option<Match>;

    /// Returns the number of the next scanner mode if a transition is defined for the token type.
    /// If no transition is defined, None returned.
    fn has_transition(&self, token_type: usize) -> Option<usize>;

    /// Logs the compiled DFAs or NFAs as a Graphviz DOT file with the help of the `log` crate.
    /// To enable debug output compliled automaton as dot file set the environment variable
    /// `RUST_LOG` to `scnr::internal::scanner_impl=debug`.
    fn log_compiled_automata_as_dot(&self, modes: &[ScannerMode]) -> Result<()>;

    /// Generates the compiled DFAs or NFAs as a Graphviz DOT files.
    /// The DOT files are written to the target folder.
    /// The file names are derived from the scanner mode names and the index of the automaton.
    fn generate_compiled_automata_as_dot(
        &self,
        modes: &[ScannerMode],
        target_folder: &Path,
    ) -> Result<()>;

    /// Clones the scanner implementation.
    fn dyn_clone(&self) -> Box<dyn ScannerImplTrait>;
}

/// A Scanner.
/// It consists of multiple DFAs resp. NFAs that are used to search for matches.
///
/// Each DFA/NFA corresponds to a terminal symbol (token type) the lexer/scanner can recognize.
/// All these FSMs are advanced in parallel to search for matches.
/// It further constists of at least one scanner mode. Scanners support multiple scanner modes.
/// This feature is known from Flex as *Start conditions* and provides more
/// flexibility by defining several scanners for several parts of your grammar.
/// See <https://www.cs.princeton.edu/~appel/modern/c/software/flex/flex.html#SEC11>
/// for more information.
///
/// To create a scanner, you should use the `ScannerBuilder` to add scanner mode data.
/// At least one scanner mode must be added to the scanner. This single mode is usually named
/// `INITIAL`.
#[derive(Debug)]
pub struct Scanner {
    pub(crate) inner: Box<dyn ScannerImplTrait>,
}

impl Scanner {
    /// Creates a new scanner.
    /// The scanner is created with the given scanner modes.
    /// The ScannerImpl is created from the scanner modes and the use_nfa flag determines if the
    /// scanner uses DFAs or NFAs for the pattern matching.
    pub fn try_new(scanner_modes: Vec<ScannerMode>, use_nfa: bool) -> Result<Self> {
        Ok(Scanner {
            inner: if use_nfa {
                Box::new(ScannerNfaImpl::try_from(scanner_modes)?)
            } else {
                Box::new(ScannerImpl::try_from(scanner_modes)?)
            },
        })
    }

    /// Returns an iterator over all non-overlapping matches.
    /// The iterator yields a [`Match`] value until no more matches could be found.
    pub fn find_iter<'h>(&self, input: &'h str) -> FindMatches<'h> {
        self.inner.find_iter(input)
    }

    /// Logs the compiled FSMs as a Graphviz DOT file with the help of the `log` crate.
    /// To enable debug output compliled FSMs as dot file set the environment variable `RUST_LOG` to
    /// `scnr::internal::scanner_impl=debug`.
    pub fn log_compiled_automata_as_dot(&self, modes: &[ScannerMode]) -> Result<()> {
        self.inner.log_compiled_automata_as_dot(modes)
    }

    /// Generates the compiled FSMs as a Graphviz DOT files.
    /// The DOT files are written to the target folder.
    /// The file names are derived from the scanner mode names and the index of the regarding FSM.
    pub fn generate_compiled_automata_as_dot(
        &self,
        modes: &[ScannerMode],
        target_folder: &Path,
    ) -> Result<()> {
        self.inner
            .generate_compiled_automata_as_dot(modes, target_folder)
    }
}

impl ScannerModeSwitcher for Scanner {
    /// Returns the current scanner mode.
    fn current_mode(&self) -> usize {
        self.inner.current_mode()
    }

    /// Sets the current scanner mode.
    ///
    /// A parser can explicitly set the scanner mode to switch to a different set of FSMs.
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

// impl Debug for Scanner {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Scanner")
//     }
// }

#[cfg(test)]
mod tests {
    use crate::{Pattern, ScannerBuilder};

    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_scanner_builder_with_single_mode() {
        init();
        let scanner_mode = ScannerMode::new(
            "INITIAL",
            vec![
                Pattern::new(r"\r\n|\r|\n".to_string(), 1),
                Pattern::new(r"(//.*(\r\n|\r|\n))".to_string(), 3),
            ],
            vec![(1, 1), (3, 1)],
        );
        let scanner = ScannerBuilder::new()
            .add_scanner_mode(scanner_mode)
            .build()
            .unwrap();
        assert_eq!(Some("INITIAL"), scanner.inner.mode_name(0));
    }

    #[test]
    fn test_scanner_builder_nfa_with_single_mode() {
        init();
        let scanner_mode = ScannerMode::new(
            "INITIAL",
            vec![
                Pattern::new(r"\r\n|\r|\n".to_string(), 1),
                Pattern::new(r"(//.*(\r\n|\r|\n))".to_string(), 3),
            ],
            vec![(1, 1), (3, 1)],
        );
        let scanner = ScannerBuilder::new()
            .add_scanner_mode(scanner_mode)
            .use_nfa()
            .build()
            .unwrap();
        assert_eq!(Some("INITIAL"), scanner.inner.mode_name(0));
    }

    #[test]
    // Test the correct sharing of the current mode between the scanner and the scanner impl.
    fn test_scanner_current_mode() {
        init();
        let scanner_mode = ScannerMode::new(
            "INITIAL",
            vec![
                Pattern::new(r"\r\n|\r|\n".to_string(), 1),
                Pattern::new(r"(//.*(\r\n|\r|\n))".to_string(), 3),
            ],
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
        assert_eq!(1, scanner.inner.dyn_clone().current_mode());

        find_iter.set_mode(1);
        assert_eq!(1, find_iter.current_mode());
        scanner.set_mode(0);
        assert_eq!(0, scanner.current_mode());
        assert_eq!(0, scanner.inner.current_mode());
        assert_eq!(0, scanner.inner.dyn_clone().current_mode());
    }

    #[test]
    // Test the correct sharing of the current mode between the scanner and the scanner impl.
    fn test_scanner_nfa_current_mode() {
        init();
        let scanner_mode = ScannerMode::new(
            "INITIAL",
            vec![
                Pattern::new(r"\r\n|\r|\n".to_string(), 1),
                Pattern::new(r"(//.*(\r\n|\r|\n))".to_string(), 3),
            ],
            vec![(1, 1), (3, 1)],
        );
        let mut scanner = ScannerBuilder::new()
            .add_scanner_mode(scanner_mode)
            .use_nfa()
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
        assert_eq!(1, scanner.inner.dyn_clone().current_mode());

        find_iter.set_mode(1);
        assert_eq!(1, find_iter.current_mode());
        scanner.set_mode(0);
        assert_eq!(0, scanner.current_mode());
        assert_eq!(0, scanner.inner.current_mode());
        assert_eq!(0, scanner.inner.dyn_clone().current_mode());
    }
}
