use crate::{Result, ScannerMode};

use super::{CharacterClassRegistry, CompiledDfa, ScannerModeID, TerminalID};

/// A compiled scanner mode that can be used to scan a string.
#[derive(Debug, Clone)]
pub(crate) struct CompiledScannerMode {
    /// The name of the scanner mode.
    pub(crate) name: String,
    /// The regular expression that can match valid token types in this mode.
    /// The priorities of the patterns are determined by their order in the vector. Lower indices
    /// have higher priority if multiple patterns match the input and have the same length.
    pub(crate) dfa: CompiledDfa,
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
        let dfa = CompiledDfa::try_from_patterns(&patterns, character_class_registry)?;
        Ok(Self {
            name,
            dfa,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScannerMode;

    /// A macro that simplifies the rendering of a dot file for a DFA.
    macro_rules! compiled_dfa_render_to {
        ($nfa:expr, $label:expr, $reg:ident) => {
            let label = format!("{}Dfa", $label);
            let mut f = std::fs::File::create(format!("target/{}CompiledDfa.dot", $label)).unwrap();
            $crate::internal::dot::compiled_dfa_render($nfa, &label, &$reg, &mut f);
        };
    }

    #[test]
    fn test_compile_to_dfa() {
        let mut character_class_registry = CharacterClassRegistry::new();
        let pattern = "(//.*(\r\n|\r|\n))";
        let compiled_dfa =
            CompiledDfa::try_from_pattern(pattern, 0usize.into(), &mut character_class_registry)
                .unwrap();
        compiled_dfa_render_to!(&compiled_dfa, "LineComment_", character_class_registry);
        // assert_eq!(compiled_dfa.accepting_states.len(), 1);
    }

    #[test]
    fn test_compiled_scanner_mode() {
        let mut character_class_registry = CharacterClassRegistry::new();
        let scanner_mode = ScannerMode {
            name: "test".to_string(),
            patterns: vec![("a".to_string(), 0usize.into())],
            transitions: vec![(0usize.into(), 1.into())],
        };
        let compiled_scanner_mode =
            CompiledScannerMode::try_from_scanner_mode(scanner_mode, &mut character_class_registry)
                .unwrap();
        assert_eq!(compiled_scanner_mode.name, "test");
        assert_eq!(compiled_scanner_mode.transitions.len(), 1);
    }

    #[test]
    fn test_compiled_scanner_mode_error() {
        let mut character_class_registry = CharacterClassRegistry::new();
        let scanner_mode = ScannerMode {
            name: "test".to_string(),
            patterns: vec![("[".to_string(), 0usize.into())],
            transitions: vec![(0usize.into(), 1.into())],
        };
        let compiled_scanner_mode =
            CompiledScannerMode::try_from_scanner_mode(scanner_mode, &mut character_class_registry);
        assert!(compiled_scanner_mode.is_err());
    }

    #[test]
    fn test_compiled_scanner_mode_transition() {
        let mut character_class_registry = CharacterClassRegistry::new();
        let scanner_mode = ScannerMode {
            name: "test".to_string(),
            patterns: vec![("a".to_string(), 0usize.into())],
            transitions: vec![(0usize.into(), 1.into()), (1usize.into(), 2.into())],
        };
        let compiled_scanner_mode =
            CompiledScannerMode::try_from_scanner_mode(scanner_mode, &mut character_class_registry)
                .unwrap();
        assert_eq!(compiled_scanner_mode.has_transition(0), Some(1));
        assert_eq!(compiled_scanner_mode.has_transition(1), Some(2));
        assert_eq!(compiled_scanner_mode.has_transition(2), None);
        assert_eq!(compiled_scanner_mode.has_transition(3), None);
    }
}
