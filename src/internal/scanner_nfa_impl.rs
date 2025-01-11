use std::sync::Arc;

use log::{debug, trace};

use crate::{Match, Result, ScannerMode, ScannerModeSwitcher};

use super::{
    compiled_scanner_mode::CompiledScannerMode, CharClassID, CharacterClassRegistry, TerminalIDBase,
};

/// ScannerNfaImpl instances are always created by the Scanner::try_new method and of course by
/// the clone method.
#[derive(Clone)]
pub(crate) struct ScannerNfaImpl {
    pub(crate) character_classes: Arc<CharacterClassRegistry>,
    pub(crate) scanner_modes: Vec<CompiledScannerMode>,
    // The function used to match characters against character classes.
    pub(crate) match_char_class: Arc<dyn (Fn(CharClassID, char) -> bool) + 'static + Send + Sync>,
    // The current mode is private and thereby makes the free creation of ScannerNfaImpl instances
    // impossible.
    current_mode: usize,
}
impl ScannerNfaImpl {
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
    ) -> Result<Box<dyn (Fn(CharClassID, char) -> bool) + 'static + Send + Sync>> {
        self.character_classes.create_match_char_class()
    }

    pub(crate) fn reset(&mut self) {
        self.current_mode = 0;
    }

    /// Executes a leftmost search and returns the first match that is found, if one exists.
    /// It starts the search at the position of the given CharIndices iterator.
    /// During the search, all NFAs are tested in parallel.
    pub(crate) fn find_from(
        &mut self,
        input: &str,
        char_indices: std::str::CharIndices,
    ) -> Option<crate::Match> {
        if let Some(matched) = self.peek_from(input, char_indices.clone()) {
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
        &mut self,
        input: &str,
        char_indices: std::str::CharIndices,
    ) -> Option<crate::Match> {
        let nfa = &mut self.scanner_modes[self.current_mode].nfa;

        let cloned_char_indices = char_indices.clone();
        // We clone the char_indices iterator for each NFA.
        if let Some(matched) =
            nfa.find_from(input, cloned_char_indices.clone(), &*self.match_char_class)
        {
            let mut iter = char_indices.clone();
            for _ in 0..matched.len() {
                iter.next();
            }
            if matched.is_empty() {
                panic!(
                    r#"
    An empty token was matched. This leads to an infinite loop.
    Avoid regexes that match empty tokens.
    Please, check regex {} for token type {}"#,
                    nfa.pattern((matched.token_type() as TerminalIDBase).into())
                        .escape_default(),
                    matched.token_type()
                );
            }
            return Some(matched);
        }
        None
    }

    pub(crate) fn has_transition(&self, token_type: usize) -> Option<usize> {
        self.scanner_modes[self.current_mode].has_transition(token_type)
    }

    /// Traces the compiled NFAs as dot format.
    /// The output is written to the log.
    /// This function is used for debugging purposes.
    pub(crate) fn log_compiled_automata_as_dot(&self) -> crate::Result<()> {
        use std::io::Read;
        for (i, scanner_mode) in self.scanner_modes.iter().enumerate() {
            debug!("Compiled NFA: Mode {} \n{}", i, {
                let mut cursor = std::io::Cursor::new(Vec::new());
                let title = format!("Compiled NFA {}", scanner_mode.name);
                super::dot::compiled_nfa_render(
                    &scanner_mode.nfa,
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

    /// Generates the compiled NFAs as dot files.
    /// The dot files are written to the target folder.
    pub(crate) fn generate_compiled_automata_as_dot(
        &self,
        prefix: &str,
        target_folder: &std::path::Path,
    ) -> crate::Result<()> {
        use std::fs::File;
        for scanner_mode in self.scanner_modes.iter() {
            let title = format!("Compiled NFA {}", scanner_mode.name);
            let file_name = format!(
                "{}/{}_{}.dot",
                target_folder.to_str().unwrap(),
                prefix,
                scanner_mode.name
            );
            let mut file = File::create(file_name)?;
            super::dot::compiled_nfa_render(
                &scanner_mode.nfa,
                &title,
                &self.character_classes,
                &mut file,
            );
        }
        Ok(())
    }
}

impl ScannerModeSwitcher for ScannerNfaImpl {
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

impl std::fmt::Debug for ScannerNfaImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScannerNfaImpl")
            .field("character_classes", &self.character_classes)
            .field("scanner_modes", &self.scanner_modes)
            .finish()
    }
}

impl TryFrom<Vec<ScannerMode>> for ScannerNfaImpl {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Pattern, ScannerMode};
    use std::{convert::TryInto, fs, path::Path, sync::Once};

    static INIT: Once = Once::new();

    const TARGET_FOLDER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/target/testout/string_nfas");

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
        let scanner_impl: ScannerNfaImpl = scanner_modes.try_into().unwrap();
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
        let scanner_impl: ScannerNfaImpl = scanner_modes.try_into().unwrap();
        let match_char_class = scanner_impl.create_match_char_class().unwrap();
        assert!(match_char_class((0).into(), 'a'));
        assert!(!match_char_class((0).into(), 'b'));
        assert!(!match_char_class((0).into(), 'c'));
        assert!(!match_char_class((1).into(), 'a'));
        assert!(match_char_class((1).into(), 'b'));
        assert!(!match_char_class((1).into(), 'c'));
    }

    #[test]
    fn test_generate_dot_files() {
        init();
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/string.json");
        let file = fs::File::open(path).unwrap();

        let scanner_modes: Vec<ScannerMode> = serde_json::from_reader(file)
            .unwrap_or_else(|e| panic!("**** Failed to read json file {path}: {e}"));

        let scanner_impl: ScannerNfaImpl = scanner_modes.clone().try_into().unwrap();

        // Generate the compiled NFAs as dot files.
        scanner_impl
            .generate_compiled_automata_as_dot("String", Path::new(TARGET_FOLDER))
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
