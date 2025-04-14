use regex_syntax::hir::ClassUnicodeRange;

use crate::{Result, ScnrError};

use super::HirWithPattern;

macro_rules! unsupported {
    ($feature:expr) => {
        ScnrError::new($crate::ScnrErrorKind::UnsupportedFeature(
            $feature.to_string(),
        ))
    };
}

pub(crate) struct MatchFn(Box<dyn Fn(char) -> bool + 'static + Send + Sync>);

impl MatchFn {
    pub(crate) fn new<F>(f: F) -> Self
    where
        F: Fn(char) -> bool + 'static + Send + Sync,
    {
        MatchFn(Box::new(f))
    }

    /// Flatten the MatchFn to a function pointer for performance.
    /// This is safe because MatchFn is a thin wrapper around a function pointer.
    #[inline]
    pub(crate) fn inner(&self) -> &(dyn Fn(char) -> bool + 'static + Send + Sync) {
        &*self.0
    }
}

/// A function that takes a character and returns a boolean.
pub(crate) struct MatchFunction {
    pub(crate) match_fn: MatchFn,
}

impl MatchFunction {
    /// Create a new match function from a closure.
    pub(crate) fn new<F>(f: F) -> Self
    where
        F: Fn(char) -> bool + 'static + Send + Sync,
    {
        MatchFunction {
            match_fn: MatchFn::new(f),
        }
    }

    /// Call the match function with a character.
    #[inline]
    pub(crate) fn call(&self, c: char) -> bool {
        self.match_fn.inner()(c)
    }
}

impl TryFrom<&HirWithPattern> for MatchFunction {
    type Error = ScnrError;

    #[inline(always)]
    fn try_from(hir: &HirWithPattern) -> Result<Self> {
        // Translate the AST to a Hir and create a match function from it.
        let match_function = match &hir.hir.kind() {
            regex_syntax::hir::HirKind::Empty => {
                // An empty AST matches everything.
                Self::new(|_| true)
            }
            regex_syntax::hir::HirKind::Literal(literal) => {
                // A literal AST matches a single character.
                // We need to clone the bytes because the Hir is borrowed.
                let bytes = literal.0.clone();
                Self::new(move |ch| {
                    let mut buffer = [0; 4];
                    let utf8_bytes = ch.encode_utf8(&mut buffer);
                    &*bytes == utf8_bytes.as_bytes()
                })
            }
            regex_syntax::hir::HirKind::Class(class) => match class {
                regex_syntax::hir::Class::Unicode(class_unicode) => {
                    let ranges = class_unicode.ranges().to_vec();
                    let match_fn = MatchFn::new(move |ch| {
                        for range in &ranges {
                            if range.start() <= ch && ch <= range.end() {
                                return true;
                            }
                        }
                        false
                    });
                    Self { match_fn }
                }
                regex_syntax::hir::Class::Bytes(class_bytes) => {
                    let ranges: Vec<ClassUnicodeRange> = class_bytes
                        .ranges()
                        .iter()
                        .map(|r| ClassUnicodeRange::new(r.start().into(), r.end().into()))
                        .collect();
                    let match_fn = MatchFn::new(move |ch| {
                        for range in &ranges {
                            if range.start() <= ch && ch <= range.end() {
                                return true;
                            }
                        }
                        false
                    });
                    Self { match_fn }
                }
            },
            _ => return Err(unsupported!(format!("{:#?}", hir))),
        };
        Ok(match_function)
    }
}

impl std::fmt::Debug for MatchFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MatchFunction")
    }
}

#[cfg(test)]
mod tests {
    use crate::internal::parse_regex_syntax;

    use super::*;

    #[test]
    fn test_match_function_unicode_class() {
        let pattern = r"\pL";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('A'));
        assert!(!match_function.call('1'));
        assert!(!match_function.call(' '));
    }

    #[test]
    fn test_match_function_perl_class() {
        let pattern = r"\d";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(match_function.call('1'));
        assert!(!match_function.call('a'));
    }

    #[test]
    fn test_match_function_bracketed_class() {
        let pattern = r"[a-z]";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('z'));
        assert!(!match_function.call('A'));
        assert!(!match_function.call('1'));
    }

    #[test]
    fn test_match_function_binary_op_class_intersection() {
        // Intersection (matching x or y)
        let pattern = r"[a-y&&xyz]";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(match_function.call('x'));
        assert!(match_function.call('y'));
        assert!(!match_function.call('a'));
        assert!(!match_function.call('z'));
    }

    #[test]
    fn test_match_function_union_class() {
        let pattern = r"[0-9a-z]";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('z'));
        assert!(match_function.call('0'));
        assert!(match_function.call('9'));
        assert!(!match_function.call('!'));
    }

    #[test]
    fn test_match_function_negated_bracketed_class() {
        let pattern = r"[^a-z]";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(!match_function.call('a'));
        assert!(!match_function.call('z'));
        assert!(match_function.call('A'));
        assert!(match_function.call('1'));
    }

    #[test]
    fn test_match_function_negated_binary_op_class() {
        let pattern = r"[a-z&&[^aeiou]]";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(!match_function.call('a'));
        assert!(!match_function.call('e'));
        assert!(match_function.call('z'));
        assert!(!match_function.call('1'));
    }

    // [[:alpha:]]   ASCII character class ([A-Za-z])
    #[test]
    fn test_match_function_ascii_class() {
        let pattern = r"[[:alpha:]]";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('A'));
        assert!(!match_function.call('ä'));
        assert!(!match_function.call('1'));
        assert!(!match_function.call(' '));
    }

    // [[:^alpha:]]  Negated ASCII character class ([^A-Za-z])
    #[test]
    fn test_match_function_negated_ascii_class() {
        let pattern = r"[^[:alpha:]]";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(!match_function.call('a'));
        assert!(!match_function.call('A'));
        assert!(match_function.call('ä'));
        assert!(match_function.call('1'));
        assert!(match_function.call(' '));
    }

    #[test]
    fn test_match_function_empty() {
        let pattern = r"";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('A'));
        assert!(match_function.call('1'));
        assert!(match_function.call(' '));
    }

    // [x[^xyz]]     Nested/grouping character class (matching any character except y and z)
    #[test]
    fn test_nested_classes() {
        let pattern = r"[x[^xyz]]";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('x'));
        assert!(match_function.call('1'));
        assert!(match_function.call(' '));
        assert!(!match_function.call('y'));
        assert!(!match_function.call('z'));
    }

    // [0-9&&[^4]]   Subtraction using intersection and negation (matching 0-9 except 4)
    #[test]
    fn test_subtraction() {
        let pattern = r"[0-9&&[^4]]";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(match_function.call('0'));
        assert!(match_function.call('9'));
        assert!(!match_function.call('4'));
        assert!(!match_function.call('a'));
    }

    // [0-9--4]      Direct subtraction (matching 0-9 except 4)
    #[test]
    fn test_direct_subtraction() {
        let pattern = r"[0-9--4]";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(match_function.call('0'));
        assert!(match_function.call('9'));
        assert!(!match_function.call('4'));
        assert!(!match_function.call('a'));
    }

    // [a-g~~b-h]    Symmetric difference (matching `a` and `h` only)
    #[test]
    fn test_symmetric_difference() {
        let pattern = r"[a-g~~b-h]";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('h'));
        assert!(!match_function.call('b'));
        assert!(!match_function.call('z'));
    }

    // [\[\]]        Escaping in character classes (matching [ or ])
    #[test]
    fn test_escaping() {
        let pattern = r"[\[\]]";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(match_function.call('['));
        assert!(match_function.call(']'));
        assert!(!match_function.call('a'));
        assert!(!match_function.call('1'));
    }

    // [a&&b]        An empty character class matching nothing
    #[test]
    fn test_empty_intersection() {
        let pattern = r"[a&&b]";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(!match_function.call('a'));
        assert!(!match_function.call('b'));
        assert!(!match_function.call('1'));
        assert!(!match_function.call(' '));
    }

    // [^a--b]       A negated subtraction character class (matching b and everything but a)
    //               This is equivalent to [^a]
    #[test]
    fn test_negated_subtraction() {
        let pattern = r"[^a--b]";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(!match_function.call('a'));
        assert!(match_function.call('b'));
        assert!(match_function.call('1'));
        assert!(match_function.call(' '));
    }

    // \p{XID_Start} and \p{XID_Continue} are supported by unicode_xid
    #[test]
    fn test_named_classes() {
        let pattern = r"\p{XID_Start}";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('A'));
        assert!(!match_function.call('1'));
        assert!(!match_function.call(' '));

        let pattern1 = r"\p{XID_Continue}";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern1).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('A'));
        assert!(match_function.call('1'));
        assert!(!match_function.call(' '));
    }

    #[test]
    fn test_evaluate_general_category() {
        let pattern = r"\w";
        let hir = HirWithPattern::new(parse_regex_syntax(pattern).unwrap());
        let match_function = MatchFunction::try_from(&hir).unwrap();
        assert!(match_function.call('_'));
    }
}
