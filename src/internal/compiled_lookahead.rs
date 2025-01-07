//! Module with the compiled lookahead type and functions.

use crate::{Lookahead, Result};

use super::{
    compiled_nfa::CompiledNfa, parse_regex_syntax, CharClassID, CharacterClassRegistry, Nfa,
};

#[derive(Debug, Clone)]
pub(crate) struct CompiledLookahead {
    /// The compiled NFA for the lookahead.
    /// We need a box to break the cycle between CompiledNfa and CompiledLookahead.
    pub(crate) nfa: Box<CompiledNfa>,
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
        let nfa = Box::new(nfa.into());
        Ok(Self {
            nfa,
            is_positive: *is_positive,
        })
    }

    /// Check if the lookahead constraints are met.
    ///
    /// If the lookahead is positive, the function returns true if the input matches the lookahead.
    /// Otherwise if the lookahead is negative, the function returns true if the input does not
    /// match the lookahead.
    pub(crate) fn satisfies_lookahead(
        &mut self,
        char_indices: std::str::CharIndices,
        match_char_class: &(dyn Fn(CharClassID, char) -> bool + 'static),
    ) -> bool {
        if self.nfa.find_from(char_indices, match_char_class).is_some() {
            self.is_positive
        } else {
            !self.is_positive
        }
    }
}

impl std::fmt::Display for CompiledLookahead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} lookahead: {}",
            if self.is_positive {
                "Positive"
            } else {
                "Negative"
            },
            self.nfa
        )
    }
}
