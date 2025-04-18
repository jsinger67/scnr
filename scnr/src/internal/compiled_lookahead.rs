//! Module with the compiled lookahead type and functions.

use crate::{Lookahead, Result};

use super::{compiled_dfa::CompiledDfa, parse_regex_syntax, CharacterClassRegistry, Nfa};

#[derive(Debug, Clone)]
pub(crate) struct CompiledLookahead {
    /// The compiled DFA for the lookahead.
    /// We need a box to break the cycle between CompiledDfa and CompiledLookahead.
    pub(crate) nfa: Box<CompiledDfa>,
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
        let hir = parse_regex_syntax(pattern)?;
        let nfa: Nfa = Nfa::try_from_hir(hir, character_class_registry)?;
        let nfa = Box::new(nfa.into());
        Ok(Self {
            nfa,
            is_positive: *is_positive,
        })
    }

    /// Create a new compiled lookahead from a lookahead.
    pub(crate) fn try_from_lookahead_hir(
        lookahead: &Lookahead,
        character_class_registry: &mut CharacterClassRegistry,
    ) -> Result<Self> {
        let Lookahead {
            is_positive,
            pattern,
        } = lookahead;
        let hir = parse_regex_syntax(pattern)?;
        let nfa: Nfa = Nfa::try_from_hir(hir, character_class_registry)?;
        let nfa = Box::new(nfa.into());
        Ok(Self {
            nfa,
            is_positive: *is_positive,
        })
    }

    /// Check if the lookahead constraints are met.
    ///
    /// The function returns a tuple of (bool, usize) where the bool indicates if the lookahead
    /// is satisfied and the usize indicates the number of characters consumed.
    ///
    /// The boolean value in the returned tuple is calculated based on the value of `is_positive`.
    /// If the lookahead is positive, the value is true if the input matches the lookahead.
    /// Otherwise if the lookahead is negative, the value is true if the input does not match the
    /// lookahead.
    pub(crate) fn satisfies_lookahead(
        &mut self,
        input: &str,
        char_indices: std::str::CharIndices,
        match_char_class: &(dyn Fn(usize, char) -> bool + 'static),
    ) -> (bool, usize) {
        if let Some(ma) = self.nfa.find_from(input, char_indices, match_char_class) {
            (self.is_positive, ma.len())
        } else {
            (!self.is_positive, 0)
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
