//! Module with the pattern types and their methods.
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

macro_rules! parse_ident {
    ($input:ident, $name:ident) => {
        $input.parse().map_err(|e| {
            syn::Error::new(
                e.span(),
                concat!("expected identifier `", stringify!($name), "`"),
            )
        })?
    };
}

/// A lookahead is a regular expression that restricts a match of a pattern so that it must be
/// matched after the pattern.
///
/// If the lookahead is negative, it must not be matched after the pattern.
///
/// With the help of a positive lookahead you can define a semantic like
/// ```text
/// match pattern R only if it is followed by pattern S
/// ```
/// On the other hand with a negative lookahead you can define a semantic like
/// ```text
/// match pattern R only if it is NOT followed by pattern S
/// ```
///
/// The lookahead patterns denoted above as `S` are not considered as part of the matched string.
///
/// The lookahead is an optional member of the [crate::Pattern] struct.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lookahead {
    /// If the lookahead is positive.
    pub is_positive: bool,
    /// The lookahead pattern.
    pub pattern: String,
}

impl Lookahead {
    /// Create a new lookahead.
    pub fn new(is_positive: bool, pattern: String) -> Self {
        Self {
            is_positive,
            pattern,
        }
    }

    /// Get the pattern.
    #[inline]
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Check if the lookahead is positive.
    #[inline]
    pub fn is_positive(&self) -> bool {
        self.is_positive
    }
}

impl std::fmt::Display for Lookahead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_positive {
            write!(f, "(?={})", self.pattern.escape_default())
        } else {
            write!(f, "(?!{})", self.pattern.escape_default())
        }
    }
}

/// This is used to create a lookahead from a part of a macro input.
/// The macro input looks like this:
/// ```text
/// with lookahead positive r"!";
/// ```
/// where the `positive` term can also be `negative`.
impl syn::parse::Parse for Lookahead {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let with: syn::Ident = parse_ident!(input, with);
        let with = with.to_string();
        if with != "with" {
            return Err(input.error("expected 'with'"));
        }
        let lookahead: syn::Ident = parse_ident!(input, lookahead);
        let lookahead = lookahead.to_string();
        if lookahead != "lookahead" {
            return Err(input.error("expected 'lookahead'"));
        }
        let pos_or_neg: syn::Ident = parse_ident!(input, pos_or_neg);
        let pos_or_neg = pos_or_neg.to_string();
        let is_positive = match pos_or_neg.as_str() {
            "positive" => true,
            "negative" => false,
            _ => return Err(input.error("expected 'positive' or 'negative'")),
        };
        let pattern: syn::LitStr = input.parse()?;
        let pattern = pattern.value();
        Ok(Lookahead::new(is_positive, pattern))
    }
}

/// A pattern that is used to match the input.
/// The pattern is represented by a regular expression and a token type number.
/// The token type number is used to identify the pattern in the scanner.
/// The pattern also has an optional [Lookahead].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Pattern {
    pattern: String,
    token_type: usize,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    lookahead: Option<Lookahead>,
}

impl Pattern {
    /// Create a new pattern.
    pub fn new(pattern: String, token_type: usize) -> Self {
        Self {
            pattern,
            token_type,
            lookahead: None,
        }
    }

    /// Set the token type of the pattern.
    pub fn set_token_type(&mut self, token_type: usize) {
        self.token_type = token_type;
    }

    /// Create a new pattern with lookahead.
    pub fn with_lookahead(self, lookahead: Lookahead) -> Self {
        Self {
            pattern: self.pattern,
            token_type: self.token_type,
            lookahead: Some(lookahead),
        }
    }

    /// Get the pattern.
    #[inline]
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Get the token type.
    #[inline]
    pub fn terminal_id(&self) -> usize {
        self.token_type
    }

    /// Get the lookahead.
    #[inline]
    pub fn lookahead(&self) -> Option<&Lookahead> {
        self.lookahead.as_ref()
    }
}

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pattern.escape_default())?;
        if let Some(lookahead) = &self.lookahead {
            write!(f, "{}", lookahead)?
        }
        Ok(())
    }
}

impl AsRef<str> for Pattern {
    fn as_ref(&self) -> &str {
        &self.pattern
    }
}

/// This is used to create a pattern from a part of a macro input.
/// The macro input looks like this:
/// ```text
/// token r"World" => 11 with lookahead positive r"!";
/// ```
/// where the lookahead part is optional. The `with lookahead` part should be parsed with the help
/// of the `Lookahead` struct's `parse` method.
///
/// Note that the `token` keyword is not part of the pattern, but it is used to identify the
/// pattern.
impl syn::parse::Parse for Pattern {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pattern: syn::LitStr = input.parse()?;
        let pattern = pattern.value();
        input.parse::<syn::Token![=>]>()?;
        let token_type: syn::LitInt = input.parse()?;
        let token_type = token_type.base10_parse()?;
        let mut pattern = Pattern::new(pattern, token_type);
        // Check if there is a lookahead and parse it.
        if input.peek(syn::Ident) {
            // The parse implementation of the Lookahead struct will check if the ident is `with`.
            // If it is not, it will return an error.
            let lookahead: Lookahead = input.parse()?;
            pattern = pattern.with_lookahead(lookahead);
        }
        // Parse the semicolon at the end of the pattern.
        if input.peek(syn::Token![;]) {
            input.parse::<syn::Token![;]>()?;
        } else {
            return Err(input.error("expected ';'"));
        }
        Ok(pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest]
    #[case::without_lookahead(
        // input
        quote::quote! {
            token r"Hello" => 0;
        },
        // expected_pattern
        "Hello",
        // expected_token_type
        0,
        // lookahead
        None)]
    #[case::with_positive_lookahead(
        // input
        quote::quote! {
            token r"Hello" => 1 with lookahead positive r"!";
        },
        // expected_pattern
        "Hello",
        // expected_token_type
        1,
        // lookahead
        Some(Lookahead { is_positive: true, pattern: "!".to_string() }),)]
    #[case::with_negative_lookahead(
        // input
        quote::quote! {
            token r#"""# => 8 with lookahead negative r#"\\[\"\\bfnt]"#;
        },
        // expected_pattern
        r#"""#,
        // expected_token_type
        8,
        // lookahead
        Some(Lookahead { is_positive: false, pattern: r#"\\[\"\\bfnt]"#.to_string() }),)]
    fn test_parse_pattern(
        #[case] input: proc_macro2::TokenStream,
        #[case] expected_pattern: &str,
        #[case] expected_token_type: usize,
        #[case] lookahead: Option<Lookahead>,
    ) {
        let pattern: Pattern = syn::parse2(input).unwrap();
        assert_eq!(pattern.pattern(), expected_pattern);
        assert_eq!(pattern.terminal_id(), expected_token_type);
        assert_eq!(pattern.lookahead(), lookahead.as_ref());
    }
}
