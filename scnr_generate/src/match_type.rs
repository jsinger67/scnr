#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Position;

use super::Span;

/// A match in the haystack.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Match {
    /// The token type number associated with the match.
    token_type: usize,
    /// The underlying match span.
    span: Span,
}

impl Match {
    /// Create a new match.
    pub fn new(token_type: usize, span: Span) -> Self {
        Self { token_type, span }
    }

    /// Get the start of the match.
    #[inline]
    pub fn start(&self) -> usize {
        self.span.start
    }

    /// Get the end of the match.
    #[inline]
    pub fn end(&self) -> usize {
        self.span.end
    }

    /// Get the span of the match.
    #[inline]
    pub fn span(&self) -> Span {
        self.span
    }

    /// Get the span as range
    #[inline]
    pub fn range(&self) -> std::ops::Range<usize> {
        self.span.range()
    }

    /// Get the length of the match.
    #[inline]
    pub fn len(&self) -> usize {
        self.span.len()
    }

    /// Check if the match is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.span.is_empty()
    }

    /// Get the token type of the match.
    #[inline]
    pub fn token_type(&self) -> usize {
        self.token_type
    }

    /// Add an offset to the match.
    /// This is used to adjust the match position after the scanner has been reset.
    pub fn add_offset(&mut self, offset: usize) {
        self.span.start += offset;
        self.span.end += offset;
    }
}

/// A match with line and column information for start and end positions.
///
/// You can create a `MatchExt` iterator from a `Match` iterator by calling the `with_positions`
/// method on the iterator.
///
/// ```ignore
/// let find_iter = scanner.find_iter(INPUT).with_positions();
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MatchExt {
    /// The token type number associated with the match.
    token_type: usize,
    /// The underlying match span.
    span: Span,
    /// The position of the start of the match.
    start_position: Position,
    /// The position of the end of the match.
    /// The end position is exclusive.
    end_position: Position,
}

impl MatchExt {
    pub fn new(
        token_type: usize,
        span: Span,
        start_position: Position,
        end_position: Position,
    ) -> Self {
        Self {
            token_type,
            span,
            start_position,
            end_position,
        }
    }

    /// Get the start of the match.
    #[inline]
    pub fn start(&self) -> usize {
        self.span.start
    }

    /// Get the end of the match.
    #[inline]
    pub fn end(&self) -> usize {
        self.span.end
    }

    /// Get the span of the match.
    #[inline]
    pub fn span(&self) -> Span {
        self.span
    }

    /// Get the span as range
    #[inline]
    pub fn range(&self) -> std::ops::Range<usize> {
        self.span.range()
    }

    /// Get the length of the match.
    #[inline]
    pub fn len(&self) -> usize {
        self.span.len()
    }

    /// Check if the match is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.span.is_empty()
    }

    /// Get the token type of the match.
    #[inline]
    pub fn token_type(&self) -> usize {
        self.token_type
    }

    /// Get the start position of the match.
    #[inline]
    pub fn start_position(&self) -> Position {
        self.start_position
    }

    /// Get the end position of the match.
    #[inline]
    pub fn end_position(&self) -> Position {
        self.end_position
    }
}
