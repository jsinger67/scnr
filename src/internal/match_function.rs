use regex_syntax::ast::{
    Ast, ClassAscii, ClassAsciiKind, ClassBracketed, ClassPerl, ClassPerlKind, ClassSet,
    ClassSetBinaryOp, ClassSetBinaryOpKind, ClassSetItem, ClassSetRange, ClassSetUnion,
    ClassUnicode,
    ClassUnicodeKind::{Named, NamedValue, OneLetter},
    Literal,
};
use seshat::unicode::{props::Gc, Ucd};

use crate::{Result, ScnrError};

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

    #[inline]
    pub(crate) fn call(&self, c: char) -> bool {
        (self.0)(c)
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
        self.match_fn.call(c)
    }
}

impl TryFrom<&Literal> for MatchFn {
    type Error = ScnrError;

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

impl TryFrom<&ClassSetUnion> for MatchFn {
    type Error = ScnrError;

    fn try_from(union: &ClassSetUnion) -> Result<Self> {
        union
            .items
            .iter()
            .try_fold(MatchFn::new(|_| false), |acc, s| {
                (s, false)
                    .try_into()
                    .map(|f: MatchFn| MatchFn::new(move |ch| acc.call(ch) || f.call(ch)))
            })
    }
}

impl TryFrom<&ClassBracketed> for MatchFn {
    type Error = ScnrError;

    fn try_from(bracketed: &ClassBracketed) -> Result<Self> {
        let negated = bracketed.negated;
        match &bracketed.kind {
            ClassSet::Item(item) => (item, negated).try_into(),
            ClassSet::BinaryOp(bin_op) => (bin_op, negated).try_into(),
        }
    }
}

impl TryFrom<&ClassUnicode> for MatchFn {
    type Error = ScnrError;

    fn try_from(unicode: &ClassUnicode) -> Result<Self> {
        let kind = unicode.kind.clone();
        let match_function = match kind {
            OneLetter(ch) => {
                match ch {
                    // Unicode class for Letters
                    'L' => MatchFn::new(|ch| ch.alpha()),
                    // Unicode class for Numbers
                    'N' => MatchFn::new(|ch| ch.is_numeric()),
                    // Unicode class for Whitespace
                    'Z' => MatchFn::new(|ch| ch.is_whitespace()),
                    // Unicode class Terminal_Punctuation
                    'P' => MatchFn::new(|ch| ch.term()),
                    // Unicode class for Control characters
                    'C' => MatchFn::new(|ch| ch.is_control()),
                    _ => return Err(unsupported!(format!("Unicode class {}", ch))),
                }
            }
            Named(name) => match name.as_str() {
                "Alphabetic" => MatchFn::new(|ch| ch.alpha()),
                "ASCII_Hex_Digit" => MatchFn::new(|ch| ch.ahex()),
                "Bidi_Control" => MatchFn::new(|ch| ch.bidi_c()),
                "Case_Ignorable" => MatchFn::new(|ch| ch.ci()),
                "Cased" => MatchFn::new(|ch| ch.cased()),
                "Composition_Exclusion" => MatchFn::new(|ch| ch.ce()),
                "Dash" => MatchFn::new(|ch| ch.dash()),
                "Default_Ignorable_Code_Point" => MatchFn::new(|ch| ch.di()),
                "Deprecated" => MatchFn::new(|ch| ch.dep()),
                "Diacritic" => MatchFn::new(|ch| ch.dia()),
                "Emoji_Component" => MatchFn::new(|ch| ch.ecomp()),
                "Emoji_Modifier_Base" => MatchFn::new(|ch| ch.ebase()),
                "Emoji_Modifier" => MatchFn::new(|ch| ch.emod()),
                "Emoji_Presentation" => MatchFn::new(|ch| ch.epres()),
                "Emoji" => MatchFn::new(|ch| ch.emoji()),
                "Extended_Pictographic" => MatchFn::new(|ch| ch.ext_pict()),
                "Extender" => MatchFn::new(|ch| ch.ext()),
                "Full_Composition_Exclusion" => MatchFn::new(|ch| ch.comp_ex()),
                "Grapheme_Extend" => MatchFn::new(|ch| ch.gr_ext()),
                "Hex_Digit" => MatchFn::new(|ch| ch.hex()),
                "Hyphen" => MatchFn::new(|ch| ch.hyphen()),
                "ID_Continue" => MatchFn::new(|ch| ch.idc()),
                "ID_Start" => MatchFn::new(|ch| ch.ids()),
                "Ideographic" => MatchFn::new(|ch| ch.ideo()),
                "IDS_Binary_Operator" => MatchFn::new(|ch| ch.idsb()),
                "IDS_Trinary_Operator" => MatchFn::new(|ch| ch.idst()),
                "Join_Control" => MatchFn::new(|ch| ch.join_c()),
                "Logical_Order_Exception" => MatchFn::new(|ch| ch.loe()),
                "Lowercase" => MatchFn::new(|ch| ch.lower()),
                "Math" => MatchFn::new(|ch| ch.math()),
                "Noncharacter_Code_Point" => MatchFn::new(|ch| ch.nchar()),
                "Other_Alphabetic" => MatchFn::new(|ch| ch.oalpha()),
                "Other_Default_Ignorable_Code_Point" => MatchFn::new(|ch| ch.odi()),
                "Other_Grapheme_Extend" => MatchFn::new(|ch| ch.ogr_ext()),
                "Other_ID_Continue" => MatchFn::new(|ch| ch.oidc()),
                "Other_ID_Start" => MatchFn::new(|ch| ch.oids()),
                "Other_Lowercase" => MatchFn::new(|ch| ch.olower()),
                "Other_Math" => MatchFn::new(|ch| ch.omath()),
                "Other_Uppercase" => MatchFn::new(|ch| ch.oupper()),
                "Pattern_Syntax" => MatchFn::new(|ch| ch.pat_syn()),
                "Pattern_White_Space" => MatchFn::new(|ch| ch.pat_ws()),
                "Prepended_Concatenation_Mark" => MatchFn::new(|ch| ch.pcm()),
                "Quotation_Mark" => MatchFn::new(|ch| ch.qmark()),
                "Radical" => MatchFn::new(|ch| ch.radical()),
                "Regional_Indicator" => MatchFn::new(|ch| ch.ri()),
                "Sentence_Terminal" => MatchFn::new(|ch| ch.sterm()),
                "Soft_Dotted" => MatchFn::new(|ch| ch.sd()),
                "Terminal_Punctuation" => MatchFn::new(|ch| ch.term()),
                "Unified_Ideograph" => MatchFn::new(|ch| ch.uideo()),
                "Uppercase" => MatchFn::new(|ch| ch.upper()),
                "Variation_Selector" => MatchFn::new(|ch| ch.vs()),
                "White_Space" => MatchFn::new(|ch| ch.wspace()),
                "XID_Continue" => MatchFn::new(|ch| ch.xidc()),
                "XID_Start" => MatchFn::new(|ch| ch.xids()),
                _ => return Err(unsupported!(format!("Unicode named class {}", name))),
            },
            NamedValue { name, value, .. } => {
                return Err(unsupported!(format!("Named value {}={}", name, value)));
            }
        };
        Ok(if unicode.is_negated() {
            MatchFn::new(move |ch| !match_function.call(ch))
        } else {
            match_function
        })
    }
}

impl TryFrom<&ClassPerl> for MatchFn {
    type Error = ScnrError;

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
            MatchFn::new(move |ch| !match_function.call(ch))
        } else {
            match_function
        })
    }
}

impl TryFrom<&ClassSet> for MatchFn {
    type Error = ScnrError;

    fn try_from(set: &ClassSet) -> Result<Self> {
        let negated = false;
        match set {
            ClassSet::Item(item) => (item, negated).try_into(),
            ClassSet::BinaryOp(bin_op) => (bin_op, negated).try_into(),
        }
    }
}

impl TryFrom<(&ClassSetItem, bool)> for MatchFn {
    type Error = ScnrError;

    fn try_from((item, negated): (&ClassSetItem, bool)) -> Result<Self> {
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
                    MatchFn::new(move |ch| !match_function.call(ch))
                } else {
                    match_function
                }
            }
            ClassSetItem::Unicode(ref c) => c.try_into()?,
            ClassSetItem::Perl(ref c) => c.try_into()?,
            ClassSetItem::Bracketed(ref c) => c.as_ref().try_into()?,
            ClassSetItem::Union(ref c) => c.try_into()?,
        };
        Ok(if negated {
            MatchFn::new(move |ch| !match_function.call(ch))
        } else {
            match_function
        })
    }
}

impl TryFrom<(&ClassSetBinaryOp, bool)> for MatchFn {
    type Error = ScnrError;

    fn try_from((bin_op, negated): (&ClassSetBinaryOp, bool)) -> Result<Self> {
        let ClassSetBinaryOp { kind, lhs, rhs, .. } = bin_op;
        let lhs: MatchFn = lhs.as_ref().try_into()?;
        let rhs: MatchFn = rhs.as_ref().try_into()?;
        let match_function = match kind {
            ClassSetBinaryOpKind::Intersection => {
                MatchFn::new(move |ch| lhs.call(ch) && rhs.call(ch))
            }
            ClassSetBinaryOpKind::Difference => {
                MatchFn::new(move |ch| lhs.call(ch) && !rhs.call(ch))
            }
            ClassSetBinaryOpKind::SymmetricDifference => {
                MatchFn::new(move |ch| lhs.call(ch) != rhs.call(ch))
            }
        };
        Ok(if negated {
            MatchFn::new(move |ch| !match_function.call(ch))
        } else {
            match_function
        })
    }
}

impl TryFrom<&Ast> for MatchFunction {
    type Error = ScnrError;

    fn try_from(ast: &Ast) -> Result<Self> {
        let match_function = match ast {
            Ast::Empty(_) => {
                // An empty AST matches everything.
                MatchFunction::new(|_| true)
            }
            Ast::Dot(_) => {
                // A dot AST matches any character except newline.
                MatchFunction::new(|ch| ch != '\n' && ch != '\r')
            }
            Ast::Literal(ref l) => {
                // A literal AST matches a single character.
                MatchFunction {
                    match_fn: l.as_ref().try_into()?,
                }
            }
            Ast::ClassUnicode(ref c) => Self {
                match_fn: c.as_ref().try_into()?,
            },
            Ast::ClassPerl(ref c) => Self {
                match_fn: c.as_ref().try_into()?,
            },
            Ast::ClassBracketed(ref c) => Self {
                match_fn: c.as_ref().try_into()?,
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
        let ast = Parser::new().parse(r"\pL").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('A'));
        assert!(!match_function.call('1'));
        assert!(!match_function.call(' '));
    }

    #[test]
    fn test_match_function_perl_class() {
        let ast = Parser::new().parse(r"\d").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
        assert!(match_function.call('1'));
        assert!(!match_function.call('a'));
    }

    #[test]
    fn test_match_function_bracketed_class() {
        let ast = Parser::new().parse(r"[a-z]").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('z'));
        assert!(!match_function.call('A'));
        assert!(!match_function.call('1'));
    }

    #[test]
    fn test_match_function_binary_op_class_intersection() {
        // Intersection (matching x or y)
        let ast = Parser::new().parse(r"[a-y&&xyz]").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
        assert!(match_function.call('x'));
        assert!(match_function.call('y'));
        assert!(!match_function.call('a'));
        assert!(!match_function.call('z'));
    }

    #[test]
    fn test_match_function_union_class() {
        let ast = Parser::new().parse(r"[0-9a-z]").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('z'));
        assert!(match_function.call('0'));
        assert!(match_function.call('9'));
        assert!(!match_function.call('!'));
    }

    #[test]
    fn test_match_function_negated_bracketed_class() {
        let ast = Parser::new().parse(r"[^a-z]").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
        assert!(!match_function.call('a'));
        assert!(!match_function.call('z'));
        assert!(match_function.call('A'));
        assert!(match_function.call('1'));
    }

    #[test]
    fn test_match_function_negated_binary_op_class() {
        let ast = Parser::new().parse(r"[a-z&&[^aeiou]]").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
        assert!(!match_function.call('a'));
        assert!(!match_function.call('e'));
        assert!(match_function.call('z'));
        assert!(!match_function.call('1'));
    }

    // [[:alpha:]]   ASCII character class ([A-Za-z])
    #[test]
    fn test_match_function_ascci_class() {
        let ast = Parser::new().parse(r"[[:alpha:]]").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('A'));
        assert!(match_function.call('ä'));
        assert!(!match_function.call('1'));
        assert!(!match_function.call(' '));
    }

    // [[:^alpha:]]  Negated ASCII character class ([^A-Za-z])
    #[test]
    fn test_match_function_negated_ascii_class() {
        let ast = Parser::new().parse(r"[^[:alpha:]]").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
        assert!(!match_function.call('a'));
        assert!(!match_function.call('A'));
        assert!(!match_function.call('ä'));
        assert!(match_function.call('1'));
        assert!(match_function.call(' '));
    }

    #[test]
    fn test_match_function_empty() {
        let ast = Parser::new().parse(r"").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('A'));
        assert!(match_function.call('1'));
        assert!(match_function.call(' '));
    }

    // [x[^xyz]]     Nested/grouping character class (matching any character except y and z)
    #[test]
    fn test_nested_classes() {
        let ast = Parser::new().parse(r"[x[^xyz]]").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
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
        let ast = Parser::new().parse(r"[0-9&&[^4]]").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
        assert!(match_function.call('0'));
        assert!(match_function.call('9'));
        assert!(!match_function.call('4'));
        assert!(!match_function.call('a'));
    }

    // [0-9--4]      Direct subtraction (matching 0-9 except 4)
    #[test]
    fn test_direct_subtraction() {
        let ast = Parser::new().parse(r"[0-9--4]").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
        assert!(match_function.call('0'));
        assert!(match_function.call('9'));
        assert!(!match_function.call('4'));
        assert!(!match_function.call('a'));
    }

    // [a-g~~b-h]    Symmetric difference (matching `a` and `h` only)
    #[test]
    fn test_symmetric_difference() {
        let ast = Parser::new().parse(r"[a-g~~b-h]").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('h'));
        assert!(!match_function.call('b'));
        assert!(!match_function.call('z'));
    }

    // [\[\]]        Escaping in character classes (matching [ or ])
    #[test]
    fn test_escaping() {
        let ast = Parser::new().parse(r"[\[\]]").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
        assert!(match_function.call('['));
        assert!(match_function.call(']'));
        assert!(!match_function.call('a'));
        assert!(!match_function.call('1'));
    }

    // [a&&b]        An empty character class matching nothing
    #[test]
    fn test_empty_intersection() {
        let ast = Parser::new().parse(r"[a&&b]").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
        assert!(!match_function.call('a'));
        assert!(!match_function.call('b'));
        assert!(!match_function.call('1'));
        assert!(!match_function.call(' '));
    }

    // [^a--b]       A negated subtraction character class (matching b and everything but a)
    //               This is equivalent to [^a]
    #[test]
    fn test_negated_subtraction() {
        let ast = Parser::new().parse(r"[^a--b]").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
        assert!(!match_function.call('a'));
        assert!(match_function.call('b'));
        assert!(match_function.call('1'));
        assert!(match_function.call(' '));
    }

    // \p{XID_Start} and \p{XID_Continue} are supported by unicode_xid
    #[test]
    fn test_named_classes() {
        let ast = Parser::new().parse(r"\p{XID_Start}").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('A'));
        assert!(!match_function.call('1'));
        assert!(!match_function.call(' '));

        let ast = Parser::new().parse(r"\p{XID_Continue}").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
        assert!(match_function.call('a'));
        assert!(match_function.call('A'));
        assert!(match_function.call('1'));
        assert!(!match_function.call(' '));
    }

    #[test]
    fn test_evaluate_general_category() {
        assert_eq!('_'.gc(), Gc::Pc);
        let ast = Parser::new().parse(r"\w").unwrap();
        let match_function = MatchFunction::try_from(&ast).unwrap();
        assert!(match_function.call('_'));
    }
}
