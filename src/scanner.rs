use std::{fmt::Debug, path::Path};

use log::trace;

use crate::{internal::ScannerNfaImpl, FindMatches, Result, ScannerMode};

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
/// the parser can switch the scanner mode before scanning the next token.
///
/// A LR parser is not able to handle mode changes in the grammar because it does not know the next
/// production to parse. The parser has to decide which production to parse based on the lookahead
/// tokens. If the lookahead tokens contain a token that needed a scanner mode switch, the parser
/// is not able to switch the scanner mode before reading the next token.
///
/// An example of a parser induced mode changes is the `parol` parser generator. It is able to
/// handle mode changes in the grammar because it generates LL parsers. The parser is able to switch
/// the scanner mode before scanning the next token.
/// `parol` is also able to handle scanner induced mode changes stored as transitions in the scanner
/// modes. The scanner mode changes are then no part of the grammar but instead part of the scanner.
///
/// Furthermore `parol` can also generate LR parsers. In this case, the scanner mode changes are not
/// part of the grammar but instead part of the scanner. `parol` will prevent the LR grammar from
/// containing scanner mode changes.
///
/// See <https://github.com/jsinger67/parol> for more informationon about the `parol` parser
/// generator.
pub trait ScannerModeSwitcher {
    /// Sets the current scanner mode.
    fn set_mode(&mut self, mode: usize);
    /// Returns the current scanner mode.
    fn current_mode(&self) -> usize;
    /// Returns the name of the scanner mode with the given index.
    fn mode_name(&self, index: usize) -> Option<&str>;
}

/// A Scanner.
/// It consists of multiple DFAs resp. NFAs that are used to search for matches.
///
/// Each NFA corresponds to a terminal symbol (token type) the lexer/scanner can recognize.
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
    pub(crate) inner: ScannerNfaImpl,
}

impl Scanner {
    /// Returns an iterator over all non-overlapping matches.
    /// The iterator yields a [`Match`] value until no more matches could be found.
    pub fn find_iter<'h>(&self, input: &'h str) -> FindMatches<'h> {
        FindMatches::new(self.inner.clone(), input)
    }

    /// Logs the compiled FSMs as a Graphviz DOT file with the help of the `log` crate.
    /// To enable debug output compliled FSMs as dot file set the environment variable `RUST_LOG` to
    /// `scnr::internal::scanner_impl=debug`.
    pub fn log_compiled_automata_as_dot(&self) -> Result<()> {
        self.inner.log_compiled_automata_as_dot()
    }

    /// Generates the compiled FSMs as a Graphviz DOT files.
    /// The DOT files are written to the target folder.
    /// The file names are derived from the scanner mode names and the index of the regarding FSM.
    pub fn generate_compiled_automata_as_dot(&self, target_folder: &Path) -> Result<()> {
        self.inner.generate_compiled_automata_as_dot(target_folder)
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

/// A scanner can be created from a vector of scanner modes.
impl TryFrom<Vec<ScannerMode>> for Scanner {
    type Error = crate::ScnrError;

    fn try_from(scanner_modes: Vec<ScannerMode>) -> Result<Self> {
        Ok(Scanner {
            inner: scanner_modes.try_into()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

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
        assert_eq!(1, scanner.inner.clone().current_mode());

        find_iter.set_mode(1);
        assert_eq!(1, find_iter.current_mode());
        scanner.set_mode(0);
        assert_eq!(0, scanner.current_mode());
        assert_eq!(0, scanner.inner.current_mode());
        assert_eq!(0, scanner.inner.clone().current_mode());
    }

    // A test that checks the behavoir of the scanner when so called 'pathological regular expressions'
    // are used. These are regular expressions that are very slow to match.
    // The test checks if the scanner is able to handle these cases and does not hang.
    struct TestData {
        pattern: &'static str,
        input: &'static str,
        expected_match: Option<&'static str>,
    }

    const TEST_DATA: &[TestData] = &[
        TestData {
            pattern: r"((a*)*b)",
            input: "aaaaaaaaaaaaaaaaaaaaaaaaaab",
            expected_match: Some("aaaaaaaaaaaaaaaaaaaaaaaaaab"),
        },
        TestData {
            pattern: r"(a+)+b",
            input: "aaaaaaaaaaaaaaaaaaaaaaaaaab",
            expected_match: Some("aaaaaaaaaaaaaaaaaaaaaaaaaab"),
        },
        TestData {
            pattern: r"(a+)+b",
            input: "aaaaaaaaaaaaaaaaaaaaaaaaaa",
            expected_match: None,
        },
        TestData {
            pattern: r"(a|aa)+b",
            input: "aaaaaaaaaaaaaaaaaaaaaaaaaab",
            expected_match: Some("aaaaaaaaaaaaaaaaaaaaaaaaaab"),
        },
        TestData {
            pattern: r"(a|a?)+b",
            input: "aaaaaaaaaaaaaaaaaaaaaaaaaab",
            expected_match: Some("aaaaaaaaaaaaaaaaaaaaaaaaaab"),
        },
        TestData {
            pattern: r"((a|aa|aaa|aaaa|aaaaa)*)*b",
            input: "aaaaaaaaaaaaaaaaaaaaaaaaaab",
            expected_match: Some("aaaaaaaaaaaaaaaaaaaaaaaaaab"),
        },
        TestData {
            pattern: r"((a*a*a*a*a*a*)*)*b",
            input: "aaaaaaaaaaaaaaaaaaaaaaaaaab",
            expected_match: Some("aaaaaaaaaaaaaaaaaaaaaaaaaab"),
        },
        TestData {
            pattern: r"a{3}{3}*b",
            input: "aaaaaaaaaaaaaaaaaaaaaaaaaaab",
            expected_match: Some("aaaaaaaaaaaaaaaaaaaaaaaaaaab"),
        },
        // The following test case is only applicable for the NFA.
        // The NFA is able to handle this case but needs a view seconds to compile the automaton.
        TestData {
            pattern: r"a{5}{5}{5}{5}{5}{5}*b",
            input: "aaaaaaaaaaaaaaaaaaaaaaaaaaab",
            expected_match: Some("b"),
        },
    ];

    #[test]
    fn test_pathological_regular_expressions_dfa() {
        init();

        for (index, test) in TEST_DATA.iter().enumerate() {
            let target_folder = format!(
                "{}_{}",
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/target/test_pathological_regular_expressions_dfa"
                ),
                index
            );
            // Delete all previously generated dot files.
            let _ = fs::remove_dir_all(target_folder.clone());
            // Create the target folder.
            fs::create_dir_all(target_folder.clone()).unwrap();

            let scanner_mode = ScannerMode::new(
                "INITIAL",
                vec![Pattern::new(test.pattern.to_string(), 1)],
                vec![],
            );
            let scanner = ScannerBuilder::new()
                .add_scanner_mode(scanner_mode.clone())
                .build()
                .unwrap();

            scanner
                .generate_compiled_automata_as_dot(Path::new(&target_folder))
                .unwrap();

            let mut find_iter = scanner.find_iter(test.input);
            let match1 = find_iter.next();
            assert_eq!(
                test.expected_match,
                match1.map(|m| test.input.get(m.start()..m.end()).unwrap())
            );
        }
    }
}
