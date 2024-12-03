//! Module with the position type and functions.
//! A position is a struct that contains a line and column number.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A position in the haystack.
/// The position is represented by a line and column number.
/// The line and column numbers are 1-based.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Position {
    /// The line number of the position.
    pub line: usize,
    /// The column number of the position.
    pub column: usize,
}

impl Position {
    /// Create a new position.
    pub fn new(line: usize, column: usize) -> Self {
        debug_assert!(line > 0, "line number must be greater than 0");
        debug_assert!(column > 0, "column number must be greater than 0");
        Self { line, column }
    }

    /// Get the line number of the position.
    #[inline]
    pub fn line(&self) -> usize {
        self.line
    }

    /// Get the column number of the position.
    #[inline]
    pub fn column(&self) -> usize {
        self.column
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line: {}, column: {}", self.line, self.column)
    }
}

/// A trait for providing the line and column information of a given byte offset in the haystack.
/// It also provides a method to set the offset of the char indices iterator.
pub trait PositionProvider {
    /// Returns the position of the given offset.
    fn position(&self, offset: usize) -> Position;

    /// Set the position of the char indices iterator to the given offset. Use this to let the
    /// iterator start at a specific offset.
    fn set_offset(&mut self, offset: usize);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position() {
        let pos = Position::new(1, 1);
        assert_eq!(pos.line(), 1);
        assert_eq!(pos.column(), 1);
        assert_eq!(format!("{}", pos), "line: 1, column: 1");
    }
}
