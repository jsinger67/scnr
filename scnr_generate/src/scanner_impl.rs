use std::{io::Write, sync::Arc};

use log::trace;

use crate::{Match, Result, ScannerMode, ScannerModeSwitcher};

use super::{compiled_scanner_mode::CompiledScannerMode, CharacterClassRegistry, TerminalIDBase};

/// ScannerImpl instances are always created by `TryFrom<Vec<ScannerMode>>` or
/// `TryFrom<&[ScannerMode]>` and of course by the clone method.
#[derive(Clone)]
pub struct ScannerImpl {
    pub(crate) character_classes: Arc<CharacterClassRegistry>,
    pub(crate) scanner_modes: Vec<CompiledScannerMode>,
    // The function used to match characters against character classes.
    pub(crate) match_char_class: Arc<dyn (Fn(usize, char) -> bool) + 'static + Send + Sync>,
    // The current mode is private and thereby makes the free creation of ScannerImpl instances
    // impossible.
    current_mode: usize,
}
impl ScannerImpl {
    /// Executes a possible mode switch if a transition is defined for the token type found.
    #[inline]
    fn execute_possible_mode_switch(&mut self, current_match: &Match) {
        let current_mode = &self.scanner_modes[self.current_mode];
        // We perform a scanner mode switch if a transition is defined for the token type found.
        if let Some(next_mode) = current_mode.has_transition(current_match.token_type()) {
            trace!(
                "Switching from mode {} to mode {}",
                self.current_mode,
                next_mode
            );
            self.current_mode = next_mode;
        }
    }

    /// Creates a function that matches a character against a character class.
    /// Used in tests only.
    #[allow(dead_code)]
    pub(crate) fn create_match_char_class(
        &self,
    ) -> Result<Box<dyn (Fn(usize, char) -> bool) + 'static + Send + Sync>> {
        self.character_classes.create_match_char_class()
    }

    pub(crate) fn reset(&mut self) {
        self.current_mode = 0;
    }

    /// Executes a leftmost search and returns the first match that is found, if one exists.
    /// It starts the search at the position of the given CharIndices iterator.
    pub(crate) fn find_from(
        &mut self,
        input: &str,
        char_indices: std::str::CharIndices,
    ) -> Option<crate::Match> {
        if let Some(matched) = self.peek_from(input, char_indices) {
            self.execute_possible_mode_switch(&matched);
            return Some(matched);
        }
        None
    }

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
    pub(crate) fn peek_from(
        &self,
        input: &str,
        char_indices: std::str::CharIndices,
    ) -> Option<crate::Match> {
        let dfa = &self.scanner_modes[self.current_mode].dfa;

        if let Some(matched) = dfa.find_from(input, char_indices, &*self.match_char_class) {
            debug_assert!(
                !matched.is_empty(),
                r#"
    An empty token was matched. This leads to an infinite loop.
    It is therefore necessary to avoid regexes that can match empty tokens.
    Please, check regex '{}' for token type {} in scanner mode {}"#,
                dfa.pattern((matched.token_type() as TerminalIDBase).into())
                    .escape_default(),
                matched.token_type(),
                self.current_mode
            );
            return Some(matched);
        }
        None
    }

    pub(crate) fn has_transition(&self, token_type: usize) -> Option<usize> {
        self.scanner_modes[self.current_mode].has_transition(token_type)
    }

    /// Traces the compiled DFAs as dot format.
    /// The output is written to the log.
    /// This function is used for debugging purposes.
    #[cfg(feature = "dot_writer")]
    pub fn log_compiled_automata_as_dot(&self) -> crate::Result<()> {
        use log::debug;
        use std::io::Read;

        for (i, scanner_mode) in self.scanner_modes.iter().enumerate() {
            debug!("Compiled DFA: Mode {} \n{}", i, {
                let mut cursor = std::io::Cursor::new(Vec::new());
                let title = format!("Compiled DFA {}", scanner_mode.name);
                super::dot::compiled_dfa_render(
                    &scanner_mode.dfa,
                    &title,
                    &self.character_classes,
                    &mut cursor,
                );
                let mut dot_format = String::new();
                cursor.set_position(0);
                cursor.read_to_string(&mut dot_format)?;
                dot_format
            });
        }
        Ok(())
    }

    /// Generates the compiled DFAs as dot files.
    /// The dot files are written to the target folder.
    #[cfg(feature = "dot_writer")]
    pub fn generate_compiled_automata_as_dot(
        &self,
        prefix: &str,
        target_folder: &std::path::Path,
    ) -> crate::Result<()> {
        use std::fs::File;
        for scanner_mode in self.scanner_modes.iter() {
            let title = format!("Compiled DFA {}", scanner_mode.name);
            let file_name = format!(
                "{}/{}_{}.dot",
                target_folder.to_str().unwrap(),
                prefix,
                scanner_mode.name
            );
            let mut file = File::create(file_name)?;
            super::dot::compiled_dfa_render(
                &scanner_mode.dfa,
                &title,
                &self.character_classes,
                &mut file,
            );
        }
        Ok(())
    }

    pub fn generate_match_function_code(&self, out_file: &std::path::Path) -> Result<()> {
        // TODO: Make the function name configurable, e.g. as a parameter to the function.
        {
            let mut file = std::fs::File::create(out_file)?;
            let code = self
                .character_classes
                .generate("match_function")
                .to_string();
            file.write_all(code.as_bytes())?;
            file.write_all(b"\n")?;
        }
        crate::rust_code_formatter::try_format(out_file)?;
        Ok(())
    }

    pub fn set_match_function(&mut self, match_function: fn(usize, char) -> bool) {
        self.match_char_class = Arc::new(match_function);
    }
}

impl ScannerModeSwitcher for ScannerImpl {
    fn mode_name(&self, index: usize) -> Option<&str> {
        self.scanner_modes.get(index).map(|mode| mode.name.as_str())
    }

    #[inline]
    fn current_mode(&self) -> usize {
        self.current_mode
    }

    #[inline]
    fn set_mode(&mut self, mode: usize) {
        self.current_mode = mode;
    }
}

impl std::fmt::Debug for ScannerImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScannerImpl")
            .field("character_classes", &self.character_classes)
            .field("scanner_modes", &self.scanner_modes)
            .finish()
    }
}

impl TryFrom<Vec<ScannerMode>> for ScannerImpl {
    type Error = crate::ScnrError;
    fn try_from(scanner_modes: Vec<ScannerMode>) -> Result<Self> {
        let mut character_class_registry = CharacterClassRegistry::new();
        let mut compiled_scanner_modes = Vec::with_capacity(scanner_modes.len());
        for scanner_mode in scanner_modes {
            let compiled_scanner_mode = CompiledScannerMode::try_from_scanner_mode(
                scanner_mode,
                &mut character_class_registry,
            )?;
            compiled_scanner_modes.push(compiled_scanner_mode);
        }
        let match_char_class = Arc::new(character_class_registry.create_match_char_class()?);
        Ok(Self {
            character_classes: Arc::new(character_class_registry),
            scanner_modes: compiled_scanner_modes,
            match_char_class,
            current_mode: 0,
        })
    }
}

impl TryFrom<&[ScannerMode]> for ScannerImpl {
    type Error = crate::ScnrError;
    fn try_from(scanner_modes: &[ScannerMode]) -> Result<Self> {
        let mut character_class_registry = CharacterClassRegistry::new();
        let mut compiled_scanner_modes = Vec::with_capacity(scanner_modes.len());
        for scanner_mode in scanner_modes {
            let compiled_scanner_mode = CompiledScannerMode::try_from_scanner_mode(
                scanner_mode.clone(),
                &mut character_class_registry,
            )?;
            compiled_scanner_modes.push(compiled_scanner_mode);
        }
        let match_char_class = Arc::new(character_class_registry.create_match_char_class()?);
        Ok(Self {
            character_classes: Arc::new(character_class_registry),
            scanner_modes: compiled_scanner_modes,
            match_char_class,
            current_mode: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Pattern, ScannerMode};
    use std::{convert::TryInto, fs, sync::Once};

    static INIT: Once = Once::new();

    const TARGET_FOLDER: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../target/testout/scanner_nfa_impl_test"
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
    fn test_try_from() {
        init();
        let scanner_modes = vec![
            ScannerMode::new("mode1", vec![Pattern::new("a".to_string(), 0)], vec![]),
            ScannerMode::new("mode2", vec![Pattern::new("b".to_string(), 1)], vec![]),
        ];
        let scanner_impl: ScannerImpl = scanner_modes.try_into().unwrap();
        // assert_eq!(scanner_impl.character_classes.len(), 2);
        assert_eq!(scanner_impl.scanner_modes.len(), 2);
    }

    #[test]
    fn test_match_char_class() {
        init();
        let scanner_modes = vec![
            ScannerMode::new("mode1", vec![Pattern::new("a".to_string(), 0)], vec![]),
            ScannerMode::new("mode2", vec![Pattern::new("b".to_string(), 1)], vec![]),
        ];
        let scanner_impl: ScannerImpl = scanner_modes.try_into().unwrap();
        let match_char_class = scanner_impl.create_match_char_class().unwrap();
        assert!(match_char_class(0, 'a'));
        assert!(!match_char_class(0, 'b'));
        assert!(!match_char_class(0, 'c'));
        assert!(!match_char_class(1, 'a'));
        assert!(match_char_class(1, 'b'));
        assert!(!match_char_class(1, 'c'));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_generate_dot_files() {
        init();
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "../../scnr/tests/data/string.json"
        );
        let file = fs::File::open(path).unwrap();

        let scanner_modes: Vec<ScannerMode> = serde_json::from_reader(file)
            .unwrap_or_else(|e| panic!("**** Failed to read json file {path}: {e}"));

        let scanner_impl: ScannerImpl = scanner_modes.clone().try_into().unwrap();

        // Generate the compiled NFAs as dot files.
        scanner_impl
            .generate_compiled_automata_as_dot("String", std::path::Path::new(TARGET_FOLDER))
            .unwrap();

        // Check if the dot files are generated.
        let dot_files: Vec<_> = fs::read_dir(TARGET_FOLDER)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();

        assert_eq!(dot_files.len(), 2);
        assert_eq!(
            dot_files
                .iter()
                .filter(|p| p.extension().unwrap() == "dot")
                .count(),
            2
        );
        assert_eq!(
            dot_files
                .iter()
                .filter(|p| p.file_stem().unwrap().to_str().unwrap().contains("INITIAL"))
                .count(),
            1
        );
        assert_eq!(
            dot_files
                .iter()
                .filter(|p| p.file_stem().unwrap().to_str().unwrap().contains("STRING"))
                .count(),
            1
        );
    }
}
