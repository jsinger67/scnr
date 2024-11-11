use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{internal::nfa::Nfa, Pattern, Result, Span};

use super::{
    ids::StateSetID, parse_regex_syntax, CharClassID, CharacterClassRegistry, CompiledNfaLookahead,
    StateID, StateIDBase,
};

/// A compiled NFA.
/// It is used to represent the NFA in a way that is optimized for matching.
/// The start state is by design always 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompiledNfa {
    pub(crate) pattern: String,
    pub(crate) states: Vec<StateData>,
    // Used during NFA construction
    pub(crate) end_states: Vec<StateSetID>,
    // An optional lookahead that is used to check if the NFA should match the input.
    pub(crate) lookahead: Option<CompiledNfaLookahead>,
}

impl CompiledNfa {
    /// Simulates the NFA on the given input.
    /// Returns a match starting at the current position. No try on next character is done.
    /// The caller must do that.
    ///
    /// If no match is found, None is returned.
    ///
    /// We use a non-recursive implementation of the NFA simulation.
    /// The algorithm uses a queue to store the states that are currently active.
    /// The algorithm is as follows:
    /// 1. Add the start state to the queue.
    /// 2. Take the next character from the input.
    /// 3. For each state in the queue, check if it is an end state.
    ///     If it is, remember the current match.
    /// 4. For each state in the queue, check if there is a transition that matches the current
    ///    character.
    ///    If there is, add the target state to a seconde queue that will be used for the next
    ///    character.
    /// 5. Replace the queue with the second queue.
    /// 6. If there are more characters in the input, go to step 2.
    ///
    #[allow(dead_code)]
    pub(crate) fn find_from(
        &self,
        char_indices: std::str::CharIndices,
        match_char_class: &(dyn Fn(CharClassID, char) -> bool + 'static),
    ) -> Option<Span> {
        // trace!("NFA {}", self);
        let mut current_states: Vec<StateSetID> = Vec::with_capacity(self.states.len());
        // Push the start state to the current states.
        current_states.push(StateSetID::new(0));
        let mut next_states: Vec<StateSetID> = Vec::with_capacity(self.states.len());
        let mut match_start = None;
        let mut match_end = None;
        for (index, c) in char_indices {
            if match_start.is_none() {
                match_start = Some(index);
            }

            // trace!("-------------------");
            // trace!("Character: {}", c);
            // trace!("Current states: {:?}", current_states);
            for state in &current_states {
                // trace!("----------");
                // trace!("State: {}", state);
                if match_end.is_none() && self.end_states.contains(state) {
                    // trace!("Set end index to {} in state {}", index, state);
                    match_end = Some(index);
                }
                for (cc, next) in &self.states[state.as_usize()].transitions {
                    // trace!("Transition: #{} -> '{}'", cc.id(), next);
                    if match_char_class(*cc, c) {
                        if !next_states.contains(next) {
                            // trace!("Push next state {}", next);
                            next_states.push(*next);
                        }
                        if self.end_states.contains(next) {
                            // trace!("Update end index to {} for state {}", index, next);
                            match_end = Some(index);
                        }
                    }
                }
            }
            // trace!("Next states: {:?}", next_states);
            current_states.clear();
            std::mem::swap(&mut current_states, &mut next_states);
            if current_states.is_empty() {
                break;
            }
        }
        match_end.map(|match_end| Span::new(match_start.unwrap(), match_end + 1))
    }

    pub(crate) fn try_from_pattern(
        pattern: &Pattern,
        character_class_registry: &mut CharacterClassRegistry,
    ) -> Result<Self> {
        let ast = parse_regex_syntax(pattern.pattern())?;
        let nfa: Nfa = Nfa::try_from_ast(ast, character_class_registry)?;
        let mut nfa: CompiledNfa = nfa.into();
        if let Some(lookahead) = pattern.lookahead() {
            let lookahead =
                CompiledNfaLookahead::try_from_lookahead(lookahead, character_class_registry)?;
            nfa.lookahead = Some(lookahead);
        }
        Ok(nfa)
    }
}

impl From<Nfa> for CompiledNfa {
    // Create a dense representation of the NFA in form of match transitions between states sets.
    fn from(nfa: Nfa) -> Self {
        // A temporary map to store the state ids of the sets of states.
        let mut state_map: BTreeMap<BTreeSet<StateID>, StateSetID> = BTreeMap::new();
        // A temporary set to store the transitions of the CompiledNfa.
        // The state ids are numbers of sets of states.
        let mut transitions: BTreeSet<(StateSetID, CharClassID, StateSetID)> = BTreeSet::new();
        // The end states of the CompiledNfa.
        let mut end_states: Vec<StateSetID> = Vec::new();

        // Calculate the epsilon closure of the start state.
        let mut epsilon_closure: BTreeSet<StateID> =
            BTreeSet::from_iter(nfa.epsilon_closure(nfa.start_state));

        // The current state id is always 0.
        let current_state = StateSetID::new(StateSetID::new(0).id());

        // Add the start state to the state map.
        state_map.insert(BTreeSet::from_iter(epsilon_closure.clone()), current_state);

        // The list of target states not yet processed.
        let mut queue: VecDeque<StateSetID> = VecDeque::new();
        queue.push_back(current_state);

        while let Some(current_state) = queue.pop_front() {
            epsilon_closure = state_map
                .iter()
                .find(|(_, v)| **v == current_state)
                .unwrap()
                .0
                .clone();
            let target_states = nfa.get_match_transitions(epsilon_closure.clone().into_iter());
            let old_state_id = current_state;
            // trace!(
            //     "Current state {}: Target states of {:?} => {:?}",
            //     old_state_id,
            //     epsilon_closure,
            //     target_states
            // );
            // Transform the target states to a set of state ids by calculating their epsilon closure.
            for (cc, target_state) in target_states {
                epsilon_closure = BTreeSet::from_iter(nfa.epsilon_closure(target_state));
                // trace!("Epsilon closure of {}: {:?}", target_state, epsilon_closure);
                let mut new_state_id_candidate = StateSetID::new(state_map.len() as StateIDBase);
                if !state_map.contains_key(&epsilon_closure) {
                    state_map.insert(epsilon_closure.clone(), new_state_id_candidate);
                    // Add the new state to the queue.
                    queue.push_back(new_state_id_candidate);
                } else {
                    new_state_id_candidate = *state_map.get(&epsilon_closure).unwrap();
                }
                let current_state = new_state_id_candidate;
                // trace!("has state id: {}", current_state);
                if epsilon_closure.contains(&nfa.end_state) && !end_states.contains(&current_state)
                {
                    // trace!("End state: {:?}", current_state);
                    end_states.push(current_state);
                }
                transitions.insert((old_state_id, cc, current_state));
            }
        }

        // trace!("State map: {:?}", state_map);
        // trace!("Transitions: {:?}", transitions);
        // trace!("End states: {:?}", end_states);

        // The transitions of the CompiledNfa.
        let mut states: Vec<StateData> = Vec::with_capacity(transitions.len());
        for _ in 0..state_map.len() {
            states.push(StateData::default());
        }
        for (from, cc, to) in transitions {
            states[from.as_usize()].transitions.push((cc, to));
        }

        Self {
            pattern: nfa.pattern.clone(),
            states,
            end_states,
            lookahead: None,
        }
    }
}

impl std::fmt::Display for CompiledNfa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Pattern: {}", self.pattern)?;
        writeln!(f, "Start state: 0")?;
        writeln!(f, "End states: {:?}", self.end_states)?;
        for (i, state) in self.states.iter().enumerate() {
            writeln!(f, "State {}: {}", i, state)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct StateData {
    /// A list of transitions from this state.
    /// The state ids are numbers of sets of states.
    pub(crate) transitions: Vec<(CharClassID, StateSetID)>,
}

impl std::fmt::Display for StateData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (cc, next) in &self.transitions {
            writeln!(f, "Transition: {:?} -> {}", cc, next)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use log::trace;

    use super::*;
    use crate::{
        internal::{character_class_registry::CharacterClassRegistry, parser::parse_regex_syntax},
        Pattern,
    };

    /// A macro that simplifies the rendering of a dot file for a NFA.
    macro_rules! nfa_render_to {
        ($nfa:expr, $label:expr) => {
            let mut f = std::fs::File::create(format!("target/{}Nfa.dot", $label)).unwrap();
            $crate::internal::dot::nfa_render($nfa, $label, &mut f);
        };
    }

    /// A macro that simplifies the rendering of a dot file for a CompiledNfa.
    macro_rules! compiled_nfa_render_to {
        ($compiled_nfa:expr, $label:expr, $character_class_registry:expr) => {
            let mut f = std::fs::File::create(format!("target/{}CompiledNfa.dot", $label)).unwrap();
            $crate::internal::dot::compiled_nfa_render(
                $compiled_nfa,
                $label,
                $character_class_registry,
                &mut f,
            );
        };
    }

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    struct TestData {
        pattern: &'static str,
        name: &'static str,
        end_states: &'static [StateSetID],
        match_data: &'static [(&'static str, Option<(usize, usize)>)],
    }

    const TEST_DATA: &[TestData] = &[
        TestData {
            pattern: "(A*B|AC)D",
            name: "Sedgewick",
            end_states: &[StateSetID::new(5)],
            match_data: &[
                ("AAABD", Some((0, 5))),
                ("ACD", Some((0, 3))),
                ("CDAABCAAABD", None),
                ("CDAABC", None),
            ],
        },
        TestData {
            pattern: r#"\u{0022}(\\[\u{0022}\\/bfnrt]|u[0-9a-fA-F]{4}|[^\u{0022}\\\u0000-\u001F])*\u{0022}"#,
            name: "JsonString",
            end_states: &[StateSetID::new(2)],
            match_data: &[
                (r#""autumn""#, Some((0, 8))),
                (r#""au0075tumn""#, Some((0, 12))),
                (r#""au007xtumn""#, Some((0, 12))),
            ],
        },
        TestData {
            pattern: r"[a-zA-Z_]\w*",
            name: "Identifier",
            end_states: &[StateSetID::new(1), StateSetID::new(2)],
            match_data: &[
                ("_a", Some((0, 2))),
                ("a", Some((0, 1))),
                ("a0", Some((0, 2))),
                ("a0_", Some((0, 3))),
                ("0a", None),
                ("0", None),
            ],
        },
        TestData {
            pattern: r"(0|1)*1(0|1)",
            name: "SecondLastBitIs1",
            end_states: &[StateSetID::new(4), StateSetID::new(5)],
            match_data: &[
                ("11010", Some((0, 5))),
                ("11011", Some((0, 5))),
                ("110", Some((0, 3))),
                ("111", Some((0, 3))),
                ("10", Some((0, 2))),
                ("11", Some((0, 2))),
                ("1101", Some((0, 3))),
                ("1100", Some((0, 3))),
                ("1", None),
                ("0", None),
            ],
        },
    ];

    #[test]
    fn test_find_from() {
        init();
        for test in TEST_DATA {
            let pattern = Pattern::new(test.pattern.to_string(), 0);
            let mut character_class_registry = CharacterClassRegistry::new();
            let ast = parse_regex_syntax(pattern.pattern()).unwrap();
            let nfa: Nfa = Nfa::try_from_ast(ast, &mut character_class_registry).unwrap();
            nfa_render_to!(&nfa, test.name);
            let compiled_nfa = CompiledNfa::from(nfa);
            assert_eq!(
                compiled_nfa.end_states, test.end_states,
                "Test '{}', End states",
                test.name
            );
            compiled_nfa_render_to!(&compiled_nfa, test.name, &character_class_registry);
            eprintln!("{}", compiled_nfa);

            for (id, (input, expected)) in test.match_data.iter().enumerate() {
                let char_indices = input.char_indices();
                trace!("Matching string: {}", input);
                let match_char_class = character_class_registry.create_match_char_class().unwrap();
                let span = compiled_nfa.find_from(char_indices, &match_char_class);
                assert_eq!(
                    span,
                    expected.map(|(start, end)| Span::new(start, end)),
                    "Test '{}', Match data #{}, input '{}'",
                    test.name,
                    id,
                    input
                );
            }
        }
    }
}
