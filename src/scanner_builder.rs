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

    /// Adds only patterns to the scanner builder.
    /// This is useful for simple use cases where only one scanner mode is needed.
    /// The scanner mode is named `INITIAL` implicitly.
    /// Adding more scanner modes as well as transitions between scanner modes are not supported.
    /// Note that all previously added scanner modes will be ignored after calling this method.
    pub fn add_patterns<P, S>(self, patterns: P) -> SimpleScannerBuilder
    where
        P: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        SimpleScannerBuilder::new(patterns)
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

/// A struct used by the [ScannerBuilder] for creating a simple scanner.
/// It is used to create a scanner with a single scanner mode, which is usually sufficient for
/// simple use cases. The scanner mode is named `INITIAL` implicitly. Transitions between scanner
/// modes are not supported.
/// The token types returned by the scanner are the indices of the patterns in the order they were
/// added to the scanner.
#[derive(Debug, Clone)]
pub struct SimpleScannerBuilder {
    scanner_mode: ScannerMode,
}

impl SimpleScannerBuilder {
    /// Creates a new simple scanner builder.
    fn new<P, S>(patterns: P) -> Self
    where
        P: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let patterns = patterns
            .into_iter()
            .enumerate()
            .map(|(index, pattern)| (pattern, index))
            .collect::<Vec<_>>();
        Self {
            scanner_mode: ScannerMode::new("INITIAL", patterns, vec![]),
        }
    }

    /// Builds the scanner from the simple scanner builder.
    pub fn build(self) -> Result<Scanner> {
        Ok(Scanner {
            inner: ScannerImpl::try_from(vec![self.scanner_mode])?,
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
    fn test_scanner_builder_with_single_mode() {
        init();
        let scanner_mode = ScannerMode::new(
            "INITIAL",
            vec![(r"\r\n|\r|\n", 1), (r"(//.*(\r\n|\r|\n))", 3)],
            vec![(1, 1), (3, 1)],
        );
        let scanner = ScannerBuilder::new()
            .add_scanner_mode(scanner_mode)
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
    fn test_scanner_builder_with_multiple_modes() {
        init();
        let scanner_modes = vec![
            ScannerMode::new(
                "INITIAL",
                vec![(r"\r\n|\r|\n", 1), (r"(//.*(\r\n|\r|\n))", 3)],
                vec![(1, 1), (3, 1)],
            ),
            ScannerMode::new("STRING", vec![(r#""[^"]*""#, 2)], vec![(2, 0)]),
        ];
        let scanner = ScannerBuilder::new()
            .add_scanner_modes(&scanner_modes)
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
    fn test_simple_scanner_builder() {
        init();
        let scanner = ScannerBuilder::new()
            .add_patterns(["\r\n|\r|\n", "//.*(\r\n|\r|\n)"])
            .build()
            .unwrap();
        assert_eq!("INITIAL", scanner.inner.scanner_modes[0].name);
        let input = r#"
        // Line comment1

        // Line comment2
        "#;

        let matches: Vec<_> = scanner.find_iter(input).collect();
        assert_eq!(matches.len(), 4);
        assert_eq!(matches[0].token_type(), 0);
        assert_eq!(matches[1].token_type(), 1);
        assert_eq!(
            &input[matches[1].span().range()].to_string().trim(),
            &"// Line comment1"
        );
        assert_eq!(matches[2].token_type(), 0);
        assert_eq!(matches[3].token_type(), 1);
        assert_eq!(
            &input[matches[3].span().range()].to_string().trim(),
            &"// Line comment2"
        );
    }
}
