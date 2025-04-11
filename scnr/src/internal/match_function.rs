use regex_syntax::ast::{
    Ast, ClassAscii, ClassAsciiKind, ClassBracketed, ClassPerl, ClassPerlKind, ClassSet,
    ClassSetBinaryOp, ClassSetBinaryOpKind, ClassSetItem, ClassSetRange, ClassSetUnion,
    ClassUnicode, Literal,
};
use seshat::unicode::{props::Gc, Ucd};

use crate::{Result, ScnrError, ScnrErrorKind};

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

impl TryFrom<&Literal> for MatchFn {
    type Error = ScnrError;

    #[inline(always)]
    fn try_from(l: &Literal) -> Result<Self> {
        let Literal {
            ref c, ref kind, ..
        } = *l;
        let c = *c;
        if c == '.' && *kind == regex_syntax::ast::LiteralKind::Verbatim {
            Ok(MatchFn::new(|ch| ch != '\n' && ch != '\r'))
        } else {
            Ok(MatchFn::new(move |ch| ch == c))
        }
    }
}

impl TryFrom<(&ClassSetUnion, &str)> for MatchFn {
    type Error = ScnrError;

    #[inline(always)]
    fn try_from((union, pattern): (&ClassSetUnion, &str)) -> Result<Self> {
        union
            .items
            .iter()
            .try_fold(MatchFn::new(|_| false), |acc, s| {
                (s, false, pattern)
                    .try_into()
                    .map(|f: MatchFn| MatchFn::new(move |ch| acc.inner()(ch) || f.inner()(ch)))
            })
    }
}

impl TryFrom<(&ClassBracketed, &str)> for MatchFn {
    type Error = ScnrError;

    #[inline(always)]
    fn try_from((bracketed, pattern): (&ClassBracketed, &str)) -> Result<Self> {
        let negated = bracketed.negated;
        match &bracketed.kind {
            ClassSet::Item(item) => (item, negated, pattern).try_into(),
            ClassSet::BinaryOp(bin_op) => (bin_op, negated, pattern).try_into(),
        }
    }
}

impl TryFrom<(&ClassUnicode, &str)> for MatchFn {
    type Error = ScnrError;

    #[inline(always)]
    fn try_from((unicode, pattern): (&ClassUnicode, &str)) -> Result<Self> {
        let hir = regex_syntax::hir::translate::TranslatorBuilder::new()
            .unicode(true)
            .build()
            .translate(
                &pattern[unicode.span.start.offset..unicode.span.end.offset],
                &Ast::ClassUnicode(Box::new(unicode.clone())),
            )
            .map_err(|e| {
                ScnrError::new(ScnrErrorKind::RegexHirError(
                    e,
                    "Can't translate UnicodeClass".to_string(),
                ))
            })?;
        // Create a match function that evaluates the unicode ranges provided by the ClassUnicode.
        if let regex_syntax::hir::HirKind::Class(regex_syntax::hir::Class::Unicode(u)) = hir.kind()
        {
            let match_function = u
                .ranges()
                .iter()
                .map(|r| {
                    let start = r.start();
                    let end = r.end();
                    MatchFn::new(move |ch| start <= ch && ch <= end)
                })
                .reduce(|acc, f| MatchFn::new(move |ch| acc.inner()(ch) || f.inner()(ch)))
                .unwrap_or_else(|| MatchFn::new(|_| false));
            Ok(if unicode.is_negated() {
                MatchFn::new(move |ch| !match_function.inner()(ch))
            } else {
                match_function
            })
        } else {
            return Err(unsupported!(format!(
                "Unicode class {:?} not supported",
                unicode.kind
            )));
        }
    }
}

impl TryFrom<&ClassPerl> for MatchFn {
    type Error = ScnrError;

    #[inline(always)]
    fn try_from(perl: &ClassPerl) -> Result<Self> {
        let ClassPerl { negated, kind, .. } = perl;
        let match_function = match kind {
            ClassPerlKind::Digit => MatchFn::new(|ch| ch.is_numeric()),
            ClassPerlKind::Space => MatchFn::new(|ch| ch.is_whitespace()),
            ClassPerlKind::Word => MatchFn::new(|ch| {
                ch.is_alphanumeric() || ch.join_c() || ch.gc() == Gc::Pc || ch.gc() == Gc::Mn
            }),
        };
        Ok(if *negated {
            MatchFn::new(move |ch| !match_function.inner()(ch))
        } else {
            match_function
        })
    }
}

impl TryFrom<(&ClassSet, &str)> for MatchFn {
    type Error = ScnrError;

    #[inline(always)]
    fn try_from((set, pattern): (&ClassSet, &str)) -> Result<Self> {
        let negated = false;
        match set {
            ClassSet::Item(item) => (item, negated, pattern).try_into(),
            ClassSet::BinaryOp(bin_op) => (bin_op, negated, pattern).try_into(),
        }
    }
}

impl TryFrom<(&ClassSetItem, bool, &str)> for MatchFn {
    type Error = ScnrError;

    #[inline(always)]
    fn try_from((item, negated, pattern): (&ClassSetItem, bool, &str)) -> Result<Self> {
        let match_function = match item {
            ClassSetItem::Empty(_) => MatchFn::new(|_| false),
            ClassSetItem::Literal(ref l) => l.try_into()?,
            ClassSetItem::Range(ref r) => {
                let ClassSetRange {
                    ref start, ref end, ..
                } = *r;
                let start = start.c;
                let end = end.c;
                MatchFn::new(move |ch| start <= ch && ch <= end)
            }
            ClassSetItem::Ascii(ref a) => {
                let ClassAscii {
                    ref kind, negated, ..
                } = *a;
                let match_function = match kind {
                    ClassAsciiKind::Alnum => MatchFn::new(|ch| ch.is_alphanumeric()),
                    ClassAsciiKind::Alpha => MatchFn::new(|ch| ch.is_alphabetic()),
                    ClassAsciiKind::Ascii => MatchFn::new(|ch| ch.is_ascii()),
                    ClassAsciiKind::Blank => MatchFn::new(|ch| ch.is_ascii_whitespace()),
                    ClassAsciiKind::Cntrl => MatchFn::new(|ch| ch.is_ascii_control()),
                    ClassAsciiKind::Digit => MatchFn::new(|ch| ch.is_numeric()),
                    ClassAsciiKind::Graph => MatchFn::new(|ch| ch.is_ascii_graphic()),
                    ClassAsciiKind::Lower => MatchFn::new(|ch| ch.is_lowercase()),
                    ClassAsciiKind::Print => MatchFn::new(|ch| ch.is_ascii_graphic()),
                    ClassAsciiKind::Punct => MatchFn::new(|ch| ch.is_ascii_punctuation()),
                    ClassAsciiKind::Space => MatchFn::new(|ch| ch.is_whitespace()),
                    ClassAsciiKind::Upper => MatchFn::new(|ch| ch.is_uppercase()),
                    ClassAsciiKind::Word => MatchFn::new(|ch| {
                        ch.is_alphanumeric()
                            || ch.join_c()
                            || ch.gc() == Gc::Pc
                            || ch.gc() == Gc::Mn
                    }),
                    ClassAsciiKind::Xdigit => MatchFn::new(|ch| ch.is_ascii_hexdigit()),
                };
                if negated {
                    MatchFn::new(move |ch| !match_function.inner()(ch))
                } else {
                    match_function
                }
            }
            ClassSetItem::Unicode(ref c) => (c, pattern).try_into()?,
            ClassSetItem::Perl(ref c) => c.try_into()?,
            ClassSetItem::Bracketed(ref c) => (c.as_ref(), pattern).try_into()?,
            ClassSetItem::Union(ref c) => (c, pattern).try_into()?,
        };
        Ok(if negated {
            MatchFn::new(move |ch| !match_function.inner()(ch))
        } else {
            match_function
        })
    }
}

impl TryFrom<(&ClassSetBinaryOp, bool, &str)> for MatchFn {
    type Error = ScnrError;

    #[inline(always)]
    fn try_from((bin_op, negated, pattern): (&ClassSetBinaryOp, bool, &str)) -> Result<Self> {
        let ClassSetBinaryOp { kind, lhs, rhs, .. } = bin_op;
        let lhs: MatchFn = (lhs.as_ref(), pattern).try_into()?;
        let rhs: MatchFn = (rhs.as_ref(), pattern).try_into()?;
        let match_function = match kind {
            ClassSetBinaryOpKind::Intersection => {
                MatchFn::new(move |ch| lhs.inner()(ch) && rhs.inner()(ch))
            }
            ClassSetBinaryOpKind::Difference => {
                MatchFn::new(move |ch| lhs.inner()(ch) && !rhs.inner()(ch))
            }
            ClassSetBinaryOpKind::SymmetricDifference => {
                MatchFn::new(move |ch| lhs.inner()(ch) != rhs.inner()(ch))
            }
        };
        Ok(if negated {
            MatchFn::new(move |ch| !match_function.inner()(ch))
        } else {
            match_function
        })
    }
}

impl TryFrom<(&Ast, &str)> for MatchFunction {
    type Error = ScnrError;

    #[inline(always)]
    fn try_from((ast, pattern): (&Ast, &str)) -> Result<Self> {
        let match_function = match ast {
            Ast::Empty(_) => {
                // An empty AST matches everything.
                Self::new(|_| true)
            }
            Ast::Dot(_) => {
                // A dot AST matches any character except newline.
                Self::new(|ch| ch != '\n' && ch != '\r')
            }
            Ast::Literal(ref l) => {
                // A literal AST matches a single character.
                Self {
                    match_fn: l.as_ref().try_into()?,
                }
            }
            Ast::ClassUnicode(ref c) => Self {
                match_fn: (c.as_ref(), pattern).try_into()?,
            },
            Ast::ClassPerl(ref c) => Self {
                match_fn: c.as_ref().try_into()?,
            },
            Ast::ClassBracketed(ref c) => Self {
                match_fn: (c.as_ref(), pattern).try_into()?,
            },
            _ => return Err(unsupported!(format!("{:#?}", ast))),
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
    use super::*;
    use regex_syntax::ast::parse::Parser;

    #[test]
    fn test_match_function_unicode_class() {
        let pattern = r"\pL";
        let ast = Parser::new().parse(pattern).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern)).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('A'));
        assert!(!match_function.call('1'));
        assert!(!match_function.call(' '));
    }

    #[test]
    fn test_match_function_perl_class() {
        let pattern = r"\d";
        let ast = Parser::new().parse(pattern).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern)).unwrap();
        assert!(match_function.call('1'));
        assert!(!match_function.call('a'));
    }

    #[test]
    fn test_match_function_bracketed_class() {
        let pattern = r"[a-z]";
        let ast = Parser::new().parse(pattern).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern)).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('z'));
        assert!(!match_function.call('A'));
        assert!(!match_function.call('1'));
    }

    #[test]
    fn test_match_function_binary_op_class_intersection() {
        // Intersection (matching x or y)
        let pattern = r"[a-y&&xyz]";
        let ast = Parser::new().parse(pattern).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern)).unwrap();
        assert!(match_function.call('x'));
        assert!(match_function.call('y'));
        assert!(!match_function.call('a'));
        assert!(!match_function.call('z'));
    }

    #[test]
    fn test_match_function_union_class() {
        let pattern = r"[0-9a-z]";
        let ast = Parser::new().parse(pattern).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern)).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('z'));
        assert!(match_function.call('0'));
        assert!(match_function.call('9'));
        assert!(!match_function.call('!'));
    }

    #[test]
    fn test_match_function_negated_bracketed_class() {
        let pattern = r"[^a-z]";
        let ast = Parser::new().parse(pattern).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern)).unwrap();
        assert!(!match_function.call('a'));
        assert!(!match_function.call('z'));
        assert!(match_function.call('A'));
        assert!(match_function.call('1'));
    }

    #[test]
    fn test_match_function_negated_binary_op_class() {
        let pattern = r"[a-z&&[^aeiou]]";
        let ast = Parser::new().parse(pattern).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern)).unwrap();
        assert!(!match_function.call('a'));
        assert!(!match_function.call('e'));
        assert!(match_function.call('z'));
        assert!(!match_function.call('1'));
    }

    // [[:alpha:]]   ASCII character class ([A-Za-z])
    #[test]
    fn test_match_function_ascci_class() {
        let pattern = r"[[:alpha:]]";
        let ast = Parser::new().parse(pattern).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern)).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('A'));
        assert!(match_function.call('ä'));
        assert!(!match_function.call('1'));
        assert!(!match_function.call(' '));
    }

    // [[:^alpha:]]  Negated ASCII character class ([^A-Za-z])
    #[test]
    fn test_match_function_negated_ascii_class() {
        let pattern = r"[^[:alpha:]]";
        let ast = Parser::new().parse(pattern).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern)).unwrap();
        assert!(!match_function.call('a'));
        assert!(!match_function.call('A'));
        assert!(!match_function.call('ä'));
        assert!(match_function.call('1'));
        assert!(match_function.call(' '));
    }

    #[test]
    fn test_match_function_empty() {
        let pattern = r"";
        let ast = Parser::new().parse(pattern).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern)).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('A'));
        assert!(match_function.call('1'));
        assert!(match_function.call(' '));
    }

    // [x[^xyz]]     Nested/grouping character class (matching any character except y and z)
    #[test]
    fn test_nested_classes() {
        let pattern = r"[x[^xyz]]";
        let ast = Parser::new().parse(pattern).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern)).unwrap();
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
        let ast = Parser::new().parse(pattern).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern)).unwrap();
        assert!(match_function.call('0'));
        assert!(match_function.call('9'));
        assert!(!match_function.call('4'));
        assert!(!match_function.call('a'));
    }

    // [0-9--4]      Direct subtraction (matching 0-9 except 4)
    #[test]
    fn test_direct_subtraction() {
        let pattern = r"[0-9--4]";
        let ast = Parser::new().parse(pattern).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern)).unwrap();
        assert!(match_function.call('0'));
        assert!(match_function.call('9'));
        assert!(!match_function.call('4'));
        assert!(!match_function.call('a'));
    }

    // [a-g~~b-h]    Symmetric difference (matching `a` and `h` only)
    #[test]
    fn test_symmetric_difference() {
        let pattern = r"[a-g~~b-h]";
        let ast = Parser::new().parse(pattern).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern)).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('h'));
        assert!(!match_function.call('b'));
        assert!(!match_function.call('z'));
    }

    // [\[\]]        Escaping in character classes (matching [ or ])
    #[test]
    fn test_escaping() {
        let pattern = r"[\[\]]";
        let ast = Parser::new().parse(pattern).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern)).unwrap();
        assert!(match_function.call('['));
        assert!(match_function.call(']'));
        assert!(!match_function.call('a'));
        assert!(!match_function.call('1'));
    }

    // [a&&b]        An empty character class matching nothing
    #[test]
    fn test_empty_intersection() {
        let pattern = r"[a&&b]";
        let ast = Parser::new().parse(pattern).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern)).unwrap();
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
        let ast = Parser::new().parse(pattern).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern)).unwrap();
        assert!(!match_function.call('a'));
        assert!(match_function.call('b'));
        assert!(match_function.call('1'));
        assert!(match_function.call(' '));
    }

    // \p{XID_Start} and \p{XID_Continue} are supported by unicode_xid
    #[test]
    fn test_named_classes() {
        let pattern = r"\p{XID_Start}";
        let ast = Parser::new().parse(pattern).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern)).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('A'));
        assert!(!match_function.call('1'));
        assert!(!match_function.call(' '));

        let pattern1 = r"\p{XID_Continue}";
        let ast = Parser::new().parse(pattern1).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern1)).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('A'));
        assert!(match_function.call('1'));
        assert!(!match_function.call(' '));
    }

    #[test]
    fn test_evaluate_general_category() {
        assert_eq!('_'.gc(), Gc::Pc);
        let pattern = r"\w";
        let ast = Parser::new().parse(pattern).unwrap();
        let match_function = MatchFunction::try_from((&ast, pattern)).unwrap();
        assert!(match_function.call('_'));
    }
}
