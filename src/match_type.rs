use super::Span;

/// A match in the haystack.
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
}
