use std::collections::VecDeque;

use crate::{internal::nfa::Nfa, Span};

use super::{CharClassID, StateID};

/// A compiled NFA.
/// It is used to represent the NFA in a way that is optimized for matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompiledNfa {
    pub(crate) pattern: String,
    pub(crate) states: Vec<StateData>,
    // Used during NFA construction
    pub(crate) start_state: StateID,
    // Used during NFA construction
    pub(crate) end_state: StateID,
}

impl CompiledNfa {
    /// Simulates the NFA on the given input.
    /// Returns a match starting at the current position. No try on next character is done.
    /// The caller must do that.
    ///
    /// If no match is found, None is returned.
    ///
    /// We use the algorithm described in the book "Algorithms" by Robert Sedgewick.
    /// The algorithm is a non-deterministic finite automaton (NFA) simulation.
    /// The algorithm uses a queue to keep track of the states that are currently active.
    /// The algorithm processes the states in the queue in a loop.
    /// For each state, the algorithm checks if there is a transition on the current character.
    /// If there is a transition, the algorithm adds the target state to the queue.
    /// If there is no transition, the algorithm discards the state.
    /// The algorithm also processes epsilon transitions.
    /// The algorithm stops when the end state is reached or when there are no more states to process.
    /// The algorithm returns the span of the match if the end state is reached.
    /// Otherwise, the algorithm returns None.
    /// The algorithm is implemented in a way that is similar to the Thompson's construction algorithm.
    /// The algorithm is not optimized for performance yet.
    ///
    #[allow(dead_code)]
    pub(crate) fn find_from(
        &self,
        char_indices: std::str::CharIndices,
        match_char_class: &(dyn Fn(CharClassID, char) -> bool + 'static),
    ) -> Option<Span> {
        const SCAN: usize = usize::MAX;
        let mut state = self.start_state.as_usize();
        let mut dequeue = VecDeque::<usize>::new();

        let put = |dequeue: &mut VecDeque<usize>, state: usize| {
            if !dequeue.contains(&state) {
                // trace!("Push back: {}", state);
                dequeue.push_back(state);
                // trace!("  Dequeue: {:?}", dequeue);
            }
        };

        let push = |dequeue: &mut VecDeque<usize>, state: usize| {
            if !dequeue.contains(&state) {
                // trace!("Push front: {}", state);
                dequeue.push_front(state);
                // trace!("  Dequeue: {:?}", dequeue);
            }
        };

        let pop = |dequeue: &mut VecDeque<usize>, current_state| {
            dequeue.pop_front().unwrap_or(current_state)
            // let next_state = dequeue.pop_front().unwrap_or(current_state);
            // trace!("Pop: {}", next_state);
            // trace!("  Dequeue: {:?}", dequeue);
            // next_state
        };

        let mut start_index = None;
        let mut end_index = None;
        'FOR: for (j, aj) in char_indices {
            // state = self.start_state.as_usize();
            put(&mut dequeue, SCAN);
            // trace!("----- Character: ({}, '{}')", j, aj);
            // The inner loop processes the states in the dequeue
            loop {
                // trace!("State: {}", state);
                if state == SCAN {
                    // We read the next character from the input in the outer loop
                    state = pop(&mut dequeue, state);
                    break;
                } else if let Some(cc) = self.states[state].character_class {
                    if match_char_class(cc, aj) {
                        // Transition on character class
                        // trace!("Matched character class: {} on {}", cc.as_usize(), aj);
                        let n1 = self.states[state].next1.as_usize();
                        end_index = Some(j);
                        put(&mut dequeue, n1);
                        if start_index.is_none() {
                            // trace!("* Setting start index to {}", j);
                            start_index = Some(j);
                        }
                    } else {
                        // No transition, state is discarded
                        // trace!("No match of character class: {} on {}", cc.as_usize(), aj);
                        // trace!("~~~ Discarding state {}", state);
                    }
                } else {
                    // Epsilon transition(s)
                    // trace!("Process Epsilon transition(s) at state {}:", state);
                    let n1 = self.states[state].next1.as_usize();
                    let n2 = self.states[state].next2.as_usize();
                    push(&mut dequeue, n1);
                    if n1 != n2 {
                        push(&mut dequeue, n2);
                    }
                }
                state = pop(&mut dequeue, state);
                if state == self.end_state.as_usize() {
                    end_index = Some(j);
                    // trace!(
                    //     "Match found: {}-{} in state {}",
                    //     start_index.unwrap(),
                    //     end_index.unwrap() + 1,
                    //     state
                    // );
                    break 'FOR;
                }
                if dequeue.is_empty() {
                    // trace!("No more states to process");
                    if end_index.is_none() {
                        // trace!("* Resetting start index");
                        start_index = None;
                    }
                    break 'FOR;
                }
            }
        }
        if state == self.end_state.as_usize() {
            // trace!(
            //     "Returning match: {}-{}",
            //     start_index.unwrap(),
            //     end_index.unwrap() + 1
            // );
            Some(Span::new(start_index.unwrap(), end_index.unwrap() + 1))
        } else {
            // trace!("No match found. State is {}", state);
            None
        }
    }
}

impl From<&Nfa> for CompiledNfa {
    fn from(nfa: &Nfa) -> Self {
        let mut states = Vec::with_capacity(nfa.states.len());
        for _ in &nfa.states {
            states.push(StateData::default());
        }
        for state in &nfa.states {
            debug_assert!(state.transitions().len() <= 1);
            if !state.transitions().is_empty() {
                let transition = &state.transitions()[0];
                let next1 = transition.target_state();
                states[state.id()] = StateData::new(Some(transition.char_class()), next1, next1);
            } else if !state.epsilon_transitions().is_empty() {
                let next1 = state.epsilon_transitions()[0].target_state();
                let next2 = if state.epsilon_transitions().len() > 1 {
                    state.epsilon_transitions()[1].target_state()
                } else {
                    next1
                };
                states[state.id()] = StateData::new(None, next1, next2);
            }
        }
        Self {
            pattern: nfa.pattern.clone(),
            states,
            start_state: nfa.start_state,
            end_state: nfa.end_state,
        }
    }
}

impl std::fmt::Display for CompiledNfa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Pattern: {}", self.pattern)?;
        writeln!(f, "Start state: {}", self.start_state)?;
        writeln!(f, "End state: {}", self.end_state)?;
        for (i, state) in self.states.iter().enumerate() {
            writeln!(f, "State {}: {}", i, state)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct StateData {
    pub(crate) character_class: Option<CharClassID>,
    pub(crate) next1: StateID,
    pub(crate) next2: StateID,
}

impl StateData {
    pub(crate) fn new(
        character_class: Option<CharClassID>,
        next1: StateID,
        next2: StateID,
    ) -> Self {
        Self {
            character_class,
            next1,
            next2,
        }
    }
}

impl std::fmt::Display for StateData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Character class: {:?}, Next1: {}, Next2: {}",
            if let Some(cl) = self.character_class.as_ref() {
                cl.as_usize().to_string()
            } else {
                "-".to_string()
            },
            self.next1.as_usize(),
            self.next2.as_usize()
        )
    }
}

#[cfg(test)]
mod tests {
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

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_find_from() {
        init();
        let pattern = Pattern::new("(A*B|AC)D".to_string(), 0);
        let mut character_class_registry = CharacterClassRegistry::new();
        let ast = parse_regex_syntax(pattern.pattern()).unwrap();
        let nfa: Nfa = Nfa::try_from_ast(ast, &mut character_class_registry).unwrap();
        nfa_render_to!(&nfa, "TestFindFrom");
        let compiled_nfa = CompiledNfa::from(&nfa);
        eprintln!("{}", compiled_nfa);

        let char_indices = "AAABD".char_indices();
        trace!("Matching string: AAABD");
        let match_char_class = character_class_registry.create_match_char_class().unwrap();
        let span = compiled_nfa.find_from(char_indices, &match_char_class);
        assert_eq!(span, Some(Span::new(0, 5)));

        let char_indices = "ACD".char_indices();
        trace!("Matching string: ACD");
        let match_char_class = character_class_registry.create_match_char_class().unwrap();
        let span = compiled_nfa.find_from(char_indices, &match_char_class);
        assert_eq!(span, Some(Span::new(0, 3)));

        let char_indices = "CDAABCAAABD".char_indices();
        trace!("Matching string: CDAABCAAABD");
        let match_char_class = character_class_registry.create_match_char_class().unwrap();
        let span = compiled_nfa.find_from(char_indices, &match_char_class);
        assert_eq!(span, None);

        let char_indices = "CDAABC".char_indices();
        trace!("Matching string: CDAABC");
        let match_char_class = character_class_registry.create_match_char_class().unwrap();
        let span = compiled_nfa.find_from(char_indices, &match_char_class);
        assert_eq!(span, None);
    }

    #[test]
    fn test_find_from_with_string_pattern() {
        init();
        let pattern = Pattern::new(
            r#"\u{0022}(\\[\u{0022}\\/bfnrt]|u[0-9a-fA-F]{4}|[^\u{0022}\\\u0000-\u001F])*\u{0022}"#
                .to_string(),
            0,
        );
        let mut character_class_registry = CharacterClassRegistry::new();
        let ast = parse_regex_syntax(pattern.pattern()).unwrap();
        let nfa: Nfa = Nfa::try_from_ast(ast, &mut character_class_registry).unwrap();
        nfa_render_to!(&nfa, "TestFindFromWithStringPattern");
        let compiled_nfa = CompiledNfa::from(&nfa);
        eprintln!("{}", compiled_nfa);

        let char_indices = r#""autumn""#.char_indices();
        trace!("Matching string: autumn");
        let match_char_class = character_class_registry.create_match_char_class().unwrap();
        let span = compiled_nfa.find_from(char_indices, &match_char_class);
        assert_eq!(span, Some(Span::new(0, 8)));

        let char_indices = r#""au0075tumn""#.char_indices();
        trace!("Matching string: au0075tumn");
        let match_char_class = character_class_registry.create_match_char_class().unwrap();
        let span = compiled_nfa.find_from(char_indices, &match_char_class);
        assert_eq!(span, Some(Span::new(0, 12)));

        let char_indices = r#""au007xtumn""#.char_indices();
        trace!("Matching string: au007xtumn");
        let match_char_class = character_class_registry.create_match_char_class().unwrap();
        let span = compiled_nfa.find_from(char_indices, &match_char_class);
        assert_eq!(span, Some(Span::new(0, 12)));
    }
}
