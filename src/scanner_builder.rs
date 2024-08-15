use crate::{scanner::Scanner, scanner_mode::ScannerMode, Result, ScannerImpl};

/// A builder for creating a scanner.
#[derive(Debug, Clone, Default)]
pub struct ScannerBuilder {
    scanner_modes: Vec<ScannerMode>,
}

impl ScannerBuilder {
    /// Creates a new scanner builder.
    pub fn new() -> Self {
        Self {
            scanner_modes: Vec::new(),
        }
    }

    /// Adds a scanner mode to the scanner builder.
    pub fn add_scanner_mode(mut self, scanner_mode: ScannerMode) -> Self {
        self.scanner_modes.push(scanner_mode);
        self
    }

    /// Adds multiple scanner modes to the scanner builder.
    pub fn add_scanner_modes(mut self, scanner_modes: &[ScannerMode]) -> Self {
        self.scanner_modes.extend_from_slice(scanner_modes);
        self
    }

    /// Builds the scanner from the scanner builder.
    pub fn build(self) -> Result<Scanner> {
        Ok(Scanner {
            inner: ScannerImpl::try_from(self.scanner_modes)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A macro that simplifies the rendering of a dot file for a DFA.
    macro_rules! compiled_dfa_render_to {
        ($dfa:expr, $label:expr, $reg:expr) => {
            let label = format!("{} Compiled DFA", $label);
            let mut f =
                std::fs::File::create(format!("target/{}CompiledDfaFromScannerMode.dot", $label))
                    .unwrap();
            $crate::internal::dot::compiled_dfa_render($dfa, &label, &$reg, &mut f);
        };
    }
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_scanner_builder() {
        init();
        let scanner_mode = ScannerMode::new(
            "INITIAL",
            vec![(r"\r\n|\r|\n", 1), (r"(//.*(\r\n|\r|\n))", 3)],
            vec![],
        );
        let scanner = ScannerBuilder::new()
            .add_scanner_modes(&[scanner_mode])
            .build()
            .unwrap();
        assert_eq!("INITIAL", scanner.inner.scanner_modes[0].name);
        let compiled_dfa = &scanner.inner.scanner_modes[0].patterns[1].0;

        compiled_dfa_render_to!(
            &compiled_dfa,
            "LineComment",
            scanner.inner.character_classes
        );
    }

    #[test]
    fn test_scanner_mode_serialization() {
        init();
        let scanner_mode = ScannerMode::new(
            "INITIAL",
            vec![(r"\r\n|\r|\n", 1), (r"(//.*(\r\n|\r|\n))", 3)],
            vec![],
        );

        let serialized = serde_json::to_string(&scanner_mode).unwrap();
        eprintln!("{}", serialized);
        let deserialized: ScannerMode = serde_json::from_str(&serialized).unwrap();
        assert_eq!(scanner_mode, deserialized);
    }
}
