//! Module with the pattern types and their methods.
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
