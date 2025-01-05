//! This module contains the implementation of the multi-pattern NFA.
//! The multi-pattern NFA is used to find all matches of multiple patterns in a text.
//! The implemenation is based on the nfa module. The Nfa there has only one end state, but the
//! multi-pattern NFA has one end state for each pattern.

use super::{nfa::EpsilonTransition, CharClassID, Nfa, StateID};
use crate::{Pattern, Result};

#[derive(Debug, Clone, Default)]
pub(crate) struct MultiPatternNfa {
    pub(crate) patterns: Vec<Pattern>,
    /// Transitions from the start state to the start states of the NFAs.
    /// The start state of the multi-pattern NFA is always 0.
    pub(crate) start_transitions: Vec<EpsilonTransition>,
    pub(crate) nfas: Vec<Nfa>,
}

impl MultiPatternNfa {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub fn add_pattern(&mut self, pattern: Pattern) {
        self.patterns.push(pattern);
    }

    fn add_nfa(&mut self, nfa: Nfa) {
        self.nfas.push(nfa);
    }

    pub(crate) fn try_from_patterns(
        patterns: &[Pattern],
        character_class_registry: &mut super::CharacterClassRegistry,
    ) -> Result<Self> {
        let mut multi_pattern_nfa = Self::new();
        let mut next_state = 1;
        for pattern in patterns {
            let ast = super::parse_regex_syntax(pattern.pattern())?;
            let mut nfa = Nfa::try_from_ast(ast, character_class_registry)?;

            let (s, e) = nfa.shift_ids(next_state);
            debug_assert_eq!(e.id(), nfa.highest_state_number());
            next_state = e.id() as usize + 1;

            multi_pattern_nfa
                .start_transitions
                .push(EpsilonTransition::new(s));

            multi_pattern_nfa.add_pattern(pattern.clone());
            multi_pattern_nfa.add_nfa(nfa);
        }
        Ok(multi_pattern_nfa)
    }

    /// Returns the patterns as a slice of (pattern, terminal_id) tuples.
    pub(crate) fn patterns(&self) -> &[Pattern] {
        &self.patterns
    }

    /// Returns the start transitions of the MultiPatternNfa.
    pub(crate) fn start_transitions(&self) -> &[EpsilonTransition] {
        &self.start_transitions
    }

    /// Checks if the given state is an accepting state of one of the NFAs.
    pub(crate) fn is_accepting_state(&self, state: StateID) -> bool {
        self.nfas.iter().any(|nfa| nfa.end_state() == state)
    }

    /// Calculate the epsilon closure of a state.
    /// This is the set of states that can be reached from the state by following epsilon
    /// transitions transitively.
    ///
    /// The result is a vector of unique states.
    ///
    /// This function handles all states other than 0 by calling the epsilon_closure function of the
    /// corresponding NFA.
    /// The epsilon closure of state 0 is the united set of all epsilon_closure of alle start states
    /// of the NFAs.
    pub(crate) fn epsilon_closure(&self, state: StateID) -> Vec<StateID> {
        if state.id() == 0 {
            let mut result = Vec::new();
            for nfa in &self.nfas {
                let start_state = nfa.start_state();
                let epsilon_closure = nfa.epsilon_closure(start_state);
                for state in epsilon_closure {
                    if !result.contains(&state) {
                        result.push(state);
                    }
                }
            }
            result.sort_unstable();
            result
        } else {
            // Find the nfa that contains the state and call epsilon_closure on it.
            self.nfas
                .iter()
                .find(|nfa| nfa.contains_state(state))
                .map(|f| f.epsilon_closure(state))
                .unwrap_or_default()
        }
    }

    /// Calculate the epsilon closure of a set of states and return the unique states.
    ///
    /// This function handles all states other than 0 by calling the epsilon_closure_set function of
    /// the corresponding NFA.
    /// The epsilon closure of state 0 is the united set of all epsilon_closure of alle start states
    /// of the NFAs.
    pub(crate) fn epsilon_closure_set<I>(&self, states: I) -> Vec<StateID>
    where
        I: IntoIterator<Item = StateID>,
    {
        // Collect all states in a vector and check if state 0 is in the set.
        let mut states: Vec<StateID> = states.into_iter().collect();
        if states.contains(&0.into()) {
            // If state 0 is in the set, add all start states of the NFAs to the set.
            for nfa in &self.nfas {
                let start_state = nfa.start_state();
                if !states.contains(&start_state) {
                    states.push(start_state);
                }
            }
        }
        // Calculate the epsilon closure of the united set of states.
        let mut result = Vec::new();
        for state in states {
            let epsilon_closure = self.epsilon_closure(state);
            for state in epsilon_closure {
                if !result.contains(&state) {
                    result.push(state);
                }
            }
        }
        result.sort_unstable();
        result
    }

    /// Calculate move(T, a) for a set of states T and a character class a.
    /// This is the set of states that can be reached from T by matching a.
    pub(crate) fn move_set(&self, states: &[StateID], char_class: CharClassID) -> Vec<StateID> {
        // Combine the move of all NFAs that contain the states.
        let mut result = Vec::new();
        for nfa in &self.nfas {
            let states_of_nfa: Vec<_> = states
                .iter()
                .filter(|s| nfa.contains_state(**s))
                .cloned()
                .collect();
            let nfa_states = nfa.move_set(&states_of_nfa, char_class);
            for state in nfa_states {
                if !result.contains(&state) {
                    result.push(state);
                }
            }
        }
        result.sort_unstable();
        result
    }

    pub(crate) fn get_match_transitions(
        &self,
        start_states: impl Iterator<Item = StateID>,
    ) -> Vec<(CharClassID, StateID)> {
        let mut target_states = Vec::new();
        for state in start_states {
            if state.id() == 0 {
                for transition in &self.start_transitions {
                    self.find_nfa(transition.target_state()).map(|nfa| {
                        for state in nfa.states[nfa.start_state()].transitions() {
                            target_states.push((state.char_class(), state.target_state()));
                        }
                    });
                }
            } else {
                self.find_nfa(state).map(|nfa| {
                    for state in nfa.states[state].transitions() {
                        target_states.push((state.char_class(), state.target_state()));
                    }
                });
            }
        }
        target_states.sort_unstable();
        target_states.dedup();
        target_states
    }

    /// Find the NFA that contains the state and return the state.
    pub(crate) fn find_nfa(&self, state: StateID) -> Option<&Nfa> {
        self.nfas.iter().find(|nfa| nfa.contains_state(state))
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, sync::LazyLock};

    use super::*;
    use crate::{internal::character_class_registry::CharacterClassRegistry, ScannerMode};

    /// A macro that simplifies the rendering of a dot file for a MultiPatternNfa.
    macro_rules! mp_nfa_render_to {
        ($nfa:expr, $label:expr, $char_class:ident) => {
            let label = format!("{}MpNfa", $label);
            let mut f = std::fs::File::create(format!("target/{}MpNfa.dot", $label)).unwrap();
            $crate::internal::dot::multi_pattern_nfa_render($nfa, &label, &$char_class, &mut f);
        };
    }

    static SCANNER_MODES: LazyLock<Vec<ScannerMode>> = LazyLock::new(|| {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/benches/veryl_modes.json");
        let file = fs::File::open(path).unwrap();
        serde_json::from_reader(file).unwrap()
    });

    #[test]
    fn test_epsilon_closure() {
        let mut character_class_registry = CharacterClassRegistry::new();
        let ast = super::super::parse_regex_syntax("a|b").unwrap();
        let nfa = Nfa::try_from_ast(ast, &mut character_class_registry).unwrap();
        let mut multi_pattern_nfa = MultiPatternNfa::new();
        multi_pattern_nfa.add_nfa(nfa);

        let epsilon_closure = multi_pattern_nfa.epsilon_closure(0.into());
        assert_eq!(epsilon_closure, vec![0.into(), 2.into(), 4.into()]);
        let epsilon_closure = multi_pattern_nfa.epsilon_closure(1.into());
        assert_eq!(epsilon_closure, vec![1.into(), 5.into()]);
        let epsilon_closure = multi_pattern_nfa.epsilon_closure(2.into());
        assert_eq!(epsilon_closure, vec![2.into()]);
    }

    #[test]
    fn test_epsilon_closure_set() {
        let mut character_class_registry = CharacterClassRegistry::new();
        let ast = super::super::parse_regex_syntax("a|b").unwrap();
        let nfa = Nfa::try_from_ast(ast, &mut character_class_registry).unwrap();
        let mut multi_pattern_nfa = MultiPatternNfa::new();
        multi_pattern_nfa.add_nfa(nfa);

        let epsilon_closure = multi_pattern_nfa.epsilon_closure_set(vec![0.into()]);
        assert_eq!(epsilon_closure, vec![0.into(), 2.into(), 4.into()]);
        let epsilon_closure = multi_pattern_nfa.epsilon_closure_set(vec![1.into()]);
        assert_eq!(epsilon_closure, vec![1.into(), 5.into()]);
        let epsilon_closure = multi_pattern_nfa.epsilon_closure_set(vec![2.into()]);
        assert_eq!(epsilon_closure, vec![2.into()]);
        let epsilon_closure = multi_pattern_nfa.epsilon_closure_set(vec![0.into(), 1.into()]);
        assert_eq!(
            epsilon_closure,
            vec![0.into(), 1.into(), 2.into(), 4.into(), 5.into()]
        );
        let epsilon_closure = multi_pattern_nfa.epsilon_closure_set(vec![0.into(), 2.into()]);
        assert_eq!(epsilon_closure, vec![0.into(), 2.into(), 4.into()]);
        let epsilon_closure = multi_pattern_nfa.epsilon_closure_set(vec![1.into(), 2.into()]);
        assert_eq!(epsilon_closure, vec![1.into(), 2.into(), 5.into()]);
    }

    #[test]
    fn test_move_set() {
        let mut character_class_registry = CharacterClassRegistry::new();
        let ast = super::super::parse_regex_syntax("a|b").unwrap();
        let nfa = Nfa::try_from_ast(ast, &mut character_class_registry).unwrap();
        let mut multi_pattern_nfa = MultiPatternNfa::new();
        multi_pattern_nfa.add_nfa(nfa);

        let move_set = multi_pattern_nfa.move_set(&[0.into()], 0.into());
        assert_eq!(move_set, vec![1.into()]);
        let move_set = multi_pattern_nfa.move_set(&[1.into()], 0.into());
        assert_eq!(move_set, vec![]);
        let move_set = multi_pattern_nfa.move_set(&[2.into()], 0.into());
        assert_eq!(move_set, Vec::<StateID>::new());
    }

    #[test]
    fn test_try_from_patterns_single() {
        let mut character_class_registry = CharacterClassRegistry::new();
        let multi_pattern_nfa = MultiPatternNfa::try_from_patterns(
            &[Pattern::new("a+".to_string(), 0)],
            &mut character_class_registry,
        )
        .unwrap();
        assert_eq!(multi_pattern_nfa.patterns().len(), 1);
        assert_eq!(multi_pattern_nfa.start_transitions().len(), 1);
        mp_nfa_render_to!(&multi_pattern_nfa, "APlus", character_class_registry);
    }

    #[test]
    fn test_try_from_patterns() {
        let mut character_class_registry = CharacterClassRegistry::new();
        let multi_pattern_nfa = MultiPatternNfa::try_from_patterns(
            &SCANNER_MODES.iter().next().unwrap().patterns,
            &mut character_class_registry,
        )
        .unwrap();
        assert_eq!(multi_pattern_nfa.patterns().len(), 115);
        assert_eq!(multi_pattern_nfa.start_transitions().len(), 115);
        mp_nfa_render_to!(&multi_pattern_nfa, "Veryl", character_class_registry);
    }
}
