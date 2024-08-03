//! This module contains the NFA (Non-deterministic Finite Automaton) implementation.
//! The NFA is used to represent the regex syntax as a finite automaton.
//! The NFA is later converted to a DFA (Deterministic Finite Automaton) for matching strings.

use std::vec;

use regex_syntax::ast::{Ast, RepetitionKind, RepetitionRange};

use crate::{Result, ScnrError};

use super::{
    ids::{CharClassIDBase, StateIDBase},
    CharClassID, CharacterClass, ComparableAst, StateID,
};

#[derive(Debug, Clone, Default)]
pub(crate) struct Nfa {
    pub(crate) pattern: String,
    pub(crate) states: Vec<NfaState>,
    // Used during NFA construction
    pub(crate) start_state: StateID,
    // Used during NFA construction
    pub(crate) end_state: StateID,
    // Character classes that are used in the NFA
    pub(crate) char_classes: Vec<CharacterClass>,
}

impl Nfa {
    pub(crate) fn new() -> Self {
        Self {
            pattern: String::new(),
            states: vec![NfaState::default()],
            start_state: StateID::default(),
            end_state: StateID::default(),
            char_classes: Vec::new(),
        }
    }

    // Returns true if the NFA is empty, i.e. no states and no transitions have been added.
    pub(crate) fn is_empty(&self) -> bool {
        self.start_state == StateID::default()
            && self.end_state == StateID::default()
            && self.states.len() == 1
            && self.states[0].is_empty()
    }

    pub(crate) fn start_state(&self) -> StateID {
        self.start_state
    }

    pub(crate) fn end_state(&self) -> StateID {
        self.end_state
    }

    pub(crate) fn states(&self) -> &[NfaState] {
        &self.states
    }

    /// Get the character classes.
    pub(crate) fn char_classes(&self) -> &[CharacterClass] {
        &self.char_classes
    }

    #[allow(dead_code)]
    pub(crate) fn pattern(&self) -> &str {
        &self.pattern
    }

    pub(crate) fn set_pattern(&mut self, pattern: &str) {
        self.pattern = pattern.to_string();
    }

    pub(crate) fn add_state(&mut self, state: NfaState) {
        self.states.push(state);
    }

    pub(crate) fn set_start_state(&mut self, state: StateID) {
        self.start_state = state;
    }

    pub(crate) fn set_end_state(&mut self, state: StateID) {
        self.end_state = state;
    }

    fn register_char_class(&mut self, ast: ComparableAst) -> CharClassID {
        if let Some(class_id) = self.char_classes.iter().position(|c| c.ast == ast) {
            CharClassID::new(class_id as CharClassIDBase)
        } else {
            let class_id = CharClassID::new(self.char_classes.len() as CharClassIDBase);
            self.char_classes
                .push(CharacterClass::new(class_id, ast.0.clone()));
            class_id
        }
    }

    pub(crate) fn add_transition(&mut self, from: StateID, chars: Ast, target_state: StateID) {
        let char_class = self.register_char_class(ComparableAst(chars.clone()));
        self.states[from].transitions.push(NfaTransition {
            ast: ComparableAst(chars),
            char_class,
            target_state,
        });
    }

    pub(crate) fn add_epsilon_transition(&mut self, from: StateID, target_state: StateID) {
        self.states[from]
            .epsilon_transitions
            .push(EpsilonTransition { target_state });
    }

    pub(crate) fn new_state(&mut self) -> StateID {
        let state = StateID::new(self.states.len() as StateIDBase);
        self.add_state(NfaState::new(state));
        state
    }

    /// Apply an offset to every state number.
    pub(crate) fn shift_ids(&mut self, offset: usize) -> (StateID, StateID) {
        for state in self.states.iter_mut() {
            state.offset(offset);
        }
        self.start_state = StateID::new(self.start_state.id() + offset as StateIDBase);
        self.end_state = StateID::new(self.end_state.id() + offset as StateIDBase);
        (self.start_state, self.end_state)
    }

    /// Concatenates the current NFA with another NFA.
    pub(crate) fn concat(&mut self, mut nfa: Nfa) {
        if self.is_empty() {
            // If the current NFA is empty, set the start and end states of the current NFA to the
            // start and end states of the new NFA
            self.set_start_state(nfa.start_state);
            self.set_end_state(nfa.end_state);
            self.states = nfa.states;
            return;
        }

        // Apply an offset to the state numbers of the given NFA
        let (nfa_start_state, nfa_end_state) = nfa.shift_ids(self.states.len());
        // Move the states of the given NFA to the current NFA
        self.append(nfa);

        // Connect the end state of the current NFA to the start state of the new NFA
        self.add_epsilon_transition(self.end_state, nfa_start_state);

        // Update the end state of the current NFA to the end state of the new NFA
        self.set_end_state(nfa_end_state);
    }

    pub(crate) fn alternation(&mut self, mut nfa: Nfa) {
        if self.is_empty() {
            // If the current NFA is empty, set the start and end states of the current NFA to the
            // start and end states of the new NFA
            self.set_start_state(nfa.start_state);
            self.set_end_state(nfa.end_state);
            self.states = nfa.states;
            return;
        }

        // Apply an offset to the state numbers of the given NFA
        let (nfa_start_state, nfa_end_state) = nfa.shift_ids(self.states.len());

        // Move the states of given the NFA to the current NFA
        self.append(nfa);

        // Create a new start state
        let start_state = self.new_state();
        // Connect the new start state to the start state of the current NFA
        self.add_epsilon_transition(start_state, self.start_state);
        // Connect the new start state to the start state of the new NFA
        self.add_epsilon_transition(start_state, nfa_start_state);

        // Create a new end state
        let end_state = self.new_state();
        // Connect the end state of the current NFA to the new end state
        self.add_epsilon_transition(self.end_state, end_state);
        // Connect the end state of the new NFA to the new end state
        self.add_epsilon_transition(nfa_end_state, end_state);

        // Update the start and end states of the current NFA
        self.set_start_state(start_state);
        self.set_end_state(end_state);
    }

    pub(crate) fn zero_or_one(&mut self) {
        // Create a new start state
        let start_state = self.new_state();
        // Connect the new start state to the start state of the current NFA
        self.add_epsilon_transition(start_state, self.start_state);
        // Connect the new start state to the end state of the current NFA
        self.add_epsilon_transition(start_state, self.end_state);

        // Update the start and end states of the current NFA
        self.set_start_state(start_state);
    }

    pub(crate) fn one_or_more(&mut self) {
        // Create a new start state
        let start_state = self.new_state();
        // Connect the new start state to the start state of the current NFA
        self.add_epsilon_transition(start_state, self.start_state);

        // Create a new end state
        let end_state = self.new_state();
        // Connect the end state of the current NFA to the new end state
        self.add_epsilon_transition(self.end_state, end_state);
        // Connect the end state of the current NFA to the start state of the current NFA
        self.add_epsilon_transition(self.end_state, self.start_state);

        // Update the start and end states of the current NFA
        self.set_start_state(start_state);
        self.set_end_state(end_state);
    }

    pub(crate) fn zero_or_more(&mut self) {
        // Create a new start state
        let start_state = self.new_state();
        // Connect the new start state to the start state of the current NFA
        self.add_epsilon_transition(start_state, self.start_state);
        // Connect the new start state to the end state of the current NFA
        self.add_epsilon_transition(start_state, self.end_state);

        // Create a new end state
        let end_state = self.new_state();
        // Connect the end state of the current NFA to the new end state
        self.add_epsilon_transition(self.end_state, end_state);
        // Connect the end state of the current NFA to the start state of the current NFA
        self.add_epsilon_transition(self.end_state, self.start_state);

        // Update the start and end states of the current NFA
        self.set_start_state(start_state);
        self.set_end_state(end_state);
    }

    /// Move the states of the given NFA to the current NFA and thereby consume the NFA.
    pub(crate) fn append(&mut self, mut nfa: Nfa) {
        self.states.append(nfa.states.as_mut());
        // Check the index constraints
        debug_assert!(self
            .states
            .iter()
            .enumerate()
            .all(|(i, s)| s.id().as_usize() == i));
    }

    /// Calculate the epsilon closure of a state.
    pub(crate) fn epsilon_closure(&self, state: StateID) -> Vec<StateID> {
        // The state itself is always part of the ε-closure
        let mut closure = vec![state];
        let mut i = 0;
        while i < closure.len() {
            let current_state = closure[i];
            for epsilon_transition in self.states[current_state].epsilon_transitions() {
                if !closure.contains(&epsilon_transition.target_state()) {
                    closure.push(epsilon_transition.target_state());
                }
            }
            i += 1;
        }
        closure
    }

    /// Calculate the epsilon closure of a set of states and return the unique states.
    pub(crate) fn epsilon_closure_set<I>(&self, states: I) -> Vec<StateID>
    where
        I: IntoIterator<Item = StateID>,
    {
        let mut closure: Vec<StateID> = states.into_iter().collect();
        let mut i = 0;
        while i < closure.len() {
            let current_state = closure[i];
            for epsilon_transition in self.states[current_state].epsilon_transitions() {
                if !closure.contains(&epsilon_transition.target_state()) {
                    closure.push(epsilon_transition.target_state());
                }
            }
            i += 1;
        }
        closure.sort_unstable();
        closure.dedup();
        closure
    }

    /// Calculate move(T, a) for a set of states T and a character class a.
    /// This is the set of states that can be reached from T by matching a.
    pub(crate) fn move_set(&self, states: &[StateID], char_class: CharClassID) -> Vec<StateID> {
        let mut move_set = Vec::new();
        for state in states {
            for transition in self.states()[*state].transitions() {
                if transition.char_class() == char_class {
                    move_set.push(transition.target_state());
                }
            }
        }
        move_set
    }
}

macro_rules! unsupported {
    ($feature:expr) => {
        ScnrError::new($crate::ScnrErrorKind::UnsupportedFeature(
            $feature.to_string(),
        ))
    };
}

impl TryFrom<Ast> for Nfa {
    type Error = ScnrError;

    fn try_from(ast: Ast) -> Result<Self> {
        let mut nfa = Nfa::new();
        nfa.set_pattern(&ast.to_string());
        match ast {
            Ast::Empty(_) => Ok(nfa),
            Ast::Flags(_) => Err(unsupported!(format!("{:?}", ast))),
            Ast::Literal(ref l) => {
                let start_state = nfa.end_state();
                let end_state = nfa.new_state();
                nfa.set_end_state(end_state);
                nfa.add_transition(start_state, Ast::Literal(l.clone()), end_state);
                Ok(nfa)
            }
            Ast::Dot(ref d) => {
                let start_state = nfa.end_state();
                let end_state = nfa.new_state();
                nfa.set_end_state(end_state);
                nfa.add_transition(start_state, Ast::Dot(d.clone()), end_state);
                Ok(nfa)
            }
            Ast::Assertion(ref a) => Err(unsupported!(format!("Assertion {:?}", a.kind))),
            Ast::ClassUnicode(_) | Ast::ClassPerl(_) | Ast::ClassBracketed(_) => {
                let start_state = nfa.end_state();
                let end_state = nfa.new_state();
                nfa.set_end_state(end_state);
                nfa.add_transition(start_state, ast.clone(), end_state);
                Ok(nfa)
            }
            Ast::Repetition(ref r) => {
                let mut nfa2: Nfa = r.ast.as_ref().clone().try_into()?;
                match &r.op.kind {
                    RepetitionKind::ZeroOrOne => {
                        nfa2.zero_or_one();
                        nfa = nfa2;
                    }
                    RepetitionKind::ZeroOrMore => {
                        nfa2.zero_or_more();
                        nfa = nfa2;
                    }
                    RepetitionKind::OneOrMore => {
                        nfa2.one_or_more();
                        nfa = nfa2;
                    }
                    RepetitionKind::Range(r) => match r {
                        RepetitionRange::Exactly(c) => {
                            for _ in 0..*c {
                                nfa.concat(nfa2.clone());
                            }
                        }
                        RepetitionRange::AtLeast(c) => {
                            for _ in 0..*c {
                                nfa.concat(nfa2.clone());
                            }
                            let mut nfa_zero_or_more: Nfa = nfa2.clone();
                            nfa_zero_or_more.zero_or_more();
                            nfa.concat(nfa_zero_or_more);
                        }
                        RepetitionRange::Bounded(least, most) => {
                            for _ in 0..*least {
                                nfa.concat(nfa2.clone());
                            }
                            let mut nfa_zero_or_one: Nfa = nfa2.clone();
                            nfa_zero_or_one.zero_or_one();
                            for _ in *least..*most {
                                nfa.concat(nfa_zero_or_one.clone());
                            }
                        }
                    },
                }
                Ok(nfa)
            }
            Ast::Group(ref g) => {
                nfa = g.ast.as_ref().clone().try_into()?;
                Ok(nfa)
            }
            Ast::Alternation(ref a) => {
                for ast in a.asts.iter() {
                    let nfa2: Nfa = ast.clone().try_into()?;
                    nfa.alternation(nfa2);
                }
                Ok(nfa)
            }
            Ast::Concat(ref c) => {
                for ast in c.asts.iter() {
                    let nfa2: Nfa = ast.clone().try_into()?;
                    nfa.concat(nfa2);
                }
                Ok(nfa)
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct NfaState {
    state: StateID,
    epsilon_transitions: Vec<EpsilonTransition>,
    transitions: Vec<NfaTransition>,
}

impl NfaState {
    pub(crate) fn new(state: StateID) -> Self {
        Self {
            state,
            epsilon_transitions: Vec::new(),
            transitions: Vec::new(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.transitions.is_empty() && self.epsilon_transitions.is_empty()
    }

    pub(crate) fn id(&self) -> StateID {
        self.state
    }

    pub(crate) fn transitions(&self) -> &[NfaTransition] {
        &self.transitions
    }

    pub(crate) fn epsilon_transitions(&self) -> &[EpsilonTransition] {
        &self.epsilon_transitions
    }

    /// Apply an offset to every state number.
    pub(crate) fn offset(&mut self, offset: usize) {
        self.state = StateID::new(self.state.id() + offset as StateIDBase);
        for transition in self.transitions.iter_mut() {
            transition.target_state =
                StateID::new(transition.target_state.id() + offset as StateIDBase);
        }
        for epsilon_transition in self.epsilon_transitions.iter_mut() {
            epsilon_transition.target_state =
                StateID::new(epsilon_transition.target_state.id() + offset as StateIDBase);
        }
    }
}

/// A transition in the NFA.
#[derive(Debug, Clone)]
pub(crate) struct NfaTransition {
    /// This can be a Literal or a CharacterClass
    /// We will later generate a predicate from this that determines if a character matches this
    /// transition
    ast: ComparableAst,
    /// The next state to transition to
    target_state: StateID,
    /// The characters to match, is filled in after the all DFAs have been constructed and the
    /// character classes are known
    char_class: CharClassID,
}

impl NfaTransition {
    pub(crate) fn target_state(&self) -> StateID {
        self.target_state
    }

    pub(crate) fn ast(&self) -> &ComparableAst {
        &self.ast
    }

    pub(crate) fn char_class(&self) -> CharClassID {
        self.char_class
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct EpsilonTransition {
    pub(crate) target_state: StateID,
}

impl EpsilonTransition {
    pub(crate) fn target_state(&self) -> StateID {
        self.target_state
    }
}

impl From<StateID> for EpsilonTransition {
    fn from(state: StateID) -> Self {
        Self {
            target_state: state,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::internal::parse_regex_syntax;

    use super::*;

    #[test]
    fn test_nfa_from_ast() {
        // Create an example AST
        let ast = parse_regex_syntax("a").unwrap();

        // Convert the AST to an NFA
        let nfa: Nfa = ast.try_into().unwrap();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 2);
        assert_eq!(nfa.start_state.as_usize(), 0);
        assert_eq!(nfa.end_state.as_usize(), 1);
    }

    #[test]
    fn test_nfa_from_ast_concat() {
        // Create an example AST and convert the AST to an NFA
        let nfa: Nfa = parse_regex_syntax("ab").unwrap().try_into().unwrap();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.start_state.as_usize(), 0);
        assert_eq!(nfa.end_state.as_usize(), 3);
    }

    #[test]
    fn test_nfa_concat() {
        // Create two example ASTs and convert them to an NFAs
        let mut nfa1: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        let nfa2: Nfa = parse_regex_syntax("b").unwrap().try_into().unwrap();
        nfa1.concat(nfa2);

        // Add assertions here to validate the NFA
        assert_eq!(nfa1.states.len(), 4);
        assert_eq!(nfa1.start_state.as_usize(), 0);
        assert_eq!(nfa1.end_state.as_usize(), 3);
    }

    #[test]
    fn test_nfa_alternation() {
        // Create two example ASTs and convert them to an NFAs
        let mut nfa1: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        let nfa2: Nfa = parse_regex_syntax("b").unwrap().try_into().unwrap();
        nfa1.alternation(nfa2);

        // Add assertions here to validate the NFA
        assert_eq!(nfa1.states.len(), 6);
        assert_eq!(nfa1.start_state.as_usize(), 4);
        assert_eq!(nfa1.end_state.as_usize(), 5);
    }

    #[test]
    fn test_nfa_repetition() {
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        nfa.zero_or_more();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.start_state.as_usize(), 2);
        assert_eq!(nfa.end_state.as_usize(), 3);
    }

    #[test]
    fn test_nfa_zero_or_one() {
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        nfa.zero_or_one();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 3);
        assert_eq!(nfa.start_state.as_usize(), 2);
        assert_eq!(nfa.end_state.as_usize(), 1);
    }

    #[test]
    fn test_nfa_one_or_more() {
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        nfa.one_or_more();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.start_state.as_usize(), 2);
        assert_eq!(nfa.end_state.as_usize(), 3);
    }

    #[test]
    fn test_nfa_zero_or_more() {
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        nfa.zero_or_more();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.start_state.as_usize(), 2);
        assert_eq!(nfa.end_state.as_usize(), 3);
    }

    #[test]
    fn test_complex_nfa() {
        // Create an example AST and convert the AST to an NFA
        let nfa: Nfa = parse_regex_syntax("(a|b)*abb").unwrap().try_into().unwrap();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 14);
        assert_eq!(nfa.start_state.as_usize(), 6);
        assert_eq!(nfa.end_state.as_usize(), 13);
    }

    #[test]
    fn test_nfa_offset_states() {
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa = parse_regex_syntax("a").unwrap().try_into().unwrap();
        nfa.shift_ids(10);

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 2);
        assert_eq!(nfa.start_state.as_usize(), 10);
        assert_eq!(nfa.end_state.as_usize(), 11);
    }

    #[test]
    fn test_nfa_repetition_at_least() {
        // Create an example AST and convert the AST to an NFA
        let nfa: Nfa = parse_regex_syntax("a{3,}").unwrap().try_into().unwrap();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 10);
        assert_eq!(nfa.start_state.as_usize(), 0);
        assert_eq!(nfa.end_state.as_usize(), 9);
    }

    #[test]
    fn test_nfa_repetition_bounded() {
        // Create an example AST and convert the AST to an NFA
        let nfa: Nfa = parse_regex_syntax("a{3,5}").unwrap().try_into().unwrap();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 12);
        assert_eq!(nfa.start_state.as_usize(), 0);
        assert_eq!(nfa.end_state.as_usize(), 10);
    }

    // Ascii character class are not yet implemented
    #[test]
    fn test_character_class_expression() {
        // Create an example AST and convert the AST to an NFA
        let nfa: Nfa = parse_regex_syntax(r"[[:digit:]]")
            .unwrap()
            .try_into()
            .unwrap();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 2);
        assert_eq!(nfa.start_state.as_usize(), 0);
        assert_eq!(nfa.end_state.as_usize(), 1);
    }
}

#[cfg(test)]
mod tests_try_from {
    use std::convert::TryFrom;

    use crate::internal::parse_regex_syntax;

    use super::*;

    struct TestData {
        input: &'static str,
        expected_states: usize,
        expected_start_state: usize,
        expected_end_state: usize,
    }

    const TEST_DATA: [TestData; 7] = [
        TestData {
            input: "a",
            expected_states: 2,
            expected_start_state: 0,
            expected_end_state: 1,
        },
        TestData {
            input: "ab",
            expected_states: 4,
            expected_start_state: 0,
            expected_end_state: 3,
        },
        TestData {
            input: "a|b",
            expected_states: 6,
            expected_start_state: 4,
            expected_end_state: 5,
        },
        TestData {
            input: "a*",
            expected_states: 4,
            expected_start_state: 2,
            expected_end_state: 3,
        },
        TestData {
            input: "a?",
            expected_states: 3,
            expected_start_state: 2,
            expected_end_state: 1,
        },
        TestData {
            input: "a+",
            expected_states: 4,
            expected_start_state: 2,
            expected_end_state: 3,
        },
        TestData {
            input: "(a|b)*abb",
            expected_states: 14,
            expected_start_state: 6,
            expected_end_state: 13,
        },
    ];

    #[test]
    fn test_try_from_ast() {
        for data in TEST_DATA.iter() {
            let ast = parse_regex_syntax(data.input).unwrap();
            let nfa: Nfa = Nfa::try_from(ast).unwrap();

            assert_eq!(
                nfa.states.len(),
                data.expected_states,
                "input: {}",
                data.input
            );
            assert_eq!(
                nfa.start_state.as_usize(),
                data.expected_start_state,
                "input: {}",
                data.input
            );
            assert_eq!(
                nfa.end_state.as_usize(),
                data.expected_end_state,
                "input: {}",
                data.input
            );
        }
    }
}
