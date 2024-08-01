/// A span in a source file.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Span {
    /// The start offset of the span, inclusive.
    pub start: usize,
    /// The end offset of the span, exclusive.
    pub end: usize,
}
impl Span {
    /// Create a new span.
    #[inline]
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    /// Check if the span is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Get the length of the span.
    #[inline]
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Get the span as range.
    #[inline]
    pub fn range(self) -> std::ops::Range<usize> {
        self.start..self.end
    }
}

impl<T> From<std::ops::Range<T>> for Span
where
    T: Into<usize>,
{
    fn from(range: std::ops::Range<T>) -> Self {
        Span {
            start: range.start.into(),
            end: range.end.into(),
        }
    }
}

impl<T> From<std::ops::RangeInclusive<T>> for Span
where
    T: Into<usize> + Copy,
{
    fn from(range: std::ops::RangeInclusive<T>) -> Self {
        Span {
            start: (*range.start()).into(),
            end: (*range.end()).into() + 1,
        }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
