use crate::ScnrError;

use super::{dfa::Dfa, matching_state::MatchingState, CharClassID, StateID};

/// A compiled DFA that can be used to match a string.
///
/// The DFA is compiled from a DFA by creating match functions for all character classes.
/// The match functions are used to decide if a character is in a character class.
/// Furthermore, the compile creates optimized data structures for the DFA to speed up matching.
///
/// MatchFunctions are not Clone nor Copy, so we aggregate them into a new struct CompiledDfa
/// which is Clone and Copy neither.
#[derive(Default, Debug)]
pub(crate) struct CompiledDfa {
    /// The pattern matched by the DFA.
    pattern: String,
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
    /// Creates a new compiled DFA with the given pattern and accepting states.
    /// Returns the pattern matched by the DFA.
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Returns the accepting states of the DFA.
    pub fn accepting_states(&self) -> &[StateID] {
        &self.accepting_states
    }

    /// Returns the state ranges of the DFA.
    pub fn state_ranges(&self) -> &[(usize, usize)] {
        &self.state_ranges
    }

    /// Returns the transitions of the DFA.
    pub fn transitions(&self) -> &[(CharClassID, StateID)] {
        &self.transitions
    }

    /// Returns the matching state of the DFA.
    pub fn matching_state(&self) -> &MatchingState<StateID> {
        &self.matching_state
    }
}

impl TryFrom<Dfa> for CompiledDfa {
    type Error = ScnrError;

    fn try_from(dfa: Dfa) -> Result<Self, Self::Error> {
        let Dfa {
            pattern,
            states,
            accepting_states,
            transitions,
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
            pattern,
            accepting_states,
            state_ranges,
            transitions: compiled_transitions,
            matching_state,
        })
    }
}
