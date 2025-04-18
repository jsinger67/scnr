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

    pub(crate) fn generate(&self) -> proc_macro2::TokenStream {
        let id = self.id.as_usize();
        match &self.hir.hir.kind() {
            regex_syntax::hir::HirKind::Empty => {
                quote::quote! {
                    #id => |_c: char| -> bool {
                        // An empty Hir matches everything.
                        true
                    }
                }
            }
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
                quote::quote! {
                     #id => |c: char| -> bool {
                        #lit == c as u32
                    }
                }
            }
            regex_syntax::hir::HirKind::Class(class) => match class {
                regex_syntax::hir::Class::Unicode(class_unicode) => {
                    let ranges = class_unicode.ranges().iter().fold(
                        proc_macro2::TokenStream::new(),
                        |mut acc, r| {
                            let start: char = r.start();
                            let end: char = r.end();
                            if start == end {
                                acc.extend(quote::quote! {
                                    if c == #start {
                                        return true;
                                    }
                                });
                            } else {
                                acc.extend(quote::quote! {
                                    if c >= #start && c <= #end {
                                        return true;
                                    }
                                });
                            }
                            acc
                        },
                    );
                    quote::quote! {
                        #id => |c: char| -> bool {
                            #ranges
                            false
                        }
                    }
                }
                regex_syntax::hir::Class::Bytes(class_bytes) => {
                    let ranges = class_bytes.ranges().iter().fold(
                        proc_macro2::TokenStream::new(),
                        |mut acc, r| {
                            let start: char = r.start().into();
                            let end: char = r.end().into();
                            if start == end {
                                acc.extend(quote::quote! {
                                    if c == #start {
                                        return true;
                                    }
                                });
                            } else {
                                acc.extend(quote::quote! {
                                    if c >= #start && c <= #end {
                                        return true;
                                    }
                                });
                            }
                            acc
                        },
                    );
                    quote::quote! {
                        #id => |c: char| -> bool {
                            #ranges
                            false
                        }
                    }
                }
            },
            _ => {
                panic!("Unsupported Hir kind: {:?}", self.hir.hir.kind())
            }
        }
    }
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
