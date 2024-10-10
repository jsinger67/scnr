use std::sync::Arc;

use log::{debug, trace};

use crate::{FindMatches, Match, Result, ScannerMode, ScnrError};

use super::{CharClassID, CharacterClassRegistry, CompiledScannerMode, MatchFunction};

#[derive(Clone)]
pub(crate) struct ScannerImpl {
    pub(crate) character_classes: CharacterClassRegistry,
    pub(crate) scanner_modes: Vec<CompiledScannerMode>,
    // The function used to match characters to character classes.
    pub(crate) match_char_class: Arc<dyn (Fn(CharClassID, char) -> bool) + 'static + Send + Sync>,
    // The current mode is private and thereby makes the free creation of ScannerImpl instances
    // impossible.
    // ScannerImpl instances are always created by the Scanner::try_new method and of course by
    // the clone method. So the current mode is always shared between all ScannerImpl instances of
    // the same Scanner instance.
    current_mode: usize,
}

impl ScannerImpl {
    /// Creates a new scanner implementation from the given scanner modes.

    /// Returns an iterator over all non-overlapping matches.
    /// The iterator yields a [`Match`] value until no more matches could be found.
    pub(crate) fn find_iter(scanner_impl: Self, input: &str) -> FindMatches<'_> {
        FindMatches::new(scanner_impl, input)
    }

    pub(crate) fn create_match_char_class(
        &self,
    ) -> Result<Box<dyn (Fn(CharClassID, char) -> bool) + 'static + Send + Sync>> {
        let match_functions =
            self.character_classes
                .iter()
                .try_fold(Vec::new(), |mut acc, cc| {
                    trace!("Create match function for char class {:?}", cc);
                    let match_function: MatchFunction = cc.ast().try_into()?;
                    acc.push(match_function);
                    Ok::<Vec<MatchFunction>, ScnrError>(acc)
                })?;
        Ok(Box::new(move |char_class, c| {
            let res = match_functions[char_class.as_usize()].call(c);
            if res {
                trace!("Match char class: {:?} {:?} -> {:?}", char_class, c, res);
            }
            res
        }))
    }

    /// Executes a leftmost search and returns the first match that is found, if one exists.
    /// It starts the search at the position of the given CharIndices iterator.
    /// During the search, all DFAs are advanced in parallel by one character at a time.
    pub(crate) fn find_from(&mut self, char_indices: std::str::CharIndices) -> Option<Match> {
        let dfa = &mut self.scanner_modes[self.current_mode].dfa;
        let ma = dfa.find_match(char_indices, &*self.match_char_class);
        if let Some(ref m) = ma {
            self.execute_possible_mode_switch(m);
        }
        ma
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
    pub(crate) fn peek_from(&mut self, char_indices: std::str::CharIndices) -> Option<Match> {
        let dfa = &mut self.scanner_modes[self.current_mode].dfa;
        dfa.find_match(char_indices, &*self.match_char_class)
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

    /// Returns the number of the next scanner mode if a transition is defined for the token type.
    /// If no transition is defined, None returned.
    pub(crate) fn has_transition(&self, token_type: usize) -> Option<usize> {
        self.scanner_modes[self.current_mode].has_transition(token_type)
    }

    /// Returns the name of the scanner mode with the given index.
    /// If the index is out of bounds, None is returned.
    pub(crate) fn mode_name(&self, index: usize) -> Option<&str> {
        self.scanner_modes.get(index).map(|mode| mode.name.as_str())
    }

    /// Returns the current scanner mode. Used for tests and debugging purposes.
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn current_mode(&self) -> usize {
        self.current_mode
    }

    /// Traces the compiled DFAs as dot format.
    /// The output is written to the log.
    /// This function is used for debugging purposes.
    pub(crate) fn log_compiled_dfas_as_dot(&self, modes: &[ScannerMode]) -> Result<()> {
        use std::io::Read;
        for (i, scanner_mode) in self.scanner_modes.iter().enumerate() {
            debug!("Compiled DFA: Mode {}\n{}", i, {
                let mut cursor = std::io::Cursor::new(Vec::new());
                let title = format!("Compiled DFA {}", modes[i].name);
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

    /// Generates the compiled DFAs as a Graphviz DOT files.
    /// The DOT files are written to the target folder.
    /// The file names are derived from the scanner mode names and the index of the DFA.
    pub(crate) fn generate_compiled_dfas_as_dot<T>(
        &self,
        modes: &[ScannerMode],
        target_folder: T,
    ) -> Result<()>
    where
        T: AsRef<std::path::Path>,
    {
        use std::fs::File;
        for (i, scanner_mode) in self.scanner_modes.iter().enumerate() {
            let title = format!("Compiled DFA {}", modes[i].name);
            let file_name = format!(
                "{}/{}.dot",
                target_folder.as_ref().to_str().unwrap(),
                modes[i].name,
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

    /// Resets the scanner to the initial state.
    #[inline]
    pub(crate) fn reset(&mut self) {
        self.current_mode = 0;
    }

    pub(crate) fn set_mode(&mut self, mode: usize) {
        self.current_mode = mode;
    }
}

impl TryFrom<Vec<ScannerMode>> for ScannerImpl {
    type Error = crate::ScnrError;
    fn try_from(scanner_modes: Vec<ScannerMode>) -> Result<Self> {
        let mut character_classes = CharacterClassRegistry::new();
        let scanner_modes =
            scanner_modes
                .into_iter()
                .try_fold(Vec::new(), |mut acc, scanner_mode| {
                    acc.push(CompiledScannerMode::try_from_scanner_mode(
                        scanner_mode,
                        &mut character_classes,
                    )?);
                    Ok::<Vec<CompiledScannerMode>, ScnrError>(acc)
                })?;

        let mut me = Self {
            character_classes,
            scanner_modes,
            current_mode: 0,
            match_char_class: Arc::new(|_, _| false),
        };
        me.match_char_class = Arc::new(Self::create_match_char_class(&me)?);
        Ok(me)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScannerMode;
    use std::{convert::TryInto, fs};

    #[test]
    fn test_try_from() {
        let scanner_modes = vec![
            ScannerMode::new("mode1", vec![("a".to_string(), 0)], vec![]),
            ScannerMode::new("mode2", vec![("b".to_string(), 1)], vec![]),
        ];
        let scanner_impl: ScannerImpl = scanner_modes.try_into().unwrap();
        assert_eq!(scanner_impl.character_classes.len(), 2);
        assert_eq!(scanner_impl.scanner_modes.len(), 2);
    }

    #[test]
    fn test_match_char_class() {
        let scanner_modes = vec![
            ScannerMode::new("mode1", vec![("a".to_string(), 0)], vec![]),
            ScannerMode::new("mode2", vec![("b".to_string(), 1)], vec![]),
        ];
        let scanner_impl: ScannerImpl = scanner_modes.try_into().unwrap();
        let match_char_class = scanner_impl.create_match_char_class().unwrap();
        assert!(match_char_class(0usize.into(), 'a'));
        assert!(!match_char_class(0usize.into(), 'b'));
        assert!(!match_char_class(0usize.into(), 'c'));
        assert!(!match_char_class(1usize.into(), 'a'));
        assert!(match_char_class(1usize.into(), 'b'));
        assert!(!match_char_class(1usize.into(), 'c'));
    }

    #[test]
    fn test_generate_dot_files() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/string.json");
        let file = fs::File::open(path).unwrap();

        let scanner_modes: Vec<ScannerMode> = serde_json::from_reader(file)
            .unwrap_or_else(|e| panic!("**** Failed to read json file {path}: {e}"));

        let scanner_impl: ScannerImpl = scanner_modes.clone().try_into().unwrap();
        let target_folder = concat!(env!("CARGO_MANIFEST_DIR"), "/target/string_dfas");

        // Delete all previously generated dot files.
        let _ = fs::remove_dir_all(target_folder);
        // Create the target folder.
        fs::create_dir_all(target_folder).unwrap();

        // Generate the compiled DFAs as dot files.
        scanner_impl
            .generate_compiled_dfas_as_dot(&scanner_modes, target_folder)
            .unwrap();

        // Check if the dot files are generated.
        let dot_files: Vec<_> = fs::read_dir(target_folder)
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
