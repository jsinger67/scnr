//! The `dot` module contains the conversion from an finite automata to a graphviz dot format.
//! The functions in this module are used for testing and debugging purposes.

use std::io::Write;

use dot_writer::{Attributes, DotWriter, RankDirection};

use crate::internal::{CompiledDfa, StateID, StateIDBase};

use super::{
    dfa::Dfa, multi_pattern_nfa::MultiPatternNfa, nfa::Nfa, CharClassID, CharacterClassRegistry,
};

/// Render the NFA to a graphviz dot format.
#[allow(dead_code)]
pub(crate) fn nfa_render<W: Write>(nfa: &Nfa, label: &str, output: &mut W) {
    let mut writer = DotWriter::from(output);
    writer.set_pretty_print(true);
    let mut digraph = writer.digraph();
    digraph
        .set_label(label)
        .set_rank_direction(RankDirection::LeftRight);
    for state in nfa.states() {
        let source_id = {
            let mut source_node = digraph.node_auto();
            source_node.set_label(&state.id().as_usize().to_string());
            if state.id() == nfa.start_state() {
                source_node
                    .set_shape(dot_writer::Shape::Circle)
                    .set_color(dot_writer::Color::Blue)
                    .set_pen_width(3.0);
            }
            if state.id() == nfa.end_state() {
                source_node
                    .set_shape(dot_writer::Shape::Circle)
                    .set_color(dot_writer::Color::Red)
                    .set_pen_width(3.0);
            }
            source_node.id()
        };
        for transition in state.transitions() {
            let target_state = transition.target_state();
            digraph
                .edge(
                    source_id.clone(),
                    &format!("node_{}", target_state.as_usize()),
                )
                .attributes()
                .set_label(&format!("{}", transition.ast()).escape_default().to_string());
        }
        for epsilon_transition in state.epsilon_transitions() {
            let target_state = epsilon_transition.target_state();
            digraph
                .edge(
                    source_id.clone(),
                    &format!("node_{}", target_state.as_usize()),
                )
                .attributes()
                .set_label("ε");
        }
    }
}

/// Render a MultiPatternNfa to a graphviz dot format.
#[allow(dead_code)]
pub(crate) fn multi_pattern_nfa_render<W: Write>(
    multi_pattern_nfa: &MultiPatternNfa,
    label: &str,
    character_class_registry: &CharacterClassRegistry,
    output: &mut W,
) {
    let mut writer = DotWriter::from(output);
    writer.set_pretty_print(true);
    let mut digraph = writer.digraph();
    digraph
        .set_label(label)
        .set_rank_direction(RankDirection::LeftRight);

    let node_ids = multi_pattern_nfa.nfas.iter().fold(vec![0], |mut acc, nfa| {
        nfa.states()
            .iter()
            .for_each(|state| acc.push(state.id().as_usize()));
        acc
    });

    // Add the start transitions of the MultiPatternNfa to the epsilon transitions
    let mut epsilon_transitions = multi_pattern_nfa.start_transitions().iter().fold(
        Vec::<(usize, usize)>::new(),
        |mut acc, t| {
            acc.push((0, t.target_state().as_usize()));
            acc
        },
    );
    // Add the epsilon transitions of the NFAs to the epsilon transitions
    multi_pattern_nfa.nfas.iter().for_each(|nfa| {
        nfa.states().iter().for_each(|state| {
            state.epsilon_transitions().iter().for_each(|t| {
                epsilon_transitions.push((state.id().as_usize(), t.target_state().as_usize()));
            });
        });
    });

    // Add the transitions of the NFAs to the transitions
    let transitions = multi_pattern_nfa.nfas.iter().fold(
        Vec::<(StateID, StateID, CharClassID)>::new(),
        |mut acc, nfa| {
            nfa.states().iter().for_each(|state| {
                state.transitions().iter().for_each(|t| {
                    acc.push((state.id(), t.target_state(), t.char_class()));
                });
            });
            acc
        },
    );

    for node_id in node_ids {
        let mut source_node = digraph.node_auto();
        source_node.set_label(&node_id.to_string());
        if node_id == 0 {
            source_node
                .set_shape(dot_writer::Shape::Circle)
                .set_color(dot_writer::Color::Blue)
                .set_pen_width(3.0);
        }
        if multi_pattern_nfa.is_accepting_state(node_id.into()) {
            source_node
                .set_shape(dot_writer::Shape::Circle)
                .set_color(dot_writer::Color::Red)
                .set_pen_width(3.0);
        }
    }

    for (source_id, target_id) in epsilon_transitions {
        digraph
            .edge(
                &format!("node_{}", source_id),
                &format!("node_{}", target_id),
            )
            .attributes()
            .set_label("ε");
    }

    for (source_id, target_id, char_class_id) in transitions {
        digraph
            .edge(
                &format!("node_{}", source_id.as_usize()),
                &format!("node_{}", target_id.as_usize()),
            )
            .attributes()
            .set_label(&format!(
                "{}:{}",
                character_class_registry
                    .get_character_class(char_class_id)
                    .map_or("-".to_string(), |cc| cc
                        .ast()
                        .to_string()
                        .escape_default()
                        .to_string()),
                char_class_id.id()
            ));
    }
}

/// Render a DFA to a graphviz dot format.
#[allow(dead_code)]
pub(crate) fn dfa_render<W: Write>(
    dfa: &Dfa,
    label: &str,
    character_class_registry: &CharacterClassRegistry,
    output: &mut W,
) {
    let mut writer = DotWriter::from(output);
    writer.set_pretty_print(true);
    let mut digraph = writer.digraph();
    digraph
        .set_label(label)
        .set_rank_direction(RankDirection::LeftRight);
    // Render the states of the DFA
    for state in dfa.states() {
        let mut source_node = digraph.node_auto();
        source_node.set_label(&state.id().to_string());
        if state.id() == 0usize.into() {
            source_node
                .set_shape(dot_writer::Shape::Circle)
                .set_color(dot_writer::Color::Blue)
                .set_pen_width(3.0);
        }
        if dfa.is_accepting(state.id()) {
            source_node
                .set_color(dot_writer::Color::Red)
                .set_pen_width(3.0)
                .set_label(&format!(
                    "{}\n'{}'",
                    state.id(),
                    dfa.pattern(state.terminal_id().unwrap())
                        .unwrap_or_default()
                        .escape_default(),
                ));
        }
    }
    // Render the transitions of the DFA
    for (source_id, targets) in dfa.transitions() {
        for (char_id, target_id) in targets.iter() {
            // Label the edge with the character class used to transition to the target state.
            digraph
                .edge(
                    &format!("node_{}", source_id.as_usize()),
                    &format!("node_{}", target_id.as_usize()),
                )
                .attributes()
                .set_label(&format!(
                    "{}:{}",
                    character_class_registry
                        .get_character_class(*char_id)
                        .map_or("-".to_string(), |cc| cc
                            .ast()
                            .to_string()
                            .escape_debug()
                            .to_string()),
                    char_id.id()
                ));
        }
    }
}

// Render a compiled DFA
pub(crate) fn compiled_dfa_render<W: Write>(
    compiled_dfa: &CompiledDfa,
    label: &str,
    character_class_registry: &CharacterClassRegistry,
    output: &mut W,
) {
    let mut writer = DotWriter::from(output);
    writer.set_pretty_print(true);
    let mut digraph = writer.digraph();
    digraph
        .set_label(label)
        .set_rank_direction(RankDirection::LeftRight);
    // Render the states of the DFA
    for state_id in 0..compiled_dfa.state_ranges().len() {
        let mut source_node = digraph.node_auto();
        source_node.set_label(&state_id.to_string());
        if state_id == 0 {
            source_node
                .set_shape(dot_writer::Shape::Circle)
                .set_color(dot_writer::Color::Blue)
                .set_pen_width(3.0);
        }
        if compiled_dfa.is_accepting(StateID::new(state_id as StateIDBase)) {
            source_node
                .set_color(dot_writer::Color::Red)
                .set_pen_width(3.0)
                .set_label(&format!("{}", state_id,));
        }
    }
    // Render the transitions of the compiled DFA
    for (source_id, (s, e)) in compiled_dfa.state_ranges().iter().enumerate() {
        for (char_id, target_id) in compiled_dfa.transitions()[*s..*e].iter() {
            // Label the edge with the character class used to transition to the target state.
            digraph
                .edge(
                    &format!("node_{}", source_id),
                    &format!("node_{}", target_id.as_usize()),
                )
                .attributes()
                .set_label(&format!(
                    "{}:{}",
                    character_class_registry
                        .get_character_class(*char_id)
                        .map_or("-".to_string(), |cc| cc
                            .ast()
                            .to_string()
                            .escape_debug()
                            .to_string()),
                    char_id.id()
                ));
        }
    }
}
