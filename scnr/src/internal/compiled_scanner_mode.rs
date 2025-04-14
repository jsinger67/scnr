use crate::{Result, ScannerMode};

use super::{compiled_dfa::CompiledDfa, CharacterClassRegistry, ScannerModeID, TerminalID};

/// A compiled scanner mode that can be used to scan a string.
#[derive(Debug, Clone)]
pub(crate) struct CompiledScannerMode {
    /// The name of the scanner mode.
    pub(crate) name: String,
    /// The regular expressions that are valid token types in this mode, bundled with their token
    /// type numbers.
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

    pub(crate) fn try_from_scanner_mode_hir(
        scanner_mode: &ScannerMode,
        character_class_registry: &mut CharacterClassRegistry,
    ) -> Result<Self> {
        let ScannerMode {
            name,
            patterns,
            transitions,
        } = scanner_mode;
        let dfa = CompiledDfa::try_from_patterns(patterns, character_class_registry)?;
        Ok(Self {
            name: name.clone(),
            dfa,
            transitions: transitions.clone(),
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
    use std::{fs, sync::Once};

    use super::*;
    use crate::{Pattern, ScannerMode};

    static INIT: Once = Once::new();

    const TARGET_FOLDER: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../target/testout/compiled_scanner_mode_test"
    );

    fn init() {
        INIT.call_once(|| {
            let _ = env_logger::builder().is_test(true).try_init();
            // Delete all previously generated dot files.
            let _ = fs::remove_dir_all(TARGET_FOLDER);
            // Create the target folder.
            fs::create_dir_all(TARGET_FOLDER).unwrap();
        });
    }

    /// A macro that simplifies the rendering of a dot file for a DFA.
    #[cfg(feature = "dot_writer")]
    macro_rules! compiled_dfa_render_to {
        ($nfa:expr, $label:expr, $reg:ident) => {
            let label = format!("{}Dfa", $label);
            let mut f =
                std::fs::File::create(format!("{}/{}CompiledDfa.dot", TARGET_FOLDER, $label))
                    .unwrap();
            $crate::internal::dot::compiled_dfa_render($nfa, &label, &$reg, &mut f);
        };
    }

    #[cfg(feature = "dot_writer")]
    #[test]
    fn test_compile_to_nfa() {
        init();
        let mut character_class_registry = CharacterClassRegistry::new();
        let pattern = Pattern::new("(//.*(\r\n|\r|\n))".to_string(), 0);
        let compiled_dfa =
            CompiledDfa::try_from_pattern(&pattern, &mut character_class_registry).unwrap();
        compiled_dfa_render_to!(&compiled_dfa, "LineComment_", character_class_registry);
        // assert_eq!(compiled_dfa.accepting_states.len(), 1);
    }

    #[test]
    fn test_compiled_dfa_scanner_mode() {
        init();
        let mut character_class_registry = CharacterClassRegistry::new();
        let scanner_mode = ScannerMode {
            name: "test".to_string(),
            patterns: vec![Pattern::new("a".to_string(), 0)],
            transitions: vec![(0.into(), 1.into())],
        };
        let compiled_scanner_mode =
            CompiledScannerMode::try_from_scanner_mode(scanner_mode, &mut character_class_registry)
                .unwrap();
        assert_eq!(compiled_scanner_mode.name, "test");
        assert_eq!(compiled_scanner_mode.transitions.len(), 1);
    }

    #[test]
    fn test_compiled_dfa_scanner_mode_error() {
        init();
        let mut character_class_registry = CharacterClassRegistry::new();
        let scanner_mode = ScannerMode {
            name: "test".to_string(),
            patterns: vec![Pattern::new("[".to_string(), 0)],
            transitions: vec![(0.into(), 1.into())],
        };
        let compiled_scanner_mode =
            CompiledScannerMode::try_from_scanner_mode(scanner_mode, &mut character_class_registry);
        assert!(compiled_scanner_mode.is_err());
    }

    #[test]
    fn test_compiled_dfa_scanner_mode_transition() {
        init();
        let mut character_class_registry = CharacterClassRegistry::new();
        let scanner_mode = ScannerMode {
            name: "test".to_string(),
            patterns: vec![Pattern::new("a".to_string(), 0)],
            transitions: vec![(0.into(), 1.into()), (1.into(), 2.into())],
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
