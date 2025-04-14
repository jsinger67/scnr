use super::{CharClassID, HirWithPattern};

/// A character class that can match a character.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct CharacterClass {
    pub(crate) id: CharClassID,
    pub(crate) hir: HirWithPattern,
}

impl CharacterClass {
    pub(crate) fn new(id: CharClassID, hir: HirWithPattern) -> Self {
        CharacterClass { id, hir }
    }

    // #[inline]
    // pub(crate) fn hir(&self) -> &regex_syntax::hir::Hir {
    //     &self.hir.hir
    // }

    // #[inline]
    // pub(crate) fn pattern(&self) -> &str {
    //     self.hir.pattern()
    // }
}

impl std::fmt::Debug for CharacterClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CharacterClass {{ id: {:?}, hir: {:?} }}",
            self.id, self.hir
        )
    }
}

impl std::fmt::Display for CharacterClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.hir)
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
    use crate::internal::parser::parse_regex_syntax;

    use super::*;

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
        let hir1 = HirLiteral!('a');
        let hir2 = HirLiteral!('a');
        let hir3 = HirLiteral!('b');
        let class1 = CharacterClass::new(0.into(), hir1);
        let class2 = CharacterClass::new(0.into(), hir2);
        let class3 = CharacterClass::new(1.into(), hir3);
        assert_eq!(class1, class2);
        assert_ne!(class1, class3);
    }

    #[test]
    fn test_character_class_equality_hir() {
        let ast1 = HirLiteral!('a');
        let ast2 = HirLiteral!('a');
        let ast3 = HirLiteral!('b');
        let class1 = CharacterClass::new(0.into(), ast1);
        let class2 = CharacterClass::new(0.into(), ast2);
        let class3 = CharacterClass::new(1.into(), ast3);
        assert_eq!(class1, class2);
        assert_ne!(class1, class3);
    }

    #[test]
    fn test_character_class_equality_special() {
        let hir1 = HirWithPattern::new(parse_regex_syntax("\r").unwrap());
        if let regex_syntax::hir::HirKind::Literal(_) = hir1.hir.clone().into_kind() {
            let class1 = CharacterClass::new(0.into(), hir1.clone());
            let class2 = CharacterClass::new(0.into(), HirLiteral!('\r'));
            eprintln!("{:?} <=> {:?}", &class1.hir, &class2.hir);
            assert_eq!(class1, class2);
        } else {
            panic!("Expected a literal AST.");
        }
    }

    #[test]
    fn test_character_class_equality_special_hir() {
        let hir1 = parse_regex_syntax("\r").unwrap();
        if let regex_syntax::hir::HirKind::Literal(_) = hir1.clone().into_kind() {
            let class1 = CharacterClass::new(0.into(), HirWithPattern::new(hir1.clone()));
            let class2 = CharacterClass::new(0.into(), HirLiteral!('\r'));
            eprintln!("{:?} <=> {:?}", &class1, &class2);
            assert_eq!(class1, class2);
        } else {
            panic!("Expected a literal Hir.");
        }
    }

    #[test]
    fn test_character_class_ordering() {
        let hir1 = HirLiteral!('a');
        let hir2 = HirLiteral!('b');
        let class1 = CharacterClass::new(0.into(), hir1);
        let class2 = CharacterClass::new(1.into(), hir2);
        assert!(class1 < class2);
        assert!(class2 > class1);
    }

    #[test]
    fn test_character_class_ordering_hir() {
        let hir1 = HirLiteral!('a');
        let hir2 = HirLiteral!('b');
        let class1 = CharacterClass::new(0.into(), hir1);
        let class2 = CharacterClass::new(1.into(), hir2);
        assert!(class1 < class2);
        assert!(class2 > class1);
    }
}
