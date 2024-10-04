//! This module contains the DFA implementation.
//! The DFA is used to match a string against a regex pattern.
//! The DFA is generated from the NFA using the subset construction algorithm.

use log::trace;
use std::collections::{BTreeMap, BTreeSet};

use crate::Result;

use super::{
    multi_pattern_nfa::MultiPatternNfa, CharClassID, CharacterClassRegistry, Nfa, StateID,
    StateIDBase, TerminalID,
};

// The type definitions for the subset construction algorithm.
pub(crate) type StateGroup = BTreeSet<StateID>;
pub(crate) type Partition = Vec<StateGroup>;

// A data type that is calcuated from the transitions of a DFA state so that for each character
// class the target state is mapped to the partition group it belongs to.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct TransitionsToPartitionGroups(pub(crate) Vec<(CharClassID, usize)>);

impl TransitionsToPartitionGroups {
    pub(crate) fn new() -> Self {
        TransitionsToPartitionGroups(Vec::new())
    }

    pub(crate) fn insert(&mut self, char_class: CharClassID, partition_group: usize) {
        self.0.push((char_class, partition_group));
    }
}

/// The DFA implementation.
/// The DFA is created from an NFA using the subset construction algorithm.
/// It matches exactly one pattern.
#[derive(Debug, Default, Clone)]
pub(crate) struct Dfa {
    // The states of the DFA. The start state is always the first state in the vector, i.e. state 0.
    pub(crate) states: Vec<DfaState>,
    // The patterns for the accepting states.
    pub(crate) patterns: Vec<(String, TerminalID)>,
    // The accepting states of the DFA as well as the corresponding pattern id.
    pub(crate) accepting_states: BTreeSet<(StateID, TerminalID)>,
    // The transitions of the DFA.
    pub(crate) transitions: BTreeMap<StateID, BTreeMap<CharClassID, StateID>>,
}

impl Dfa {
    /// Get the states of the DFA.
    /// It is used for debugging purposes mostly in the [crate::internal::dot] module.
    #[allow(unused)]
    pub(crate) fn states(&self) -> &[DfaState] {
        &self.states
    }

    /// Get the pattern for the accepting states.
    /// It is used for debugging purposes mostly in the [crate::internal::dot] module.
    #[allow(unused)]
    pub(crate) fn pattern(&self, terminal_id: TerminalID) -> Option<&str> {
        self.patterns
            .iter()
            .find(|(_, id)| *id == terminal_id)
            .map(|(pattern, _)| pattern.as_str())
    }

    /// Get the pattern id if the given state is an accepting state.
    #[inline]
    pub(crate) fn is_accepting(&self, state_id: StateID) -> bool {
        self.accepting_states.iter().any(|s| s.0 == state_id)
    }

    /// Get the transitions of the DFA.
    /// It is used for debugging purposes mostly in the [crate::internal::dot] module.
    #[allow(unused)]
    pub(crate) fn transitions(&self) -> &BTreeMap<StateID, BTreeMap<CharClassID, StateID>> {
        &self.transitions
    }

    /// Create a DFA from an NFA.
    /// The DFA is created using the subset construction algorithm.
    pub(crate) fn try_from_nfa(
        nfa: Nfa,
        terminal_id: TerminalID,
        character_class_registry: &mut CharacterClassRegistry,
    ) -> Result<Self> {
        trace!("Character classes: {:?}", character_class_registry);
        let Nfa {
            pattern,
            start_state,
            end_state,
            ..
        } = nfa.clone();
        let mut dfa = Dfa {
            states: Vec::new(),
            patterns: vec![(pattern.clone(), terminal_id)],
            accepting_states: BTreeSet::new(),
            transitions: BTreeMap::new(),
        };
        // The initial state of the DFA is the epsilon closure of the start state of the NFA.
        let start_state = nfa.epsilon_closure(start_state);
        let accepting_states = vec![(end_state, terminal_id)];
        // The initial state is the start state of the DFA.
        let initial_state = dfa.add_state_if_new(start_state, &accepting_states)?;
        // The work list is used to keep track of the states that need to be processed.
        let mut work_list = vec![initial_state];
        // The marked flag is used to mark a state as visited during the subset construction algorithm.
        dfa.states[initial_state].marked = true;

        while let Some(state_id) = work_list.pop() {
            let nfa_states = dfa.states[state_id].nfa_states.clone();
            // let terminal_id = dfa.states[state_id].terminal_id;
            for char_class in character_class_registry.iter() {
                let target_states =
                    nfa.epsilon_closure_set(nfa.move_set(&nfa_states, char_class.id()));
                if !target_states.is_empty() {
                    let target_state = dfa.add_state_if_new(target_states, &accepting_states)?;
                    dfa.transitions
                        .entry(state_id)
                        .or_default()
                        .insert(char_class.id(), target_state);
                    if !dfa.states[target_state].marked {
                        dfa.states[target_state].marked = true;
                        work_list.push(target_state);
                    }
                }
            }
        }

        Ok(dfa)
    }

    pub(crate) fn try_from_mp_nfa(
        nfa: MultiPatternNfa,
        character_class_registry: &mut CharacterClassRegistry,
    ) -> Result<Self> {
        trace!("Character classes: {:?}", character_class_registry);
        let patterns = nfa.patterns();
        let nfas = nfa.nfas.clone();
        let accepting_states = nfas
            .iter()
            .map(|nfa| nfa.end_state())
            .zip(patterns.iter().map(|p| p.1))
            .collect::<Vec<_>>();
        debug_assert_eq!(accepting_states.len(), patterns.len());
        let mut dfa = Dfa {
            states: Vec::new(),
            patterns: patterns.to_vec(),
            accepting_states: BTreeSet::new(),
            transitions: BTreeMap::new(),
        };
        // The initial state of the DFA is the epsilon closure of the start state of the NFA.
        let start_state = nfa.epsilon_closure(0usize.into());
        // The initial state is the start state of the DFA.
        let initial_state = dfa.add_state_if_new(start_state, &accepting_states)?;
        // The work list is used to keep track of the states that need to be processed.
        let mut work_list = vec![initial_state];
        // The marked flag is used to mark a state as visited during the subset construction algorithm.
        dfa.states[initial_state].set_marked(true);

        while let Some(state_id) = work_list.pop() {
            let nfa_states = dfa.states[state_id].nfa_states().to_vec();
            for char_class in character_class_registry.iter() {
                let target_states =
                    nfa.epsilon_closure_set(nfa.move_set(&nfa_states, char_class.id()));
                if !target_states.is_empty() {
                    let target_state = dfa.add_state_if_new(target_states, &accepting_states)?;
                    dfa.transitions
                        .entry(state_id)
                        .or_default()
                        .insert(char_class.id(), target_state);
                    if !dfa.states[target_state].marked() {
                        dfa.states[target_state].set_marked(true);
                        work_list.push(target_state);
                    }
                }
            }
        }

        Ok(dfa)
    }

    /// Add a state to the DFA if it does not already exist.
    /// The state is identified by the NFA states that constitute the DFA state.
    /// The accepting state is used to determine if the DFA state is an accepting state.
    /// The NFA only has one accepting state, it is the end state of the NFA.
    pub(crate) fn add_state_if_new<I>(
        &mut self,
        nfa_states: I,
        accepting_nfa_states: &[(StateID, TerminalID)],
    ) -> Result<StateID>
    where
        I: IntoIterator<Item = StateID>,
    {
        let mut nfa_states: Vec<StateID> = nfa_states.into_iter().collect();
        nfa_states.sort_unstable();
        nfa_states.dedup();
        if let Some(state_id) = self
            .states
            .iter()
            .position(|state| state.nfa_states == nfa_states)
        {
            return Ok(StateID::new(state_id as StateIDBase));
        }

        let state_id = StateID::new(self.states.len() as StateIDBase);
        let mut state = DfaState::new(state_id, nfa_states);

        // Check if the constraint holds that only one pattern can match, i.e. the DFA
        // state only contains one accpting NFA state. This should always be the case since
        // the NFA is a single-pattern NFA.
        debug_assert!(
            state
                .nfa_states
                .iter()
                .filter(|nfa_state_id| accepting_nfa_states.iter().any(|a| a.0 == **nfa_state_id))
                .count()
                <= 1
        );

        // Check if the state contains an accepting state and if so add it to the accepting states.
        for nfa_state_id in &state.nfa_states {
            if let Some((_, t)) = accepting_nfa_states.iter().find(|a| a.0 == *nfa_state_id) {
                // The state is an accepting state.
                trace!(
                    "* State {} is an accepting state for terminal {}.",
                    state_id.id(),
                    t
                );
                self.accepting_states.insert((state_id, *t));
                state.terminal_id = Some(*t);
            }
        }

        trace!("Add state: {}: {:?}", state.id.as_usize(), state.nfa_states);

        self.states.push(state);
        Ok(state_id)
    }

    /// Add a representative state to the DFA.
    /// The representative state is the first state in the group.
    /// The accepting states are used to determine if the DFA state is an accepting state.
    /// The new state id is returned.
    fn add_representive_state(
        &mut self,
        group: &BTreeSet<StateID>,
        accepting_states: &BTreeSet<(StateID, TerminalID)>,
    ) -> Result<StateID> {
        let state_id = StateID::new(self.states.len() as StateIDBase);
        let mut state = DfaState::new(state_id, Vec::new());

        // First state in group is the representative state.
        let representative_state_id = group.first().unwrap();

        trace!(
            "Add representive state {} with id {} and terminal id {:?}.",
            representative_state_id.id(),
            state_id.id(),
            state.terminal_id
        );

        // Insert the representative state into the accepting states if any state in its group is
        // an accepting state.
        for state_in_group in group.iter() {
            if let Some((_, t)) = accepting_states.iter().find(|a| a.0 == *state_in_group) {
                trace!(
                    "* State {} is accepting state (from state {}) and terminal id {:?}.",
                    state_id.as_usize(),
                    state_in_group.as_usize(),
                    state.terminal_id
                );
                state.terminal_id = Some(*t);
                self.accepting_states.insert((state_id, *t));
            }
        }

        self.states.push(state);
        Ok(state_id)
    }

    /// Trace out a partition of the DFA.
    #[allow(dead_code)]
    fn trace_partition(context: &str, partition: &[StateGroup]) {
        trace!("Partition {}:", context);
        for (i, group) in partition.iter().enumerate() {
            trace!("Group {}: {:?}", i, group);
        }
    }

    #[allow(dead_code)]
    fn trace_transitions_to_groups(
        state_id: StateID,
        transitions_to_groups: &TransitionsToPartitionGroups,
    ) {
        trace!("Transitions of state {} to groups:", state_id.as_usize());
        for (char_class, group) in &transitions_to_groups.0 {
            trace!("{} -> {}", char_class, group);
        }
    }

    /// Minimize the DFA.
    /// The Nfa states are removed from the DFA states during minimization. They are not needed
    /// anymore after the DFA is created.
    pub(crate) fn minimize(&self) -> Result<Self> {
        // Check in DEBUG mode that the DFA states have increasing ids.
        debug_assert!(self
            .states
            .iter()
            .enumerate()
            .all(|(i, state)| state.id.as_usize() == i));

        trace!("Minimize DFA ----------------------------");
        trace!("Initial DFA:\n{}", self);
        let mut partition_old = self.calculate_initial_partition();
        Self::trace_partition("initial", &partition_old);
        let mut partition_new = Partition::new();
        let mut changed = true;

        while changed {
            partition_new = self.calculate_new_partition(&partition_old);
            Self::trace_partition("new", &partition_new);
            changed = partition_new != partition_old;
            partition_old.clone_from(&partition_new);
        }

        self.create_from_partition(&partition_new)
    }

    /// The start partition is created as follows:
    /// 1. The accepting states are put each in a partition with states having the same terminal id
    ///    they match.
    /// 2. The non-accepting states are put together in one partition that has the id 1.
    ///
    /// The partitions are stored in a vector of vectors.
    ///
    /// The key building function for the Itertools::chunk_by method is used to create the
    /// partitions. For accepting states the key is the state id, for non-accepting states
    /// the key is the state id of the first non-accepting state.
    fn calculate_initial_partition(&self) -> Partition {
        // Group 0
        let mut group_non_accepting_states = StateGroup::new();
        let mut group_accepting_states_groups = BTreeMap::<TerminalID, StateGroup>::new();

        self.states.clone().into_iter().for_each(|state| {
            if self.is_accepting(state.id) {
                group_accepting_states_groups
                    .entry(state.terminal_id.unwrap())
                    .or_default()
                    .insert(state.id);
            } else {
                group_non_accepting_states.insert(state.id);
            }
        });
        let mut initial_partition = vec![group_non_accepting_states];
        for group in group_accepting_states_groups.values() {
            initial_partition.push(group.clone());
        }
        initial_partition
    }

    /// Calculate the new partition based on the old partition.
    /// We try to split the groups of the partition based on the transitions of the DFA.
    /// The new partition is calculated by iterating over the old partition and the states
    /// in the groups. For each state in a group we check if the transitions to the states in the
    /// old partition's groups are the same. If the transitions are the same, the state is put in
    /// the same group as the other states with the same transitions. If the transitions are
    /// different, the state is put in a new group.
    /// The new partition is returned.
    fn calculate_new_partition(&self, partition: &[StateGroup]) -> Partition {
        let mut new_partition = Partition::new();
        for (index, group) in partition.iter().enumerate() {
            // The new group receives the states from the old group which are distiguishable from
            // the other states in group.
            self.split_group(index, group, partition)
                .into_iter()
                .for_each(|new_group| {
                    new_partition.push(new_group);
                });
        }
        new_partition
    }

    fn split_group(
        &self,
        group_index: usize,
        group: &StateGroup,
        partition: &[StateGroup],
    ) -> Partition {
        // If the group contains only one state, the group can't be split further.
        if group.len() == 1 {
            return vec![group.clone()];
        }
        trace!("Split group {}: {:?}", group_index, group);
        let mut transition_map_to_states: BTreeMap<TransitionsToPartitionGroups, StateGroup> =
            BTreeMap::new();
        for state_id in group {
            let transitions_to_partition =
                self.build_transitions_to_partition_group(*state_id, partition);
            transition_map_to_states
                .entry(transitions_to_partition)
                .or_default()
                .insert(*state_id);
        }
        transition_map_to_states
            .into_values()
            .collect::<Partition>()
    }

    /// Build a modified transition data structure of a given DFA state that maps states to the
    /// partition group.
    /// The partition group is the index of the group in the partition.
    /// The modified transition data structure is returned.
    /// The modified transition data structure is used to determine if two states are distinguish
    /// based on the transitions of the DFA.
    fn build_transitions_to_partition_group(
        &self,
        state_id: StateID,
        partition: &[StateGroup],
    ) -> TransitionsToPartitionGroups {
        if let Some(transitions_of_state) = self.transitions.get(&state_id) {
            let mut transitions_to_partition_groups = TransitionsToPartitionGroups::new();
            for transition in transitions_of_state {
                let partition_group = self.find_group(*transition.1, partition).unwrap();
                transitions_to_partition_groups.insert(*transition.0, partition_group);
            }
            Self::trace_transitions_to_groups(state_id, &transitions_to_partition_groups);
            transitions_to_partition_groups
        } else {
            trace!("** State {} has no transitions.", state_id.as_usize());
            TransitionsToPartitionGroups::new()
        }
    }

    fn find_group(&self, state_id: StateID, partition: &[StateGroup]) -> Option<usize> {
        partition.iter().position(|group| group.contains(&state_id))
    }

    /// Create a DFA from a partition.
    /// If a StateGroup contains more than one state, the states are merged into one state.
    /// The transitions are updated accordingly.
    /// The accepting states are updated accordingly.
    /// The new DFA is returned.
    fn create_from_partition(&self, partition: &[StateGroup]) -> Result<Dfa> {
        trace!("Create DFA ------------------------------");
        trace!("from partition {:?}", partition);
        let mut dfa = Dfa {
            states: Vec::new(),
            patterns: self.patterns.clone(),
            accepting_states: BTreeSet::new(),
            transitions: self.transitions.clone(),
        };

        // Reorder the groups so that the start state is in the first group (0).
        // The representative state of the first group must be the start state of the minimized DFA,
        // even after minimization.
        let mut partition = partition.to_vec();
        partition.sort_by(|a, b| {
            if a.contains(&StateID::new(0)) {
                return std::cmp::Ordering::Less;
            }
            if b.contains(&StateID::new(0)) {
                return std::cmp::Ordering::Greater;
            }
            std::cmp::Ordering::Equal
        });

        // Then add the representative states to the DFA from the other groups.
        for group in &partition {
            // For each group we add a representative state to the DFA.
            // It's id is the index of the group in the partition.
            // This function also updates the accepting states of the DFA.
            dfa.add_representive_state(group, &self.accepting_states)?;
        }

        // Then renumber the states in the transitions.
        dfa.update_transitions(&partition);

        trace!("Minimized DFA:\n{}", dfa);

        Ok(dfa)
    }

    fn update_transitions(&mut self, partition: &[StateGroup]) {
        // Create a vector because we dont want to mess the transitins map while renumbering.
        let mut transitions = self
            .transitions
            .iter()
            .map(|(s, t)| (*s, t.clone()))
            .collect::<Vec<_>>();

        Self::merge_transitions(partition, &mut transitions);
        Self::renumber_states_in_transitions(partition, &mut transitions);

        self.transitions = transitions.into_iter().collect();
    }

    fn merge_transitions(
        partition: &[StateGroup],
        transitions: &mut Vec<(StateID, BTreeMap<CharClassID, StateID>)>,
    ) {
        // Remove all transitions that do not belong to the representive states of a group.
        // The representive states are the first states in the groups.
        for group in partition {
            debug_assert!(!group.is_empty());
            if group.len() == 1 {
                continue;
            }
            let representive_state_id = group.first().unwrap();
            for state_id in group.iter().skip(1) {
                Self::merge_transitions_of_state(*state_id, *representive_state_id, transitions);
            }
        }
    }

    fn merge_transitions_of_state(
        state_id: StateID,
        representive_state_id: StateID,
        transitions: &mut Vec<(StateID, BTreeMap<CharClassID, StateID>)>,
    ) {
        if let Some(rep_pos) = transitions
            .iter()
            .position(|(s, _)| *s == representive_state_id)
        {
            let mut rep_trans = transitions.get_mut(rep_pos).unwrap().1.clone();
            if let Some(pos) = transitions.iter().position(|(s, _)| *s == state_id) {
                let (_, transitions_of_state) = transitions.get_mut(pos).unwrap();
                for (char_class, target_state) in transitions_of_state.iter() {
                    rep_trans.insert(*char_class, *target_state);
                }
                // Remove the transitions of the state that is merged into the representative state.
                transitions.remove(pos);
            }
            transitions[rep_pos].1 = rep_trans;
        }
    }

    fn renumber_states_in_transitions(
        partition: &[StateGroup],
        transitions: &mut [(StateID, BTreeMap<CharClassID, StateID>)],
    ) {
        let find_group_of_state = |state_id: StateID| -> StateID {
            for (group_id, group) in partition.iter().enumerate() {
                if group.contains(&state_id) {
                    return StateID::new(group_id as StateIDBase);
                }
            }
            panic!("State {} not found in partition.", state_id.as_usize());
        };

        for transition in transitions.iter_mut() {
            transition.0 = find_group_of_state(transition.0);
            for target_state in transition.1.values_mut() {
                *target_state = find_group_of_state(*target_state);
            }
        }
    }
}

impl std::fmt::Display for Dfa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "DFA")?;
        writeln!(f, "States:")?;
        for state in &self.states {
            writeln!(f, "{:?}", state)?;
        }
        writeln!(f, "Pattern:")?;
        writeln!(f, "{:?}", self.patterns)?;
        writeln!(f, "Accepting states:")?;
        for state_id in &self.accepting_states {
            writeln!(f, "{}:T{}", state_id.0.id(), state_id.1.id())?;
        }
        writeln!(f, "Transitions:")?;
        for (source_id, targets) in &self.transitions {
            write!(f, "{} -> ", source_id.as_usize())?;
            for (char_class, target_id) in targets {
                write!(f, "{}:{}, ", char_class.id(), target_id.as_usize())?;
            }
            writeln!(f)?
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct DfaState {
    id: StateID,
    // The ids of the NFA states that constitute this DFA state. The id can only be used as indices
    // into the NFA states.
    nfa_states: Vec<StateID>,
    // The marked flag is used to mark a state as visited during the subset construction algorithm.
    marked: bool,
    /// The terminal id of the state if it is an accepting state.
    terminal_id: Option<TerminalID>,
}

impl DfaState {
    /// Create a new DFA state solely from the NFA states that constitute the DFA state.
    pub(crate) fn new(id: StateID, nfa_states: Vec<StateID>) -> Self {
        DfaState {
            id,
            nfa_states,
            ..Default::default()
        }
    }

    /// Get the id of the DFA state.
    #[allow(dead_code)]
    pub(crate) fn id(&self) -> StateID {
        self.id
    }

    /// Get the NFA states that constitute the DFA state.
    #[allow(dead_code)]
    pub(crate) fn nfa_states(&self) -> &[StateID] {
        &self.nfa_states
    }

    /// Get the marked flag of the DFA state.
    #[allow(dead_code)]
    pub(crate) fn marked(&self) -> bool {
        self.marked
    }

    /// Set the marked flag of the DFA state.
    #[allow(dead_code)]
    pub(crate) fn set_marked(&mut self, marked: bool) {
        self.marked = marked;
    }

    /// Set the terminal id of the DFA state.
    /// The terminal id is used to determine the pattern that matched the input string.
    pub(crate) fn set_terminal_id(&mut self, terminal_id: TerminalID) {
        self.terminal_id = Some(terminal_id);
    }

    /// Get the terminal id of the DFA state.
    /// The terminal id is used to determine the pattern that matched the input string.
    pub(crate) fn terminal_id(&self) -> Option<TerminalID> {
        self.terminal_id
    }
}

#[cfg(test)]
mod tests {

    use std::{fs, sync::LazyLock};

    use crate::{
        internal::{
            parser::parse_regex_syntax, CharClassID, CharacterClass, CharacterClassRegistry,
        },
        ScannerMode,
    };

    use super::*;

    static SCANNER_MODES: LazyLock<Vec<ScannerMode>> = LazyLock::new(|| {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/parol.json");
        let file = fs::File::open(path).unwrap();
        serde_json::from_reader(file).unwrap()
    });

    struct TestData {
        // Use pascal case for the name because the name is used also as dot file name.
        // Also, the name should be unique.
        name: &'static str,
        pattern: &'static str,
        states: Vec<StateID>,
        accepting_states: Vec<(StateID, TerminalID)>,
        char_classes: Vec<CharacterClass>,
        transitions: BTreeMap<StateID, BTreeMap<CharClassID, StateID>>,
    }

    /// A macro that parses a regex pattern and creates a Ast from it.
    macro_rules! parse {
        ($pattern:expr) => {
            $crate::internal::parse_regex_syntax($pattern).unwrap()
        };
    }

    /// A macro that simplifies the rendering of a dot file for a DFA.
    macro_rules! dfa_render_to {
        ($dfa:expr, $label:expr, $reg:ident) => {
            let label = format!("{}Dfa", $label);
            let mut f = std::fs::File::create(format!("target/{}Dfa.dot", $label)).unwrap();
            $crate::internal::dot::dfa_render($dfa, &label, &$reg, &mut f);
        };
    }

    /// A macro that simplifies the rendering of a dot file for a NFA.
    macro_rules! nfa_render_to {
        ($nfa:expr, $label:expr) => {
            let label = format!("{}Nfa", $label);
            let mut f = std::fs::File::create(format!("target/{}Nfa.dot", $label)).unwrap();
            $crate::internal::dot::nfa_render($nfa, &label, &mut f);
        };
    }

    /// A macro that simplifies the rendering of a dot file for a MultiPatternNfa.
    macro_rules! mp_nfa_render_to {
        ($nfa:expr, $label:expr, $char_class:ident) => {
            let label = format!("{}MpNfa", $label);
            let mut f = std::fs::File::create(format!("target/{}MpNfa.dot", $label)).unwrap();
            $crate::internal::dot::multi_pattern_nfa_render($nfa, &label, &$char_class, &mut f);
        };
    }

    static TEST_DATA: LazyLock<[TestData; 10]> = LazyLock::new(|| {
        [
            TestData {
                name: "SingleCharacter",
                pattern: "a",
                states: vec![StateID::new(0), StateID::new(1)],
                accepting_states: vec![(StateID::new(1), TerminalID::new(0))],
                char_classes: vec![CharacterClass::new(CharClassID::new(0), parse!("a"))],
                transitions: BTreeMap::from([(
                    StateID::new(0),
                    BTreeMap::from([(CharClassID::new(0), StateID::new(1))]),
                )]),
            },
            TestData {
                name: "Alternation",
                pattern: "a|b",
                states: vec![StateID::new(0), StateID::new(1)],
                accepting_states: vec![(StateID::new(1), TerminalID::new(0))],
                char_classes: vec![
                    CharacterClass::new(CharClassID::new(0), parse!("a")),
                    CharacterClass::new(CharClassID::new(1), parse!("b")),
                ],
                transitions: BTreeMap::from([(
                    StateID::new(0),
                    BTreeMap::from([
                        (CharClassID::new(0), StateID::new(1)),
                        (CharClassID::new(1), StateID::new(1)),
                    ]),
                )]),
            },
            TestData {
                name: "Concatenation",
                pattern: "ab",
                states: vec![StateID::new(0), StateID::new(1), StateID::new(2)],
                accepting_states: vec![(StateID::new(2), TerminalID::new(0))],
                char_classes: vec![
                    CharacterClass::new(CharClassID::new(0), parse!("a")),
                    CharacterClass::new(CharClassID::new(1), parse!("b")),
                ],
                transitions: BTreeMap::from([
                    (
                        StateID::new(0),
                        BTreeMap::from([(CharClassID::new(0), StateID::new(1))]),
                    ),
                    (
                        StateID::new(1),
                        BTreeMap::from([(CharClassID::new(1), StateID::new(2))]),
                    ),
                ]),
            },
            TestData {
                name: "KleeneStar",
                pattern: "a*",
                states: vec![StateID::new(0)],
                accepting_states: vec![(StateID::new(0), TerminalID::new(0))],
                char_classes: vec![CharacterClass::new(CharClassID::new(0), parse!("a"))],
                transitions: BTreeMap::from([(
                    StateID::new(0),
                    BTreeMap::from([(CharClassID::new(0), StateID::new(0))]),
                )]),
            },
            TestData {
                name: "KleeneStarAlternation",
                pattern: "(a|b)*",
                states: vec![StateID::new(0)],
                accepting_states: vec![(StateID::new(0), TerminalID::new(0))],
                char_classes: vec![
                    CharacterClass::new(CharClassID::new(0), parse!("a")),
                    CharacterClass::new(CharClassID::new(1), parse!("b")),
                ],
                transitions: BTreeMap::from([(
                    StateID::new(0),
                    BTreeMap::from([
                        (CharClassID::new(0), StateID::new(0)),
                        (CharClassID::new(1), StateID::new(0)),
                    ]),
                )]),
            },
            TestData {
                name: "KleeneStarConcatenation",
                pattern: "(ab)*",
                states: vec![StateID::new(0), StateID::new(1)],
                accepting_states: vec![(StateID::new(0), TerminalID::new(0))],
                char_classes: vec![
                    CharacterClass::new(CharClassID::new(0), parse!("a")),
                    CharacterClass::new(CharClassID::new(1), parse!("b")),
                ],
                transitions: BTreeMap::from([
                    (
                        StateID::new(0),
                        BTreeMap::from([(CharClassID::new(0), StateID::new(1))]),
                    ),
                    (
                        StateID::new(1),
                        BTreeMap::from([(CharClassID::new(1), StateID::new(0))]),
                    ),
                ]),
            },
            TestData {
                name: "KleeneStarConcatenationAlternation",
                pattern: "(a|b)*c",
                states: vec![StateID::new(0), StateID::new(1)],
                accepting_states: vec![(StateID::new(1), TerminalID::new(0))],
                char_classes: vec![
                    CharacterClass::new(CharClassID::new(0), parse!("a")),
                    CharacterClass::new(CharClassID::new(1), parse!("b")),
                    CharacterClass::new(CharClassID::new(2), parse!("c")),
                ],
                transitions: BTreeMap::from([(
                    StateID::new(0),
                    BTreeMap::from([
                        (CharClassID::new(0), StateID::new(0)),
                        (CharClassID::new(1), StateID::new(0)),
                        (CharClassID::new(2), StateID::new(1)),
                    ]),
                )]),
            },
            TestData {
                name: "Complex",
                pattern: "(a|b)*abb",
                states: vec![
                    StateID::new(0),
                    StateID::new(1),
                    StateID::new(2),
                    StateID::new(3),
                ],
                accepting_states: vec![(StateID::new(3), TerminalID::new(0))],
                char_classes: vec![
                    CharacterClass::new(CharClassID::new(0), parse!("a")),
                    CharacterClass::new(CharClassID::new(1), parse!("b")),
                ],
                transitions: BTreeMap::from([
                    (
                        StateID::new(0),
                        BTreeMap::from([
                            (CharClassID::new(0), StateID::new(1)),
                            (CharClassID::new(1), StateID::new(0)),
                        ]),
                    ),
                    (
                        StateID::new(1),
                        BTreeMap::from([
                            (CharClassID::new(0), StateID::new(1)),
                            (CharClassID::new(1), StateID::new(2)),
                        ]),
                    ),
                    (
                        StateID::new(2),
                        BTreeMap::from([
                            (CharClassID::new(0), StateID::new(1)),
                            (CharClassID::new(1), StateID::new(3)),
                        ]),
                    ),
                    (
                        StateID::new(3),
                        BTreeMap::from([
                            (CharClassID::new(0), StateID::new(1)),
                            (CharClassID::new(1), StateID::new(0)),
                        ]),
                    ),
                ]),
            },
            TestData {
                name: "LineComment",
                pattern: "(//.*(\r\n|\r|\n))",
                states: vec![
                    StateID::new(0),
                    StateID::new(1),
                    StateID::new(2),
                    StateID::new(3),
                    StateID::new(5),
                ],
                accepting_states: vec![
                    (StateID::new(3), TerminalID::new(0)),
                    (StateID::new(4), TerminalID::new(0)),
                ],
                char_classes: vec![
                    CharacterClass::new(CharClassID::new(0), parse!("/")),
                    CharacterClass::new(CharClassID::new(1), parse!(".")),
                    CharacterClass::new(CharClassID::new(2), parse!("\r")),
                    CharacterClass::new(CharClassID::new(3), parse!("\n")),
                ],
                transitions: BTreeMap::from([
                    (
                        StateID::new(0),
                        BTreeMap::from([(CharClassID::new(0), StateID::new(1))]),
                    ),
                    (
                        StateID::new(1),
                        BTreeMap::from([(CharClassID::new(0), StateID::new(2))]),
                    ),
                    (
                        StateID::new(2),
                        BTreeMap::from([
                            (CharClassID::new(1), StateID::new(2)),
                            (CharClassID::new(3), StateID::new(3)),
                            (CharClassID::new(2), StateID::new(4)),
                        ]),
                    ),
                    (
                        StateID::new(4),
                        BTreeMap::from([(CharClassID::new(3), StateID::new(3))]),
                    ),
                ]),
            },
            TestData {
                name: "BlockComment",
                pattern: r"/\*([.\r\n--*]|\*[^/])*\*/",
                states: vec![
                    StateID::new(0),
                    StateID::new(1),
                    StateID::new(2),
                    StateID::new(3),
                    StateID::new(4),
                ],
                accepting_states: vec![(StateID::new(4), TerminalID::new(0))],
                char_classes: vec![
                    CharacterClass::new(CharClassID::new(0), parse!("/")),
                    CharacterClass::new(CharClassID::new(1), parse!(r"\*")),
                    CharacterClass::new(CharClassID::new(2), parse!(r"[.\r\n--*]")),
                    CharacterClass::new(CharClassID::new(3), parse!("[^/]")),
                ],
                transitions: BTreeMap::from([
                    (
                        StateID::new(0),
                        BTreeMap::from([(CharClassID::new(0), StateID::new(2))]),
                    ),
                    (
                        StateID::new(1),
                        BTreeMap::from([
                            (CharClassID::new(0), StateID::new(4)),
                            (CharClassID::new(3), StateID::new(3)),
                        ]),
                    ),
                    (
                        StateID::new(2),
                        BTreeMap::from([(CharClassID::new(1), StateID::new(3))]),
                    ),
                    (
                        StateID::new(3),
                        BTreeMap::from([
                            (CharClassID::new(1), StateID::new(1)),
                            (CharClassID::new(2), StateID::new(3)),
                        ]),
                    ),
                ]),
            },
        ]
    });

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_try_from_nfa() {
        init();
        for data in TEST_DATA.iter() {
            let mut char_class_registry = CharacterClassRegistry::new();
            let nfa = Nfa::try_from_ast(
                parse_regex_syntax(data.pattern).unwrap(),
                &mut char_class_registry,
            )
            .unwrap();
            nfa_render_to!(&nfa, data.name);
            let dfa = Dfa::try_from_nfa(nfa, 0usize.into(), &mut char_class_registry).unwrap();
            dfa_render_to!(&dfa, data.name, char_class_registry);
            let dfa = Dfa::minimize(&dfa).unwrap();
            dfa_render_to!(&dfa, format!("{}Min", data.name), char_class_registry);
            assert_eq!(
                dfa.states.len(),
                data.states.len(),
                "dfa state count for '{}:{}' is wrong",
                data.name,
                data.pattern.escape_default()
            );
            assert_eq!(
                dfa.accepting_states.iter().cloned().collect::<Vec<_>>(),
                data.accepting_states,
                "dfa accepting states for '{}:{}' are wrong",
                data.name,
                data.pattern.escape_default()
            );

            assert_eq!(
                char_class_registry.character_classes().to_vec(),
                data.char_classes,
                "dfa char classes for '{}:{}' are wrong",
                data.name,
                data.pattern.escape_default()
            );
            assert_eq!(
                dfa.transitions,
                data.transitions,
                "dfa transitions for '{}:{}' are wrong",
                data.name,
                data.pattern.escape_default()
            );
        }
    }

    #[test]
    fn test_try_from_mp_nfa_simple() {
        init();
        let mut character_class_registry = CharacterClassRegistry::new();
        let multi_pattern_nfa = MultiPatternNfa::try_from_patterns(
            &[
                ("aab".to_owned(), 0usize.into()),
                ("abb".to_owned(), 1usize.into()),
                // ("abc", 2),
                // ("bbc", 3),
                // ("bca", 4),
            ],
            &mut character_class_registry,
        )
        .unwrap();
        mp_nfa_render_to!(
            &multi_pattern_nfa,
            "SimpleMultiPattern",
            character_class_registry
        );
        let dfa = Dfa::try_from_mp_nfa(multi_pattern_nfa, &mut character_class_registry).unwrap();
        dfa_render_to!(&dfa, "SimpleMultiPattern", character_class_registry);
        let dfa = Dfa::minimize(&dfa).unwrap();
        dfa_render_to!(&dfa, "SimpleMultiPatternMin", character_class_registry);
    }

    #[test]
    fn test_try_from_mp_nfa() {
        init();
        let mut character_class_registry = CharacterClassRegistry::new();
        let multi_pattern_nfa = MultiPatternNfa::try_from_patterns(
            &SCANNER_MODES.iter().next().unwrap().patterns,
            &mut character_class_registry,
        )
        .unwrap();
        let dfa = Dfa::try_from_mp_nfa(multi_pattern_nfa, &mut character_class_registry).unwrap();
        dfa_render_to!(&dfa, "MultiPattern", character_class_registry);
        let dfa = Dfa::minimize(&dfa).unwrap();
        dfa_render_to!(&dfa, "MultiPatternMin", character_class_registry);
    }
}
