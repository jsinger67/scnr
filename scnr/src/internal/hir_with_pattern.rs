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

    /// Returns the string representation of the AST.
    pub(crate) fn pattern(&self) -> &str {
        &self.pattern
    }
}

impl std::hash::Hash for HirWithPattern {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the string representation of the AST.
        self.hir.to_string().hash(state);
    }
}

impl std::fmt::Display for HirWithPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hir)
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
