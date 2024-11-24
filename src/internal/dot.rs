//! The `dot` module contains the conversion from an finite automata to a graphviz dot format.
//! The functions in this module are used for testing and debugging purposes.

use std::io::Write;

use dot_writer::{Attributes, DotWriter, RankDirection};

use crate::internal::{compiled_nfa::CompiledNfa, StateIDBase};

use super::{ids::StateSetID, nfa::Nfa, CharacterClassRegistry};

/// Render the NFA to a graphviz dot format.
#[allow(dead_code)]
pub(crate) fn nfa_render<W: Write>(nfa: &Nfa, label: &str, output: &mut W) {
    let mut writer = DotWriter::from(output);
    writer.set_pretty_print(true);
    let mut digraph = writer.digraph();
    digraph
        .set_label(format!("{}: {}", label, nfa.pattern()).as_str())
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
                    format!("node_{}", target_state.as_usize()),
                )
                .attributes()
                .set_label(
                    &format!("#{}:'{}'", transition.char_class(), transition.ast())
                        .escape_default()
                        .to_string(),
                );
        }
        for epsilon_transition in state.epsilon_transitions() {
            let target_state = epsilon_transition.target_state();
            digraph
                .edge(
                    source_id.clone(),
                    format!("node_{}", target_state.as_usize()),
                )
                .attributes()
                .set_label("ε");
        }
    }
}

// Render a compiled NFA
pub(crate) fn compiled_nfa_render<W: Write>(
    compiled_nfa: &CompiledNfa,
    label: &str,
    character_class_registry: &CharacterClassRegistry,
    output: &mut W,
) {
    let mut writer = DotWriter::from(output);
    writer.set_pretty_print(true);
    let mut digraph = writer.digraph();
    digraph
        .set_label(format!("{}: {}", label, compiled_nfa.pattern.escape_default()).as_str())
        .set_rank_direction(RankDirection::LeftRight);
    // Render the states of the NFA
    for id in 0..compiled_nfa.states.len() {
        let mut source_node = digraph.node_auto();
        source_node.set_label(&id.to_string());
        if id == 0 {
            // Start state of the compiled NFA
            source_node
                .set_shape(dot_writer::Shape::Circle)
                .set_color(dot_writer::Color::Blue)
                .set_pen_width(3.0);
        }
        if compiled_nfa
            .end_states
            .contains(&StateSetID::new(id as StateIDBase))
        {
            source_node
                .set_shape(dot_writer::Shape::Circle)
                .set_color(dot_writer::Color::Red)
                .set_pen_width(3.0);
        }
    }
    // Render the transitions of the NFA
    for (id, state) in compiled_nfa.states.iter().enumerate() {
        for (cc, next) in state.transitions.iter() {
            // Label the edge with the character class used to transition to the target state.
            digraph
                .edge(format!("node_{}", id), format!("node_{}", next.as_usize()))
                .attributes()
                .set_label(&format!(
                    "{}:#{}",
                    character_class_registry.get_character_class(*cc).map_or(
                        "-".to_string(),
                        |cc| cc.ast().to_string().escape_debug().to_string()
                    ),
                    cc.id()
                ));
        }
    }
}
