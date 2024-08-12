use regex_syntax::ast::{Ast, Position, Span};

/// A comparable AST in regard of a character class.
/// It only compares AST types that are relevant for handling of character classes.
#[derive(Clone, Eq)]
pub(crate) struct ComparableAst(pub(crate) Ast);

impl PartialEq for ComparableAst {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Ast::ClassUnicode(_), Ast::ClassUnicode(_))
            | (Ast::ClassPerl(_), Ast::ClassPerl(_))
            | (Ast::ClassBracketed(_), Ast::ClassBracketed(_)) => {
                // Compare the string representation of the ASTs.
                // This is a workaround because the AST's implementation of PartialEq also
                // compares the span, which is not relevant for the character class handling here.
                self.0.to_string().escape_default().to_string()
                    == other.0.to_string().escape_default().to_string()
            }
            (Ast::Empty(_), Ast::Empty(_)) => true,
            (Ast::Literal(l), Ast::Literal(r)) => l.c == r.c && l.kind == r.kind,
            (Ast::Dot(_), Ast::Dot(_)) => true,
            _ => false,
        }
    }
}

impl From<&Ast> for ComparableAst {
    fn from(ast: &Ast) -> Self {
        ComparableAst(ast.clone())
    }
}

impl std::hash::Hash for ComparableAst {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the string representation of the AST.
        self.0.to_string().hash(state);
    }
}

impl Default for ComparableAst {
    fn default() -> Self {
        ComparableAst(Ast::Empty(Box::new(Span {
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
        })))
    }
}

impl std::fmt::Display for ComparableAst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string().escape_default())
    }
}

impl std::fmt::Debug for ComparableAst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_string().escape_default())
    }
}
