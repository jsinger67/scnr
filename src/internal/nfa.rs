//! This module contains the NFA (Non-deterministic Finite Automaton) implementation.
//! The NFA is used to represent the regex syntax as a finite automaton.

use std::vec;

use regex_syntax::ast::{Ast, FlagsItemKind, GroupKind, RepetitionKind, RepetitionRange};

use crate::{Pattern, Result, ScnrError};

use super::{ids::StateIDBase, CharClassID, CharacterClassRegistry, ComparableAst, StateID};

macro_rules! unsupported {
    ($feature:expr) => {
        ScnrError::new($crate::ScnrErrorKind::UnsupportedFeature(
            $feature.to_string(),
        ))
    };
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Nfa {
    /// The pattern that the NFA represents.
    pub(crate) pattern: Pattern,
    pub(crate) states: Vec<NfaState>,
    // Used during NFA construction
    pub(crate) start_state: StateID,
    // Used during NFA construction
    pub(crate) end_state: StateID,
}

impl Nfa {
    pub(crate) fn new() -> Self {
        Self {
            pattern: Pattern::default(),
            states: vec![NfaState::default()],
            start_state: StateID::default(),
            end_state: StateID::default(),
        }
    }

    // Returns true if the NFA is empty, i.e. no states and no transitions have been added.
    pub(crate) fn is_empty(&self) -> bool {
        self.start_state == StateID::default()
            && self.end_state == StateID::default()
            && self.states.len() == 1
            && self.states[0].is_empty()
    }

    /// Returns the start state of the NFA.
    /// It is used for debugging purposes mostly in the [crate::internal::dot] module.
    #[allow(unused)]
    pub(crate) fn start_state(&self) -> StateID {
        self.start_state
    }

    pub(crate) fn end_state(&self) -> StateID {
        self.end_state
    }

    pub(crate) fn states(&self) -> &[NfaState] {
        &self.states
    }

    #[allow(dead_code)]
    pub(crate) fn pattern(&self) -> &str {
        self.pattern.pattern()
    }

    pub(crate) fn set_pattern(&mut self, pattern: &str) {
        self.pattern = Pattern::new(pattern.to_string(), self.pattern.terminal_id());
    }

    pub(crate) fn set_terminal_id(&mut self, terminal_id: usize) {
        self.pattern.set_token_type(terminal_id);
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

    pub(crate) fn add_transition(
        &mut self,
        from: StateID,
        chars: Ast,
        target_state: StateID,
        char_class_registry: &mut CharacterClassRegistry,
    ) {
        let char_class = char_class_registry.add_character_class(&chars);
        self.states[from].transitions.push(NfaTransition {
            ast: ComparableAst(chars),
            char_class,
            target_state,
        });
        debug_assert!(
            self.states[from].epsilon_transitions.len() + self.states[from].transitions.len() <= 2
        );
    }

    pub(crate) fn add_epsilon_transition(&mut self, from: StateID, target_state: StateID) {
        self.states[from]
            .epsilon_transitions
            .push(EpsilonTransition { target_state });
        debug_assert!(
            self.states[from].epsilon_transitions.len() + self.states[from].transitions.len() <= 2
        );
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

    /// Returns the number of the highest state.
    /// If no states are present, it returns 0.
    /// It is used for debugging purposes during multi-pattern NFA construction.
    #[allow(dead_code)]
    pub(crate) fn highest_state_number(&self) -> StateIDBase {
        self.states
            .iter()
            .max_by(|x, y| x.id().cmp(&y.id()))
            .map_or(0, |s| s.id().id())
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

    pub(crate) fn try_from_ast(
        ast: Ast,
        char_class_registry: &mut CharacterClassRegistry,
    ) -> Result<Self> {
        let mut nfa = Nfa::new();
        nfa.set_pattern(&ast.to_string());
        match ast {
            Ast::Empty(_) => Ok(nfa),
            Ast::Flags(ref f) => Err(unsupported!(format!("{:?}", f.flags.items))),
            Ast::Literal(ref l) => {
                let start_state = nfa.end_state();
                let end_state = nfa.new_state();
                nfa.set_end_state(end_state);
                nfa.add_transition(
                    start_state,
                    Ast::Literal(l.clone()),
                    end_state,
                    char_class_registry,
                );
                Ok(nfa)
            }
            Ast::Dot(ref d) => {
                let start_state = nfa.end_state();
                let end_state = nfa.new_state();
                nfa.set_end_state(end_state);
                nfa.add_transition(
                    start_state,
                    Ast::Dot(d.clone()),
                    end_state,
                    char_class_registry,
                );
                Ok(nfa)
            }
            Ast::Assertion(ref a) => Err(unsupported!(format!("Assertion {:?}", a.kind))),
            Ast::ClassUnicode(_) | Ast::ClassPerl(_) | Ast::ClassBracketed(_) => {
                let start_state = nfa.end_state();
                let end_state = nfa.new_state();
                nfa.set_end_state(end_state);
                nfa.add_transition(start_state, ast.clone(), end_state, char_class_registry);
                Ok(nfa)
            }
            Ast::Repetition(ref r) => {
                let mut nfa2: Nfa = Self::try_from_ast((*r.ast).clone(), char_class_registry)?;
                if !r.greedy {
                    Err(unsupported!(
                        format!("{}: Non-greedy repetionions. Consider using different scanner modes instead.", ast)))?;
                }
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
                if let GroupKind::NonCapturing(flags) = &g.kind {
                    if flags
                        .items
                        .iter()
                        .any(|f| matches!(f.kind, FlagsItemKind::Flag(_)))
                    {
                        Err(unsupported!(format!(
                            "{:?}: Flags in non-capturing group",
                            flags.items
                        )))?;
                    }
                }
                nfa = Self::try_from_ast((*g.ast).clone(), char_class_registry)?;
                Ok(nfa)
            }
            Ast::Alternation(ref a) => {
                for ast in a.asts.iter() {
                    let nfa2: Nfa = Self::try_from_ast(ast.clone(), char_class_registry)?;
                    nfa.alternation(nfa2);
                }
                Ok(nfa)
            }
            Ast::Concat(ref c) => {
                for ast in c.asts.iter() {
                    let nfa2: Nfa = Self::try_from_ast(ast.clone(), char_class_registry)?;
                    nfa.concat(nfa2);
                }
                Ok(nfa)
            }
        }
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
        closure.sort_unstable();
        closure.dedup();
        closure
    }

    /// Calculate move(T, a) for a set of states T and a character class a.
    /// This is the set of states that can be reached from T by matching a.
    pub(crate) fn move_set(&self, states: &[StateID], char_class: CharClassID) -> Vec<StateID> {
        let mut move_set = Vec::new();
        for state in states {
            if let Some(state) = self.find_state(*state) {
                for transition in state.transitions() {
                    if transition.char_class() == char_class {
                        move_set.push(transition.target_state());
                    }
                }
            } else {
                panic!("State not found: {:?}", state);
            }
        }
        move_set.sort_unstable();
        move_set.dedup();
        move_set
    }

    pub(crate) fn get_match_transitions(
        &self,
        start_state: impl Iterator<Item = StateID>,
    ) -> Vec<(CharClassID, StateID)> {
        let mut target_states = Vec::new();
        for state in start_state {
            for transition in self.states()[state].transitions() {
                target_states.push((transition.char_class(), transition.target_state()));
            }
        }
        target_states.sort_unstable();
        target_states.dedup();
        target_states
    }

    pub(crate) fn contains_state(&self, state: StateID) -> bool {
        self.states.iter().any(|s| s.id() == state)
    }

    fn find_state(&self, state: StateID) -> Option<&NfaState> {
        self.states.iter().find(|s| s.id() == state)
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
    /// transition. It is used for debugging purposes mostly in the [crate::internal::dot] module.
    #[allow(unused)]
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

    #[allow(unused)]
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
    /// Create a new epsilon transition to the given state.
    #[inline]
    pub(crate) fn new(target_state: StateID) -> Self {
        Self { target_state }
    }

    #[inline]
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
    use std::collections::BTreeSet;

    use crate::internal::parser::parse_regex_syntax;

    use super::*;

    #[test]
    fn test_nfa_from_ast() {
        // Create an example AST
        let ast = parse_regex_syntax("a").unwrap();
        // Create a character class registry
        let mut char_class_registry = CharacterClassRegistry::new();

        // Convert the AST to an NFA
        let nfa: Nfa = Nfa::try_from_ast(ast, &mut char_class_registry).unwrap();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 2);
        assert_eq!(nfa.start_state.as_usize(), 0);
        assert_eq!(nfa.end_state.as_usize(), 1);
    }

    #[test]
    fn test_nfa_from_ast_concat() {
        // Create a character class registry
        let mut char_class_registry = CharacterClassRegistry::new();
        // Create an example AST and convert the AST to an NFA
        let nfa: Nfa =
            Nfa::try_from_ast(parse_regex_syntax("ab").unwrap(), &mut char_class_registry).unwrap();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.start_state.as_usize(), 0);
        assert_eq!(nfa.end_state.as_usize(), 3);
    }

    #[test]
    fn test_nfa_concat() {
        // Create a character class registry
        let mut char_class_registry = CharacterClassRegistry::new();
        // Create two example ASTs and convert them to an NFAs
        let mut nfa1: Nfa =
            Nfa::try_from_ast(parse_regex_syntax("a").unwrap(), &mut char_class_registry).unwrap();
        let nfa2: Nfa =
            Nfa::try_from_ast(parse_regex_syntax("b").unwrap(), &mut char_class_registry).unwrap();
        nfa1.concat(nfa2);

        // Add assertions here to validate the NFA
        assert_eq!(nfa1.states.len(), 4);
        assert_eq!(nfa1.start_state.as_usize(), 0);
        assert_eq!(nfa1.end_state.as_usize(), 3);
    }

    #[test]
    fn test_nfa_alternation() {
        // Create a character class registry
        let mut char_class_registry = CharacterClassRegistry::new();
        // Create two example ASTs and convert them to an NFAs
        let mut nfa1: Nfa =
            Nfa::try_from_ast(parse_regex_syntax("a").unwrap(), &mut char_class_registry).unwrap();
        let nfa2: Nfa =
            Nfa::try_from_ast(parse_regex_syntax("b").unwrap(), &mut char_class_registry).unwrap();
        nfa1.alternation(nfa2);

        // Add assertions here to validate the NFA
        assert_eq!(nfa1.states.len(), 6);
        assert_eq!(nfa1.start_state.as_usize(), 4);
        assert_eq!(nfa1.end_state.as_usize(), 5);
    }

    #[test]
    fn test_nfa_repetition() {
        // Create a character class registry
        let mut char_class_registry = CharacterClassRegistry::new();
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa =
            Nfa::try_from_ast(parse_regex_syntax("a").unwrap(), &mut char_class_registry).unwrap();

        nfa.zero_or_more();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.start_state.as_usize(), 2);
        assert_eq!(nfa.end_state.as_usize(), 3);
    }

    #[test]
    fn test_nfa_zero_or_one() {
        // Create a character class registry
        let mut char_class_registry = CharacterClassRegistry::new();
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa =
            Nfa::try_from_ast(parse_regex_syntax("a").unwrap(), &mut char_class_registry).unwrap();
        nfa.zero_or_one();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 3);
        assert_eq!(nfa.start_state.as_usize(), 2);
        assert_eq!(nfa.end_state.as_usize(), 1);
    }

    #[test]
    fn test_nfa_one_or_more() {
        // Create a character class registry
        let mut char_class_registry = CharacterClassRegistry::new();
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa =
            Nfa::try_from_ast(parse_regex_syntax("a").unwrap(), &mut char_class_registry).unwrap();
        nfa.one_or_more();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.start_state.as_usize(), 2);
        assert_eq!(nfa.end_state.as_usize(), 3);
    }

    #[test]
    fn test_nfa_zero_or_more() {
        // Create a character class registry
        let mut char_class_registry = CharacterClassRegistry::new();
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa =
            Nfa::try_from_ast(parse_regex_syntax("a").unwrap(), &mut char_class_registry).unwrap();
        nfa.zero_or_more();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.start_state.as_usize(), 2);
        assert_eq!(nfa.end_state.as_usize(), 3);
    }

    #[test]
    fn test_complex_nfa() {
        // Create a character class registry
        let mut char_class_registry = CharacterClassRegistry::new();
        // Create an example AST and convert the AST to an NFA
        let nfa: Nfa = Nfa::try_from_ast(
            parse_regex_syntax("(a|b)*abb").unwrap(),
            &mut char_class_registry,
        )
        .unwrap();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 14);
        assert_eq!(nfa.start_state.as_usize(), 6);
        assert_eq!(nfa.end_state.as_usize(), 13);
    }

    #[test]
    fn test_nfa_offset_states() {
        // Create a character class registry
        let mut char_class_registry = CharacterClassRegistry::new();
        // Create an example AST and convert the AST to an NFA
        let mut nfa: Nfa =
            Nfa::try_from_ast(parse_regex_syntax("a").unwrap(), &mut char_class_registry).unwrap();
        nfa.shift_ids(10);

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 2);
        assert_eq!(nfa.start_state.as_usize(), 10);
        assert_eq!(nfa.end_state.as_usize(), 11);
    }

    #[test]
    fn test_nfa_repetition_at_least() {
        // Create a character class registry
        let mut char_class_registry = CharacterClassRegistry::new();
        // Create an example AST and convert the AST to an NFA
        let nfa: Nfa = Nfa::try_from_ast(
            parse_regex_syntax("a{3,}").unwrap(),
            &mut char_class_registry,
        )
        .unwrap();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 10);
        assert_eq!(nfa.start_state.as_usize(), 0);
        assert_eq!(nfa.end_state.as_usize(), 9);
    }

    #[test]
    fn test_nfa_repetition_bounded() {
        // Create a character class registry
        let mut char_class_registry = CharacterClassRegistry::new();
        // Create an example AST and convert the AST to an NFA
        let nfa: Nfa = Nfa::try_from_ast(
            parse_regex_syntax("a{3,5}").unwrap(),
            &mut char_class_registry,
        )
        .unwrap();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 12);
        assert_eq!(nfa.start_state.as_usize(), 0);
        assert_eq!(nfa.end_state.as_usize(), 10);
    }

    // Ascii character class are not yet implemented
    #[test]
    fn test_character_class_expression() {
        // Create a character class registry
        let mut char_class_registry = CharacterClassRegistry::new();
        // Create an example AST and convert the AST to an NFA
        let nfa: Nfa = Nfa::try_from_ast(
            parse_regex_syntax(r"[[:digit:]]").unwrap(),
            &mut char_class_registry,
        )
        .unwrap();

        // Add assertions here to validate the NFA
        assert_eq!(nfa.states.len(), 2);
        assert_eq!(nfa.start_state.as_usize(), 0);
        assert_eq!(nfa.end_state.as_usize(), 1);
    }

    #[test]
    fn test_clousure_of_states() {
        // Create a character class registry
        let mut char_class_registry = CharacterClassRegistry::new();
        // Create an example AST and convert the AST to an NFA
        let nfa: Nfa = Nfa::try_from_ast(
            // SecondLastBitIs1
            parse_regex_syntax(r"(0|1)*1(0|1)").unwrap(),
            &mut char_class_registry,
        )
        .unwrap();

        // Calculate the epsilon closure of the start state
        let closure = nfa.epsilon_closure(nfa.start_state);

        // Validate the NFA closure
        assert_eq!(closure.len(), 7);
        assert_eq!(BTreeSet::<StateID>::from_iter(closure.iter().cloned()), {
            let mut set = BTreeSet::new();
            set.insert(StateID::new(0));
            set.insert(StateID::new(2));
            set.insert(StateID::new(4));
            set.insert(StateID::new(5));
            set.insert(StateID::new(6));
            set.insert(StateID::new(7));
            set.insert(StateID::new(8));
            set
        });

        // Calculate the transitions on a character classes in the closure
        let matching_states = closure.iter().fold(BTreeSet::new(), |mut set, state| {
            for transition in nfa.states()[*state].transitions() {
                set.insert((transition.char_class(), transition.target_state()));
            }
            set
        });

        // Validate the transitions from the closure of the start state
        assert_eq!(matching_states.len(), 3);
        assert_eq!(matching_states, {
            let mut set = BTreeSet::new();
            set.insert((CharClassID::new(0), StateID::new(1)));
            set.insert((CharClassID::new(1), StateID::new(3)));
            set.insert((CharClassID::new(1), StateID::new(9)));
            set
        });

        // Calculate the epsilon closure of each target state of the transitions
        let closures = matching_states.iter().fold(
            Vec::<(CharClassID, BTreeSet<StateID>)>::new(),
            |mut set, (char_class, state)| {
                let closure = nfa.epsilon_closure(*state);
                set.push((*char_class, BTreeSet::from_iter(closure.iter().cloned())));
                set
            },
        );

        eprintln!("{:?}", closures);

        // Validate the epsilon closures of the target states
        assert_eq!(closures.len(), 3);
        assert_eq!(closures, {
            let mut vec = Vec::new();
            vec.push((CharClassID::new(0), {
                BTreeSet::from_iter(vec![
                    StateID::new(0),
                    StateID::new(1),
                    StateID::new(2),
                    StateID::new(4),
                    StateID::new(5),
                    StateID::new(7),
                    StateID::new(8),
                ])
            }));
            vec.push((CharClassID::new(1), {
                BTreeSet::from_iter(vec![
                    StateID::new(0),
                    StateID::new(2),
                    StateID::new(3),
                    StateID::new(4),
                    StateID::new(5),
                    StateID::new(7),
                    StateID::new(8),
                ])
            }));
            vec.push((CharClassID::new(1), {
                BTreeSet::from_iter(vec![
                    StateID::new(9),
                    StateID::new(10),
                    StateID::new(12),
                    StateID::new(14),
                ])
            }));
            vec
        });

        // Calculate the transitions on a character classes in the closures
        let matching_states = closures
            .iter()
            .fold(Vec::new(), |mut vec, (char_class, closure)| {
                for state in closure {
                    for transition in nfa.states()[*state].transitions() {
                        vec.push((
                            *char_class,
                            transition.char_class(),
                            transition.target_state(),
                        ));
                    }
                }
                vec
            });

        // Validate the transitions from the closures of the target states
        assert_eq!(matching_states.len(), 8);
        assert_eq!(
            BTreeSet::<(CharClassID, CharClassID, StateID)>::from_iter(
                matching_states.iter().cloned()
            ),
            {
                let mut set = BTreeSet::new();
                set.insert((CharClassID::new(0), CharClassID::new(0), StateID::new(1)));
                set.insert((CharClassID::new(0), CharClassID::new(1), StateID::new(3)));
                set.insert((CharClassID::new(0), CharClassID::new(1), StateID::new(9)));
                set.insert((CharClassID::new(1), CharClassID::new(0), StateID::new(1)));
                set.insert((CharClassID::new(1), CharClassID::new(1), StateID::new(3)));
                set.insert((CharClassID::new(1), CharClassID::new(0), StateID::new(11)));
                set.insert((CharClassID::new(1), CharClassID::new(1), StateID::new(9)));
                set.insert((CharClassID::new(1), CharClassID::new(1), StateID::new(13)));
                set
            }
        );
    }

    // Test error on greedy repetition
    #[test]
    fn test_nfa_repetition_non_greedy() {
        // Create a character class registry
        let mut char_class_registry = CharacterClassRegistry::new();
        // Create an example AST and convert the AST to an NFA
        let result =
            Nfa::try_from_ast(parse_regex_syntax("a*?").unwrap(), &mut char_class_registry);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Non-greedy"));

        // Create a character class registry
        let mut char_class_registry = CharacterClassRegistry::new();
        // Create an example AST and convert the AST to an NFA
        let result =
            Nfa::try_from_ast(parse_regex_syntax("a+?").unwrap(), &mut char_class_registry);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Non-greedy"));
    }
}

#[cfg(test)]
mod tests_try_from {

    use crate::internal::parser::parse_regex_syntax;

    use super::*;

    /// A macro that simplifies the rendering of a dot file for a NFA.
    macro_rules! nfa_render_to {
        ($nfa:expr, $label:expr) => {
            let mut f = std::fs::File::create(format!("target/{}Nfa.dot", $label)).unwrap();
            $crate::internal::dot::nfa_render($nfa, $label, &mut f);
        };
    }

    struct TestData {
        // Use pascal case for the name because the name is used also as dot file name.
        // Also, the name should be unique.
        name: &'static str,
        input: &'static str,
        expected_states: usize,
        expected_start_state: usize,
        expected_end_state: usize,
        expected_char_classes: usize,
    }

    const TEST_DATA: &[TestData] = &[
        TestData {
            name: "SingleCharacter1",
            input: "a",
            expected_states: 2,
            expected_start_state: 0,
            expected_end_state: 1,
            expected_char_classes: 1,
        },
        TestData {
            name: "Concatenation",
            input: "ab",
            expected_states: 4,
            expected_start_state: 0,
            expected_end_state: 3,
            expected_char_classes: 2,
        },
        TestData {
            name: "Alternation1",
            input: "a|b",
            expected_states: 6,
            expected_start_state: 4,
            expected_end_state: 5,
            expected_char_classes: 2,
        },
        TestData {
            name: "Repetition",
            input: "a*",
            expected_states: 4,
            expected_start_state: 2,
            expected_end_state: 3,
            expected_char_classes: 1,
        },
        TestData {
            name: "ZeroOrOne",
            input: "a?",
            expected_states: 3,
            expected_start_state: 2,
            expected_end_state: 1,
            expected_char_classes: 1,
        },
        TestData {
            name: "OneOrMore",
            input: "a+",
            expected_states: 4,
            expected_start_state: 2,
            expected_end_state: 3,
            expected_char_classes: 1,
        },
        TestData {
            name: "Complex1",
            input: "(a|b)*abb",
            expected_states: 14,
            expected_start_state: 6,
            expected_end_state: 13,
            expected_char_classes: 2,
        },
        TestData {
            name: "String",
            input: r"\u{0022}(\\[\u{0022}\\/bfnrt]|u[0-9a-fA-F]{4}|[^\u{0022}\\\u0000-\u001F])*\u{0022}",
            expected_states: 26,
            expected_start_state: 0,
            expected_end_state: 25,
            expected_char_classes: 6,
        },
        TestData {
            name: "Example1",
            input: "(A*B|AC)D",
            expected_states: 14,
            expected_start_state: 10,
            expected_end_state: 13,
            expected_char_classes: 4,
        },
    ];

    #[test]
    fn test_try_from_ast() {
        for data in TEST_DATA.iter() {
            let mut char_class_registry = CharacterClassRegistry::new();
            let nfa: Nfa = Nfa::try_from_ast(
                parse_regex_syntax(data.input).unwrap(),
                &mut char_class_registry,
            )
            .unwrap();

            nfa_render_to!(&nfa, data.name);

            assert_eq!(
                nfa.states.len(),
                data.expected_states,
                "expected state count: {}:{}",
                data.name,
                data.input
            );
            assert_eq!(
                nfa.start_state.as_usize(),
                data.expected_start_state,
                "expected start state: {}:{}",
                data.name,
                data.input
            );
            assert_eq!(
                nfa.end_state.as_usize(),
                data.expected_end_state,
                "expected end state: {}:{}",
                data.name,
                data.input
            );
            assert_eq!(
                char_class_registry.len(),
                data.expected_char_classes,
                "expected char classes: {}:{}",
                data.name,
                data.input
            );
        }
    }
}
