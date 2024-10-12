//! Module with the compiled lookahead type and functions.

use crate::{Lookahead, Result};

use super::{dfa::Dfa, parse_regex_syntax, CharClassID, CharacterClassRegistry, CompiledDfa, Nfa};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompiledLookahead {
    /// The compiled DFA for the lookahead.
    /// We need a box to break the cycle between CompiledDfa and CompiledLookahead.
    pub(crate) dfa: Box<CompiledDfa>,
    /// If the lookahead is positive or negative.
    pub(crate) is_positive: bool,
}

impl CompiledLookahead {
    /// Create a new compiled lookahead from a lookahead.
    pub(crate) fn try_from_lookahead(
        lookahead: &Lookahead,
        character_class_registry: &mut CharacterClassRegistry,
    ) -> Result<Self> {
        let Lookahead {
            is_positive,
            pattern,
        } = lookahead;
        let ast = parse_regex_syntax(pattern)?;
        let nfa: Nfa = Nfa::try_from_ast(ast, character_class_registry)?;
        let dfa: Dfa = Dfa::try_from_nfa(nfa, character_class_registry)?;
        let dfa = dfa.minimize()?;
        let dfa = Box::new(CompiledDfa::try_from(dfa)?);
        Ok(Self {
            dfa,
            is_positive: *is_positive,
        })
    }

    /// Check if the lookahead matches the input.
    pub(crate) fn matches(
        &mut self,
        char_indices: std::str::CharIndices,
        match_char_class: &(dyn Fn(CharClassID, char) -> bool + 'static),
    ) -> bool {
        for (c_pos, c) in char_indices {
            self.dfa.advance(c_pos, c, match_char_class);
            if !self.dfa.search_for_longer_match() {
                if self.is_positive {
                    return self.dfa.matching_state().is_longest_match();
                } else {
                    return self.dfa.matching_state().is_no_match();
                }
            }
        }
        false
    }
}
