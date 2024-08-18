use super::{CharClassID, ScannerImpl};
use crate::{Match, PeekResult};

/// An iterator over all non-overlapping matches.
pub(crate) struct FindMatchesImpl<'h> {
    // The scanner used to find matches.
    scanner: ScannerImpl,
    // The function used to match characters to character classes.
    match_char_class: Box<dyn Fn(CharClassID, char) -> bool + 'static>,
    // The input haystack.
    char_indices: std::str::CharIndices<'h>,
}

impl<'h> FindMatchesImpl<'h> {
    /// Creates a new `FindMatches` iterator.
    pub(crate) fn new(scanner_impl: &ScannerImpl, input: &'h str) -> Self {
        Self {
            scanner: scanner_impl.clone(),
            match_char_class: scanner_impl.create_match_char_class().unwrap(),
            char_indices: input.char_indices(),
        }
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
            result = self
                .scanner
                .find_from(&self.match_char_class, self.char_indices.clone());
            if let Some(matched) = result {
                self.advance_beyond_match(matched);
                break;
            } else if self.char_indices.next().is_none() {
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
            let result = self
                .scanner
                .peek_from(&self.match_char_class, char_indices.clone());
            if let Some(matched) = result {
                matches.push(matched);
                Self::advance_char_indices_beyond_match(&mut char_indices, matched);
                if let Some(mode) = self.scanner.has_transition(matched.token_type()) {
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
        for (i, c) in self.char_indices.by_ref() {
            if i + c.len_utf8() >= end {
                // Stop at the end of the match.
                break;
            }
        }
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
    use crate::{ScannerBuilder, ScannerMode};

    static MODES: LazyLock<[ScannerMode; 2]> = LazyLock::new(|| {
        [
            ScannerMode::new(
                "INITIAL",
                vec![
                    (r"\r\n|\r|\n", 0),                 // Newline
                    (r"[\s--\r\n]+", 1),                // Whitespace
                    (r"//.*(\r\n|\r|\n)", 2),           // Line comment
                    (r"/\*([.\r\n--*]|\*[^/])*\*/", 3), // Block comment
                    (r"[a-zA-Z_]\w*", 4),               // Identifier
                    (r"\u{22}", 8),                     // String delimiter
                    (r".", 9),                          // Error
                ],
                vec![
                    (8, 1), // Token "String delimiter" -> Mode "STRING"
                ],
            ),
            ScannerMode::new(
                "STRING",
                vec![
                    (r"\u{5c}[\u{22}\u{5c}bfnt]", 5), // Escape sequence
                    (r"\u{5c}[\s^\n\r]*\r?\n", 6),    // Line continuation
                    (r"[^\u{22}\u{5c}]+", 7),         // String content
                    (r"\u{22}", 8),                   // String delimiter
                    (r".", 9),                        // Error
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

    #[test]
    fn test_find_matches_impl() {
        println!("{}", serde_json::to_string(&*MODES).unwrap());
        let scanner = ScannerBuilder::new()
            .add_scanner_modes(&*MODES)
            .build()
            .unwrap();

        let find_matches = FindMatchesImpl::new(&scanner.inner, INPUT);
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
                Match::new(0, (20usize..21).into()),
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
                Match::new(4, (1usize..4).into()),
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
                    Match::new(8, (5usize..6).into()),
                ],
                1
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
                    Match::new(8, (5usize..6).into()),
                ],
                1
            ))
        );
        let _ = find_iter.by_ref().take(7).collect::<Vec<_>>();
        let peeked = find_iter.peek_n(4);
        assert_eq!(
            peeked,
            PeekResult::MatchesReachedEnd(vec![
                Match::new(4, (17usize..20).into()),
                Match::new(0, (20usize..21).into()),
            ])
        );
    }
}
