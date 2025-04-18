use std::{
    collections::{BTreeMap, BTreeSet},
    vec,
};

use log::trace;

use crate::internal::compiled_dfa::StateData;

use super::{
    compiled_dfa::CompiledDfa,
    ids::{StateGroupID, StateGroupIDBase, StateID},
    CharClassID, StateIDBase, TerminalID,
};

// The type definitions for the subset construction algorithm.

// A state group is a sorted set of states that are in the same partition group.
type StateGroup = BTreeSet<StateID>;
// A partition is a vector of state groups.
type Partition = Vec<StateGroup>;

// A transition map is a map of state ids to a map of character class ids to state set ids.
type TransitionMap = BTreeMap<StateID, BTreeMap<CharClassID, Vec<StateID>>>;

// A data type that is calculated from the transitions of a DFA state so that for each character
// class the target state is mapped to the partition group it belongs to.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct TransitionsToPartitionGroups(pub(crate) Vec<(CharClassID, StateGroupID)>);

impl TransitionsToPartitionGroups {
    pub(crate) fn new() -> Self {
        TransitionsToPartitionGroups::default()
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        TransitionsToPartitionGroups(Vec::with_capacity(capacity))
    }

    pub(crate) fn insert(&mut self, char_class: CharClassID, partition_group: StateGroupID) {
        self.0.push((char_class, partition_group));
    }
}

// The minimizer is a struct that is used to minimize the number of states in a DFA.
#[derive(Debug)]
pub(crate) struct Minimizer;

impl Minimizer {
    /// Minimize the DFA.
    /// The minimization is done using the subset construction algorithm.
    /// The method takes a DFA and returns a minimized DFA.
    pub(crate) fn minimize(dfa: CompiledDfa) -> CompiledDfa {
        trace!("Minimize DFA ----------------------------");
        trace!("Initial DFA:\n{}", dfa);
        // The transitions of the DFA in a convenient data structure.
        let mut transitions = TransitionMap::new();
        dfa.states.iter().enumerate().for_each(|(id, state)| {
            transitions.entry((id as StateIDBase).into()).or_default();
            for t in &state.transitions {
                let t_of_s = transitions.get_mut(&(id as StateIDBase).into()).unwrap();
                t_of_s.entry(t.0).or_default().push(t.1.id().into());
                t_of_s.get_mut(&t.0).unwrap().sort();
                t_of_s.get_mut(&t.0).unwrap().dedup();
            }
        });

        trace!("Transitions: {:?}", transitions);

        // The initial partition is created.
        let mut partition_old = Self::calculate_initial_partition(&dfa);
        Self::trace_partition("initial", &partition_old);
        let mut partition_new = Partition::new();
        let mut changed = true;
        while changed {
            partition_new = Self::calculate_new_partition(&partition_old, &transitions);
            Self::trace_partition("new", &partition_new);
            changed = partition_new != partition_old;
            partition_old.clone_from(&partition_new);
        }

        Self::create_from_partition(dfa, &partition_new, &transitions)
    }

    /// The start partition is created as follows:
    /// 1. The accepting states are put each in a separate group with group id set to terminal
    ///    id + 1.
    ///    This follows from the constraint of the DFA that multiple patterns can match.
    /// 2. The non-accepting states are put together in one group with the id 0.
    ///
    /// The partitions are stored in a vector of vectors.
    fn calculate_initial_partition(dfa: &CompiledDfa) -> Partition {
        let mut terminal_map = dfa
            .end_states
            .iter()
            .filter_map(|(accept, id)| if *accept { Some(*id) } else { None })
            .collect::<Vec<_>>();
        terminal_map.sort();
        terminal_map.dedup();
        let number_of_end_states = terminal_map.len();
        let mut initial_partition = vec![StateGroup::new(); number_of_end_states + 1];

        for state in 0..dfa.states.len() {
            let state: StateID = (state as u32).into();
            if dfa.end_states[state].0 {
                let terminal_id = dfa.end_states[state].1;
                let index = terminal_map
                    .iter()
                    .position(|id| *id == terminal_id)
                    .unwrap();
                initial_partition[index + 1].insert(state);
            } else {
                initial_partition[0].insert(state);
            }
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
    fn calculate_new_partition(partition: &[StateGroup], transitions: &TransitionMap) -> Partition {
        let mut new_partition = Partition::new();
        for (index, group) in partition.iter().enumerate() {
            // The new group receives the states from the old group which are distinguishable from
            // the other states in group.
            Self::split_group(index, group, partition, transitions)
                .into_iter()
                .for_each(|new_group| {
                    new_partition.push(new_group);
                });
        }
        new_partition
    }

    fn split_group(
        group_index: usize,
        group: &StateGroup,
        partition: &[StateGroup],
        transitions: &TransitionMap,
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
                Self::build_transitions_to_partition_group(*state_id, partition, transitions);
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
        state_id: StateID,
        partition: &[StateGroup],
        transitions: &TransitionMap,
    ) -> TransitionsToPartitionGroups {
        if let Some(transitions_of_state) = transitions.get(&state_id) {
            let mut transitions_to_partition_groups =
                TransitionsToPartitionGroups::with_capacity(transitions_of_state.len());
            for transition in transitions_of_state {
                for target_state in transition.1.iter() {
                    let partition_group = Self::find_group(*target_state, partition).unwrap();
                    transitions_to_partition_groups.insert(*transition.0, partition_group);
                }
            }
            Self::trace_transitions_to_groups(state_id, &transitions_to_partition_groups);
            transitions_to_partition_groups
        } else {
            trace!("** State {} has no transitions.", state_id);
            TransitionsToPartitionGroups::new()
        }
    }

    fn find_group(state_id: StateID, partition: &[StateGroup]) -> Option<StateGroupID> {
        partition
            .iter()
            .position(|group| group.contains(&state_id))
            .map(|id| (id as StateGroupIDBase).into())
    }

    /// Create a DFA from a partition.
    /// If a StateGroup contains more than one state, the states are merged into one state.
    /// The transitions are updated accordingly.
    /// The accepting states are updated accordingly.
    /// The new DFA is returned.
    fn create_from_partition(
        dfa: CompiledDfa,
        partition: &[StateGroup],
        transitions: &TransitionMap,
    ) -> CompiledDfa {
        trace!("Create DFA ------------------------------");
        trace!("from partition {:?}", partition);
        let CompiledDfa {
            patterns,
            terminal_ids,
            states: _,
            end_states,
            lookaheads,
            ..
        } = dfa;
        let mut dfa = CompiledDfa::new(
            patterns,
            terminal_ids,
            vec![StateData::new(); partition.len()],
            vec![(false, 0.into()); partition.len()],
            lookaheads,
        );

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
        for (id, group) in partition.iter().enumerate() {
            // For each group we add a representative state to the DFA.
            // It's id is the index of the group in the partition.
            // This function also updates the accepting states of the DFA.
            Self::add_representative_state(
                &mut dfa,
                (id as StateGroupIDBase).into(),
                group,
                &end_states,
            );
        }

        // Then renumber the states in the transitions.
        Self::update_transitions(&mut dfa, &partition, transitions);

        trace!("Minimized DFA:\n{}", dfa);

        dfa
    }

    /// Add a representative state to the DFA.
    /// The representative state is the first state in the group.
    /// The accepting states are used to determine if the DFA state is an accepting state.
    /// The new state id is returned.
    fn add_representative_state(
        dfa: &mut CompiledDfa,
        group_id: StateGroupID,
        group: &BTreeSet<StateID>,
        end_states: &[(bool, TerminalID)],
    ) -> StateID {
        let state_id = StateID::new(group_id.id() as StateIDBase);
        let state = StateData::new();

        // First state in group is the representative state.
        let representative_state_id = group.first().unwrap();

        trace!(
            "Add representative state {} with id {}",
            representative_state_id.as_usize(),
            state_id.as_usize()
        );

        // Insert the representative state into the accepting states if any state in its group is
        // an accepting state.
        for state_in_group in group.iter() {
            if end_states[*state_in_group].0 {
                dfa.end_states[state_id] = (true, end_states[*state_in_group].1);
            }
        }

        dfa.states[state_id] = state;
        state_id
    }

    fn update_transitions(
        dfa: &mut CompiledDfa,
        partition: &[StateGroup],
        transitions: &TransitionMap,
    ) {
        // Create a vector because we dont want to mess the transitions map while renumbering.
        let mut transitions = transitions
            .iter()
            .map(|(s, t)| (*s, t.clone()))
            .collect::<Vec<_>>();

        Self::merge_transitions(partition, &mut transitions);
        Self::renumber_states_in_transitions(partition, &mut transitions);

        // Update the transitions of the DFA.
        for (state_id, transitions_of_state) in transitions {
            let state_id = state_id.as_usize();
            for (char_class, target_states) in transitions_of_state.iter() {
                for target_state in target_states {
                    trace!(
                        "Add transition {} --{}--> {}",
                        state_id,
                        char_class,
                        target_state
                    );
                    if !dfa.states[state_id]
                        .transitions
                        .contains(&(*char_class, target_state.id().into()))
                    {
                        dfa.states[state_id]
                            .transitions
                            .push((*char_class, target_state.id().into()));
                    }
                }
            }
        }
    }

    fn merge_transitions(
        partition: &[StateGroup],
        transitions: &mut Vec<(StateID, BTreeMap<CharClassID, Vec<StateID>>)>,
    ) {
        // Remove all transitions that do not belong to the representative states of a group.
        // The representative states are the first states in the groups.
        for group in partition {
            debug_assert!(!group.is_empty());
            if group.len() == 1 {
                continue;
            }
            let representative_state_id = group.first().unwrap();
            for state_id in group.iter().skip(1) {
                Self::merge_transitions_of_state(*state_id, *representative_state_id, transitions);
            }
        }
    }

    fn merge_transitions_of_state(
        state_id: StateID,
        representative_state_id: StateID,
        transitions: &mut Vec<(StateID, BTreeMap<CharClassID, Vec<StateID>>)>,
    ) {
        if let Some(rep_pos) = transitions
            .iter()
            .position(|(s, _)| *s == representative_state_id)
        {
            let mut rep_trans = transitions.get_mut(rep_pos).unwrap().1.clone();
            if let Some(pos) = transitions.iter().position(|(s, _)| *s == state_id) {
                let (_, transitions_of_state) = transitions.get_mut(pos).unwrap();
                for (char_class, target_states) in transitions_of_state.iter() {
                    rep_trans
                        .entry(*char_class)
                        .and_modify(|e| {
                            for s in target_states {
                                if !e.contains(s) {
                                    e.push(*s)
                                }
                            }
                        })
                        .or_insert(target_states.clone());
                }
                // Remove the transitions of the state that is merged into the representative state.
                transitions.remove(pos);
            }
            transitions[rep_pos].1 = rep_trans;
        }
    }

    fn renumber_states_in_transitions(
        partition: &[StateGroup],
        transitions: &mut [(StateID, BTreeMap<CharClassID, Vec<StateID>>)],
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
            for target_states in transition.1.values_mut() {
                for target_state in target_states.iter_mut() {
                    *target_state = find_group_of_state(*target_state);
                }
            }
        }
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
        trace!("  Transitions of state {} to groups:", state_id.as_usize());
        for (char_class, group) in &transitions_to_groups.0 {
            trace!("    cc# {} -> gr# {}", char_class, group);
        }
    }
}

#[cfg(test)]
mod tests {
    use rustc_hash::FxHashMap;

    use super::*;
    use crate::internal::compiled_dfa::StateData;

    #[test]
    fn test_calculate_initial_partition() {
        let dfa = CompiledDfa::new(
            vec![],
            vec![0.into(), 1.into(), 2.into()],
            vec![StateData::new(); 5],
            vec![
                (true, 0.into()),
                (true, 1.into()),
                (false, 0.into()),
                (false, 0.into()),
                (true, 2.into()),
            ],
            FxHashMap::default(),
        );

        let partition = Minimizer::calculate_initial_partition(&dfa);
        assert_eq!(partition.len(), 4);
        assert_eq!(partition[0].len(), 2);
        assert_eq!(partition[1].len(), 1);
        assert_eq!(partition[2].len(), 1);
        assert_eq!(partition[3].len(), 1);
    }

    #[test]
    fn test_calculate_new_partition() {
        let dfa = CompiledDfa::new(
            vec![],
            vec![0.into(), 1.into(), 2.into()],
            vec![StateData::new(); 5],
            vec![
                (true, 0.into()),
                (true, 1.into()),
                (false, 0.into()),
                (false, 0.into()),
                (true, 2.into()),
            ],
            FxHashMap::default(),
        );

        let transitions: TransitionMap = vec![
            (
                0.into(),
                vec![(0.into(), vec![1.into()]), (1.into(), vec![2.into()])]
                    .into_iter()
                    .collect(),
            ),
            (
                1.into(),
                vec![(0.into(), vec![1.into()]), (1.into(), vec![2.into()])]
                    .into_iter()
                    .collect(),
            ),
            (
                2.into(),
                vec![(0.into(), vec![1.into()]), (1.into(), vec![2.into()])]
                    .into_iter()
                    .collect(),
            ),
            (
                3.into(),
                vec![(0.into(), vec![1.into()]), (1.into(), vec![2.into()])]
                    .into_iter()
                    .collect(),
            ),
            (
                4.into(),
                vec![(0.into(), vec![1.into()]), (1.into(), vec![2.into()])]
                    .into_iter()
                    .collect(),
            ),
        ]
        .into_iter()
        .collect();

        let partition_old = Minimizer::calculate_initial_partition(&dfa);
        let partition_new = Minimizer::calculate_new_partition(&partition_old, &transitions);
        assert_eq!(partition_new.len(), 4);
        assert_eq!(partition_new[0].len(), 2);
        assert_eq!(partition_new[1].len(), 1);
        assert_eq!(partition_new[2].len(), 1);
        assert_eq!(partition_new[3].len(), 1);
    }
}
