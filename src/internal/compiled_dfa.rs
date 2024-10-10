use crate::{
    internal::{parse_regex_syntax, Nfa},
    Match, Result, ScnrError,
};

use super::{
    dfa::Dfa, matching_state::MatchingState, multi_pattern_nfa::MultiPatternNfa, CharClassID,
    CharacterClassRegistry, StateID, TerminalID,
};

/// A compiled DFA that can be used to match a string.
///
/// The DFA is compiled from a DFA by creating match functions for all character classes.
/// The match functions are used to decide if a character is in a character class.
/// Furthermore, the compile creates optimized data structures for the DFA to speed up matching.
///
/// MatchFunctions are not Clone nor Copy, so we aggregate them into a new struct CompiledDfa
/// which is Clone and Copy neither.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompiledDfa {
    /// The accepting states of the DFA as well as the corresponding pattern id.
    /// The vector is sorted by priority of the terminal. The lower the index, the higher the
    /// priority.
    accepting_states: Vec<(StateID, TerminalID)>,
    /// Each entry in the vector represents a state in the DFA. The entry is a tuple of first and
    /// last index into the transitions vector.
    state_ranges: Vec<(usize, usize)>,
    /// The transitions of the DFA. The indices that are relevant for a state are stored in the
    /// state_ranges vector.
    transitions: Vec<(CharClassID, StateID)>,
}

impl CompiledDfa {
    /// Get the pattern id if the given state is an accepting state.
    /// It is used for debugging purposes mostly in the [crate::internal::dot] module.
    #[inline]
    pub(crate) fn is_accepting(&self, state_id: StateID) -> bool {
        self.accepting_states.iter().any(|a| a.0 == state_id)
    }

    /// Returns the state ranges of the DFA.
    /// It is used for debugging purposes mostly in the [crate::internal::dot] module.
    #[allow(unused)]
    pub(crate) fn state_ranges(&self) -> &[(usize, usize)] {
        &self.state_ranges
    }

    /// Returns the transitions of the DFA.
    /// It is used for debugging purposes mostly in the [crate::internal::dot] module.
    #[allow(unused)]
    pub(crate) fn transitions(&self) -> &[(CharClassID, StateID)] {
        &self.transitions
    }

    /// Find a match in a multi-pattern DFA.
    /// Takes a CharIndices iterator and returns an Option<Match>.
    /// The Match contains the start and end position of the match as well as the pattern id.
    /// If no match is found, None is returned.
    /// The DFA is advanced by one character at a time.
    /// A list of MatchingStates is used to keep track of the current state of the DFA for each
    /// pattern.
    /// The DFA is reset before the search starts.
    pub(crate) fn find_match(
        &self,
        mut char_indices: std::str::CharIndices,
        match_char_class: &(dyn Fn(CharClassID, char) -> bool + 'static),
    ) -> Option<Match> {
        let mut matching_states = Vec::new();
        // Initialize the matching states after reading one character from the input.
        let (c_pos, c) = char_indices.next()?;
        // Take all transitions from the start state and initialize the matching states.
        for transition in self.state_ranges[0].0..self.state_ranges[0].1 {
            let (char_class, next_state) = self.transitions[transition];
            if match_char_class(char_class, c) {
                let mut matching_state = MatchingState::new();
                if self.is_accepting(next_state) {
                    matching_state.transition_to_accepting(c_pos, c);
                } else {
                    matching_state.transition_to_non_accepting(c_pos);
                }
                matching_state.set_current_state(next_state);
                matching_states.push(matching_state);
            }
        }
        // Advance the DFA by one character at a time for each remaining matching state.
        for (c_pos, c) in char_indices {
            for matching_state in &mut matching_states {
                if matching_state.is_longest_match() {
                    continue;
                }
                if let Some(next_state) = self.find_transition(matching_state, c, match_char_class)
                {
                    if self.is_accepting(next_state) {
                        matching_state.transition_to_accepting(c_pos, c);
                    } else {
                        matching_state.transition_to_non_accepting(c_pos);
                    }
                    matching_state.set_current_state(next_state);
                } else {
                    matching_state.no_transition();
                }
            }
            matching_states.retain(|m| !m.is_no_match());
            if matching_states.is_empty() {
                break;
            }
            if matching_states.iter().all(|m| m.is_longest_match()) {
                break;
            }
        }
        // Find the longest match.
        // If there are more than one longest matches with the same length, the one with the
        // highest priority (i.e. the lowest third element in the tripple in the
        // accepting_states vector) is returned.
        if !matching_states.is_empty() {
            matching_states.sort_by(|a, b| {
                a.last_match()
                    .unwrap()
                    .len()
                    .cmp(&b.last_match().unwrap().len())
                    .reverse()
            });
            let max_len = matching_states[0].last_match().unwrap().len();
            let mut priority = usize::MAX;
            let mut index = usize::MAX;
            for (i, matching_state) in matching_states.iter().enumerate() {
                if matching_state.last_match().unwrap().len() < max_len {
                    break;
                }
                let match_priority = self.terminal_priority(matching_state.current_state());
                if match_priority < priority {
                    priority = match_priority;
                    index = i;
                }
            }
            if index != usize::MAX {
                let state = matching_states[index].current_state();
                let terminal_id = self.terminal_of_accepting_state(state);
                let span = matching_states[index].last_match().unwrap();
                return Some(Match::new(terminal_id.as_usize(), span));
            }
        }
        None
    }

    fn terminal_priority(&self, state_id: StateID) -> usize {
        self.accepting_states
            .iter()
            .position(|(state, _)| *state == state_id)
            .unwrap_or(usize::MAX)
    }

    fn terminal_of_accepting_state(&self, state_id: StateID) -> TerminalID {
        self.accepting_states
            .iter()
            .find(|(state, _)| *state == state_id)
            .map(|(_, terminal_id)| *terminal_id)
            .unwrap()
    }

    /// Returns the target state of the transition for the given character.
    fn find_transition(
        &self,
        matching_state: &MatchingState<StateID>,
        c: char,
        match_char_class: &(dyn Fn(CharClassID, char) -> bool + 'static),
    ) -> Option<StateID> {
        let (start, end) = self.state_ranges[matching_state.current_state().as_usize()];
        let transitions = &self.transitions[start..end];
        for (char_class, target_state) in transitions {
            if match_char_class(*char_class, c) {
                return Some(*target_state);
            }
        }
        None
    }

    pub(crate) fn try_from_pattern(
        pattern: &str,
        terminal_id: TerminalID,
        character_class_registry: &mut CharacterClassRegistry,
    ) -> Result<CompiledDfa> {
        let ast = parse_regex_syntax(pattern)?;
        let nfa: Nfa = Nfa::try_from_ast(ast, character_class_registry)?;
        let dfa: Dfa = Dfa::try_from_nfa(nfa, terminal_id, character_class_registry)?;
        let dfa = dfa.minimize()?;
        let compiled_dfa = CompiledDfa::try_from(dfa)?;
        Ok(compiled_dfa)
    }

    /// Create a new compiled DFA from a slice of (pattern, pattern_id) tuples.
    /// This function is used to create a compiled DFA from a scanner mode, just like the
    /// Flex scanner generator does.
    pub(crate) fn try_from_patterns(
        patterns: &[(String, TerminalID)],
        character_class_registry: &mut CharacterClassRegistry,
    ) -> Result<Self> {
        let nfa = MultiPatternNfa::try_from_patterns(patterns, character_class_registry)?;
        let dfa = Dfa::try_from_mp_nfa(nfa, character_class_registry)?;
        let dfa = dfa.minimize()?;
        CompiledDfa::try_from(dfa)
    }
}

impl TryFrom<Dfa> for CompiledDfa {
    type Error = ScnrError;

    fn try_from(dfa: Dfa) -> std::result::Result<Self, Self::Error> {
        let Dfa {
            patterns,
            states,
            accepting_states,
            transitions,
            ..
        } = dfa;

        let mut state_ranges = Vec::new();
        let mut compiled_transitions = Vec::new();

        // Initialize state ranges with dummy values.
        for _ in 0..states.len() {
            state_ranges.push((0, 0));
        }

        // Iterate over the transitions and fill the state ranges and transitions vector by
        // maintaining the sort order of the transitions and char classes for each state.
        for (state, state_transitions) in transitions {
            let start = compiled_transitions.len();
            state_ranges[state] = (start, start + state_transitions.len());
            let mut transitions_for_state = state_transitions.iter().try_fold(
                Vec::new(),
                |mut acc, (char_class, target_state)| {
                    acc.push((*char_class, *target_state));
                    Ok::<Vec<(CharClassID, StateID)>, ScnrError>(acc)
                },
            )?;
            compiled_transitions.append(&mut transitions_for_state);
        }

        // Sort the accepting states by the index of the terminal id in the pattern vector.
        let mut accepting_states: Vec<(StateID, TerminalID)> = accepting_states
            .iter()
            .map(|(state, terminal_id)| (*state, *terminal_id))
            .collect();
        accepting_states.sort_by(|a, b| {
            patterns
                .iter()
                .position(|(_, id)| *id == a.1)
                .unwrap()
                .cmp(&patterns.iter().position(|(_, id)| *id == b.1).unwrap())
        });

        Ok(Self {
            accepting_states,
            state_ranges,
            transitions: compiled_transitions,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::{internal::CharacterClassRegistry, ScannerImpl, ScannerMode, Span};

    /// A macro that simplifies the rendering of a dot file for a DFA.
    macro_rules! compiled_dfa_render_to {
        ($nfa:expr, $label:expr, $reg:ident) => {
            let label = format!("{}Dfa", $label);
            let mut f = std::fs::File::create(format!("target/{}CompiledDfa.dot", $label)).unwrap();
            $crate::internal::dot::compiled_dfa_render($nfa, &label, &$reg, &mut f);
        };
    }

    #[test]
    fn test_compiled_dfa() {
        let mut character_class_registry = CharacterClassRegistry::new();
        let pattern = "(//.*(\\r\\n|\\r|\\n))";
        let compiled_dfa =
            CompiledDfa::try_from_pattern(pattern, 0usize.into(), &mut character_class_registry)
                .unwrap();
        compiled_dfa_render_to!(&compiled_dfa, "LineComment", character_class_registry);
    }

    #[test]
    fn test_compiled_dfa_from_patterns() {
        let mut character_class_registry = CharacterClassRegistry::new();
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/parol.json");
        let file = fs::File::open(path).unwrap();

        let scanner_modes: Vec<ScannerMode> = serde_json::from_reader(file)
            .unwrap_or_else(|e| panic!("**** Failed to read json file {path}: {e}"));

        assert_eq!(scanner_modes.len(), 1);

        let mut compiled_dfa = CompiledDfa::try_from_patterns(
            &scanner_modes[0].patterns,
            &mut character_class_registry,
        )
        .unwrap();
        compiled_dfa_render_to!(&compiled_dfa, "ParolMpDfa", character_class_registry);

        // Assert that the accepting states are sorted in the same order as the patterns.
        let mut index_in_patterns = usize::MAX;
        let accepting_states = &compiled_dfa.accepting_states;
        for (_, terminal_id) in accepting_states.iter() {
            let pattern_id = scanner_modes[0]
                .patterns
                .iter()
                .position(|(_, id)| *id == *terminal_id)
                .unwrap();
            if index_in_patterns == usize::MAX {
                index_in_patterns = pattern_id;
            } else {
                assert!(
                    index_in_patterns <= pattern_id,
                    "Accepting states are not sorted: {} <= {}",
                    index_in_patterns,
                    pattern_id
                );
                index_in_patterns = pattern_id;
            }
        }

        let match_char_class = ScannerImpl::try_from(scanner_modes)
            .unwrap()
            .create_match_char_class()
            .unwrap();

        let find_iter = compiled_dfa.find_match("::".char_indices(), &*match_char_class);
        assert_eq!(find_iter, Some(Match::new(18, Span::new(0, 2))));
    }
}
