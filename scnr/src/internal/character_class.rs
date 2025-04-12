use super::{AstWithPattern, CharClassID, HirWithPattern};

//
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum AstOrHir {
    Ast(AstWithPattern),
    Hir(HirWithPattern),
}

impl Default for AstOrHir {
    fn default() -> Self {
        AstOrHir::Ast(AstWithPattern::default())
    }
}

impl std::fmt::Display for AstOrHir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstOrHir::Ast(ast) => write!(f, "{}", ast),
            AstOrHir::Hir(hir) => write!(f, "{}", hir),
        }
    }
}

/// A character class that can match a character.
#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CharacterClass {
    pub(crate) id: CharClassID,
    pub(crate) ast_or_hir: AstOrHir,
}

impl CharacterClass {
    pub(crate) fn new_ast(id: CharClassID, ast: AstWithPattern) -> Self {
        CharacterClass {
            id,
            ast_or_hir: AstOrHir::Ast(ast),
        }
    }

    pub(crate) fn new_hir(id: CharClassID, hir: HirWithPattern) -> Self {
        CharacterClass {
            id,
            ast_or_hir: AstOrHir::Hir(hir),
        }
    }

    #[inline]
    pub(crate) fn ast(&self) -> Option<&regex_syntax::ast::Ast> {
        if let AstOrHir::Ast(ref ast) = self.ast_or_hir {
            Some(&ast.ast)
        } else {
            None
        }
    }

    #[inline]
    pub(crate) fn hir(&self) -> Option<&regex_syntax::hir::Hir> {
        if let AstOrHir::Hir(ref hir) = self.ast_or_hir {
            Some(&hir.hir)
        } else {
            None
        }
    }

    #[inline]
    pub(crate) fn pattern(&self) -> &str {
        match self.ast_or_hir {
            AstOrHir::Ast(ref ast) => &ast.pattern(),
            AstOrHir::Hir(ref hir) => &hir.pattern(),
        }
    }
}

impl std::fmt::Debug for CharacterClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.ast_or_hir {
            AstOrHir::Ast(ref ast) => {
                write!(f, "CharacterClass {{ id: {:?}, ast: {:?} }}", self.id, ast)
            }
            AstOrHir::Hir(ref hir) => {
                write!(f, "CharacterClass {{ id: {:?}, hir: {:?} }}", self.id, hir)
            }
        }
    }
}

impl std::fmt::Display for CharacterClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.ast_or_hir {
            AstOrHir::Ast(ref ast) => write!(f, "{}", ast),
            AstOrHir::Hir(ref hir) => write!(f, "{}", hir),
        }
    }
}

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
    use crate::internal::{parse_regex_syntax, parser::parse_regex_syntax_hir};

    use super::*;

    // Helper macro to create a literal AST.
    macro_rules! AstLiteral {
        ($c:literal) => {
            $crate::internal::ast_with_pattern::AstWithPattern::new(
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
                })),
            )
        };
    }

    // Helper macro to create a literal Hir.
    macro_rules! HirLiteral {
        ($c:literal) => {{
            let mut buffer = [0; 4];
            let utf8_bytes = $c.encode_utf8(&mut buffer);

            $crate::internal::hir_with_pattern::HirWithPattern::new(
                regex_syntax::hir::Hir::literal(utf8_bytes.as_bytes().to_vec()),
            )
        }};
    }

    #[test]
    fn test_character_class_equality() {
        let ast1 = AstLiteral!('a');
        let ast2 = AstLiteral!('a');
        let ast3 = AstLiteral!('b');
        let class1 = CharacterClass::new_ast(0.into(), ast1);
        let class2 = CharacterClass::new_ast(0.into(), ast2);
        let class3 = CharacterClass::new_ast(1.into(), ast3);
        assert_eq!(class1, class2);
        assert_ne!(class1, class3);
    }

    #[test]
    fn test_character_class_equality_hir() {
        let ast1 = HirLiteral!('a');
        let ast2 = HirLiteral!('a');
        let ast3 = HirLiteral!('b');
        let class1 = CharacterClass::new_hir(0.into(), ast1);
        let class2 = CharacterClass::new_hir(0.into(), ast2);
        let class3 = CharacterClass::new_hir(1.into(), ast3);
        assert_eq!(class1, class2);
        assert_ne!(class1, class3);
    }

    #[test]
    fn test_character_class_equality_special() {
        let ast1 = AstWithPattern::new(parse_regex_syntax("\r").unwrap());
        if let AstWithPattern {
            ast: regex_syntax::ast::Ast::Literal(_),
            ..
        } = &ast1
        {
            let class1 = CharacterClass::new_ast(0.into(), ast1.clone());
            let class2 = CharacterClass::new_ast(0.into(), AstLiteral!('\r'));
            eprintln!("{:?} <=> {:?}", class1.ast(), class2.ast());
            assert_eq!(class1, class2);
        } else {
            panic!("Expected a literal AST.");
        }
    }

    #[test]
    fn test_character_class_equality_special_hir() {
        let hir1 = parse_regex_syntax_hir("\r").unwrap();
        if let regex_syntax::hir::HirKind::Literal(_) = hir1.clone().into_kind() {
            let class1 = CharacterClass::new_hir(0.into(), HirWithPattern::new(hir1.clone()));
            let class2 = CharacterClass::new_hir(0.into(), HirLiteral!('\r'));
            eprintln!("{:?} <=> {:?}", class1.hir(), class2.hir());
            assert_eq!(class1, class2);
        } else {
            panic!("Expected a literal Hir.");
        }
    }

    #[test]
    fn test_character_class_ordering() {
        let ast1 = AstLiteral!('a');
        let ast2 = AstLiteral!('b');
        let class1 = CharacterClass::new_ast(0.into(), ast1);
        let class2 = CharacterClass::new_ast(1.into(), ast2);
        assert!(class1 < class2);
        assert!(class2 > class1);
    }

    #[test]
    fn test_character_class_ordering_hir() {
        let ast1 = HirLiteral!('a');
        let ast2 = HirLiteral!('b');
        let class1 = CharacterClass::new_hir(0.into(), ast1);
        let class2 = CharacterClass::new_hir(1.into(), ast2);
        assert!(class1 < class2);
        assert!(class2 > class1);
    }
}
