use crate::{Result, ScannerMode, ScnrError};

use super::{
    parse_regex_syntax, CharacterClassRegistry, CompiledDfa, Dfa, Nfa, ScannerModeID, TerminalID,
};

/// A compiled scanner mode that can be used to scan a string.
#[derive(Debug, Clone)]
pub(crate) struct CompiledScannerMode {
    /// The name of the scanner mode.
    pub(crate) name: String,
    /// The regular expressions that are valid token types in this mode, bundled with their token
    /// type numbers.
    /// The priorities of the patterns are determined by their order in the vector. Lower indices
    /// have higher priority if multiple patterns match the input and have the same length.
    pub(crate) patterns: Vec<(CompiledDfa, TerminalID)>,
    pub(crate) transitions: Vec<(TerminalID, ScannerModeID)>,
}

impl CompiledScannerMode {
    /// Create a new compiled scanner mode.
    pub(crate) fn try_from_scanner_mode(
        scanner_mode: ScannerMode,
        character_class_registry: &mut CharacterClassRegistry,
    ) -> Result<Self> {
        let ScannerMode {
            name,
            patterns,
            transitions,
        } = scanner_mode;
        let patterns =
            patterns
                .iter()
                .try_fold(Vec::new(), |mut acc, (pattern, terminal_id)| {
                    let ast = parse_regex_syntax(pattern)?;
                    let nfa: Nfa = Nfa::try_from_ast(ast, character_class_registry)?;
                    let dfa: Dfa = Dfa::try_from_nfa(nfa, character_class_registry)?;
                    let compiled_dfa = CompiledDfa::try_from(dfa)?;
                    acc.push((compiled_dfa, *terminal_id));
                    Ok::<Vec<(CompiledDfa, TerminalID)>, ScnrError>(acc)
                })?;
        Ok(Self {
            name,
            patterns,
            transitions,
        })
    }

    /// Check if the scanner configuration has a transition on the given terminal index
    pub(crate) fn has_transition(&self, token_type: usize) -> Option<usize> {
        for (tok_type, scanner) in &self.transitions {
            match token_type.cmp(&tok_type.as_usize()) {
                std::cmp::Ordering::Less => return None,
                std::cmp::Ordering::Equal => return Some(scanner.as_usize()),
                std::cmp::Ordering::Greater => continue,
            }
        }
        None
    }
}
