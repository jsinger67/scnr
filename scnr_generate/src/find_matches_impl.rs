use log::trace;

use super::ScannerImpl;
use crate::scanner_mode_switcher::ScannerModeSwitcher;
use crate::{Match, PeekResult, Position};

/// An iterator over all non-overlapping matches.
pub struct FindMatchesImpl<'h> {
    // The scanner used to find matches.
    scanner_impl: ScannerImpl,
    // The input haystack.
    input: &'h str,
    // The char_indices iterator of the input haystack.
    char_indices: std::str::CharIndices<'h>,
    // The last position of the char_indices iterator.
    last_position: usize,
    // The last iterated character.
    last_char: char,
    // The vector of offsets of the char_indices iterator that mark the start of a line.
    // It is used to calculate line and column numbers of offsets.
    // The line number is the index of the line offset in the vector plus one.
    line_offsets: Vec<usize>,
    // The offset of the char_indices iterator in bytes.
    // It is used to calculate the start position of each match.
    offset: usize,
}

impl<'h> FindMatchesImpl<'h> {
    /// Creates a new `FindMatches` iterator.
    pub fn new(scanner_impl: ScannerImpl, input: &'h str) -> Self {
        let mut me = Self {
            scanner_impl,
            input,
            char_indices: input.char_indices(),
            last_position: 0,
            last_char: '\0',
            line_offsets: vec![0],
            offset: 0,
        };
        me.scanner_impl.reset();
        me
    }

    /// Sets an offset to the current position of the scanner. This offset is added to the start
    /// position of each match.
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.set_offset(offset);
        self
    }

    /// Set the offset of the char indices iterator to the given position on the current instance.
    /// The function is used to set the position of the char_indices iterator to the given position.
    pub fn set_offset(&mut self, offset: usize) {
        trace!("Set offset to {}", offset);
        if offset <= self.input.len() {
            // Split the input a byte position `offset` and create a new char_indices iterator.
            self.char_indices = self.input[offset..].char_indices();
        } else {
            // The position is greater than the length of the haystack.
            // Take an empty slice after the haystack to create an empty char_indices iterator.
            self.char_indices = self.input[self.input.len()..self.input.len()].char_indices();
        }
        self.last_position = 0;
        self.offset = offset;
    }

    /// Returns the next match in the haystack.
    ///
    /// If no match is found, `None` is returned.
    ///
    /// The function calls the `find_from` method of the scanner to find the next match.
    /// If a match is found, the function advances the char_indices iterator to the end of the match.
    /// If no match is found, the function repeatedly advances the char_indices iterator by one
    /// and tries again until a match is found or the iterator is exhausted.
    #[inline]
    pub fn next_match(&mut self) -> Option<Match> {
        let mut result;
        trace!("Find next match from offset {}", self.offset);
        loop {
            result = self
                .scanner_impl
                .find_from(self.input, self.char_indices.clone());
            if let Some(mut matched) = result {
                self.advance_beyond_match(matched);
                matched.add_offset(self.offset);
                return Some(matched);
            } else if let Some((i, c)) = self.char_indices.next() {
                self.record_line_offset(i + self.offset, c);
            } else {
                // The iterator is exhausted.
                // We should update the line offsets with the last character of the haystack.
                self.record_line_offset(self.last_position + self.offset, '\0');
                break;
            }
        }
        result
    }

    /// Peeks n matches ahead without consuming the matches.
    /// The function returns [PeekResult].
    ///
    /// The peek operation always stops at the end of the haystack or when a mode switch is
    /// triggered by the last match. The mode switch is not conducted by the peek operation to not
    /// change the state of the scanner as well as to avoid a mix of tokens from different modes
    /// being returned.
    pub fn peek_n(&mut self, n: usize) -> PeekResult {
        let mut char_indices = self.char_indices.clone();
        let mut matches = Vec::with_capacity(n);
        let mut mode_switch = false;
        let mut new_mode = 0;
        for _ in 0..n {
            let result = self
                .scanner_impl
                .peek_from(self.input, char_indices.clone());
            if let Some(mut matched) = result {
                let token_type = matched.token_type();
                Self::advance_char_indices_beyond_match(&mut char_indices, matched);
                matched.add_offset(self.offset);
                matches.push(matched);
                if let Some(mode) = self.scanner_impl.has_transition(token_type) {
                    mode_switch = true;
                    new_mode = mode;
                    break;
                }
            } else {
                break;
            }
        }
        if mode_switch {
            PeekResult::MatchesReachedModeSwitch((matches, new_mode))
        } else if matches.len() == n {
            PeekResult::Matches(matches)
        } else if matches.is_empty() {
            PeekResult::NotFound
        } else {
            PeekResult::MatchesReachedEnd(matches)
        }
    }

    // Advance the char_indices iterator to the end of the match.
    #[inline]
    fn advance_beyond_match(&mut self, matched: Match) {
        if matched.is_empty() {
            return;
        }
        let end = matched.span().end;
        self.advance_to(end);
    }

    /// Advances the given char_indices iterator to the end of the given match.
    fn advance_char_indices_beyond_match(char_indices: &mut std::str::CharIndices, matched: Match) {
        if matched.is_empty() {
            return;
        }
        let end = matched.span().end;
        for (i, c) in char_indices {
            if i + c.len_utf8() >= end {
                // Stop at the end of the match.
                break;
            }
        }
    }

    /// Advance the char_indices iterator to the given position.
    /// The function is used to skip a given number of characters in the haystack.
    /// It can be used after a peek operation to skip the characters of the peeked matches.
    /// The function returns the new position of the char_indices iterator.
    /// If the new position is greater than the length of the haystack, the function returns the
    /// length of the haystack.
    /// If the new position is less than the current position of the char_indices iterator, the
    /// function returns the current position of the char_indices iterator.
    pub fn advance_to(&mut self, position: usize) -> usize {
        if position < self.last_position {
            // The new position is less than the current position of the char_indices iterator.
            // The iterator is advanced by one character and the next character is not returned by
            // the iterator.
            return self.last_position;
        }
        let mut new_position = 0;
        let mut line_start_offsets = vec![];
        let mut last_char = self.last_char;
        for (i, c) in self.char_indices.by_ref() {
            if last_char == '\n' {
                line_start_offsets.push(i + self.offset);
            }
            last_char = c;
            new_position = i;
            if i + c.len_utf8() >= position {
                break;
            }
        }
        if !line_start_offsets.is_empty() {
            // Merge the line start offsets with the current line start offsets.
            // We need to merge them because the lines could be read multiple times due to
            // possible resets of the char_indices iterator (see `with_offset`).
            self.merge_line_offsets(line_start_offsets);
        }
        self.last_char = last_char;
        self.last_position = new_position;
        new_position
    }

    /// Retrieve the total offset of the char indices iterator in bytes.
    pub fn offset(&self) -> usize {
        self.last_position + self.offset
    }

    /// Returns the line and column numbers of the given offset.
    /// The line number is the index of the line offset in the vector plus one.
    /// The column number is actually the number of characters found from the line offset to the
    /// offset plus one. See note below for more about the inaccuracy of the column number.
    /// If the offset is greater than the length of the haystack, the function returns the last
    /// recorded line and the column number is calculated from the last recorded position.
    ///
    /// *Note:*
    ///
    /// This function simply calculates the column number by subtracting the last recorded line
    /// offset from the given offset. This is not always the correct column number, especially when
    /// the line contains characters with more then one byte length.
    /// This inaccuracy is accepted in favor of performance.
    pub fn position(&self, offset: usize) -> Position {
        match self.line_offsets.binary_search_by(|&x| x.cmp(&offset)) {
            Ok(i) => Position::new(i + 1, offset.saturating_sub(self.line_offsets[i]) + 1),
            Err(i) => Position::new(i, offset.saturating_sub(self.line_offsets[i - 1]) + 1),
        }
    }

    /// Records the offset of a line in the haystack.
    fn record_line_offset(&mut self, i: usize, c: char) {
        if self.last_char == '\n' {
            self.merge_line_offsets(vec![i]);
        }
        self.last_char = c;
    }

    /// Returns the current scanner mode. Used for tests and debugging purposes.
    #[allow(dead_code)]
    #[inline]
    pub fn current_mode(&self) -> usize {
        self.scanner_impl.current_mode()
    }

    pub fn set_mode(&mut self, mode: usize) {
        self.scanner_impl.set_mode(mode);
    }

    pub fn mode_name(&self, index: usize) -> Option<&str> {
        self.scanner_impl.mode_name(index)
    }

    /// Merges the given line start offsets with the current line start offsets.
    /// The function is used to merge the given line start offsets.
    /// Already existing line start offsets are not added to the vector.
    /// New line start offsets are added to the vector while maintaining the ascending order.
    fn merge_line_offsets(&mut self, line_start_offsets: Vec<usize>) {
        // The line offsets are always sorted in ascending order.
        debug_assert!(self.line_offsets.windows(2).all(|w| w[0] < w[1]));
        for offset in line_start_offsets {
            match self.line_offsets.binary_search(&offset) {
                Ok(_) => {}
                Err(i) => {
                    trace!("Insert line offset at index {}: {}", i, offset);
                    self.line_offsets.insert(i, offset)
                }
            }
        }
    }
}

impl std::fmt::Debug for FindMatchesImpl<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FindMatchesImpl").finish()
    }
}
