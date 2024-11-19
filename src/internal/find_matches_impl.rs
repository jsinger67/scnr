use crate::{scanner::ScannerImplTrait, Match, PeekResult, Position};

/// An iterator over all non-overlapping matches.
pub(crate) struct FindMatchesImpl<'h> {
    // The scanner used to find matches.
    scanner_impl: Box<dyn ScannerImplTrait>,
    // The input haystack.
    char_indices: std::str::CharIndices<'h>,
    // The last position of the char_indices iterator.
    last_position: usize,
    // The last iterated character.
    last_char: char,
    // The vector of offsets of the char_indices iterator that mark the start of a line.
    // It is used to calculate line and column numbers of offsets.
    // The line number is the index of the line offset in the vector plus one.
    line_offsets: Vec<usize>,
}

impl<'h> FindMatchesImpl<'h> {
    /// Creates a new `FindMatches` iterator.
    pub(crate) fn new(scanner_impl: Box<dyn ScannerImplTrait>, input: &'h str) -> Self {
        let mut me = Self {
            scanner_impl,
            char_indices: input.char_indices(),
            last_position: 0,
            last_char: '\0',
            line_offsets: vec![0],
        };
        me.scanner_impl.reset();
        me
    }

    pub(crate) fn with_offset(self, offset: usize) -> Self {
        let mut me = Self {
            scanner_impl: self.scanner_impl,
            char_indices: self.char_indices,
            last_position: self.last_position,
            last_char: self.last_char,
            line_offsets: self.line_offsets,
        };
        me.advance_to(offset);
        me
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
    pub(crate) fn next_match(&mut self) -> Option<Match> {
        let mut result;
        loop {
            result = self.scanner_impl.find_from(self.char_indices.clone());
            if let Some(matched) = result {
                self.advance_beyond_match(matched);
                break;
            } else if let Some((i, c)) = self.char_indices.next() {
                self.record_line_offset(i, c);
            } else {
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
    /// change the state of the scanner as well as to aviod a mix of tokens from different modes
    /// being returned.
    pub(crate) fn peek_n(&mut self, n: usize) -> PeekResult {
        let mut char_indices = self.char_indices.clone();
        let mut matches = Vec::with_capacity(n);
        let mut mode_switch = false;
        let mut new_mode = 0;
        for _ in 0..n {
            let result = self.scanner_impl.peek_from(char_indices.clone());
            if let Some(matched) = result {
                matches.push(matched);
                Self::advance_char_indices_beyond_match(&mut char_indices, matched);
                if let Some(mode) = self.scanner_impl.has_transition(matched.token_type()) {
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

    /// Advane the char_indices iterator to the given position.
    /// The function is used to skip a given number of characters in the haystack.
    /// It can be used after a peek operation to skip the characters of the peeked matches.
    /// The function returns the new position of the char_indices iterator.
    /// If the new position is greater than the length of the haystack, the function returns the
    /// length of the haystack.
    /// If the new position is less than the current position of the char_indices iterator, the
    /// function returns the current position of the char_indices iterator.
    pub(crate) fn advance_to(&mut self, position: usize) -> usize {
        let mut new_position = 0;
        if position < self.last_position {
            // The new position is less than the current position of the char_indices iterator.
            // The iterator is advanced by one character and the next character is not returned by
            // the iterator.
            return self.last_position;
        }
        let mut line_start_offsets = vec![];
        let mut last_char = self.last_char;
        for (i, c) in self.char_indices.by_ref() {
            if last_char == '\n' {
                line_start_offsets.push(i);
            }
            last_char = c;
            new_position = i;
            if i + c.len_utf8() >= position {
                break;
            }
        }
        self.line_offsets.append(&mut line_start_offsets);
        self.last_char = last_char;
        self.last_position = new_position;
        new_position
    }

    /// Retrieve the total offset of the char indices iterator in bytes.
    pub(crate) fn offset(&self) -> usize {
        self.last_position
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
    pub(crate) fn position(&self, offset: usize) -> Position {
        match self.line_offsets.binary_search_by(|&x| x.cmp(&offset)) {
            Ok(i) => Position::new(i + 1, offset.saturating_sub(self.line_offsets[i]) + 1),
            Err(_) => Position::new(
                self.line_offsets.len() - 1 + 1,
                offset.saturating_sub(*self.line_offsets.last().unwrap()) + 1,
            ),
        }
    }

    /// Records the offset of a line in the haystack.
    fn record_line_offset(&mut self, i: usize, c: char) {
        if self.last_char == '\n' {
            self.line_offsets.push(i);
        }
        self.last_char = c;
    }

    /// Returns the current scanner mode. Used for tests and debugging purposes.
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn current_mode(&self) -> usize {
        self.scanner_impl.current_mode()
    }

    pub(crate) fn set_mode(&mut self, mode: usize) {
        self.scanner_impl.set_mode(mode);
    }

    pub(crate) fn mode_name(&self, index: usize) -> Option<&str> {
        self.scanner_impl.mode_name(index)
    }
}

impl std::fmt::Debug for FindMatchesImpl<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FindMatchesImpl").finish()
    }
}

impl Iterator for FindMatchesImpl<'_> {
    type Item = Match;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_match()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use super::*;
    use crate::{MatchExt, MatchExtIterator, Pattern, ScannerBuilder, ScannerMode};

    static MODES: LazyLock<[ScannerMode; 2]> = LazyLock::new(|| {
        [
            ScannerMode::new(
                "INITIAL",
                vec![
                    Pattern::new(r"\r\n|\r|\n".to_string(), 0),  // Newline
                    Pattern::new(r"[\s--\r\n]+".to_string(), 1), // Whitespace
                    Pattern::new(r"//.*(\r\n|\r|\n)".to_string(), 2), // Line comment
                    Pattern::new(r"/\*([.\r\n--*]|\*[^/])*\*/".to_string(), 3), // Block comment
                    Pattern::new(r"[a-zA-Z_]\w*".to_string(), 4), // Identifier
                    Pattern::new(r"\u{22}".to_string(), 8),      // String delimiter
                    Pattern::new(r".".to_string(), 9),           // Error
                ],
                vec![
                    (8, 1), // Token "String delimiter" -> Mode "STRING"
                ],
            ),
            ScannerMode::new(
                "STRING",
                vec![
                    Pattern::new(r"\u{5c}[\u{22}\u{5c}bfnt]".to_string(), 5), // Escape sequence
                    Pattern::new(r"\u{5c}[\s^\n\r]*\r?\n".to_string(), 6),    // Line continuation
                    Pattern::new(r"[^\u{22}\u{5c}]+".to_string(), 7),         // String content
                    Pattern::new(r"\u{22}".to_string(), 8),                   // String delimiter
                    Pattern::new(r".".to_string(), 9),                        // Error
                ],
                vec![
                    (8, 0), // Token "String delimiter" -> Mode "INITIAL"
                ],
            ),
        ]
    });

    // The input string contains a string which triggers a mode switch from "INITIAL" to "STRING"
    // and back to "INITIAL".
    const INPUT: &str = r#"
Id1
"1. String"
Id2
"#;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_find_matches_impl() {
        init();
        println!("{}", serde_json::to_string(&*MODES).unwrap());
        let scanner = ScannerBuilder::new()
            .add_scanner_modes(&*MODES)
            .build()
            .unwrap();

        let find_matches = scanner.find_iter(INPUT);
        let matches: Vec<Match> = find_matches.collect();
        assert_eq!(matches.len(), 9);
        assert_eq!(
            matches,
            vec![
                Match::new(0, (0usize..1).into()),
                Match::new(4, (1usize..4).into()),
                Match::new(0, (4usize..5).into()),
                Match::new(8, (5usize..6).into()),
                Match::new(7, (6usize..15).into()),
                Match::new(8, (15usize..16).into()),
                Match::new(0, (16usize..17).into()),
                Match::new(4, (17usize..20).into()),
                Match::new(0, (20usize..21).into())
            ]
        );
        assert_eq!(
            matches
                .iter()
                .map(|m| {
                    let rng = m.span().start..m.span().end;
                    INPUT.get(rng).unwrap()
                })
                .collect::<Vec<_>>(),
            vec![
                "\n",
                "Id1",
                "\n",
                "\"",
                "1. String",
                "\"",
                "\n",
                "Id2",
                "\n"
            ]
        );
    }

    #[test]
    fn test_find_matches_impl_nfa() {
        init();
        println!("{}", serde_json::to_string(&*MODES).unwrap());
        let scanner = ScannerBuilder::new()
            .add_scanner_modes(&*MODES)
            .use_nfa()
            .build()
            .unwrap();

        eprintln!("Input: '{}'", INPUT);
        let find_matches = scanner.find_iter(INPUT);
        let matches: Vec<Match> = find_matches.collect();
        // assert_eq!(matches.len(), 9);
        assert_eq!(
            matches,
            vec![
                Match::new(0, (0usize..1).into()),
                Match::new(4, (1usize..4).into()),
                Match::new(0, (4usize..5).into()),
                Match::new(8, (5usize..6).into()),
                Match::new(7, (6usize..15).into()),
                Match::new(8, (15usize..16).into()),
                Match::new(0, (16usize..17).into()),
                Match::new(4, (17usize..20).into()),
                Match::new(0, (20usize..21).into())
            ]
        );
        assert_eq!(
            matches
                .iter()
                .map(|m| {
                    let rng = m.span().start..m.span().end;
                    INPUT.get(rng).unwrap()
                })
                .collect::<Vec<_>>(),
            vec![
                "\n",
                "Id1",
                "\n",
                "\"",
                "1. String",
                "\"",
                "\n",
                "Id2",
                "\n"
            ]
        );
    }

    #[test]
    fn test_peek_n() {
        init();
        let scanner = ScannerBuilder::new()
            .add_scanner_modes(&*MODES)
            .build()
            .unwrap();
        let mut find_iter = scanner.find_iter(INPUT);
        let peeked = find_iter.peek_n(2);
        assert_eq!(
            peeked,
            PeekResult::Matches(vec![
                Match::new(0, (0usize..1).into()),
                Match::new(4, (1usize..4).into())
            ])
        );
        let peeked = find_iter.peek_n(4);
        assert_eq!(
            peeked,
            PeekResult::MatchesReachedModeSwitch((
                vec![
                    Match::new(0, (0usize..1).into()),
                    Match::new(4, (1usize..4).into()),
                    Match::new(0, (4usize..5).into()),
                    Match::new(8, (5usize..6).into())
                ],
                1,
            ))
        );
        let peeked = find_iter.peek_n(5);
        assert_eq!(
            peeked,
            PeekResult::MatchesReachedModeSwitch((
                vec![
                    Match::new(0, (0usize..1).into()),
                    Match::new(4, (1usize..4).into()),
                    Match::new(0, (4usize..5).into()),
                    Match::new(8, (5usize..6).into())
                ],
                1,
            ))
        );
        let _ = find_iter.by_ref().take(7).collect::<Vec<_>>();
        let peeked = find_iter.peek_n(4);
        assert_eq!(
            peeked,
            PeekResult::MatchesReachedEnd(vec![
                Match::new(4, (17usize..20).into()),
                Match::new(0, (20usize..21).into())
            ])
        );
    }

    #[test]
    fn test_peek_n_nfa() {
        init();
        let scanner = ScannerBuilder::new()
            .add_scanner_modes(&*MODES)
            .use_nfa()
            .build()
            .unwrap();
        let mut find_iter = scanner.find_iter(INPUT);
        let peeked = find_iter.peek_n(2);
        assert_eq!(
            peeked,
            PeekResult::Matches(vec![
                Match::new(0, (0usize..1).into()),
                Match::new(4, (1usize..4).into())
            ])
        );
        let peeked = find_iter.peek_n(4);
        assert_eq!(
            peeked,
            PeekResult::MatchesReachedModeSwitch((
                vec![
                    Match::new(0, (0usize..1).into()),
                    Match::new(4, (1usize..4).into()),
                    Match::new(0, (4usize..5).into()),
                    Match::new(8, (5usize..6).into())
                ],
                1,
            ))
        );
        let peeked = find_iter.peek_n(5);
        assert_eq!(
            peeked,
            PeekResult::MatchesReachedModeSwitch((
                vec![
                    Match::new(0, (0usize..1).into()),
                    Match::new(4, (1usize..4).into()),
                    Match::new(0, (4usize..5).into()),
                    Match::new(8, (5usize..6).into())
                ],
                1,
            ))
        );
        let _ = find_iter.by_ref().take(7).collect::<Vec<_>>();
        let peeked = find_iter.peek_n(4);
        assert_eq!(
            peeked,
            PeekResult::MatchesReachedEnd(vec![
                Match::new(4, (17usize..20).into()),
                Match::new(0, (20usize..21).into())
            ])
        );
    }

    #[test]
    fn test_peek_does_not_effect_the_iterator() {
        init();
        let scanner = ScannerBuilder::new()
            .add_scanner_modes(&*MODES)
            .build()
            .unwrap();
        let mut find_iter = scanner.find_iter(INPUT);
        let peeked = find_iter.peek_n(2);
        assert_eq!(
            peeked,
            PeekResult::Matches(vec![
                Match::new(0, (0usize..1).into()),
                Match::new(4, (1usize..4).into())
            ])
        );
        let peeked = find_iter.peek_n(2);
        assert_eq!(
            peeked,
            PeekResult::Matches(vec![
                Match::new(0, (0usize..1).into()),
                Match::new(4, (1usize..4).into())
            ])
        );
    }

    #[test]
    fn test_peek_nfa_does_not_effect_the_iterator() {
        init();
        let scanner = ScannerBuilder::new()
            .add_scanner_modes(&*MODES)
            .use_nfa()
            .build()
            .unwrap();
        let mut find_iter = scanner.find_iter(INPUT);
        let peeked = find_iter.peek_n(2);
        assert_eq!(
            peeked,
            PeekResult::Matches(vec![
                Match::new(0, (0usize..1).into()),
                Match::new(4, (1usize..4).into())
            ])
        );
        let peeked = find_iter.peek_n(2);
        assert_eq!(
            peeked,
            PeekResult::Matches(vec![
                Match::new(0, (0usize..1).into()),
                Match::new(4, (1usize..4).into())
            ])
        );
    }

    #[test]
    fn test_advance_to() {
        init();
        let scanner = ScannerBuilder::new()
            .add_scanner_modes(&*MODES)
            .build()
            .unwrap();
        let mut find_iter = scanner.find_iter(INPUT);
        let peeked = find_iter.peek_n(2);
        assert_eq!(
            peeked,
            PeekResult::Matches(vec![
                Match::new(0, (0usize..1).into()),
                Match::new(4, (1usize..4).into())
            ])
        );
        let new_position = find_iter.advance_to(4);
        assert_eq!(new_position, 3);
        let peeked = find_iter.peek_n(3);
        assert_eq!(
            peeked,
            PeekResult::MatchesReachedModeSwitch((
                vec![
                    Match::new(0, (4usize..5).into()),
                    Match::new(8, (5usize..6).into())
                ],
                1,
            ))
        );
    }

    #[test]
    fn test_nfa_advance_to() {
        init();
        let scanner = ScannerBuilder::new()
            .add_scanner_modes(&*MODES)
            .use_nfa()
            .build()
            .unwrap();
        let mut find_iter = scanner.find_iter(INPUT);
        let peeked = find_iter.peek_n(2);
        assert_eq!(
            peeked,
            PeekResult::Matches(vec![
                Match::new(0, (0usize..1).into()),
                Match::new(4, (1usize..4).into())
            ])
        );
        let new_position = find_iter.advance_to(4);
        assert_eq!(new_position, 3);
        let peeked = find_iter.peek_n(3);
        assert_eq!(
            peeked,
            PeekResult::MatchesReachedModeSwitch((
                vec![
                    Match::new(0, (4usize..5).into()),
                    Match::new(8, (5usize..6).into())
                ],
                1,
            ))
        );
    }

    // Test the WithPositions iterator.
    #[test]
    fn test_with_positions() {
        init();
        let scanner = ScannerBuilder::new()
            .add_scanner_modes(&*MODES)
            .build()
            .unwrap();
        let find_iter = scanner.find_iter(INPUT).with_positions();
        let matches: Vec<MatchExt> = find_iter.collect();
        assert_eq!(matches.len(), 9);
        assert_eq!(
            matches,
            vec![
                MatchExt::new(
                    0,
                    (0usize..1).into(),
                    Position::new(1, 1),
                    Position::new(1, 2)
                ),
                MatchExt::new(
                    4,
                    (1usize..4).into(),
                    Position::new(2, 1),
                    Position::new(2, 4)
                ),
                MatchExt::new(
                    0,
                    (4usize..5).into(),
                    Position::new(2, 4),
                    Position::new(2, 5)
                ),
                MatchExt::new(
                    8,
                    (5usize..6).into(),
                    Position::new(3, 1),
                    Position::new(3, 2)
                ),
                MatchExt::new(
                    7,
                    (6usize..15).into(),
                    Position::new(3, 2),
                    Position::new(3, 11)
                ),
                MatchExt::new(
                    8,
                    (15usize..16).into(),
                    Position::new(3, 11),
                    Position::new(3, 12)
                ),
                MatchExt::new(
                    0,
                    (16usize..17).into(),
                    Position::new(3, 12),
                    Position::new(3, 13)
                ),
                MatchExt::new(
                    4,
                    (17usize..20).into(),
                    Position::new(4, 1),
                    Position::new(4, 4)
                ),
                MatchExt::new(
                    0,
                    (20usize..21).into(),
                    Position::new(4, 4),
                    Position::new(4, 5)
                )
            ]
        );
        assert_eq!(
            matches
                .iter()
                .map(|m| {
                    let rng = m.span().start..m.span().end;
                    INPUT.get(rng).unwrap()
                })
                .collect::<Vec<_>>(),
            vec![
                "\n",
                "Id1",
                "\n",
                "\"",
                "1. String",
                "\"",
                "\n",
                "Id2",
                "\n"
            ]
        );
    }

    // Test the WithPositions iterator with NFA scanner.
    #[test]
    fn test_nfa_with_positions() {
        init();
        let scanner = ScannerBuilder::new()
            .add_scanner_modes(&*MODES)
            .use_nfa()
            .build()
            .unwrap();
        let find_iter = scanner.find_iter(INPUT).with_positions();
        let matches: Vec<MatchExt> = find_iter.collect();
        assert_eq!(matches.len(), 9);
        assert_eq!(
            matches,
            vec![
                MatchExt::new(
                    0,
                    (0usize..1).into(),
                    Position::new(1, 1),
                    Position::new(1, 2)
                ),
                MatchExt::new(
                    4,
                    (1usize..4).into(),
                    Position::new(2, 1),
                    Position::new(2, 4)
                ),
                MatchExt::new(
                    0,
                    (4usize..5).into(),
                    Position::new(2, 4),
                    Position::new(2, 5)
                ),
                MatchExt::new(
                    8,
                    (5usize..6).into(),
                    Position::new(3, 1),
                    Position::new(3, 2)
                ),
                MatchExt::new(
                    7,
                    (6usize..15).into(),
                    Position::new(3, 2),
                    Position::new(3, 11)
                ),
                MatchExt::new(
                    8,
                    (15usize..16).into(),
                    Position::new(3, 11),
                    Position::new(3, 12)
                ),
                MatchExt::new(
                    0,
                    (16usize..17).into(),
                    Position::new(3, 12),
                    Position::new(3, 13)
                ),
                MatchExt::new(
                    4,
                    (17usize..20).into(),
                    Position::new(4, 1),
                    Position::new(4, 4)
                ),
                MatchExt::new(
                    0,
                    (20usize..21).into(),
                    Position::new(4, 4),
                    Position::new(4, 5)
                )
            ]
        );
        assert_eq!(
            matches
                .iter()
                .map(|m| {
                    let rng = m.span().start..m.span().end;
                    INPUT.get(rng).unwrap()
                })
                .collect::<Vec<_>>(),
            vec![
                "\n",
                "Id1",
                "\n",
                "\"",
                "1. String",
                "\"",
                "\n",
                "Id2",
                "\n"
            ]
        );
    }
}
