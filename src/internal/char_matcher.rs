// TODO: Remove this line once the file is implemented.
#![allow(dead_code)]

use regex_syntax::ast::{
    Ast, ClassAsciiKind, ClassPerl, ClassPerlKind, ClassSet, ClassSetBinaryOpKind, ClassSetItem,
    ClassUnicode, ClassUnicodeKind, LiteralKind,
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

#[derive(Debug, Clone, Copy)]
pub(crate) enum UnicodeClassNames {
    Alphabetic,
    AsciiHexDigit,
    BidiControl,
    CaseIgnorable,
    Cased,
    CompositionExclusion,
    Dash,
    DefaultIgnorableCodePoint,
    Deprecated,
    Diacritic,
    EmojiComponent,
    EmojiModifierBase,
    EmojiModifier,
    EmojiPresentation,
    Emoji,
    ExtendedPictographic,
    Extender,
    FullCompositionExclusion,
    GraphemeExtend,
    HexDigit,
    Hyphen,
    IdContinue,
    IdStart,
    Ideographic,
    IdsBinaryOperator,
    IdsTrinaryOperator,
    JoinControl,
    LogicalOrderException,
    Lowercase,
    Math,
    NoncharacterCodePoint,
    OtherAlphabetic,
    OtherDefaultIgnorableCodePoint,
    OtherGraphemeExtend,
    OtherIdContinue,
    OtherIdStart,
    OtherLowercase,
    OtherMath,
    OtherUppercase,
    PatternSyntax,
    PatternWhiteSpace,
    PrependedConcatenationMark,
    QuotationMark,
    Radical,
    RegionalIndicator,
    SentenceTerminal,
    SoftDotted,
    TerminalPunctuation,
    UnifiedIdeograph,
    Uppercase,
    VariationSelector,
    WhiteSpace,
    XidContinue,
    XidStart,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum UnicodeClassOneLetter {
    L,
    N,
    Z,
    P,
    C,
}

/// A character class is a set of characters that can be matched by a regular expression.
/// This enum represents a character class specialized for fast matching.
#[derive(Debug, Clone)]
pub(crate) enum CharMatcher {
    /// Matches any character except carriage return and newline.
    Dot,
    /// Negated dot, matches carriage return and newline.
    NegatedDot,
    /// Matches exactly the given character.
    Literal(char),
    /// Negated literal character.
    NegatedLiteral(char),
    /// Matches the union of the given character matchers.
    Union(Vec<CharMatcher>),
    /// A negated union of character matchers.
    NegatedUnion(Vec<CharMatcher>),
    /// Unicode class with one letter abbreviation, e.g. \pL.
    UnicodeClassOneLetter(UnicodeClassOneLetter),
    /// Negated unicode class with one letter abbreviation, e.g. \pL.
    NegatedUnicodeClassOneLetter(UnicodeClassOneLetter),
    /// A binary property, general category or script.
    NamedUnicodeClass(UnicodeClassNames),
    /// A negated binary property, general category or script.
    NegatedNamedUnicodeClass(UnicodeClassNames),
    /// Perl classes
    ClassPerl(ClassPerlKind),
    /// Negated Perl classes
    NegatedClassPerl(ClassPerlKind),
    /// Class set items
    /// Empty class set item
    ClassSetItemEmpty,
    /// Negated empty class set item
    NegatedClassSetItemEmpty,
    /// Class set item with a range of characters
    ClassSetItemRange(char, char),
    /// Negated class set item with a range of characters
    NegatedClassSetItemRange(char, char),
    /// Class set item with ASCII characters
    SetItemAscii(ClassAsciiKind),
    /// Negated class set item with ASCII characters
    NegatedSetItemAscii(ClassAsciiKind),
    /// A binary operator.
    ClassBinaryOperator(ClassSetBinaryOpKind, Box<CharMatcher>, Box<CharMatcher>),
    /// A negated binary operator.
    NegatedClassBinaryOperator(ClassSetBinaryOpKind, Box<CharMatcher>, Box<CharMatcher>),
}

impl CharMatcher {
    /// Returns true if the given character matches the character class.
    #[inline(always)]
    pub(crate) fn matches(&self, c: char) -> bool {
        match self {
            CharMatcher::Dot => c != '\r' && c != '\n',
            CharMatcher::NegatedDot => c == '\r' || c == '\n',
            CharMatcher::Literal(literal) => *literal == c,
            CharMatcher::NegatedLiteral(literal) => *literal != c,
            CharMatcher::Union(matchers) => matchers.iter().any(|m| m.matches(c)),
            CharMatcher::NegatedUnion(matchers) => matchers.iter().all(|m| !m.matches(c)),
            CharMatcher::UnicodeClassOneLetter(ch) => match_letter_unicode_class(c, ch),
            CharMatcher::NegatedUnicodeClassOneLetter(ch) => {
                match_negated_letter_unicode_class(c, ch)
            }
            CharMatcher::NamedUnicodeClass(name) => match_unicode_class(c, name),
            CharMatcher::NegatedNamedUnicodeClass(name) => !match_unicode_class(c, name),
            CharMatcher::ClassPerl(kind) => match_perl_class(c, kind),
            CharMatcher::NegatedClassPerl(kind) => !match_perl_class(c, kind),
            CharMatcher::ClassSetItemEmpty => true,
            CharMatcher::NegatedClassSetItemEmpty => false,
            CharMatcher::ClassSetItemRange(start, end) => *start <= c && c <= *end,
            CharMatcher::NegatedClassSetItemRange(start, end) => *start > c || c > *end,
            CharMatcher::SetItemAscii(kind) => match_ascii_class(c, kind),
            CharMatcher::NegatedSetItemAscii(kind) => !match_ascii_class(c, kind),
            CharMatcher::ClassBinaryOperator(
                class_set_binary_op_kind,
                char_matcher,
                char_matcher1,
            ) => match_class_set_binary_operation(
                c,
                class_set_binary_op_kind,
                char_matcher,
                char_matcher1,
            ),
            CharMatcher::NegatedClassBinaryOperator(
                class_set_binary_op_kind,
                char_matcher,
                char_matcher1,
            ) => !match_class_set_binary_operation(
                c,
                class_set_binary_op_kind,
                char_matcher,
                char_matcher1,
            ),
        }
    }
}

impl TryFrom<&Ast> for CharMatcher {
    type Error = ScnrError;

    fn try_from(ast: &Ast) -> Result<Self> {
        let char_matcher = match ast {
            Ast::Empty(_) => {
                // An empty AST matches everything.
                CharMatcher::ClassSetItemEmpty
            }
            Ast::Dot(_) => {
                // A dot AST matches any character except newline.
                CharMatcher::Dot
            }
            Ast::Literal(ref literal) => {
                // A literal AST matches a single character.
                let c = literal.c;
                if c == '.' && literal.kind == LiteralKind::Verbatim {
                    CharMatcher::Dot
                } else {
                    CharMatcher::Literal(c)
                }
            }
            Ast::ClassUnicode(ref unicode) => try_from_unicode_class(unicode)?,
            Ast::ClassPerl(ref perl) => from_perl_class(perl),
            Ast::ClassBracketed(ref bracketed) => {
                try_from_class_set(&bracketed.kind, bracketed.negated)?
            }
            _ => return Err(unsupported!(format!("{:#?}", ast))),
        };
        Ok(char_matcher)
    }
}

impl std::fmt::Display for CharMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CharMatcher::Dot => write!(f, "."),
            CharMatcher::NegatedDot => write!(f, "[^\\r\\n]"),
            CharMatcher::Literal(c) => write!(f, "{}", c),
            CharMatcher::NegatedLiteral(c) => write!(f, "[^{}]", c),
            CharMatcher::Union(matchers) => {
                write!(f, "[")?;
                for matcher in matchers {
                    write!(f, "{}", matcher)?;
                }
                write!(f, "]")
            }
            CharMatcher::NegatedUnion(matchers) => {
                write!(f, "[^")?;
                for matcher in matchers {
                    write!(f, "{}", matcher)?;
                }
                write!(f, "]")
            }
            CharMatcher::UnicodeClassOneLetter(ch) => write!(f, "\\p{}", ch),
            CharMatcher::NegatedUnicodeClassOneLetter(ch) => write!(f, "\\P{}", ch),
            CharMatcher::NamedUnicodeClass(name) => write!(f, "\\p{{{}:}}", name),
            CharMatcher::NegatedNamedUnicodeClass(name) => write!(f, "\\P{{{}:}}", name),
            CharMatcher::ClassPerl(kind) => write!(f, "\\{:?}", kind),
            CharMatcher::NegatedClassPerl(kind) => write!(f, "\\{:?}", kind),
            CharMatcher::ClassSetItemEmpty => write!(f, ""),
            CharMatcher::NegatedClassSetItemEmpty => write!(f, ""),
            CharMatcher::ClassSetItemRange(start, end) => write!(f, "[{}-{}]", start, end),
            CharMatcher::NegatedClassSetItemRange(start, end) => write!(f, "[^{}-{}]", start, end),
            CharMatcher::SetItemAscii(kind) => write!(f, "[[:{:?}:]]", kind),
            CharMatcher::NegatedSetItemAscii(kind) => write!(f, "[^[:{:?}:]]", kind),
            CharMatcher::ClassBinaryOperator(kind, char_matcher, char_matcher1) => {
                write!(f, "{}", char_matcher)?;
                match kind {
                    ClassSetBinaryOpKind::Intersection => write!(f, "&&")?,
                    ClassSetBinaryOpKind::Difference => write!(f, "--")?,
                    ClassSetBinaryOpKind::SymmetricDifference => write!(f, "^")?,
                }
                write!(f, "{}", char_matcher1)
            }
            CharMatcher::NegatedClassBinaryOperator(kind, char_matcher, char_matcher1) => {
                write!(f, "{}", char_matcher)?;
                write!(f, "{:?}", kind)?;
                write!(f, "{}", char_matcher1)
            }
        }
    }
}

impl std::fmt::Display for UnicodeClassNames {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnicodeClassNames::Alphabetic => write!(f, "Alphabetic"),
            UnicodeClassNames::AsciiHexDigit => write!(f, "ASCII_Hex_Digit"),
            UnicodeClassNames::BidiControl => write!(f, "Bidi_Control"),
            UnicodeClassNames::CaseIgnorable => write!(f, "Case_Ignorable"),
            UnicodeClassNames::Cased => write!(f, "Cased"),
            UnicodeClassNames::CompositionExclusion => write!(f, "Composition_Exclusion"),
            UnicodeClassNames::Dash => write!(f, "Dash"),
            UnicodeClassNames::DefaultIgnorableCodePoint => {
                write!(f, "Default_Ignorable_Code_Point")
            }
            UnicodeClassNames::Deprecated => write!(f, "Deprecated"),
            UnicodeClassNames::Diacritic => write!(f, "Diacritic"),
            UnicodeClassNames::EmojiComponent => write!(f, "Emoji_Component"),
            UnicodeClassNames::EmojiModifierBase => write!(f, "Emoji_Modifier_Base"),
            UnicodeClassNames::EmojiModifier => write!(f, "Emoji_Modifier"),
            UnicodeClassNames::EmojiPresentation => write!(f, "Emoji_Presentation"),
            UnicodeClassNames::Emoji => write!(f, "Emoji"),
            UnicodeClassNames::ExtendedPictographic => write!(f, "Extended_Pictographic"),
            UnicodeClassNames::Extender => write!(f, "Extender"),
            UnicodeClassNames::FullCompositionExclusion => write!(f, "Full_Composition_Exclusion"),
            UnicodeClassNames::GraphemeExtend => write!(f, "Grapheme_Extend"),
            UnicodeClassNames::HexDigit => write!(f, "Hex_Digit"),
            UnicodeClassNames::Hyphen => write!(f, "Hyphen"),
            UnicodeClassNames::IdContinue => write!(f, "ID_Continue"),
            UnicodeClassNames::IdStart => write!(f, "ID_Start"),
            UnicodeClassNames::Ideographic => write!(f, "Ideographic"),
            UnicodeClassNames::IdsBinaryOperator => write!(f, "IDS_Binary_Operator"),
            UnicodeClassNames::IdsTrinaryOperator => write!(f, "IDS_Trinary_Operator"),
            UnicodeClassNames::JoinControl => write!(f, "Join_Control"),
            UnicodeClassNames::LogicalOrderException => write!(f, "Logical_Order_Exception"),
            UnicodeClassNames::Lowercase => write!(f, "Lowercase"),
            UnicodeClassNames::Math => write!(f, "Math"),
            UnicodeClassNames::NoncharacterCodePoint => write!(f, "Noncharacter_Code_Point"),
            UnicodeClassNames::OtherAlphabetic => write!(f, "Other_Alphabetic"),
            UnicodeClassNames::OtherDefaultIgnorableCodePoint => {
                write!(f, "Other_Default_Ignorable_Code_Point")
            }
            UnicodeClassNames::OtherGraphemeExtend => write!(f, "Other_Grapheme_Extend"),
            UnicodeClassNames::OtherIdContinue => write!(f, "Other_ID_Continue"),
            UnicodeClassNames::OtherIdStart => write!(f, "Other_ID_Start"),
            UnicodeClassNames::OtherLowercase => write!(f, "Other_Lowercase"),
            UnicodeClassNames::OtherMath => write!(f, "Other_Math"),
            UnicodeClassNames::OtherUppercase => write!(f, "Other_Uppercase"),
            UnicodeClassNames::PatternSyntax => write!(f, "Pattern_Syntax"),
            UnicodeClassNames::PatternWhiteSpace => write!(f, "Pattern_White_Space"),
            UnicodeClassNames::PrependedConcatenationMark => {
                write!(f, "Prepended_Concatenation_Mark")
            }
            UnicodeClassNames::QuotationMark => write!(f, "Quotation_Mark"),
            UnicodeClassNames::Radical => write!(f, "Radical"),
            UnicodeClassNames::RegionalIndicator => write!(f, "Regional_Indicator"),
            UnicodeClassNames::SentenceTerminal => write!(f, "Sentence_Terminal"),
            UnicodeClassNames::SoftDotted => write!(f, "Soft_Dotted"),
            UnicodeClassNames::TerminalPunctuation => write!(f, "Terminal_Punctuation"),
            UnicodeClassNames::UnifiedIdeograph => write!(f, "Unified_Ideograph"),
            UnicodeClassNames::Uppercase => write!(f, "Uppercase"),
            UnicodeClassNames::VariationSelector => write!(f, "Variation_Selector"),
            UnicodeClassNames::WhiteSpace => write!(f, "White_Space"),
            UnicodeClassNames::XidContinue => write!(f, "XID_Continue"),
            UnicodeClassNames::XidStart => write!(f, "XID_Start"),
        }
    }
}

impl std::fmt::Display for UnicodeClassOneLetter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnicodeClassOneLetter::L => write!(f, "L"),
            UnicodeClassOneLetter::N => write!(f, "N"),
            UnicodeClassOneLetter::Z => write!(f, "Z"),
            UnicodeClassOneLetter::P => write!(f, "P"),
            UnicodeClassOneLetter::C => write!(f, "C"),
        }
    }
}

fn try_from_class_set(class_set: &ClassSet, negated: bool) -> Result<CharMatcher> {
    Ok(match class_set {
        ClassSet::Item(class_set_item) => try_from_class_set_item(class_set_item, negated)?,
        ClassSet::BinaryOp(class_set_binary_op) => {
            let kind = class_set_binary_op.kind;
            if !negated {
                CharMatcher::ClassBinaryOperator(
                    kind,
                    Box::new(try_from_class_set(&class_set_binary_op.lhs, false)?),
                    Box::new(try_from_class_set(&class_set_binary_op.rhs, false)?),
                )
            } else {
                CharMatcher::NegatedClassBinaryOperator(
                    kind,
                    Box::new(try_from_class_set(&class_set_binary_op.lhs, false)?),
                    Box::new(try_from_class_set(&class_set_binary_op.rhs, false)?),
                )
            }
        }
    })
}

fn try_from_class_set_item(class_set_item: &ClassSetItem, negated: bool) -> Result<CharMatcher> {
    Ok(match class_set_item {
        ClassSetItem::Empty(_) => {
            if !negated {
                CharMatcher::ClassSetItemEmpty
            } else {
                CharMatcher::NegatedClassSetItemEmpty
            }
        }
        ClassSetItem::Literal(literal) => {
            // A literal AST matches a single character.
            let c = literal.c;
            if !negated {
                if c == '.' && literal.kind == LiteralKind::Verbatim {
                    CharMatcher::Dot
                } else {
                    CharMatcher::Literal(c)
                }
            } else if c == '.' && literal.kind == LiteralKind::Verbatim {
                CharMatcher::NegatedDot
            } else {
                CharMatcher::NegatedLiteral(c)
            }
        }
        ClassSetItem::Range(class_set_range) => {
            let start = class_set_range.start.c;
            let end = class_set_range.end.c;
            if !negated {
                CharMatcher::ClassSetItemRange(start, end)
            } else {
                CharMatcher::NegatedClassSetItemRange(start, end)
            }
        }
        ClassSetItem::Ascii(class_ascii) => {
            let negated = class_ascii.negated ^ negated;
            if !negated {
                CharMatcher::SetItemAscii(class_ascii.kind.clone())
            } else {
                CharMatcher::NegatedSetItemAscii(class_ascii.kind.clone())
            }
        }
        ClassSetItem::Unicode(unicode) => try_from_unicode_class(unicode)?,
        ClassSetItem::Perl(class_perl) => from_perl_class(class_perl),
        ClassSetItem::Bracketed(class_bracketed) => {
            try_from_class_set(&class_bracketed.kind, class_bracketed.negated)?
        }
        ClassSetItem::Union(class_set_union) => {
            let matchers = class_set_union
                .items
                .iter()
                .map(|item| try_from_class_set_item(item, false))
                .collect::<Result<Vec<CharMatcher>>>()?;
            if !negated {
                CharMatcher::Union(matchers)
            } else {
                CharMatcher::NegatedUnion(matchers)
            }
        }
    })
}

fn from_perl_class(perl: &ClassPerl) -> CharMatcher {
    let kind = perl.kind.clone();
    if !perl.negated {
        CharMatcher::ClassPerl(kind)
    } else {
        CharMatcher::NegatedClassPerl(kind)
    }
}

fn try_from_unicode_class(unicode: &ClassUnicode) -> Result<CharMatcher> {
    Ok(match &unicode.kind {
        ClassUnicodeKind::Named(name) => {
            let class_name = match name.as_str() {
                "Alphabetic" => UnicodeClassNames::Alphabetic,
                "ASCII_Hex_Digit" => UnicodeClassNames::AsciiHexDigit,
                "Bidi_Control" => UnicodeClassNames::BidiControl,
                "Case_Ignorable" => UnicodeClassNames::CaseIgnorable,
                "Cased" => UnicodeClassNames::Cased,
                "Composition_Exclusion" => UnicodeClassNames::CompositionExclusion,
                "Dash" => UnicodeClassNames::Dash,
                "Default_Ignorable_Code_Point" => UnicodeClassNames::DefaultIgnorableCodePoint,
                "Deprecated" => UnicodeClassNames::Deprecated,
                "Diacritic" => UnicodeClassNames::Diacritic,
                "Emoji_Component" => UnicodeClassNames::EmojiComponent,
                "Emoji_Modifier_Base" => UnicodeClassNames::EmojiModifierBase,
                "Emoji_Modifier" => UnicodeClassNames::EmojiModifier,
                "Emoji_Presentation" => UnicodeClassNames::EmojiPresentation,
                "Emoji" => UnicodeClassNames::Emoji,
                "Extended_Pictographic" => UnicodeClassNames::ExtendedPictographic,
                "Extender" => UnicodeClassNames::Extender,
                "Full_Composition_Exclusion" => UnicodeClassNames::FullCompositionExclusion,
                "Grapheme_Extend" => UnicodeClassNames::GraphemeExtend,
                "Hex_Digit" => UnicodeClassNames::HexDigit,
                "Hyphen" => UnicodeClassNames::Hyphen,
                "ID_Continue" => UnicodeClassNames::IdContinue,
                "ID_Start" => UnicodeClassNames::IdStart,
                "Ideographic" => UnicodeClassNames::Ideographic,
                "IDS_Binary_Operator" => UnicodeClassNames::IdsBinaryOperator,
                "IDS_Trinary_Operator" => UnicodeClassNames::IdsTrinaryOperator,
                "Join_Control" => UnicodeClassNames::JoinControl,
                "Logical_Order_Exception" => UnicodeClassNames::LogicalOrderException,
                "Lowercase" => UnicodeClassNames::Lowercase,
                "Math" => UnicodeClassNames::Math,
                "Noncharacter_Code_Point" => UnicodeClassNames::NoncharacterCodePoint,
                "Other_Alphabetic" => UnicodeClassNames::OtherAlphabetic,
                "Other_Default_Ignorable_Code_Point" => {
                    UnicodeClassNames::OtherDefaultIgnorableCodePoint
                }
                "Other_Grapheme_Extend" => UnicodeClassNames::OtherGraphemeExtend,
                "Other_ID_Continue" => UnicodeClassNames::OtherIdContinue,
                "Other_ID_Start" => UnicodeClassNames::OtherIdStart,
                "Other_Lowercase" => UnicodeClassNames::OtherLowercase,
                "Other_Math" => UnicodeClassNames::OtherMath,
                "Other_Uppercase" => UnicodeClassNames::OtherUppercase,
                "Pattern_Syntax" => UnicodeClassNames::PatternSyntax,
                "Pattern_White_Space" => UnicodeClassNames::PatternWhiteSpace,
                "Prepended_Concatenation_Mark" => UnicodeClassNames::PrependedConcatenationMark,
                "Quotation_Mark" => UnicodeClassNames::QuotationMark,
                "Radical" => UnicodeClassNames::Radical,
                "Regional_Indicator" => UnicodeClassNames::RegionalIndicator,
                "Sentence_Terminal" => UnicodeClassNames::SentenceTerminal,
                "Soft_Dotted" => UnicodeClassNames::SoftDotted,
                "Terminal_Punctuation" => UnicodeClassNames::TerminalPunctuation,
                "Unified_Ideograph" => UnicodeClassNames::UnifiedIdeograph,
                "Uppercase" => UnicodeClassNames::Uppercase,
                "Variation_Selector" => UnicodeClassNames::VariationSelector,
                "White_Space" => UnicodeClassNames::WhiteSpace,
                "XID_Continue" => UnicodeClassNames::XidContinue,
                "XID_Start" => UnicodeClassNames::XidStart,
                _ => return Err(unsupported!(format!("Unicode named class {}", name))),
            };
            if !unicode.is_negated() {
                CharMatcher::NamedUnicodeClass(class_name)
            } else {
                CharMatcher::NegatedNamedUnicodeClass(class_name)
            }
        }
        ClassUnicodeKind::OneLetter(ch) => {
            let letter = match ch {
                'L' => UnicodeClassOneLetter::L,
                'N' => UnicodeClassOneLetter::N,
                'Z' => UnicodeClassOneLetter::Z,
                'P' => UnicodeClassOneLetter::P,
                'C' => UnicodeClassOneLetter::C,
                _ => return Err(unsupported!(format!("Unicode class one letter {}", ch))),
            };
            if !unicode.is_negated() {
                CharMatcher::UnicodeClassOneLetter(letter)
            } else {
                CharMatcher::NegatedUnicodeClassOneLetter(letter)
            }
        }
        ClassUnicodeKind::NamedValue { name, value, .. } => {
            return Err(unsupported!(format!("Named value {}={}", name, value)));
        }
    })
}

#[inline(always)]
fn match_class_set_binary_operation(
    c: char,
    class_set_binary_op_kind: &ClassSetBinaryOpKind,
    char_matcher: &CharMatcher,
    char_matcher1: &CharMatcher,
) -> bool {
    match class_set_binary_op_kind {
        ClassSetBinaryOpKind::Intersection => char_matcher.matches(c) && char_matcher1.matches(c),
        ClassSetBinaryOpKind::Difference => char_matcher.matches(c) && !char_matcher1.matches(c),
        ClassSetBinaryOpKind::SymmetricDifference => {
            (char_matcher.matches(c) && !char_matcher1.matches(c))
                || (!char_matcher.matches(c) && char_matcher1.matches(c))
        }
    }
}

#[inline(always)]
fn match_perl_class(c: char, kind: &ClassPerlKind) -> bool {
    match kind {
        ClassPerlKind::Digit => c.is_numeric(),
        ClassPerlKind::Space => c.is_whitespace(),
        ClassPerlKind::Word => {
            c.is_alphanumeric() || c.join_c() || c.gc() == Gc::Pc || c.gc() == Gc::Mn
        }
    }
}

#[inline(always)]
fn match_ascii_class(c: char, kind: &ClassAsciiKind) -> bool {
    match kind {
        ClassAsciiKind::Alnum => c.is_alphanumeric(),
        ClassAsciiKind::Alpha => c.is_alphabetic(),
        ClassAsciiKind::Ascii => c.is_ascii(),
        ClassAsciiKind::Blank => c.is_ascii_whitespace(),
        ClassAsciiKind::Cntrl => c.is_ascii_control(),
        ClassAsciiKind::Digit => c.is_numeric(),
        ClassAsciiKind::Graph => c.is_ascii_graphic(),
        ClassAsciiKind::Lower => c.is_lowercase(),
        ClassAsciiKind::Print => c.is_ascii_graphic(),
        ClassAsciiKind::Punct => c.is_ascii_punctuation(),
        ClassAsciiKind::Space => c.is_whitespace(),
        ClassAsciiKind::Upper => c.is_uppercase(),
        ClassAsciiKind::Word => {
            c.is_alphanumeric() || c.join_c() || c.gc() == Gc::Pc || c.gc() == Gc::Mn
        }
        ClassAsciiKind::Xdigit => c.is_ascii_hexdigit(),
    }
}

#[inline(always)]
fn match_letter_unicode_class(c: char, ch: &UnicodeClassOneLetter) -> bool {
    match ch {
        // Unicode class for Letters
        UnicodeClassOneLetter::L => c.alpha(),
        // Unicode class for Numbers
        UnicodeClassOneLetter::N => c.is_numeric(),
        // Unicode class for Whitespace
        UnicodeClassOneLetter::Z => c.is_whitespace(),
        // Unicode class Terminal_Punctuation
        UnicodeClassOneLetter::P => c.term(),
        // Unicode class for Control characters
        UnicodeClassOneLetter::C => c.is_control(),
    }
}

#[inline(always)]
fn match_negated_letter_unicode_class(c: char, ch: &UnicodeClassOneLetter) -> bool {
    match ch {
        // Unicode class for Letters
        UnicodeClassOneLetter::L => !c.alpha(),
        // Unicode class for Numbers
        UnicodeClassOneLetter::N => !c.is_numeric(),
        // Unicode class for Whitespace
        UnicodeClassOneLetter::Z => !c.is_whitespace(),
        // Unicode class Terminal_Punctuation
        UnicodeClassOneLetter::P => !c.term(),
        // Unicode class for Control characters
        UnicodeClassOneLetter::C => !c.is_control(),
    }
}

#[inline(always)]
fn match_unicode_class(c: char, name: &UnicodeClassNames) -> bool {
    match *name {
        UnicodeClassNames::Alphabetic => c.alpha(),
        UnicodeClassNames::AsciiHexDigit => c.ahex(),
        UnicodeClassNames::BidiControl => c.bidi_c(),
        UnicodeClassNames::CaseIgnorable => c.ci(),
        UnicodeClassNames::Cased => c.cased(),
        UnicodeClassNames::CompositionExclusion => c.ce(),
        UnicodeClassNames::Dash => c.dash(),
        UnicodeClassNames::DefaultIgnorableCodePoint => c.di(),
        UnicodeClassNames::Deprecated => c.dep(),
        UnicodeClassNames::Diacritic => c.dia(),
        UnicodeClassNames::EmojiComponent => c.ecomp(),
        UnicodeClassNames::EmojiModifierBase => c.ebase(),
        UnicodeClassNames::EmojiModifier => c.emod(),
        UnicodeClassNames::EmojiPresentation => c.epres(),
        UnicodeClassNames::Emoji => c.emoji(),
        UnicodeClassNames::ExtendedPictographic => c.ext_pict(),
        UnicodeClassNames::Extender => c.ext(),
        UnicodeClassNames::FullCompositionExclusion => c.comp_ex(),
        UnicodeClassNames::GraphemeExtend => c.gr_ext(),
        UnicodeClassNames::HexDigit => c.hex(),
        UnicodeClassNames::Hyphen => c.hyphen(),
        UnicodeClassNames::IdContinue => c.idc(),
        UnicodeClassNames::IdStart => c.ids(),
        UnicodeClassNames::Ideographic => c.ideo(),
        UnicodeClassNames::IdsBinaryOperator => c.idsb(),
        UnicodeClassNames::IdsTrinaryOperator => c.idst(),
        UnicodeClassNames::JoinControl => c.join_c(),
        UnicodeClassNames::LogicalOrderException => c.loe(),
        UnicodeClassNames::Lowercase => c.lower(),
        UnicodeClassNames::Math => c.math(),
        UnicodeClassNames::NoncharacterCodePoint => c.nchar(),
        UnicodeClassNames::OtherAlphabetic => c.oalpha(),
        UnicodeClassNames::OtherDefaultIgnorableCodePoint => c.odi(),
        UnicodeClassNames::OtherGraphemeExtend => c.ogr_ext(),
        UnicodeClassNames::OtherIdContinue => c.oidc(),
        UnicodeClassNames::OtherIdStart => c.oids(),
        UnicodeClassNames::OtherLowercase => c.olower(),
        UnicodeClassNames::OtherMath => c.omath(),
        UnicodeClassNames::OtherUppercase => c.oupper(),
        UnicodeClassNames::PatternSyntax => c.pat_syn(),
        UnicodeClassNames::PatternWhiteSpace => c.pat_ws(),
        UnicodeClassNames::PrependedConcatenationMark => c.pcm(),
        UnicodeClassNames::QuotationMark => c.qmark(),
        UnicodeClassNames::Radical => c.radical(),
        UnicodeClassNames::RegionalIndicator => c.ri(),
        UnicodeClassNames::SentenceTerminal => c.sterm(),
        UnicodeClassNames::SoftDotted => c.sd(),
        UnicodeClassNames::TerminalPunctuation => c.term(),
        UnicodeClassNames::UnifiedIdeograph => c.uideo(),
        UnicodeClassNames::Uppercase => c.upper(),
        UnicodeClassNames::VariationSelector => c.vs(),
        UnicodeClassNames::WhiteSpace => c.wspace(),
        UnicodeClassNames::XidContinue => c.xidc(),
        UnicodeClassNames::XidStart => c.xids(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex_syntax::ast::parse::Parser;

    #[test]
    fn test_match_function_unicode_class() {
        let ast = Parser::new().parse(r"\pL").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(char_matcher.matches('a'));
        assert!(char_matcher.matches('A'));
        assert!(!char_matcher.matches('1'));
        assert!(!char_matcher.matches(' '));
    }

    #[test]
    fn test_match_function_perl_class() {
        let ast = Parser::new().parse(r"\d").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(char_matcher.matches('1'));
        assert!(!char_matcher.matches('a'));
    }

    #[test]
    fn test_match_function_bracketed_class() {
        let ast = Parser::new().parse(r"[a-z]").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(char_matcher.matches('a'));
        assert!(char_matcher.matches('z'));
        assert!(!char_matcher.matches('A'));
        assert!(!char_matcher.matches('1'));
    }

    #[test]
    fn test_match_function_binary_op_class_intersection() {
        // Intersection (matching x or y)
        let ast = Parser::new().parse(r"[a-y&&xyz]").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(char_matcher.matches('x'));
        assert!(char_matcher.matches('y'));
        assert!(!char_matcher.matches('a'));
        assert!(!char_matcher.matches('z'));
    }

    #[test]
    fn test_match_function_union_class() {
        let ast = Parser::new().parse(r"[0-9a-z]").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(char_matcher.matches('a'));
        assert!(char_matcher.matches('z'));
        assert!(char_matcher.matches('0'));
        assert!(char_matcher.matches('9'));
        assert!(!char_matcher.matches('!'));
    }

    #[test]
    fn test_match_function_negated_bracketed_class() {
        let ast = Parser::new().parse(r"[^a-z]").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(!char_matcher.matches('a'));
        assert!(!char_matcher.matches('z'));
        assert!(char_matcher.matches('A'));
        assert!(char_matcher.matches('1'));
    }

    #[test]
    fn test_match_function_negated_binary_op_class() {
        let ast = Parser::new().parse(r"[a-z&&[^aeiou]]").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(!char_matcher.matches('a'));
        assert!(!char_matcher.matches('e'));
        assert!(char_matcher.matches('z'));
        assert!(!char_matcher.matches('1'));
    }

    // [[:alpha:]]   ASCII character class ([A-Za-z])
    #[test]
    fn test_match_function_ascci_class() {
        let ast = Parser::new().parse(r"[[:alpha:]]").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(char_matcher.matches('a'));
        assert!(char_matcher.matches('A'));
        assert!(char_matcher.matches('ä'));
        assert!(!char_matcher.matches('1'));
        assert!(!char_matcher.matches(' '));
    }

    // [^[:alpha:]]  Negated ASCII character class ([^A-Za-z])
    #[test]
    fn test_match_function_negated_ascii_class() {
        let ast = Parser::new().parse(r"[^[:alpha:]]").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        println!("{}", char_matcher);
        assert!(!char_matcher.matches('a'));
        assert!(!char_matcher.matches('A'));
        assert!(!char_matcher.matches('ä'));
        assert!(char_matcher.matches('1'));
        assert!(char_matcher.matches(' '));
    }

    // [[:^alpha:]]  Negated ASCII character class ([^A-Za-z])
    #[test]
    fn test_match_function_negated_ascii_class_2() {
        let ast = Parser::new().parse(r"[[:^alpha:]]").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(!char_matcher.matches('a'));
        assert!(!char_matcher.matches('A'));
        assert!(!char_matcher.matches('ä'));
        assert!(char_matcher.matches('1'));
        assert!(char_matcher.matches(' '));
    }

    #[test]
    fn test_match_function_empty() {
        let ast = Parser::new().parse(r"").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(char_matcher.matches('a'));
        assert!(char_matcher.matches('A'));
        assert!(char_matcher.matches('1'));
        assert!(char_matcher.matches(' '));
    }

    // [x[^xyz]]     Nested/grouping character class (matching any character except y and z)
    #[test]
    fn test_nested_classes() {
        let ast = Parser::new().parse(r"[x[^xyz]]").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(char_matcher.matches('a'));
        assert!(char_matcher.matches('x'));
        assert!(char_matcher.matches('1'));
        assert!(char_matcher.matches(' '));
        assert!(!char_matcher.matches('y'));
        assert!(!char_matcher.matches('z'));
    }

    // [0-9&&[^4]]   Subtraction using intersection and negation (matching 0-9 except 4)
    #[test]
    fn test_subtraction() {
        let ast = Parser::new().parse(r"[0-9&&[^4]]").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(char_matcher.matches('0'));
        assert!(char_matcher.matches('9'));
        assert!(!char_matcher.matches('4'));
        assert!(!char_matcher.matches('a'));
    }

    // [0-9--4]      Direct subtraction (matching 0-9 except 4)
    #[test]
    fn test_direct_subtraction() {
        let ast = Parser::new().parse(r"[0-9--4]").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(char_matcher.matches('0'));
        assert!(char_matcher.matches('9'));
        assert!(!char_matcher.matches('4'));
        assert!(!char_matcher.matches('a'));
    }

    // [a-g~~b-h]    Symmetric difference (matching `a` and `h` only)
    #[test]
    fn test_symmetric_difference() {
        let ast = Parser::new().parse(r"[a-g~~b-h]").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(char_matcher.matches('a'));
        assert!(char_matcher.matches('h'));
        assert!(!char_matcher.matches('b'));
        assert!(!char_matcher.matches('z'));
    }

    // [\[\]]        Escaping in character classes (matching [ or ])
    #[test]
    fn test_escaping() {
        let ast = Parser::new().parse(r"[\[\]]").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(char_matcher.matches('['));
        assert!(char_matcher.matches(']'));
        assert!(!char_matcher.matches('a'));
        assert!(!char_matcher.matches('1'));
    }

    // [a&&b]        An empty character class matching nothing
    #[test]
    fn test_empty_intersection() {
        let ast = Parser::new().parse(r"[a&&b]").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(!char_matcher.matches('a'));
        assert!(!char_matcher.matches('b'));
        assert!(!char_matcher.matches('1'));
        assert!(!char_matcher.matches(' '));
    }

    // [^a--b]       A negated subtraction character class (matching b and everything but a)
    //               This is equivalent to [^a]
    #[test]
    fn test_negated_subtraction() {
        let ast = Parser::new().parse(r"[^a--b]").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(!char_matcher.matches('a'));
        assert!(char_matcher.matches('b'));
        assert!(char_matcher.matches('1'));
        assert!(char_matcher.matches(' '));
    }

    // \p{XID_Start} and \p{XID_Continue} are supported by unicode_xid
    #[test]
    fn test_named_classes() {
        let ast = Parser::new().parse(r"\p{XID_Start}").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(char_matcher.matches('a'));
        assert!(char_matcher.matches('A'));
        assert!(!char_matcher.matches('1'));
        assert!(!char_matcher.matches(' '));

        let ast = Parser::new().parse(r"\p{XID_Continue}").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(char_matcher.matches('a'));
        assert!(char_matcher.matches('A'));
        assert!(char_matcher.matches('1'));
        assert!(!char_matcher.matches(' '));
    }

    #[test]
    fn test_evaluate_general_category() {
        assert_eq!('_'.gc(), Gc::Pc);
        let ast = Parser::new().parse(r"\w").unwrap();
        let char_matcher = CharMatcher::try_from(&ast).unwrap();
        assert!(char_matcher.matches('_'));
    }
}
