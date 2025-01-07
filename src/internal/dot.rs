//! The `dot` module contains the conversion from an finite automata to a graphviz dot format.
//! The functions in this module are used for testing and debugging purposes.

use std::io::Write;

use dot_writer::{Attributes, DotWriter, RankDirection, Scope};

use crate::internal::compiled_nfa::CompiledNfa;

use super::{nfa::Nfa, CharClassID, CharacterClassRegistry, MultiPatternNfa, StateID};

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

fn render_compiled_nfa(
    compiled_nfa: &CompiledNfa,
    node_prefix: &str,
    character_class_registry: &CharacterClassRegistry,
    graph: &mut Scope,
) {
    // Render the states of the NFA
    for id in 0..compiled_nfa.states.len() {
        let node_name = format!("\"{}{}\"", node_prefix, id);
        let mut source_node = graph.node_named(&node_name);
        if id == 0 {
            // Start state of the compiled NFA
            source_node
                .set_shape(dot_writer::Shape::Circle)
                .set_color(dot_writer::Color::Blue)
                .set_pen_width(3.0);
            source_node.set_label(&id.to_string());
        } else if compiled_nfa.end_states[id].0 {
            source_node
                .set_shape(dot_writer::Shape::Circle)
                .set_color(dot_writer::Color::Red)
                .set_pen_width(3.0);
            source_node.set_label(&format!("{}:T{}", id, compiled_nfa.end_states[id].1));
        } else {
            source_node.set_label(&id.to_string());
        }
    }
    // Render the transitions of the NFA
    for (id, state) in compiled_nfa.states.iter().enumerate() {
        for (cc, next) in state.transitions.iter() {
            // Label the edge with the character class used to transition to the target state.
            graph
                .edge(
                    format!("\"{}{}\"", node_prefix, id),
                    format!("\"{}{}\"", node_prefix, next.as_usize()),
                )
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
        .set_label(
            format!(
                "{}: {}...",
                label,
                compiled_nfa.pattern(0.into()).escape_default()
            )
            .as_str(),
        )
        .set_rank_direction(RankDirection::LeftRight);

    render_compiled_nfa(compiled_nfa, "", character_class_registry, &mut digraph);

    // Render the lookaheads of the NFA each into a separate cluster
    for (terminal_id, lookahead) in compiled_nfa.lookaheads.iter() {
        let mut cluster = digraph.cluster();
        cluster.set_label(&format!(
            "LA for T{}({})",
            terminal_id,
            if lookahead.is_positive { "Pos" } else { "Neg" }
        ));
        let node_prefix = format!("{}_", terminal_id);
        render_compiled_nfa(
            &lookahead.nfa,
            &node_prefix,
            character_class_registry,
            &mut cluster,
        );
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
            .for_each(|state| acc.push(state.id().id()));
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
            .edge(format!("node_{}", source_id), format!("node_{}", target_id))
            .attributes()
            .set_label("ε");
    }

    for (source_id, target_id, char_class_id) in transitions {
        digraph
            .edge(
                format!("node_{}", source_id.as_usize()),
                format!("node_{}", target_id.as_usize()),
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
