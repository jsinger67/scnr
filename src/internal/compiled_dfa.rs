use crate::{
    internal::{parse_regex_syntax, Nfa},
    Match, Result, ScnrError, Span,
};

use super::{
    dfa::Dfa,
    matching_state::{self, MatchingState},
    multi_pattern_nfa::MultiPatternNfa,
    CharClassID, CharacterClassRegistry, StateID, TerminalID,
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
    /// The pattern matched by the DFA.
    // pattern: String,
    /// The accepting states of the DFA as well as the corresponding pattern id with the index of
    /// the terminal in the patterns vector, which is indirectly proportional to the priority of
    /// the matched terminal.
    accepting_states: Vec<(StateID, TerminalID, usize)>,
    /// Each entry in the vector represents a state in the DFA. The entry is a tuple of first and
    /// last index into the transitions vector.
    state_ranges: Vec<(usize, usize)>,
    /// The transitions of the DFA. The indices that are relevant for a state are stored in the
    /// state_ranges vector.
    transitions: Vec<(CharClassID, StateID)>,
    /// The state of matching
    matching_state: MatchingState<StateID>,
}

impl CompiledDfa {
    // Creates a new compiled DFA with the given pattern and accepting states.
    // Returns the pattern matched by the DFA.
    // pub fn pattern(&self) -> &str {
    //     &self.pattern
    // }

    // Returns the accepting states of the DFA.
    // pub fn accepting_states(&self) -> &[StateID] {
    //     &self.accepting_states
    // }

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

    /// Returns the matching state of the DFA.
    /// It is used for debugging purposes.
    #[allow(unused)]
    pub(crate) fn matching_state(&self) -> &MatchingState<StateID> {
        &self.matching_state
    }

    /// Resets the matching state of the DFA.
    pub(crate) fn reset(&mut self) {
        self.matching_state = MatchingState::new();
    }

    /// Returns the current state of the DFA.
    /// It is used for debugging purposes.
    #[allow(unused)]
    pub(crate) fn current_state(&self) -> StateID {
        self.matching_state.current_state()
    }

    /// Returns the last match of the DFA.
    pub(crate) fn current_match(&self) -> Option<Span> {
        self.matching_state.last_match()
    }

    /// Advances the DFA by one character.
    pub(crate) fn advance(
        &mut self,
        c_pos: usize,
        c: char,
        match_char_class: &(dyn Fn(CharClassID, char) -> bool + 'static),
    ) {
        // If we already have the longest match, we can stop
        if self.matching_state.is_longest_match() {
            return;
        }
        // Get the transitions for the current state
        if let Some(next_state) = self.find_transition(&self.matching_state, c, match_char_class) {
            if self.is_accepting(next_state) {
                self.matching_state.transition_to_accepting(c_pos, c);
            } else {
                self.matching_state.transition_to_non_accepting(c_pos);
            }
            self.matching_state.set_current_state(next_state);
        } else {
            self.matching_state.no_transition();
        }
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
        &mut self,
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
                // Find the longest match.
                // If there are more than one longest matches with the same length, the one with the
                // highest priority (i.e. the lowest third element in the tripple in the
                // accepting_states vector) is returned.
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
                    let match_priority = self.priority_of_terminal(matching_state.current_state());
                    if match_priority < priority {
                        priority = match_priority;
                        index = i;
                    }
                }
                if index != usize::MAX {
                    let state = matching_states[index].current_state();
                    let (_, terminal_id, _) = self.accepting_states[state.as_usize()];
                    let span = matching_states[index].last_match().unwrap();
                    return Some(Match::new(terminal_id.as_usize(), span));
                }
            }
        }
        None
    }

    fn priority_of_terminal(&self, state_id: StateID) -> usize {
        self.accepting_states
            .iter()
            .find(|(state, _, _)| *state == state_id)
            .map(|(_, _, priotity)| *priotity)
            .unwrap_or(usize::MAX)
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

    // pub(crate) fn search_on(&self) -> bool {
    //     !self.matching_state.is_longest_match()
    // }

    /// Returns true if the search should continue on the next character if the automaton has ever
    /// been in the matching state Start.
    #[inline]
    pub(crate) fn search_for_longer_match(&self) -> bool {
        !self.matching_state.is_longest_match() && !self.matching_state.is_no_match()
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
    pub(crate) fn try_from_scanner_mode(
        patterns: &[(String, TerminalID)],
        transitions: Vec<(usize, usize)>,
        character_class_registry: &mut CharacterClassRegistry,
    ) -> Result<Self> {
        let nfa = MultiPatternNfa::try_from_patterns(patterns, character_class_registry)?;
        let dfa = Dfa::try_from_mp_nfa(nfa, character_class_registry)?;
        let dfa = dfa.minimize()?;
        let mut compiled_dfa = CompiledDfa::try_from(dfa)?;
        compiled_dfa.transitions = transitions
            .iter()
            .map(|(char_class, to_state)| ((*char_class).into(), (*to_state).into()))
            .collect();
        Ok(compiled_dfa)
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
        let matching_state = MatchingState::new();

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

        Ok(Self {
            accepting_states: accepting_states
                .iter()
                .map(|(state, terminal_id)| {
                    (
                        *state,
                        *terminal_id,
                        patterns
                            .iter()
                            .position(|(_, id)| *id == *terminal_id)
                            .unwrap(),
                    )
                })
                .collect(),
            state_ranges,
            transitions: compiled_transitions,
            matching_state,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::CharacterClassRegistry;

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
}
