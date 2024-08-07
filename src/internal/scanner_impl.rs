use crate::{FindMatches, Match, Result, ScannerMode, ScnrError};

use super::{CharClassID, CharacterClassRegistry, CompiledScannerMode, MatchFunction};

#[derive(Clone)]
pub(crate) struct ScannerImpl {
    character_classes: CharacterClassRegistry,
    pub(crate) scanner_modes: Vec<CompiledScannerMode>,
    current_mode: usize,
}

impl ScannerImpl {
    pub(crate) fn character_classes(&self) -> &CharacterClassRegistry {
        &self.character_classes
    }

    pub(crate) fn scanner_modes(&self) -> &[CompiledScannerMode] {
        &self.scanner_modes
    }

    /// Returns an iterator over all non-overlapping matches.
    /// The iterator yields a [`Match`] value until no more matches could be found.
    pub(crate) fn find_iter<'h>(&self, input: &'h str) -> Result<FindMatches<'h>> {
        FindMatches::new(self, input)
    }

    pub(crate) fn create_match_char_class(
        &self,
    ) -> Result<Box<dyn Fn(CharClassID, char) -> bool + 'static>> {
        let match_functions =
            self.character_classes
                .iter()
                .try_fold(Vec::new(), |mut acc, cc| {
                    let match_function: MatchFunction = cc.ast().try_into()?;
                    acc.push(match_function);
                    Ok::<Vec<MatchFunction>, ScnrError>(acc)
                })?;
        Ok(Box::new(move |char_class, c| {
            match_functions[char_class.as_usize()].call(c)
        }))
    }

    /// Executes a leftmost search and returns the first match that is found, if one exists.
    /// It starts the search at the position of the given CharIndices iterator.
    /// During the search, all DFAs are advanced in parallel by one character at a time.
    pub(crate) fn find_from(
        &mut self,
        match_char_class: &Box<dyn Fn(CharClassID, char) -> bool + 'static>,
        char_indices: std::str::CharIndices,
    ) -> Option<Match> {
        let patterns = &mut self.scanner_modes[self.current_mode].patterns;
        for (dfa, _) in patterns.iter_mut() {
            dfa.reset();
        }

        // All indices of the DFAs that are still active.
        let mut active_dfas = (0..patterns.len()).collect::<Vec<_>>();

        for (i, c) in char_indices {
            for dfa_index in &active_dfas {
                patterns[*dfa_index].0.advance(i, c, &match_char_class);
            }

            // We remove all DFAs from `active_dfas` that finished or did not find a match so far.
            active_dfas.retain(|&dfa_index| patterns[dfa_index].0.search_for_longer_match());

            // If all DFAs have finished, we can stop the search.
            if active_dfas.is_empty() {
                break;
            }
        }

        let current_match = self.find_first_longest_match();
        self.execute_possible_mode_switch(current_match);
        current_match
    }

    /// We evaluate the matches of the DFAs in ascending order to prioritize the matches with the
    /// lowest index.
    /// We find the pattern with the lowest start position and the longest length.
    fn find_first_longest_match(&mut self) -> Option<Match> {
        let mut current_match: Option<Match> = None;
        {
            let patterns = &self.scanner_modes[self.current_mode].patterns;
            for (dfa, tok_type) in patterns.iter() {
                if let Some(dfa_match) = dfa.current_match() {
                    if current_match.is_none()
                        || dfa_match.start < current_match.unwrap().start()
                        || dfa_match.start == current_match.unwrap().start()
                            && dfa_match.len() > current_match.unwrap().span().len()
                    {
                        // We have a match and we continue the look for a longer match.
                        current_match = Some(Match::new(tok_type.as_usize(), dfa_match));
                    }
                }
            }
        }
        current_match
    }

    /// Executes a possible mode switch if a transition is defined for the token type found.
    #[inline]
    fn execute_possible_mode_switch(&mut self, current_match: Option<Match>) {
        let current_mode = &self.scanner_modes[self.current_mode];
        if let Some(current_match) = current_match.as_ref() {
            // We perform a scanner mode switch if a transition is defined for the token type found.
            if let Some(next_mode) = current_mode.has_transition(current_match.token_type()) {
                self.current_mode = next_mode;
            }
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

    /// Sets the current scanner mode.
    ///
    /// A parser can explicitly set the scanner mode to switch to a different set of DFAs.
    /// Usually, the scanner mode is changed by the scanner itself based on the transitions defined
    /// in the scanner mode.
    pub(crate) fn set_mode(&mut self, mode: usize) {
        self.current_mode = mode;
    }

    /// Returns the current scanner mode.
    pub(crate) fn current_mode(&self) -> usize {
        self.current_mode
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
        Ok(Self {
            character_classes,
            scanner_modes,
            current_mode: 0,
        })
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
    use std::convert::TryInto;

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
        assert!(match_char_class(0.into(), 'a'));
        assert!(!match_char_class(0.into(), 'b'));
        assert!(!match_char_class(0.into(), 'c'));
        assert!(!match_char_class(1.into(), 'a'));
        assert!(match_char_class(1.into(), 'b'));
        assert!(!match_char_class(1.into(), 'c'));
    }
}
