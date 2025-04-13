use crate::{
    internal::SCANNER_CACHE, scanner::Scanner, scanner_mode::ScannerMode, Pattern, Result,
};

/// A builder for creating a scanner.
#[derive(Debug, Clone, Default)]
pub struct ScannerBuilder {
    scanner_modes: Vec<ScannerMode>,
    use_hir: bool,
}

impl ScannerBuilder {
    /// Creates a new scanner builder.
    pub fn new() -> Self {
        Self {
            scanner_modes: Vec::new(),
            use_hir: false,
        }
    }

    /// Sets the use of HIR (High-level Intermediate Representation) for the scanner.
    pub fn use_hir(mut self) -> Self {
        self.use_hir = true;
        self
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
        let patterns = patterns
            .into_iter()
            .enumerate()
            .map(|(i, pattern)| Pattern::new(pattern.as_ref().to_string(), i))
            .collect::<Vec<_>>();
        let mut simple_builder = SimpleScannerBuilder::new(patterns);
        if self.use_hir {
            simple_builder = simple_builder.use_hir();
        }
        simple_builder
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
            inner: SCANNER_CACHE
                .write()
                .unwrap()
                .get(&self.scanner_modes, self.use_hir)?,
        })
    }

    /// Builds the scanner from the scanner builder without caching it.
    /// This is useful for testing and benchmarking purposes.
    ///
    /// A user should not call this method in production code. It is recommended to use the `build`
    /// method instead.
    #[allow(dead_code)]
    pub fn build_uncached(self) -> Result<Scanner> {
        Ok(Scanner {
            inner: self.scanner_modes.try_into()?,
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
    use_hir: bool,
}

impl SimpleScannerBuilder {
    /// Creates a new simple scanner builder.
    fn new<P>(patterns: P) -> Self
    where
        P: IntoIterator<Item = Pattern>,
    {
        Self {
            scanner_mode: ScannerMode::new("INITIAL", patterns, vec![]),
            use_hir: false,
        }
    }

    /// Sets the use of HIR (High-level Intermediate Representation) for the scanner.
    pub fn use_hir(mut self) -> Self {
        self.use_hir = true;
        self
    }

    /// Builds the scanner from the simple scanner builder.
    pub fn build(self) -> Result<Scanner> {
        Ok(Scanner {
            inner: SCANNER_CACHE
                .write()
                .unwrap()
                .get(&[self.scanner_mode], self.use_hir)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{Pattern, ScannerModeSwitcher};

    use super::*;

    static INIT: std::sync::Once = std::sync::Once::new();

    const TARGET_FOLDER: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../target/testout/test_simple_scanner_builder"
    );

    fn init() {
        INIT.call_once(|| {
            let _ = env_logger::builder().is_test(true).try_init();
            // Delete all previously generated dot files.
            let _ = std::fs::remove_dir_all(TARGET_FOLDER);
            // Create the target folder.
            std::fs::create_dir_all(TARGET_FOLDER).unwrap();
        });
    }

    #[test]
    fn test_scanner_builder_with_single_mode() {
        init();
        let scanner_mode = ScannerMode::new(
            "INITIAL",
            vec![
                Pattern::new(r"\r\n|\r|\n".to_string(), 1),
                Pattern::new(r"(//.*(\r\n|\r|\n))".to_string(), 3),
            ],
            vec![(1, 1), (3, 1)],
        );
        let scanner = ScannerBuilder::new()
            .add_scanner_mode(scanner_mode)
            .build()
            .unwrap();
        assert_eq!(Some("INITIAL"), scanner.inner.mode_name(0));
    }

    #[test]
    fn test_scanner_builder_with_multiple_modes() {
        init();
        let scanner_modes = vec![
            ScannerMode::new(
                "INITIAL",
                vec![
                    Pattern::new(r"\r\n|\r|\n".to_string(), 1),
                    Pattern::new(r"(//.*(\r\n|\r|\n))".to_string(), 3),
                ],
                vec![(1, 1), (3, 1)],
            ),
            ScannerMode::new(
                "STRING",
                vec![Pattern::new(r#""[^"]*""#.to_string(), 2)],
                vec![(2, 0)],
            ),
        ];
        let scanner = ScannerBuilder::new()
            .add_scanner_modes(&scanner_modes)
            .build()
            .unwrap();
        assert_eq!(Some("INITIAL"), scanner.inner.mode_name(0));
    }

    #[test]
    fn test_simple_scanner_builder() {
        init();
        let scanner = ScannerBuilder::new()
            .add_patterns(["\r\n|\r|\n", "//.*(\r\n|\r|\n)"])
            .build()
            .unwrap();
        assert_eq!(Some("INITIAL"), scanner.inner.mode_name(0));
        let input = r#"
        // Line comment1

        // Line comment2
        "#;

        #[cfg(not(feature = "regex_automata"))]
        #[cfg(feature = "dot_writer")]
        scanner
            .generate_compiled_automata_as_dot("LineComment", std::path::Path::new(TARGET_FOLDER))
            .expect("Failed to generate compiled automata as dot");

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
