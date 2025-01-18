use regex_syntax::ast::Ast;

use super::{CharClassID, ComparableAst};

/// A character class that can match a character.
#[derive(Default, Clone)]
pub(crate) struct CharacterClass {
    pub(crate) id: CharClassID,
    pub(crate) ast: ComparableAst,
}

impl CharacterClass {
    pub(crate) fn new(id: CharClassID, ast: Ast) -> Self {
        CharacterClass {
            id,
            ast: ComparableAst(ast),
        }
    }

    #[inline]
    pub(crate) fn ast(&self) -> &Ast {
        &self.ast.0
    }
}

impl std::fmt::Debug for CharacterClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CharacterClass {{ id: {:?}, ast: {:?} }}",
            self.id, self.ast
        )
    }
}

impl std::fmt::Display for CharacterClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{} '{}'", self.id, self.ast)
    }
}

impl std::hash::Hash for CharacterClass {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.ast.hash(state);
        // Do not hash the match function, because it is not relevant for equality.
        // Actually it is calculated from the AST, so it would be redundant.
    }
}

impl PartialEq for CharacterClass {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.ast == other.ast
    }
}

impl Eq for CharacterClass {}

impl PartialOrd for CharacterClass {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl Ord for CharacterClass {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

#[cfg(test)]
mod tests {
    use crate::internal::parse_regex_syntax;

    use super::*;

    // Helper macro to create a literal AST.
    macro_rules! Literal {
        ($c:literal) => {
            regex_syntax::ast::Ast::Literal(Box::new(regex_syntax::ast::Literal {
                span: regex_syntax::ast::Span {
                    start: regex_syntax::ast::Position {
                        offset: 0,
                        line: 0,
                        column: 0,
                    },
                    end: regex_syntax::ast::Position {
                        offset: 0,
                        line: 0,
                        column: 0,
                    },
                },
                kind: regex_syntax::ast::LiteralKind::Verbatim,
                c: $c,
            }))
        };
    }

    #[test]
    fn test_character_class_equality() {
        let ast1 = Literal!('a');
        let ast2 = Literal!('a');
        let ast3 = Literal!('b');
        let class1 = CharacterClass::new(0.into(), ast1);
        let class2 = CharacterClass::new(0.into(), ast2);
        let class3 = CharacterClass::new(1.into(), ast3);
        assert_eq!(class1, class2);
        assert_ne!(class1, class3);
    }

    #[test]
    fn test_character_class_equality_special() {
        let ast1 = parse_regex_syntax("\r").unwrap();
        if let Ast::Literal(_) = &ast1 {
            let class1 = CharacterClass::new(0.into(), ast1.clone());
            let class2 = CharacterClass::new(0.into(), Literal!('\r'));
            eprintln!("{:?} <=> {:?}", class1.ast(), class2.ast());
            assert_eq!(class1, class2);
        } else {
            panic!("Expected a literal AST.");
        }
    }

    #[test]
    fn test_character_class_ordering() {
        let ast1 = Literal!('a');
        let ast2 = Literal!('b');
        let class1 = CharacterClass::new(0.into(), ast1);
        let class2 = CharacterClass::new(1.into(), ast2);
        assert!(class1 < class2);
        assert!(class2 > class1);
    }
}
