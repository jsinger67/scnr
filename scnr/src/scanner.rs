#[cfg(feature = "dot_writer")]
use std::path::Path;

use std::fmt::Debug;

use log::trace;

use crate::ScannerImpl;

use crate::{FindMatches, ScannerMode};

use scnr_generate::{Result, ScannerModeSwitcher};

/// A Scanner.
/// It consists of multiple DFAs that are used to search for matches.
///
/// Each DFA corresponds to a scanner mode that can recognize the tokens that belongs to it.
/// Scanner modes are known from Flex as *Start conditions* and provides more
/// flexibility by defining several scanners for several parts of your grammar.
/// See <https://www.cs.princeton.edu/~appel/modern/c/software/flex/flex.html#SEC11>
/// for more information.
///
/// To create a scanner, you should use the `ScannerBuilder` to add scanner mode data.
/// At least one scanner mode must be added to the scanner. This single mode is usually named
/// `INITIAL`.
#[derive(Debug)]
pub struct Scanner {
    pub(crate) inner: ScannerImpl,
}

impl Scanner {
    /// Returns an iterator over all non-overlapping matches.
    /// The iterator yields a [`crate::Match`] value until no more matches could be found.
    pub fn find_iter<'h>(&self, input: &'h str) -> FindMatches<'h> {
        FindMatches::new(self.inner.clone(), input)
    }

    /// Logs the compiled FSMs as a Graphviz DOT file with the help of the `log` crate.
    /// To enable debug output compiled FSMs as dot file set the environment variable `RUST_LOG` to
    /// `scnr::scanner_impl=debug`.
    ///
    /// This is not available when the `regex_automata` feature is enabled.
    #[cfg(feature = "dot_writer")]
    pub fn log_compiled_automata_as_dot(&self) -> Result<()> {
        self.inner.log_compiled_automata_as_dot()
    }

    /// Generates the compiled FSMs as a Graphviz DOT files.
    /// The DOT files are written to the target folder.
    /// The file names are derived from the scanner mode names and the index of the regarding FSM.
    ///
    /// This is not available when the `regex_automata` feature is enabled.
    #[cfg(feature = "dot_writer")]
    pub fn generate_compiled_automata_as_dot(
        &self,
        prefix: &str,
        target_folder: &Path,
    ) -> Result<()> {
        self.inner
            .generate_compiled_automata_as_dot(prefix, target_folder)
    }

    /// Generates the code for the match function.
    /// The code is written to the target file.
    /// The code is generated in Rust and can be used to create a scanner that is able to match the
    /// input string.
    pub fn generate_match_function_code<T: AsRef<Path>>(&self, target_file: T) -> Result<()> {
        self.inner
            .generate_match_function_code(target_file.as_ref())
    }

    /// Sets the match function for the scanner.
    pub fn set_match_function(&mut self, match_function: fn(usize, char) -> bool) {
        self.inner.set_match_function(match_function)
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
    type Error = scnr_generate::ScnrError;

    fn try_from(scanner_modes: Vec<ScannerMode>) -> Result<Self> {
        Ok(Scanner {
            inner: scanner_modes.try_into()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use scnr_generate::Pattern;

    use super::*;
    use crate::ScannerBuilder;
    use std::{fs, sync::Once};

    static INIT: Once = Once::new();

    const TARGET_FOLDER: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../target/testout/test_pathological_regular_expressions_dfa"
    );

    fn init() {
        INIT.call_once(|| {
            let _ = env_logger::builder().is_test(true).try_init();
            // Delete all previously generated dot files.
            let _ = fs::remove_dir_all(TARGET_FOLDER);
            // Create the target folder.
            fs::create_dir_all(TARGET_FOLDER).unwrap();
        });
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

    // A test that checks the behavior of the scanner when so called 'pathological regular expressions'
    // are used. These are regular expressions that are very slow to match.
    // The test checks if the scanner is able to handle these cases and does not hang.
    #[cfg(feature = "dot_writer")]
    struct TestData {
        pattern: &'static str,
        input: &'static str,
        expected_match: Option<&'static str>,
    }

    #[cfg(feature = "dot_writer")]
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
        // This test is disabled because it takes too long to run.
        // It works fine when the DFA is not optimized.
        // This should be analyzed thoroughly.
        // TestData {
        //     pattern: r"a{5}{5}{5}{5}{5}{5}*b",
        //     input: "aaaaaaaaaaaaaaaaaaaaaaaaaaab",
        //     expected_match: Some("b"),
        // },
    ];

    #[cfg(feature = "dot_writer")]
    #[test]
    fn test_pathological_regular_expressions_dfa() {
        init();

        #[allow(unused_variables)]
        for (index, test) in TEST_DATA.iter().enumerate() {
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
                .generate_compiled_automata_as_dot(
                    &format!("Test{}", index),
                    Path::new(&TARGET_FOLDER),
                )
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
