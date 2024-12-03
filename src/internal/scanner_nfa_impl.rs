use std::sync::Arc;

use log::{debug, trace};

use crate::{FindMatches, Match, Result, ScannerMode, ScannerModeSwitcher};

use super::{compiled_scanner_mode::CompiledScannerMode, CharClassID, CharacterClassRegistry};

/// ScannerNfaImpl instances are always created by the Scanner::try_new method and of course by
/// the clone method.
#[derive(Clone)]
pub(crate) struct ScannerNfaImpl {
    pub(crate) character_classes: CharacterClassRegistry,
    pub(crate) scanner_modes: Vec<CompiledScannerMode>,
    // The function used to match characters against character classes.
    pub(crate) match_char_class: Arc<dyn (Fn(CharClassID, char) -> bool) + 'static + Send + Sync>,
    // The current mode is private and thereby makes the free creation of ScannerNfaImpl instances
    // impossible.
    current_mode: usize,
}
impl ScannerNfaImpl {
    /// We evaluate the matches of the NFAs in ascending order to prioritize the matches with the
    /// lowest index.
    /// We find the longest match with the lowest index in the given *matches* vector.
    fn find_first_longest_match(&self, matches: Vec<Match>) -> Option<Match> {
        let mut current_match: Option<Match> = None;
        for m in matches {
            if let Some(current) = current_match.as_ref() {
                if m.span().len() > current.span().len() {
                    current_match = Some(m);
                }
            } else {
                current_match = Some(m);
            }
        }
        trace!("Current match: {:?}", current_match);
        current_match
    }

    /// Executes a possible mode switch if a transition is defined for the token type found.
    #[inline]
    fn execute_possible_mode_switch(&mut self, current_match: &Match) {
        let current_mode = &self.scanner_modes[self.current_mode];
        // We perform a scanner mode switch if a transition is defined for the token type found.
        if let Some(next_mode) = current_mode.has_transition(current_match.token_type()) {
            self.current_mode = next_mode;
        }
    }

    pub(crate) fn create_match_char_class(
        &self,
    ) -> Result<Box<dyn (Fn(CharClassID, char) -> bool) + 'static + Send + Sync>> {
        self.character_classes.create_match_char_class()
    }

    pub(crate) fn find_iter<'h>(&self, input: &'h str) -> crate::FindMatches<'h> {
        FindMatches::new(self.clone(), input)
    }

    pub(crate) fn reset(&mut self) {
        self.current_mode = 0;
    }

    /// Executes a leftmost search and returns the first match that is found, if one exists.
    /// It starts the search at the position of the given CharIndices iterator.
    /// During the search, all NFAs are tested in parallel.
    pub(crate) fn find_from(
        &mut self,
        char_indices: std::str::CharIndices,
    ) -> Option<crate::Match> {
        let patterns = &mut self.scanner_modes[self.current_mode].nfas;

        let cloned_char_indices = char_indices.clone();
        let mut matches = Vec::with_capacity(patterns.len());
        for (nfa, terminal_id) in patterns {
            // All NFAs are tested in parallel against the input.
            // We clone the char_indices iterator for each NFA.
            if let Some(span) = nfa.find_from(cloned_char_indices.clone(), &*self.match_char_class)
            {
                let mut iter = char_indices.clone();
                for _ in 0..span.len() {
                    iter.next();
                }
                if let Some(mut lookahead) = nfa.lookahead.clone() {
                    if !lookahead.satisfies_lookahead(iter, &*self.match_char_class) {
                        continue;
                    }
                }
                if span.is_empty() {
                    panic!(
                        r#"
    An empty token was matched. This leads to an infinite loop.
    Avoid regexes that match empty tokens.
    Please, check regex {} for token type {}"#,
                        nfa.pattern.escape_default(),
                        terminal_id
                    );
                }
                matches.push(Match::new(terminal_id.as_usize(), span));
            }
        }
        let current_match = self.find_first_longest_match(matches);
        if let Some(m) = current_match.as_ref() {
            self.execute_possible_mode_switch(m);
        }
        current_match
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
        char_indices: std::str::CharIndices,
    ) -> Option<crate::Match> {
        let patterns = &mut self.scanner_modes[self.current_mode].nfas;

        let cloned_char_indices = char_indices.clone();
        let mut matches = Vec::with_capacity(patterns.len());
        for (nfa, terminal_id) in patterns {
            if let Some(span) = nfa.find_from(cloned_char_indices.clone(), &*self.match_char_class)
            {
                let mut iter = char_indices.clone();
                for _ in 0..span.len() {
                    iter.next();
                }
                if let Some(mut lookahead) = nfa.lookahead.clone() {
                    if !lookahead.satisfies_lookahead(iter, &*self.match_char_class) {
                        continue;
                    }
                }
                matches.push(Match::new(terminal_id.as_usize(), span));
            }
        }
        self.find_first_longest_match(matches)
    }

    pub(crate) fn has_transition(&self, token_type: usize) -> Option<usize> {
        self.scanner_modes[self.current_mode].has_transition(token_type)
    }

    /// Traces the compiled NFAs as dot format.
    /// The output is written to the log.
    /// This function is used for debugging purposes.
    pub(crate) fn log_compiled_automata_as_dot(
        &self,
        modes: &[crate::ScannerMode],
    ) -> crate::Result<()> {
        use std::io::Read;
        for (i, scanner_mode) in self.scanner_modes.iter().enumerate() {
            for (j, (dfa, t)) in scanner_mode.nfas.iter().enumerate() {
                debug!("Compiled NFA: Mode {} Pattern {} Token {}\n{}", i, j, t, {
                    let mut cursor = std::io::Cursor::new(Vec::new());
                    let title = format!("Compiled NFA {}::{}", modes[i].name, modes[i].patterns[j]);
                    super::dot::compiled_nfa_render(
                        dfa,
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
        }
        Ok(())
    }

    /// Generates the compiled NFAs as dot files.
    /// The dot files are written to the target folder.
    pub(crate) fn generate_compiled_automata_as_dot(
        &self,
        modes: &[crate::ScannerMode],
        target_folder: &std::path::Path,
    ) -> crate::Result<()> {
        use std::fs::File;
        for (i, scanner_mode) in self.scanner_modes.iter().enumerate() {
            for (j, (nfa, t)) in scanner_mode.nfas.iter().enumerate() {
                let title = format!("Compiled NFA {} - {}", modes[i].name, modes[i].patterns[j]);
                let file_name = format!(
                    "{}/{}_{}_{}.dot",
                    target_folder.to_str().unwrap(),
                    modes[i].name,
                    j,
                    t
                );
                let mut file = File::create(file_name)?;
                super::dot::compiled_nfa_render(nfa, &title, &self.character_classes, &mut file);
            }
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
        let mut me = Self {
            character_classes: character_class_registry,
            scanner_modes: compiled_scanner_modes,
            match_char_class: Arc::new(|_, _| false),
            current_mode: 0,
        };
        me.match_char_class = Arc::new(Self::create_match_char_class(&me)?);
        Ok(me)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Pattern, ScannerMode};
    use std::{convert::TryInto, fs, path::Path};

    #[test]
    fn test_try_from() {
        let scanner_modes = vec![
            ScannerMode::new("mode1", vec![Pattern::new("a".to_string(), 0)], vec![]),
            ScannerMode::new("mode2", vec![Pattern::new("b".to_string(), 1)], vec![]),
        ];
        let scanner_impl: ScannerNfaImpl = scanner_modes.try_into().unwrap();
        assert_eq!(scanner_impl.character_classes.len(), 2);
        assert_eq!(scanner_impl.scanner_modes.len(), 2);
    }

    #[test]
    fn test_match_char_class() {
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
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/string.json");
        let file = fs::File::open(path).unwrap();

        let scanner_modes: Vec<ScannerMode> = serde_json::from_reader(file)
            .unwrap_or_else(|e| panic!("**** Failed to read json file {path}: {e}"));

        let scanner_impl: ScannerNfaImpl = scanner_modes.clone().try_into().unwrap();
        let target_folder = concat!(env!("CARGO_MANIFEST_DIR"), "/target/string_nfas");

        // Delete all previously generated dot files.
        let _ = fs::remove_dir_all(target_folder);
        // Create the target folder.
        fs::create_dir_all(target_folder).unwrap();

        // Generate the compiled NFAs as dot files.
        scanner_impl
            .generate_compiled_automata_as_dot(&scanner_modes, Path::new(target_folder))
            .unwrap();

        // Check if the dot files are generated.
        let dot_files: Vec<_> = fs::read_dir(target_folder)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();

        assert_eq!(dot_files.len(), 12);
        assert_eq!(
            dot_files
                .iter()
                .filter(|p| p.extension().unwrap() == "dot")
                .count(),
            12
        );
        assert_eq!(
            dot_files
                .iter()
                .filter(|p| p.file_stem().unwrap().to_str().unwrap().contains("INITIAL"))
                .count(),
            7
        );
        assert_eq!(
            dot_files
                .iter()
                .filter(|p| p.file_stem().unwrap().to_str().unwrap().contains("STRING"))
                .count(),
            5
        );
    }
}
