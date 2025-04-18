use std::{
    char,
    collections::{BTreeSet, VecDeque},
};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{Match, Pattern, Result, Span};

use super::{
    ids::StateSetID, minimizer::Minimizer, parse_regex_syntax, CharClassID, CharacterClassRegistry,
    CompiledLookahead, MultiPatternNfa, Nfa, StateID, StateIDBase, TerminalID, TerminalIDBase,
};

/// A compiled DFA.
/// It represents the DFA in a way that is optimized for matching.
///
/// Although the data represent a DFA, i.e. it has no epsilon transitions, the transitions are
/// considered non-deterministic because they involve overlapping character classes. This allows a
/// transition to multiple states when reading a single character if that character belongs to
/// multiple character classes.
/// See implementation of the [find_from] method.
///
/// The start state is by design always 0.
#[derive(Debug, Clone)]
pub(crate) struct CompiledDfa {
    /// The pattern(s) of the DFA. Used for debugging purposes.
    pub(crate) patterns: Vec<String>,
    /// The terminal ids of the DFA in priority order. Lower indices have higher priority.
    pub(crate) terminal_ids: Vec<TerminalID>,
    /// The states of the DFA.
    pub(crate) states: Vec<StateData>,
    /// The accepting states of the DFA are represented by a vector of booleans and terminal ids.
    /// The index of the vector is the state id.
    /// If the value is true, the state is an accepting state.
    /// This is an optimization to avoid the need to search the end states during the simulation.
    pub(crate) end_states: Vec<(bool, TerminalID)>,
    /// An optional lookahead that is used to check if the DFA should match the input.
    pub(crate) lookaheads: FxHashMap<TerminalID, CompiledLookahead>,

    /// Current and next states of the DFA. They are used during the simulation of the DFA.
    /// For performance reasons we hold them here. This avoids the need to repeatedly allocate and
    /// drop them again during the simulation.
    current_states: Vec<StateSetID>,
    next_states: Vec<StateSetID>,
}

impl CompiledDfa {
    /// Creates a new CompiledDfa from the given patterns and character class registry.
    pub(crate) fn new(
        patterns: Vec<String>,
        terminal_ids: Vec<TerminalID>,
        states: Vec<StateData>,
        end_states: Vec<(bool, TerminalID)>,
        lookaheads: FxHashMap<TerminalID, CompiledLookahead>,
    ) -> Self {
        Self {
            patterns,
            terminal_ids,
            states,
            end_states,
            lookaheads,
            current_states: Vec::new(),
            next_states: Vec::new(),
        }
    }

    /// Simulates the DFA on the given input.
    /// Returns a match starting at the current position. No try on next character is done.
    /// The caller must do that.
    ///
    /// If no match is found, None is returned.
    ///
    /// We use a non-recursive implementation of the DFA simulation.
    /// The algorithm uses a queue to store the states that are currently active.
    /// The algorithm is as follows:
    /// 1. Add the start state to the queue.
    /// 2. Take the next character from the input.
    /// 3. If the queue is empty, stop and return the current match, if any.
    /// 4. For each state in the queue, check if it is an end state.
    ///    If it is, remember the current match if it is longer than the previous match found and
    ///    its terminal id is not higher at the same length.
    /// 5. For each state in the queue, check if there is a transition that matches the current
    ///    character.
    ///    If there is, add the target state to a second queue that will be used for the next
    ///    character.
    /// 6. Replace the queue with the second queue.
    /// 7. If there are more characters in the input, go to step 2.
    ///
    #[inline(always)]
    pub(crate) fn find_from(
        &mut self,
        input: &str,
        char_indices: std::str::CharIndices,
        match_char_class: &(dyn Fn(usize, char) -> bool + 'static),
    ) -> Option<Match> {
        self.current_states.clear();
        // Push the start state to the current states.
        self.current_states.push(StateSetID::new(0));
        self.next_states.clear();
        let mut match_start = None;
        let mut match_end = None;
        let mut match_terminal_id = None;
        for (index, c) in char_indices {
            if match_start.is_none() {
                // A potential match starts always at the first position.
                // Is is only part of a valid match if match_end is also set in the inner for loop.
                match_start = Some(index);
            }

            for state in self.current_states.iter() {
                if match_end.is_none() && self.end_states[*state].0 {
                    match_end = Some(index);
                }
                let state_data = &self.states[*state];
                for (cc, next) in &state_data.transitions {
                    if match_char_class(cc.as_usize(), c) {
                        if !self.next_states.contains(next) {
                            self.next_states.push(*next);
                        }
                        let next_end_state = self.end_states[*next];
                        if next_end_state.0 {
                            let mut lookahead_len = 0;
                            // Check if a lookahead is present and if it is satisfied.
                            if let Some(lookahead) = self.lookaheads.get(&next_end_state.1) {
                                // Create a CharIndices iterator starting from the current position.
                                if let Some((_, next_slice)) =
                                    input.split_at_checked(index + c.len_utf8())
                                {
                                    let char_indices = next_slice.char_indices();
                                    let mut lookahead = lookahead.clone();
                                    let (satisfied, len) = lookahead.satisfies_lookahead(
                                        next_slice,
                                        char_indices,
                                        match_char_class,
                                    );
                                    if !satisfied {
                                        continue;
                                    }
                                    lookahead_len = len;
                                } else {
                                    // We are at the end of the input.
                                    // If the lookahead is positive it is not satisfied, otherwise
                                    // we can accept the match.
                                    if lookahead.is_positive {
                                        continue;
                                    }
                                }
                            }
                            // Update the match end and terminal id if the match is longer or the
                            // terminal id is lower.
                            if let Some(match_end_index) = match_end.as_ref() {
                                match (index + c.len_utf8()).cmp(&(match_end_index + lookahead_len))
                                {
                                    std::cmp::Ordering::Greater => {
                                        match_end = Some(index + c.len_utf8());
                                        match_terminal_id = Some(next_end_state.1);
                                    }
                                    std::cmp::Ordering::Equal => {
                                        let terminal_id = self.priority_of(next_end_state.1);
                                        if terminal_id
                                            < self.priority_of(match_terminal_id.unwrap())
                                        {
                                            match_terminal_id = Some(next_end_state.1);
                                        }
                                    }
                                    std::cmp::Ordering::Less => {
                                        match_terminal_id = Some(next_end_state.1);
                                    }
                                }
                            } else {
                                match_end = Some(index + c.len_utf8());
                                match_terminal_id = Some(next_end_state.1);
                            }
                        }
                    }
                }
            }
            self.current_states.clear();
            std::mem::swap(&mut self.current_states, &mut self.next_states);
            if self.current_states.is_empty() {
                break;
            }
        }
        match_terminal_id.map(|match_terminal_id| {
            // If the terminal id is set, match_start and match_end must always be set as well.
            Match::new(
                match_terminal_id.as_usize(),
                Span::new(match_start.unwrap(), match_end.unwrap()),
            )
        })
    }

    /// Create a compiled NFA from a pattern.
    /// Used for testing and debugging purposes.
    #[allow(dead_code)]
    pub(crate) fn try_from_pattern(
        pattern: &Pattern,
        character_class_registry: &mut CharacterClassRegistry,
    ) -> Result<Self> {
        let hir = parse_regex_syntax(pattern.pattern())?;
        let mut nfa: Nfa = Nfa::try_from_hir(hir, character_class_registry)?;
        nfa.set_terminal_id(pattern.terminal_id());
        let mut compiled_dfa: CompiledDfa = nfa.into();
        Self::add_lookahead_from_pattern(pattern, character_class_registry, &mut compiled_dfa)?;
        Ok(compiled_dfa)
    }

    pub(crate) fn try_from_patterns(
        patterns: &[Pattern],
        character_class_registry: &mut CharacterClassRegistry,
    ) -> Result<Self> {
        let mp_nfa = MultiPatternNfa::try_from_patterns(patterns, character_class_registry)?;
        let mut compiled_dfa: CompiledDfa = mp_nfa.into();
        // Add the lookaheads to the compiled NFA.
        for pattern in patterns.iter() {
            Self::add_lookahead_from_pattern_hir(
                pattern,
                character_class_registry,
                &mut compiled_dfa,
            )?;
        }
        Ok(compiled_dfa)
    }

    /// Add a lookahead for a given terminal_id to the compiled NFA.
    #[inline(always)]
    pub(crate) fn add_lookahead(&mut self, terminal_id: TerminalID, lookahead: CompiledLookahead) {
        self.lookaheads.insert(terminal_id, lookahead);
    }

    /// Returns the pattern for the given terminal id.
    #[inline(always)]
    pub(crate) fn pattern(&self, terminal_id: TerminalID) -> &str {
        &self.patterns[terminal_id]
    }

    #[inline(always)]
    fn priority_of(&self, terminal_id: TerminalID) -> usize {
        self.terminal_ids
            .iter()
            .position(|&id| id == terminal_id)
            .unwrap()
    }

    #[inline(always)]
    fn add_lookahead_from_pattern(
        pattern: &Pattern,
        character_class_registry: &mut CharacterClassRegistry,
        compiled_dfa: &mut CompiledDfa,
    ) -> Result<()> {
        if let Some(lookahead) = pattern.lookahead() {
            let lookahead =
                CompiledLookahead::try_from_lookahead(lookahead, character_class_registry)?;
            compiled_dfa.add_lookahead((pattern.terminal_id() as TerminalIDBase).into(), lookahead);
        };
        Ok(())
    }

    #[inline(always)]
    fn add_lookahead_from_pattern_hir(
        pattern: &Pattern,
        character_class_registry: &mut CharacterClassRegistry,
        compiled_dfa: &mut CompiledDfa,
    ) -> Result<()> {
        if let Some(lookahead) = pattern.lookahead() {
            let lookahead =
                CompiledLookahead::try_from_lookahead_hir(lookahead, character_class_registry)?;
            compiled_dfa.add_lookahead((pattern.terminal_id() as TerminalIDBase).into(), lookahead);
        };
        Ok(())
    }
}

impl From<Nfa> for CompiledDfa {
    /// Create a dense representation of the NFA in form of match transitions between states sets.
    /// This is an equivalent algorithm to the subset construction for DFAs.
    ///
    /// Note that the lookahead is not set in the resulting CompiledDfa. This must be done
    /// separately because a character class registry is needed to create the lookaheads.
    /// See [CompiledDfa::try_from_pattern].
    fn from(nfa: Nfa) -> Self {
        // A temporary map to store the state ids of the sets of states.
        let mut state_map: FxHashMap<BTreeSet<StateID>, StateSetID> = FxHashMap::default();
        // A temporary set to store the transitions of the CompiledDfa.
        // The state ids are numbers of sets of states.
        let mut transitions: FxHashSet<(StateSetID, CharClassID, StateSetID)> =
            FxHashSet::default();
        // The end states of the CompiledDfa are a vector of state ids and terminal ids.
        let mut accepting_states: Vec<(StateSetID, usize)> = Vec::new();
        // Calculate the epsilon closure of the start state.
        let epsilon_closure: BTreeSet<StateID> =
            BTreeSet::from_iter(nfa.epsilon_closure(nfa.start_state));
        // The current state id is always 0.
        let current_state = StateSetID::new(0);
        // Add the start state to the state map.
        state_map.insert(epsilon_closure.clone(), current_state);

        // The list of target states not yet processed.
        let mut queue: VecDeque<StateSetID> = VecDeque::new();
        queue.push_back(current_state);

        while let Some(current_state) = queue.pop_front() {
            let epsilon_closure = state_map
                .iter()
                .find(|(_, v)| **v == current_state)
                .unwrap()
                .0
                .clone();
            let target_states = nfa.get_match_transitions(epsilon_closure.iter().cloned());
            let old_state_id = current_state;
            // Transform the target states to a set of state ids by calculating their epsilon closure.
            for (cc, target_state) in target_states {
                let epsilon_closure = BTreeSet::from_iter(nfa.epsilon_closure(target_state));
                let new_state_id_candidate = state_map.len() as StateIDBase;
                let new_state_id = *state_map.entry(epsilon_closure.clone()).or_insert_with(|| {
                    let new_state_id = StateSetID::new(new_state_id_candidate);
                    // Add the new state to the queue.
                    queue.push_back(new_state_id);
                    new_state_id
                });
                if epsilon_closure.contains(&nfa.end_state)
                    && !accepting_states.contains(&(new_state_id, nfa.pattern.terminal_id()))
                {
                    accepting_states.push((new_state_id, nfa.pattern.terminal_id()));
                }
                transitions.insert((old_state_id, cc, new_state_id));
            }
        }

        // The transitions of the CompiledDfa.
        let mut states: Vec<StateData> = Vec::with_capacity(transitions.len());
        for _ in 0..state_map.len() {
            states.push(StateData::new());
        }
        for (from, cc, to) in transitions {
            states[from].transitions.push((cc, to));
        }

        let current_states = Vec::with_capacity(states.len());
        let next_states = Vec::with_capacity(states.len());
        let mut end_states = vec![(false, TerminalID::new(0)); states.len()];
        for (state, term) in accepting_states {
            end_states[state] = (true, TerminalID::new(term as TerminalIDBase));
        }

        Minimizer::minimize(Self {
            patterns: vec![nfa.pattern.pattern().to_string()],
            terminal_ids: vec![(nfa.pattern.terminal_id() as TerminalIDBase).into()],
            states,
            end_states,
            lookaheads: FxHashMap::default(),
            current_states,
            next_states,
        })
    }
}

impl From<MultiPatternNfa> for CompiledDfa {
    /// Note that the lookahead is not set in the resulting CompiledDfa. This must be done
    /// separately because a character class registry is needed to create the lookaheads.
    /// See [CompiledDfa::try_from_patterns].
    fn from(mp_nfa: MultiPatternNfa) -> Self {
        let mut state_map: FxHashMap<BTreeSet<StateID>, StateSetID> = FxHashMap::default();
        let mut transitions: FxHashSet<(StateSetID, CharClassID, StateSetID)> =
            FxHashSet::default();
        let mut accepting_states: Vec<(StateSetID, usize)> = Vec::new();
        let mut queue: VecDeque<StateSetID> = VecDeque::new();

        // Calculate the epsilon closures of the start state of the multi-pattern NFA.
        let epsilon_closure: BTreeSet<StateID> =
            BTreeSet::from_iter(mp_nfa.epsilon_closure(0.into()));
        state_map.insert(epsilon_closure.clone(), 0.into());
        queue.push_back(StateSetID::new(0));

        while let Some(current_state) = queue.pop_front() {
            let epsilon_closure = state_map
                .iter()
                .find(|(_, v)| **v == current_state)
                .unwrap()
                .0
                .clone();
            let target_states = mp_nfa.get_match_transitions(epsilon_closure.iter().cloned());
            let old_state_id = current_state;
            for (cc, target_state) in target_states {
                let epsilon_closure = BTreeSet::from_iter(mp_nfa.epsilon_closure(target_state));
                let new_state_id_candidate = state_map.len() as StateIDBase;
                let new_state_id = *state_map.entry(epsilon_closure.clone()).or_insert_with(|| {
                    let new_state_id = StateSetID::new(new_state_id_candidate);
                    queue.push_back(new_state_id);
                    new_state_id
                });
                let target_nfa = mp_nfa.find_nfa(target_state).expect("NFA not found");
                if epsilon_closure
                    .iter()
                    .any(|s| mp_nfa.is_accepting_state(*s))
                    && !accepting_states.contains(&(new_state_id, target_nfa.terminal_id()))
                {
                    accepting_states.push((new_state_id, target_nfa.terminal_id()));
                }
                transitions.insert((old_state_id, cc, new_state_id));
            }
        }
        // The transitions of the CompiledDfa.
        let mut states: Vec<StateData> = Vec::with_capacity(transitions.len());
        for _ in 0..state_map.len() {
            states.push(StateData::new());
        }
        for (from, cc, to) in transitions {
            states[from].transitions.push((cc, to));
        }

        let current_states = Vec::with_capacity(states.len());
        let next_states = Vec::with_capacity(states.len());
        let mut end_states = vec![(false, TerminalID::new(0)); states.len()];
        for (state, term) in accepting_states {
            end_states[state] = (true, TerminalID::new(term as TerminalIDBase));
        }

        Minimizer::minimize(Self {
            patterns: vec![mp_nfa.patterns.iter().map(|p| p.pattern()).collect()],
            terminal_ids: mp_nfa
                .patterns
                .iter()
                .map(|p| (p.terminal_id() as TerminalIDBase).into())
                .collect(),
            states,
            end_states,
            lookaheads: FxHashMap::default(),
            current_states,
            next_states,
        })
    }
}

impl std::fmt::Display for CompiledDfa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Pattern: {}", self.patterns.join("|"))?;
        writeln!(f, "Start state: 0")?;
        writeln!(f, "End states: {:?}", self.end_states)?;
        for (i, state) in self.states.iter().enumerate() {
            writeln!(f, "State {}: {}", i, state)?;
        }
        writeln!(f, "Lookaheads:")?;
        for (terminal_id, lookahead) in &self.lookaheads {
            writeln!(f, "Lookahead: {} -> {}", terminal_id, lookahead)?;
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

impl StateData {
    pub(crate) fn new() -> Self {
        Self {
            // Most states have only one or two transitions.
            // Only the start state has many transitions.
            transitions: Vec::with_capacity(2),
        }
    }
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
    #[cfg(feature = "dot_writer")]
    static INIT: std::sync::Once = std::sync::Once::new();

    #[cfg(feature = "dot_writer")]
    const TARGET_FOLDER: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../target/testout/compiled_dfa_tests"
    );

    #[cfg(feature = "dot_writer")]
    fn init() {
        INIT.call_once(|| {
            let _ = env_logger::builder().is_test(true).try_init();
            // Delete all previously generated dot files.
            let _ = std::fs::remove_dir_all(TARGET_FOLDER);
            // Create the target folder.
            std::fs::create_dir_all(TARGET_FOLDER).unwrap();
        });
    }

    /// A macro that simplifies the rendering of a dot file for a NFA.
    #[cfg(feature = "dot_writer")]
    macro_rules! nfa_render_to {
        ($nfa:expr, $label:expr) => {
            let mut f =
                std::fs::File::create(format!("{}/{}Nfa.dot", TARGET_FOLDER, $label)).unwrap();
            $crate::internal::dot::nfa_render($nfa, $label, &mut f);
        };
    }

    /// A macro that simplifies the rendering of a dot file for a CompiledDfa.
    #[cfg(feature = "dot_writer")]
    macro_rules! compiled_dfa_render_to {
        ($compiled_dfa:expr, $label:expr, $character_class_registry:expr) => {
            let mut f =
                std::fs::File::create(format!("{}/{}CompiledDfa.dot", TARGET_FOLDER, $label))
                    .unwrap();
            $crate::internal::dot::compiled_dfa_render(
                $compiled_dfa,
                $label,
                $character_class_registry,
                &mut f,
            );
        };
    }

    #[cfg(feature = "dot_writer")]
    struct TestData {
        pattern: &'static str,
        name: &'static str,
        end_states: Vec<(bool, crate::internal::TerminalID)>,
        match_data: Vec<(&'static str, Option<(usize, usize)>)>,
    }

    #[cfg(feature = "dot_writer")]
    static TEST_DATA: std::sync::LazyLock<Vec<TestData>> = std::sync::LazyLock::new(|| {
        vec![
            TestData {
                pattern: "(A*B|AC)D",
                name: "Sedgewick",
                end_states: vec![
                    (false, 0.into()),
                    (false, 0.into()),
                    (false, 0.into()),
                    (false, 0.into()),
                    (true, 0.into()),
                ],
                match_data: vec![
                    ("AAABD", Some((0, 5))),
                    ("ACD", Some((0, 3))),
                    ("CDAABCAAABD", None),
                    ("CDAABC", None),
                ],
            },
            TestData {
                pattern: r#"\u{0022}(\\[\u{0022}\\/bfnrt]|u[0-9a-fA-F]{4}|[^\u{0022}\\\u0000-\u001F])*\u{0022}"#,
                name: "JsonString",
                end_states: vec![
                    (false, 0.into()),
                    (false, 0.into()),
                    (false, 0.into()),
                    (false, 0.into()),
                    (false, 0.into()),
                    (false, 0.into()),
                    (false, 0.into()),
                    (true, 0.into()),
                ],
                match_data: vec![
                    (r#""autumn""#, Some((0, 8))),
                    (r#""au0075tumn""#, Some((0, 12))),
                    (r#""au007xtumn""#, Some((0, 12))),
                ],
            },
            TestData {
                pattern: r"[a-zA-Z_]\w*",
                name: "Identifier",
                end_states: vec![(false, 0.into()), (true, 0.into())],
                match_data: vec![
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
                end_states: vec![(false, 0.into()), (false, 0.into()), (true, 0.into())],
                match_data: vec![
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
            TestData {
                pattern: r"a*(a|b)b*",
                name: "MinimalMatch",
                end_states: vec![(false, 0.into()), (true, 0.into())],
                match_data: vec![
                    ("a", Some((0, 1))),
                    ("b", Some((0, 1))),
                    ("abb", Some((0, 3))),
                    ("bbbb", Some((0, 4))),
                    ("aaaabbbbbc", Some((0, 9))),
                    ("c", None),
                    ("", None),
                ],
            },
            TestData {
                pattern: r"abc",
                name: "Concatenation",
                end_states: vec![
                    (false, 0.into()),
                    (false, 0.into()),
                    (false, 0.into()),
                    (true, 0.into()),
                ],
                match_data: vec![
                    ("a", None),
                    ("b", None),
                    ("c", None),
                    ("abc", Some((0, 3))),
                    ("ab", None),
                    ("aab", None),
                    ("aabc", None),
                ],
            },
        ]
    });

    #[cfg(feature = "dot_writer")]
    #[test]
    fn test_find_from() {
        use log::trace;

        init();
        for test in &*TEST_DATA {
            let pattern = crate::Pattern::new(test.pattern.to_string(), 0);
            let mut character_class_registry = crate::internal::CharacterClassRegistry::new();
            let hir = crate::internal::parse_regex_syntax(pattern.pattern()).unwrap();
            let nfa: crate::internal::Nfa =
                crate::internal::Nfa::try_from_hir(hir, &mut character_class_registry).unwrap();
            nfa_render_to!(&nfa, test.name);
            let mut compiled_dfa = crate::internal::compiled_dfa::CompiledDfa::from(nfa);
            assert_eq!(
                compiled_dfa.end_states, test.end_states,
                "Test '{}', End states",
                test.name
            );
            compiled_dfa_render_to!(&compiled_dfa, test.name, &character_class_registry);
            eprintln!("{}", compiled_dfa);

            for (id, (input, expected)) in test.match_data.iter().enumerate() {
                let char_indices = input.char_indices();
                trace!("Matching string: {}", input);
                let match_char_class = character_class_registry.create_match_char_class().unwrap();
                let matched = compiled_dfa.find_from(input, char_indices, &match_char_class);
                assert_eq!(
                    matched,
                    expected.map(|(start, end)| crate::Match::new(0, crate::Span::new(start, end))),
                    "Test '{}', Match data #{}, input '{}'",
                    test.name,
                    id,
                    input
                );
            }
        }
    }

    /// A test that creates a CompiledDfa from a multi-pattern NFA and writes the dot files
    /// to the target directory.
    #[cfg(feature = "serde")]
    #[test]
    fn test_multi_pattern_nfa_veryl() {
        init();
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/benches/veryl_modes.json");
        let file =
            std::fs::File::open(path).unwrap_or_else(|_| panic!("Failed to open file {}", path));
        let scanner_modes: Vec<crate::ScannerMode> = serde_json::from_reader(file).unwrap();
        assert!(scanner_modes[0].patterns[17].lookahead().is_some());
        assert_eq!(scanner_modes[0].patterns[17].terminal_id(), 20);
        let mut character_class_registry = crate::internal::CharacterClassRegistry::new();
        for scanner_mode in &scanner_modes[0..3] {
            let compiled_dfa = crate::internal::compiled_dfa::CompiledDfa::try_from_patterns(
                &scanner_mode.patterns,
                &mut character_class_registry,
            )
            .unwrap();
            if scanner_mode.name == "INITIAL" {
                assert_eq!(compiled_dfa.patterns.len(), 1);
                assert_eq!(compiled_dfa.lookaheads.len(), 1);
                println!("{}", compiled_dfa);
                assert!(compiled_dfa.lookaheads.contains_key(&20.into()));
            }
            compiled_dfa_render_to!(
                &compiled_dfa,
                &format!("Veryl_{}_", scanner_mode.name),
                &character_class_registry
            );
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_multi_pattern_nfa_parol() {
        init();
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/parol.json");
        let file =
            std::fs::File::open(path).unwrap_or_else(|_| panic!("Failed to open file {}", path));
        let scanner_modes: Vec<crate::ScannerMode> = serde_json::from_reader(file).unwrap();
        let mut character_class_registry = crate::internal::CharacterClassRegistry::new();
        let compiled_dfa = crate::internal::compiled_dfa::CompiledDfa::try_from_patterns(
            &scanner_modes[0].patterns,
            &mut character_class_registry,
        )
        .unwrap();
        assert_eq!(compiled_dfa.patterns.len(), 1);
        assert_eq!(compiled_dfa.lookaheads.len(), 0);
        println!("{}", compiled_dfa);
        compiled_dfa_render_to!(&compiled_dfa, "Parol", &character_class_registry);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_character_class_registry_data() {
        use std::io::Write;

        init();
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/benches/veryl_modes.json");
        let file =
            std::fs::File::open(path).unwrap_or_else(|_| panic!("Failed to open file {}", path));
        let scanner_modes: Vec<crate::ScannerMode> = serde_json::from_reader(file).unwrap();
        assert!(scanner_modes[0].patterns[17].lookahead().is_some());
        assert_eq!(scanner_modes[0].patterns[17].terminal_id(), 20);
        let mut character_class_registry = crate::internal::CharacterClassRegistry::new();
        let _compiled_dfa = crate::internal::compiled_dfa::CompiledDfa::try_from_patterns(
            &scanner_modes[0].patterns,
            &mut character_class_registry,
        )
        .unwrap();
        // Write the result of Display of the character class registry to a file for inspection.
        let mut f =
            std::fs::File::create(format!("{}/CharacterClassRegistry.txt", TARGET_FOLDER)).unwrap();
        writeln!(f, "Character classes deduced from veryl_modes.json").unwrap();
        writeln!(f, "{}", character_class_registry).unwrap();
    }

    /// This test failed until issue [#6](https://github.com/jsinger67/scnr/issues/6) was fixed.
    #[cfg(feature = "serde")]
    #[test]
    fn test_match_with_positive_lookahead() {
        let json = r#"
            [
            {
                "name": "INITIAL",
                "patterns": [
                { "pattern": "World", "token_type": 6 },
                {
                    "pattern": "World",
                    "token_type": 7,
                    "lookahead": { "is_positive": true, "pattern": "!" }
                }
                ],
                "transitions": []
            }
            ]"#;
        let scanner_modes: Vec<crate::ScannerMode> = serde_json::from_str(json).unwrap();
        let mut character_class_registry = crate::internal::CharacterClassRegistry::new();
        let mut compiled_dfa = crate::internal::compiled_dfa::CompiledDfa::try_from_patterns(
            &scanner_modes[0].patterns,
            &mut character_class_registry,
        )
        .expect("Failed to create compiled DFA from patterns");
        let match_char_class = character_class_registry
            .create_match_char_class()
            .expect("Failed to create match char class function");
        let input = "World!";
        let char_indices = input.char_indices();
        let matched = compiled_dfa
            .find_from(input, char_indices, &match_char_class)
            .expect("Failed to match input");
        assert_eq!(matched.token_type(), 7);
    }
}
