use regex_syntax::hir::Hir;

/// A comparable AST in regard of a character class with associated pattern string.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct HirWithPattern {
    pub(crate) hir: Hir,
    pub(crate) pattern: String,
}

impl HirWithPattern {
    /// Creates a new ComparableHir from an AST.
    pub(crate) fn new(hir: Hir) -> Self {
        let pattern = hir.to_string().escape_default().to_string();
        HirWithPattern { hir, pattern }
    }

    /// Checks if the given character is contained in the Hir.
    pub(crate) fn contains(&self, interval: &std::ops::RangeInclusive<char>) -> bool {
        match self.hir.kind() {
            regex_syntax::hir::HirKind::Empty => true, // An empty Hir matches everything.
            regex_syntax::hir::HirKind::Literal(literal) => {
                // Literals here are separated into single characters.
                let bytes = literal.0.clone();
                // We convert the first 4 bytes to a u32.
                // If the literal is smaller than 4 bytes, take will ensure we only take the bytes
                // that exist.
                let lit: u32 = bytes
                    .iter()
                    .take(4)
                    .fold(0, |acc, &b| (acc << 8) | b as u32);
                let c = char::from_u32(lit).unwrap_or('\0');
                interval.contains(&c)
            }
            regex_syntax::hir::HirKind::Class(class) => {
                // Check if the class contains any character in the interval.
                match class {
                    regex_syntax::hir::Class::Unicode(class) => {
                        // Create a ClassUnicodeRange from our RangeInclusive<char>
                        let class_unicode_range = regex_syntax::hir::ClassUnicodeRange::new(
                            *interval.start(),
                            *interval.end(),
                        );

                        let class_from_interval =
                            regex_syntax::hir::ClassUnicode::new(vec![class_unicode_range]);
                        let mut intersection = class.clone();
                        intersection.intersect(&class_from_interval);
                        intersection == class_from_interval
                    }
                    regex_syntax::hir::Class::Bytes(class) =>
                    // For byte classes, we assume they are similar.
                    {
                        // Create a ClassBytesRange from our RangeInclusive<char>
                        let class_bytes_range = regex_syntax::hir::ClassBytesRange::new(
                            *interval.start() as u8,
                            *interval.end() as u8,
                        );
                        let class_from_interval =
                            regex_syntax::hir::ClassBytes::new(vec![class_bytes_range]);
                        let mut intersection = class.clone();
                        intersection.intersect(&class_from_interval);
                        intersection == class_from_interval
                    }
                }
            }
            _ => false, // We assume other Hir kinds do not match any character.
        }
    }

    // Returns the string representation of the AST.
    // pub(crate) fn pattern(&self) -> &str {
    //     &self.pattern
    // }
}

impl std::hash::Hash for HirWithPattern {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the string representation of the AST.
        self.hir.to_string().hash(state);
    }
}

impl std::fmt::Display for HirWithPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pattern.escape_default())
    }
}

impl std::fmt::Debug for HirWithPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(HirWithPattern))
            .field("pattern", &self.pattern)
            .field("hir", &self.hir)
            .finish()
    }
}
