use std::io::{Cursor, Read};

use log::trace;

use crate::{
    internal::{parse_regex_syntax, Nfa},
    Result, ScnrError, Span,
};

use super::{
    dfa::Dfa, matching_state::MatchingState, CharClassID, CharacterClassRegistry, StateID,
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
    /// The accepting states of the DFA as well as the corresponding pattern id.
    accepting_states: Vec<StateID>,
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
    #[allow(unused)]
    pub(crate) fn is_accepting(&self, state_id: StateID) -> bool {
        self.accepting_states.contains(&state_id)
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
        if let Some(next_state) = self.find_transition(c, match_char_class) {
            if self.accepting_states.contains(&next_state) {
                self.matching_state.transition_to_accepting(c_pos, c);
            } else {
                self.matching_state.transition_to_non_accepting(c_pos);
            }
            self.matching_state.set_current_state(next_state);
        } else {
            self.matching_state.no_transition();
        }
    }

    /// Returns the target state of the transition for the given character.
    fn find_transition(
        &self,
        c: char,
        match_char_class: &(dyn Fn(CharClassID, char) -> bool + 'static),
    ) -> Option<StateID> {
        let (start, end) = self.state_ranges[self.matching_state.current_state().as_usize()];
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
    pub(crate) fn search_for_longer_match(&self) -> bool {
        !self.matching_state.is_longest_match() && !self.matching_state.is_no_match()
    }

    pub(crate) fn try_from_pattern(
        pattern: &str,
        character_class_registry: &mut CharacterClassRegistry,
    ) -> Result<CompiledDfa> {
        let ast = parse_regex_syntax(pattern)?;
        let nfa: Nfa = Nfa::try_from_ast(ast, character_class_registry)?;

        trace!("NFA:\n{}", {
            let mut cursor = Cursor::new(Vec::new());
            let title = format!("NFA for pattern '{}'", pattern.escape_default());
            super::dot::nfa_render(&nfa, &title, &mut cursor);
            let mut dot_format = String::new();
            cursor.set_position(0);
            cursor.read_to_string(&mut dot_format)?;
            dot_format
        });

        let dfa: Dfa = Dfa::try_from_nfa(nfa, character_class_registry)?;

        trace!("DFA:\n{}", {
            let mut cursor = Cursor::new(Vec::new());
            let title = format!("DFA for pattern '{}'", pattern.escape_default());
            super::dot::dfa_render(&dfa, &title, character_class_registry, &mut cursor);
            let mut dot_format = String::new();
            cursor.set_position(0);
            cursor.read_to_string(&mut dot_format)?;
            dot_format
        });

        let dfa = dfa.minimize()?;

        trace!("Minimized DFA:\n{}", {
            let mut cursor = Cursor::new(Vec::new());
            let title = format!("Minimized DFA for pattern '{}'", pattern.escape_default());
            super::dot::dfa_render(&dfa, &title, character_class_registry, &mut cursor);
            let mut dot_format = String::new();
            cursor.set_position(0);
            cursor.read_to_string(&mut dot_format)?;
            dot_format
        });

        let compiled_dfa = CompiledDfa::try_from(dfa)?;

        trace!("Compiled DFA:\n{}", {
            let mut cursor = Cursor::new(Vec::new());
            let title = format!("Compiled DFA for pattern '{}'", pattern.escape_default());
            super::dot::compiled_dfa_render(
                &compiled_dfa,
                &title,
                character_class_registry,
                &mut cursor,
            );
            let mut dot_format = String::new();
            cursor.set_position(0);
            cursor.read_to_string(&mut dot_format)?;
            dot_format
        });

        Ok(compiled_dfa)
    }
}

impl TryFrom<Dfa> for CompiledDfa {
    type Error = ScnrError;

    fn try_from(dfa: Dfa) -> std::result::Result<Self, Self::Error> {
        let Dfa {
            // pattern,
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
            // pattern,
            accepting_states: accepting_states.into_iter().collect(),
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
            CompiledDfa::try_from_pattern(pattern, &mut character_class_registry).unwrap();
        compiled_dfa_render_to!(&compiled_dfa, "LineComment", character_class_registry);
    }
}
