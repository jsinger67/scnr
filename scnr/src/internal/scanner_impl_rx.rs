use crate::{Match, Pattern, Result, ScannerMode, ScannerModeSwitcher, ScnrError};
use log::trace;
use regex_automata::{dfa::regex::Regex, Input};

use super::{ScannerModeID, TerminalID};

type ModeTransitions = Vec<(TerminalID, ScannerModeID)>;

#[derive(Clone, Debug)]
pub(crate) struct ScannerModeRx {
    rx: Regex,
    transitions: ModeTransitions,
    name: String,
    patterns: Vec<Pattern>,
    // The lookaheads are stored as a vector of options. If a lookahead is defined for a pattern,
    // the option contains the lookahead. Otherwise, the option is None.
    // The boolean value indicates if the lookahead is positive.
    // The string value of the lookahead pattern is available in the `patterns` member.
    lookaheads: Vec<Option<(bool, Regex)>>,
}

/// ScannerImpl instances are always created by the Scanner::try_new method and of course by
/// the clone method.
#[derive(Clone, Debug)]
pub(crate) struct ScannerImpl {
    pub(crate) scanner_modes: Vec<ScannerModeRx>,
    // The current mode is private and thereby makes the free creation of ScannerImpl instances
    // impossible.
    current_mode: usize,
}

impl ScannerImpl {
    /// Executes a possible mode switch if a transition is defined for the token type found.
    #[inline]
    fn execute_possible_mode_switch(&mut self, current_match: &Match) {
        // We perform a scanner mode switch if a transition is defined for the token type found.
        if let Some(next_mode) = self.has_transition(current_match.token_type()) {
            trace!(
                "Switching from mode {} to mode {}",
                self.current_mode,
                next_mode
            );
            self.current_mode = next_mode;
        }
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
        &mut self,
        _input: &str,
        char_indices: std::str::CharIndices,
    ) -> Option<crate::Match> {
        let current_mode = &mut self.scanner_modes[self.current_mode];
        let re = &mut current_mode.rx;

        let input: Input = char_indices.as_str().into();
        let input = input.anchored(regex_automata::Anchored::Yes);
        if let Some(matched) = re.find(input) {
            debug_assert!(
                !matched.is_empty(),
                r#"
    An empty token was matched. This leads to an infinite loop.
    It is therefore necessary to avoid regexes that can match empty tokens.
    Please, check regex '{}' for token type {} in scanner mode {}"#,
                current_mode.patterns[matched.pattern().as_usize()]
                    .pattern()
                    .escape_default(),
                matched.pattern().as_u32(),
                self.current_mode
            );
            let pattern_id = matched.pattern().as_usize();
            let matched_pattern = &current_mode.patterns[pattern_id];
            let mut matched: Match =
                Match::new(matched_pattern.terminal_id(), matched.range().into());
            // Check if the lookahead is satisfied.
            if let Some((is_positive, re)) = current_mode.lookaheads[pattern_id].as_ref() {
                let next_char = char_indices.clone().nth(matched.len());
                let end_of_input = next_char.is_none();
                if end_of_input {
                    // End of input reached.
                    // If the lookahead is negative, the match is valid.
                    trace!(
                        "End of input reached. Matched: {} {:?}",
                        if *is_positive { "?=" } else { "?!" },
                        matched
                    );
                    matched.add_offset(char_indices.offset());
                    return if *is_positive { None } else { Some(matched) };
                } else {
                    let lookahead_matched = {
                        let input: Input = char_indices.as_str()[matched.end()..].into();
                        let input = input.anchored(regex_automata::Anchored::Yes);
                        re.find(input).is_some()
                    };
                    matched.add_offset(char_indices.offset());
                    trace!(
                        "Lookahead matched: {} {} {:?}",
                        lookahead_matched,
                        if *is_positive { "?=" } else { "?!" },
                        matched
                    );
                    if lookahead_matched && *is_positive || !lookahead_matched && !*is_positive {
                        trace!("Lookahead is satisfied.");
                        return Some(matched);
                    } else {
                        trace!("Lookahead is not satisfied.");
                        return None;
                    }
                }
            }
            matched.add_offset(char_indices.offset());
            trace!("Matched: {:?}", matched);
            return Some(matched);
        }
        None
    }

    pub(crate) fn has_transition(&self, token_type: usize) -> Option<usize> {
        for (tok_type, scanner) in &self.scanner_modes[self.current_mode].transitions {
            match token_type.cmp(&tok_type.as_usize()) {
                std::cmp::Ordering::Less => return None,
                std::cmp::Ordering::Equal => return Some(scanner.as_usize()),
                std::cmp::Ordering::Greater => continue,
            }
        }
        None
    }

    /// Traces the compiled DFAs as dot format.
    /// The output is written to the log.
    /// This function is used for debugging purposes.
    #[cfg(feature = "dot_writer")]
    pub(crate) fn log_compiled_automata_as_dot(&self) -> crate::Result<()> {
        unimplemented!("Regex automata does not support dot output yet.");
    }

    /// Generates the compiled DFAs as dot files.
    /// The dot files are written to the target folder.
    #[cfg(feature = "dot_writer")]
    pub(crate) fn generate_compiled_automata_as_dot(
        &self,
        _prefix: &str,
        _target_folder: &std::path::Path,
    ) -> crate::Result<()> {
        unimplemented!("Regex automata does not support dot output yet.");
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

impl TryFrom<Vec<ScannerMode>> for ScannerImpl {
    type Error = crate::ScnrError;
    fn try_from(modes: Vec<ScannerMode>) -> Result<Self> {
        let mut scanner_modes = Vec::with_capacity(modes.len());

        for mode in modes {
            let rx = Regex::builder()
                .build_many(&mode.patterns)
                .map_err(|e| ScnrError::new(e.into()))?;
            let mut transitions = Vec::with_capacity(mode.transitions.len());
            for (terminal, scanner) in &mode.transitions {
                transitions.push((*terminal, *scanner));
            }
            transitions.sort_by_key(|(terminal, _)| *terminal);
            let mode = ScannerModeRx {
                rx,
                transitions,
                name: mode.name.clone(),
                patterns: mode.patterns.clone(),
                lookaheads: mode.patterns.iter().try_fold(
                    Vec::with_capacity(mode.patterns.len()),
                    |mut acc, p| {
                        if let Some(l) = p.lookahead() {
                            acc.push(Some((
                                l.is_positive,
                                Regex::builder()
                                    .build(&l.pattern)
                                    .map_err(|e| ScnrError::new(e.into()))?,
                            )));
                        } else {
                            acc.push(None);
                        }
                        Ok::<_, ScnrError>(acc)
                    },
                )?,
            };
            scanner_modes.push(mode);
        }
        Ok(Self {
            scanner_modes,
            current_mode: 0,
        })
    }
}

impl TryFrom<&[ScannerMode]> for ScannerImpl {
    type Error = crate::ScnrError;
    fn try_from(modes: &[ScannerMode]) -> Result<Self> {
        let mut scanner_modes = Vec::with_capacity(modes.len());

        for mode in modes {
            let rx = Regex::builder()
                .build_many(&mode.patterns)
                .map_err(|e| ScnrError::new(e.into()))?;
            let mut transitions = Vec::with_capacity(mode.transitions.len());
            for (terminal, scanner) in &mode.transitions {
                transitions.push((*terminal, *scanner));
            }
            transitions.sort_by_key(|(terminal, _)| *terminal);
            let mode = ScannerModeRx {
                rx,
                transitions,
                name: mode.name.clone(),
                patterns: mode.patterns.clone(),
                lookaheads: mode.patterns.iter().try_fold(
                    Vec::with_capacity(mode.patterns.len()),
                    |mut acc, p| {
                        if let Some(l) = p.lookahead() {
                            acc.push(Some((
                                l.is_positive,
                                Regex::builder()
                                    .build(&l.pattern)
                                    .map_err(|e| ScnrError::new(e.into()))?,
                            )));
                        } else {
                            acc.push(None);
                        }
                        Ok::<_, ScnrError>(acc)
                    },
                )?,
            };
            scanner_modes.push(mode);
        }
        Ok(Self {
            scanner_modes,
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
        "/../target/testout/scanner_nfa_impl_rx_test"
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
}
