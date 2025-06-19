use std::collections::{BTreeSet, VecDeque};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{ids::DisjointCharClassID, CharClassID, Match, Pattern, Result, Span};

use super::{
    ids::StateSetID, minimizer::Minimizer, parse_regex_syntax, CharacterClassRegistry,
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
}

impl CompiledDfa {
    /// Creates a new CompiledDfa from the given patterns and character class registry.
    pub(crate) fn new(
        patterns: Vec<String>,
        terminal_ids: Vec<TerminalID>,
        states: Vec<StateData>,
    ) -> Self {
        Self {
            patterns,
            terminal_ids,
            states,
        }
    }

    /// Simulates the DFA on the given input.
    /// Returns a match starting at the current position. No try on next character is done.
    /// The caller must do that.
    ///
    /// If no match is found, None is returned.
    // #[inline(always)]
    pub(crate) fn find_from(
        &self,
        input: &str,
        char_indices: std::str::CharIndices,
        character_classes: &CharacterClassRegistry,
    ) -> Option<Match> {
        // Initialize the match variables.
        let mut match_start = None;
        let mut match_end = None;
        let mut match_terminal_id = None;
        let mut state = StateSetID::new(0);

        for (index, c) in char_indices {
            if match_start.is_none() {
                // A potential match starts always at the first position.
                // Is is only part of a valid match if match_end is also set in the inner for loop.
                match_start = Some(index);
            }

            let state_data = &self.states[state];
            if match_end.is_none() && state_data.accept_data.is_some() {
                match_end = Some(index);
            }
            // let cc_ids = character_classes.get_matching_character_classes(c);
            let mut any_match = false;
            for (cc, next) in &state_data.transitions {
                // Check if the character class of the transition matches the character.
                if !character_classes.matches(*cc, c) {
                    continue;
                }
                any_match = true;
                // Go to the next state.
                state = *next;
                let next_state = &self.states[state];
                if let Some((accepted_terminal, lookahead)) = next_state.accept_data.as_ref() {
                    let mut lookahead_len = 0;
                    // Check if a lookahead is present and if it is satisfied.
                    if let Some(lookahead) = lookahead {
                        // Create a CharIndices iterator starting from the current position.
                        if let Some((_, next_slice)) = input.split_at_checked(index + c.len_utf8())
                        {
                            let char_indices = next_slice.char_indices();
                            let lookahead = lookahead.clone();
                            let (satisfied, len) = lookahead.satisfies_lookahead(
                                next_slice,
                                char_indices,
                                character_classes,
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
                        match (index + c.len_utf8()).cmp(&(match_end_index + lookahead_len)) {
                            std::cmp::Ordering::Greater => {
                                match_end = Some(index + c.len_utf8());
                                match_terminal_id = Some(accepted_terminal);
                            }
                            std::cmp::Ordering::Equal => {
                                let terminal_id_priority = self.priority_of(*accepted_terminal);
                                if terminal_id_priority
                                    < self.priority_of(*match_terminal_id.unwrap())
                                {
                                    match_terminal_id = Some(accepted_terminal);
                                }
                            }
                            std::cmp::Ordering::Less => {
                                match_terminal_id = Some(accepted_terminal);
                            }
                        }
                    } else {
                        match_end = Some(index + c.len_utf8());
                        match_terminal_id = Some(accepted_terminal);
                    }
                }
            }
            if !any_match {
                // No transition matched, so we reset the match variables.
                return None;
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

    /// Create a compiled DFA from a pattern.
    /// Used for testing and debugging purposes.
    #[allow(dead_code)]
    pub(crate) fn try_from_pattern(
        pattern: &Pattern,
        character_class_registry: &mut CharacterClassRegistry,
    ) -> Result<Self> {
        let hir = parse_regex_syntax(pattern.pattern())?;
        let mut nfa: Nfa = Nfa::try_from_hir(hir, character_class_registry)?;
        nfa.set_terminal_id(pattern.terminal_id());
        let mut compiled_dfa: CompiledDfa = Self::try_from_nfa(&nfa, character_class_registry)?;
        Self::add_lookahead_from_pattern(pattern, character_class_registry, &mut compiled_dfa)?;
        Ok(compiled_dfa)
    }

    pub(crate) fn try_from_patterns(
        patterns: &[Pattern],
        character_class_registry: &mut CharacterClassRegistry,
    ) -> Result<Self> {
        let mp_nfa = MultiPatternNfa::try_from_patterns(patterns, character_class_registry)?;
        let mut compiled_dfa: CompiledDfa =
            Self::try_from_multi_pattern_nfa(&mp_nfa, character_class_registry)?;
        // Add the lookaheads to the compiled DFA.
        for pattern in patterns.iter() {
            Self::add_lookahead_from_pattern(pattern, character_class_registry, &mut compiled_dfa)?;
        }
        Ok(compiled_dfa)
    }

    /// Create a compiled DFA from an NFA.
    /// This is a private method that is used by the public methods
    /// [`try_from_nfa`] and [`try_from_multi_pattern_nfa`].
    /// It is used to convert an NFA into a CompiledDfa.
    /// We use a subset construction algorithm to convert the NFA into a DFA.
    /// To make the transitions deterministic, we convert all transitions via `CharClassID`
    /// to possibly multiple transitions via `DisjointCharClassID`.
    fn create_from_nfa(
        nfa: &Nfa,
        character_class_registry: &mut CharacterClassRegistry,
    ) -> Result<Self> {
        // A temporary map to store the state ids of the sets of states.
        let mut state_map: FxHashMap<BTreeSet<StateID>, StateSetID> = FxHashMap::default();
        // A temporary set to store the transitions of the CompiledDfa.
        // The state ids are numbers of sets of states.
        let mut transitions: FxHashSet<(StateSetID, DisjointCharClassID, StateSetID)> =
            FxHashSet::default();
        // The end states of the CompiledDfa are a vector of state ids and terminal ids.
        let mut accepting_states: Vec<(StateSetID, TerminalID)> = Vec::new();
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
            // Group target states by character class
            let mut cc_to_targets: FxHashMap<CharClassID, FxHashSet<StateID>> =
                FxHashMap::default();
            for (cc, target_state) in target_states {
                cc_to_targets.entry(cc).or_default().insert(target_state);
            }

            // Process each character class once
            for (cc, targets) in cc_to_targets {
                let mut new_state_id_candidate = state_map.len() as StateIDBase;
                // Calculate epsilon closure of all targets
                let mut combined_epsilon_closure = BTreeSet::new();
                for target in targets {
                    combined_epsilon_closure.extend(nfa.epsilon_closure(target));
                }

                // Create a new DFA state for this combined set
                let new_state_id = *state_map
                    .entry(combined_epsilon_closure.clone())
                    .or_insert_with(|| {
                        let new_state_id = StateSetID::new(new_state_id_candidate);
                        queue.push_back(new_state_id);
                        new_state_id_candidate += 1;
                        new_state_id
                    });
                // Update accepting states if the epsilon closure contains the end state
                if combined_epsilon_closure.contains(&nfa.end_state)
                    && !accepting_states.contains(&(
                        new_state_id,
                        (nfa.pattern.terminal_id() as TerminalIDBase).into(),
                    ))
                {
                    accepting_states.push((
                        new_state_id,
                        (nfa.pattern.terminal_id() as TerminalIDBase).into(),
                    ));
                }
                // Add transitions
                for disjoint_cc in character_class_registry.get_disjoint_character_classes(cc) {
                    transitions.insert((old_state_id, *disjoint_cc, new_state_id));
                }
            }
        }
        // The transitions of the CompiledDfa.
        let mut states: Vec<StateData> = Vec::with_capacity(state_map.len());
        for _ in 0..state_map.len() {
            states.push(StateData::new());
        }
        for (from, cc, to) in transitions {
            states[from].transitions.push((cc, to));
        }
        for (state, term) in accepting_states {
            states[state].set_terminal_id(term);
        }
        // Create the CompiledDfa from the states and patterns.
        let mut compiled_dfa = CompiledDfa {
            patterns: vec![nfa.pattern.pattern().to_string()],
            terminal_ids: vec![(nfa.pattern.terminal_id() as TerminalIDBase).into()],
            states,
        };
        // Minimize the CompiledDfa.
        compiled_dfa = Minimizer::minimize(compiled_dfa);

        // Add the lookaheads to the compiled DFA.
        Self::add_lookahead_from_pattern(
            &nfa.pattern,
            character_class_registry,
            &mut compiled_dfa,
        )?;
        Ok(compiled_dfa)
    }

    /// Create a compiled DFA from an Multi pattern NFA.
    /// This is a private method that is used by the public methods
    /// [`from_nfa`] and [`from_multi_pattern_nfa`].
    /// It is used to convert an NFA into a CompiledDfa.
    /// We use the same subset construction algorithm as in `create_from_nfa`, add all
    /// lookaheads from the Nfas and set the patterns vector to the patterns of the
    /// MultiPatternNfa.
    fn create_from_multi_pattern_nfa(
        mp_nfa: &MultiPatternNfa,
        character_class_registry: &mut CharacterClassRegistry,
    ) -> Result<Self> {
        // A temporary map to store the state ids of the sets of states.
        let mut state_map: FxHashMap<BTreeSet<StateID>, StateSetID> = FxHashMap::default();
        // A temporary set to store the transitions of the CompiledDfa.
        // The state ids are numbers of sets of states.
        let mut transitions: FxHashSet<(StateSetID, DisjointCharClassID, StateSetID)> =
            FxHashSet::default();
        // The end states of the CompiledDfa are a vector of state ids and terminal ids.
        let mut accepting_states: Vec<(StateSetID, TerminalID)> = Vec::new();
        // Calculate the epsilon closure of the start state of the multi-pattern NFA.
        let epsilon_closure: BTreeSet<StateID> =
            BTreeSet::from_iter(mp_nfa.epsilon_closure(0.into()));
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
            let target_states = mp_nfa.get_match_transitions(epsilon_closure.iter().cloned());
            let old_state_id = current_state;
            // Group target states by character class
            let mut cc_to_targets: FxHashMap<CharClassID, FxHashSet<StateID>> =
                FxHashMap::default();
            for (cc, target_state) in target_states {
                cc_to_targets.entry(cc).or_default().insert(target_state);
            }

            // Process each character class once
            for (cc, targets) in cc_to_targets {
                let mut new_state_id_candidate = state_map.len() as StateIDBase;
                // Calculate epsilon closure of all targets
                let mut combined_epsilon_closure = BTreeSet::new();
                for target in targets {
                    combined_epsilon_closure.extend(mp_nfa.epsilon_closure(target));
                }

                // Create a new DFA state for this combined set
                let new_state_id = *state_map
                    .entry(combined_epsilon_closure.clone())
                    .or_insert_with(|| {
                        let new_state_id = StateSetID::new(new_state_id_candidate);
                        queue.push_back(new_state_id);
                        new_state_id_candidate += 1;
                        new_state_id
                    });
                // Update accepting states if the epsilon closure contains an end state
                if let Some(accepting_state) = combined_epsilon_closure
                    .iter()
                    .find(|s| mp_nfa.is_accepting_state(**s))
                {
                    let terminal_id: TerminalID =
                        (mp_nfa.find_nfa(*accepting_state).unwrap().terminal_id()
                            as TerminalIDBase)
                            .into();
                    if !accepting_states.contains(&(new_state_id, terminal_id)) {
                        accepting_states.push((new_state_id, terminal_id));
                    }
                }

                // Add transitions
                for disjoint_cc in character_class_registry.get_disjoint_character_classes(cc) {
                    transitions.insert((old_state_id, *disjoint_cc, new_state_id));
                }
            }
        }
        // The transitions of the CompiledDfa.
        let mut states: Vec<StateData> = Vec::with_capacity(state_map.len());
        for _ in 0..state_map.len() {
            states.push(StateData::new());
        }
        for (from, cc, to) in transitions {
            states[from].transitions.push((cc, to));
        }
        for (state, term) in accepting_states {
            states[state].set_terminal_id(term);
        }
        // Create the CompiledDfa from the states and patterns.
        let mut compiled_dfa = CompiledDfa {
            patterns: mp_nfa
                .patterns
                .iter()
                .map(|p| p.pattern().to_string())
                .collect(),
            terminal_ids: mp_nfa
                .patterns
                .iter()
                .map(|p| (p.terminal_id() as TerminalIDBase).into())
                .collect(),
            states,
        };

        // Add the lookaheads to the compiled DFA.
        for pattern in mp_nfa.patterns.iter() {
            Self::add_lookahead_from_pattern(pattern, character_class_registry, &mut compiled_dfa)?;
        }
        Ok(Minimizer::minimize(compiled_dfa))
    }

    /// Create a compiled DFA from an NFA.
    /// Calls [`create_disjoint_character_classes`] on the character class registry first
    pub(crate) fn try_from_nfa(
        nfa: &Nfa,
        character_class_registry: &mut CharacterClassRegistry,
    ) -> Result<Self> {
        character_class_registry.create_disjoint_character_classes();

        // Convert the NFA into a CompiledDfa.
        // All transitions via character classes should be converted to possibly multiple
        // transitions via DisjointCharClassID.
        let mut compiled_dfa: CompiledDfa = Self::create_from_nfa(nfa, character_class_registry)?;

        // Add the lookaheads to the compiled DFA.
        Self::add_lookahead_from_pattern(
            &nfa.pattern,
            character_class_registry,
            &mut compiled_dfa,
        )?;
        Ok(compiled_dfa)
    }

    /// Create a compiled DFA from a MultiPatternNfa.
    /// Calls [`create_disjoint_character_classes`] on the character class registry first.
    pub(crate) fn try_from_multi_pattern_nfa(
        mp_nfa: &MultiPatternNfa,
        character_class_registry: &mut CharacterClassRegistry,
    ) -> Result<Self> {
        character_class_registry.create_disjoint_character_classes();
        Self::create_from_multi_pattern_nfa(mp_nfa, character_class_registry)
    }

    /// Add a lookahead for a given terminal_id to the compiled DFA.
    #[inline(always)]
    pub(crate) fn add_lookahead(&mut self, terminal_id: TerminalID, lookahead: CompiledLookahead) {
        // Find the state data for the given terminal id.
        if let Some(state_data) = &mut self
            .states
            .iter_mut()
            .find(|s| matches!(s.accept_data, Some((id, _)) if id == terminal_id))
        {
            // Add the lookahead to the state data.
            state_data.accept_data = Some((terminal_id, Some(lookahead)));
        }
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
}

impl std::fmt::Display for CompiledDfa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Pattern: {}", self.patterns.join("|"))?;
        writeln!(f, "Start state: 0")?;
        for (i, state) in self.states.iter().enumerate() {
            writeln!(f, "State {}: {}", i, state)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct StateData {
    /// A list of transitions from this state.
    /// The state ids are numbers of sets of states.
    pub(crate) transitions: Vec<(DisjointCharClassID, StateSetID)>,

    /// A possible terminal id of the state combined with an optional lookahead.
    /// It is set only if the state is an accepting state.
    pub(crate) accept_data: Option<(TerminalID, Option<CompiledLookahead>)>,
}

impl StateData {
    pub(crate) fn new() -> Self {
        Self {
            // Most states have only one or two transitions.
            // Only the start state has many transitions.
            transitions: Vec::with_capacity(2),
            accept_data: None,
        }
    }

    /// Sets the terminal id of the state.
    /// This is used to mark the state as an accepting state.
    pub(crate) fn set_terminal_id(&mut self, terminal_id: TerminalID) {
        self.accept_data = self.accept_data.take().map_or_else(
            || Some((terminal_id, None)),
            |(old_terminal_id, old_lookahead)| {
                if old_terminal_id == terminal_id {
                    Some((old_terminal_id, old_lookahead))
                } else {
                    Some((terminal_id, old_lookahead))
                }
            },
        );
    }
}

impl std::fmt::Display for StateData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (cc, next) in &self.transitions {
            writeln!(f, "Transition: {:?} -> {}", cc, next)?;
            if let Some((terminal_id, lookahead)) = self.accept_data.as_ref() {
                writeln!(f, "  Terminal id: {}", terminal_id)?;
                if let Some(lookahead) = lookahead {
                    writeln!(f, "  Lookahead: {}", lookahead)?;
                }
            }
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
            $crate::dot::nfa_render($nfa, $label, &mut f);
        };
    }

    /// A macro that simplifies the rendering of a dot file for a CompiledDfa.
    #[cfg(feature = "dot_writer")]
    macro_rules! compiled_dfa_render_to {
        ($compiled_dfa:expr, $label:expr, $character_class_registry:expr) => {
            let mut f =
                std::fs::File::create(format!("{}/{}CompiledDfa.dot", TARGET_FOLDER, $label))
                    .unwrap();
            $crate::dot::compiled_dfa_render(
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
        end_states: Vec<(usize, crate::TerminalID)>,
        match_data: Vec<(&'static str, Option<(usize, usize)>)>,
    }

    #[cfg(feature = "dot_writer")]
    static TEST_DATA: std::sync::LazyLock<Vec<TestData>> = std::sync::LazyLock::new(|| {
        vec![
            TestData {
                pattern: "(A*B|AC)D",
                name: "Sedgewick",
                end_states: vec![(4, 0.into())],
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
                end_states: vec![(7, 0.into())],
                match_data: vec![
                    (r#""autumn""#, Some((0, 8))),
                    (r#""au0075tumn""#, Some((0, 12))),
                    (r#""au007xtumn""#, Some((0, 12))),
                ],
            },
            TestData {
                pattern: r"[a-zA-Z_]\w*",
                name: "Identifier",
                end_states: vec![(1, 0.into())],
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
                end_states: vec![(2, 0.into())],
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
                end_states: vec![(1, 0.into())],
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
                end_states: vec![(3, 0.into())],
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
            let mut character_class_registry = crate::CharacterClassRegistry::new();
            let hir = crate::parse_regex_syntax(pattern.pattern()).unwrap();
            let nfa: crate::Nfa =
                crate::Nfa::try_from_hir(hir, &mut character_class_registry).unwrap();
            nfa_render_to!(&nfa, test.name);
            let compiled_dfa =
                crate::compiled_dfa::CompiledDfa::try_from_nfa(&nfa, &mut character_class_registry)
                    .unwrap();

            let end_states = compiled_dfa
                .states
                .iter()
                .enumerate()
                .filter_map(|(i, s)| {
                    s.accept_data
                        .as_ref()
                        .map(|(terminal_id, _)| (i, *terminal_id))
                })
                .collect::<Vec<_>>();
            compiled_dfa_render_to!(&compiled_dfa, test.name, &character_class_registry);
            assert_eq!(
                end_states, test.end_states,
                "Test '{}', End states",
                test.name
            );
            eprintln!("{}", compiled_dfa);

            for (id, (input, expected)) in test.match_data.iter().enumerate() {
                let char_indices = input.char_indices();
                trace!("Matching string: {}", input);
                let matched =
                    compiled_dfa.find_from(input, char_indices, &character_class_registry);
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

    #[test]
    fn test_create_from_nfa() {
        use log::trace;

        init();
        let pattern = crate::Pattern::new("ab|ac".to_string(), 0);
        let mut character_class_registry = crate::CharacterClassRegistry::new();
        let hir = crate::parse_regex_syntax(pattern.pattern()).unwrap();
        let nfa: crate::Nfa = crate::Nfa::try_from_hir(hir, &mut character_class_registry).unwrap();
        nfa_render_to!(&nfa, "CreateFromNfa");
        let compiled_dfa =
            crate::compiled_dfa::CompiledDfa::try_from_nfa(&nfa, &mut character_class_registry)
                .unwrap();
        compiled_dfa_render_to!(&compiled_dfa, "CreateFromNfa", &character_class_registry);
        trace!("{}", compiled_dfa);
        assert_eq!(compiled_dfa.patterns.len(), 1);
        assert_eq!(compiled_dfa.pattern(0.into()), "(?:(?:ab)|(?:ac))");
        assert_eq!(compiled_dfa.terminal_ids.len(), 1);
        assert_eq!(compiled_dfa.terminal_ids[0], 0.into());
        assert_eq!(compiled_dfa.states.len(), 3);
        assert!(compiled_dfa.states[2]
            .accept_data
            .as_ref()
            .is_some_and(|(id, _)| *id == 0.into()));
    }

    /// A test that creates a CompiledDfa from a multi-pattern NFA and writes the dot files
    /// to the target directory.
    #[cfg(feature = "serde")]
    #[test]
    fn test_multi_pattern_nfa_veryl() {
        init();
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "../../scnr/benches/veryl_modes.json"
        );
        let file =
            std::fs::File::open(path).unwrap_or_else(|_| panic!("Failed to open file {}", path));
        let scanner_modes: Vec<crate::ScannerMode> = serde_json::from_reader(file).unwrap();
        assert!(scanner_modes[0].patterns[17].lookahead().is_some());
        assert_eq!(scanner_modes[0].patterns[17].terminal_id(), 20);
        let mut character_class_registry = crate::CharacterClassRegistry::new();
        for scanner_mode in &scanner_modes[0..3] {
            let compiled_dfa = crate::compiled_dfa::CompiledDfa::try_from_patterns(
                &scanner_mode.patterns,
                &mut character_class_registry,
            )
            .unwrap();
            if scanner_mode.name == "INITIAL" {
                assert_eq!(compiled_dfa.patterns.len(), 115);
                assert_eq!(
                    compiled_dfa
                        .states
                        .iter()
                        .filter(|s| s.accept_data.as_ref().is_some_and(|a| a.1.is_some()))
                        .count(),
                    1
                );
                println!("{}", compiled_dfa);
                assert!(compiled_dfa
                    .states
                    .iter()
                    .filter_map(|s| s.accept_data.as_ref().map(|a| a.0))
                    .any(|a| a == 20.into()));
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
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "../../scnr/tests/data/parol.json"
        );
        let file =
            std::fs::File::open(path).unwrap_or_else(|_| panic!("Failed to open file {}", path));
        let scanner_modes: Vec<crate::ScannerMode> = serde_json::from_reader(file).unwrap();
        let mut character_class_registry = crate::CharacterClassRegistry::new();
        let compiled_dfa = crate::compiled_dfa::CompiledDfa::try_from_patterns(
            &scanner_modes[0].patterns,
            &mut character_class_registry,
        )
        .unwrap();
        assert_eq!(compiled_dfa.patterns.len(), 1);
        assert_eq!(
            compiled_dfa
                .states
                .iter()
                .filter(|s| s.accept_data.as_ref().is_some_and(|a| a.1.is_some()))
                .count(),
            0
        );
        println!("{}", compiled_dfa);
        compiled_dfa_render_to!(&compiled_dfa, "Parol", &character_class_registry);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_character_class_registry_data() {
        use std::io::Write;

        init();
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "../../scnr/benches/veryl_modes.json"
        );
        let file =
            std::fs::File::open(path).unwrap_or_else(|_| panic!("Failed to open file {}", path));
        let scanner_modes: Vec<crate::ScannerMode> = serde_json::from_reader(file).unwrap();
        assert!(scanner_modes[0].patterns[17].lookahead().is_some());
        assert_eq!(scanner_modes[0].patterns[17].terminal_id(), 20);
        let mut character_class_registry = crate::CharacterClassRegistry::new();
        let _compiled_dfa = crate::compiled_dfa::CompiledDfa::try_from_patterns(
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
        let mut character_class_registry = crate::CharacterClassRegistry::new();
        let compiled_dfa = crate::compiled_dfa::CompiledDfa::try_from_patterns(
            &scanner_modes[0].patterns,
            &mut character_class_registry,
        )
        .expect("Failed to create compiled DFA from patterns");
        let input = "World!";
        let char_indices = input.char_indices();
        let matched = compiled_dfa
            .find_from(input, char_indices, &character_class_registry)
            .expect("Failed to match input");
        assert_eq!(matched.token_type(), 7);
    }
}
