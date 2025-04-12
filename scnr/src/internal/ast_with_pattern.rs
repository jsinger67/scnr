use regex_syntax::ast::{Ast, Position, Span};

/// A comparable AST in regard of a character class with associated pattern string.
#[derive(Clone, Eq)]
pub(crate) struct AstWithPattern {
    pub(crate) ast: Ast,
    pub(crate) pattern: String,
}

impl AstWithPattern {
    /// Creates a new ComparableAst from an AST.
    pub(crate) fn new(ast: Ast) -> Self {
        let pattern = ast.to_string().escape_default().to_string();
        AstWithPattern { ast, pattern }
    }

    /// Returns the string representation of the AST.
    pub(crate) fn pattern(&self) -> &str {
        &self.pattern
    }
}

/// It only compares AST types that are relevant for handling of character classes.
impl PartialEq for AstWithPattern {
    fn eq(&self, other: &Self) -> bool {
        match (&self.ast, &other.ast) {
            (Ast::ClassUnicode(_), Ast::ClassUnicode(_))
            | (Ast::ClassPerl(_), Ast::ClassPerl(_))
            | (Ast::ClassBracketed(_), Ast::ClassBracketed(_)) => {
                // Compare the string representation of the ASTs.
                // This is a workaround because the AST's implementation of PartialEq also
                // compares the span, which is not relevant for the character class handling here.
                self.ast.to_string().escape_default().to_string()
                    == other.ast.to_string().escape_default().to_string()
            }
            (Ast::Empty(_), Ast::Empty(_)) => true,
            (Ast::Literal(l), Ast::Literal(r)) => l.c == r.c && l.kind == r.kind,
            (Ast::Dot(_), Ast::Dot(_)) => true,
            _ => false,
        }
    }
}

impl std::hash::Hash for AstWithPattern {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the string representation of the AST.
        self.ast.to_string().hash(state);
    }
}

impl Default for AstWithPattern {
    fn default() -> Self {
        AstWithPattern {
            ast: Ast::Empty(Box::new(Span {
                start: Position {
                    offset: 0,
                    line: 0,
                    column: 0,
                },
                end: Position {
                    offset: 0,
                    line: 0,
                    column: 0,
                },
            })),
            pattern: String::new(),
        }
    }
}

impl std::fmt::Display for AstWithPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ast.to_string().escape_default())
    }
}

impl std::fmt::Debug for AstWithPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ast.to_string().escape_default())
    }
}
