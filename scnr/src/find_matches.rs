use log::trace;
use scnr_generate::{FindMatchesImpl, PeekResult};

use crate::{Match, Position, PositionProvider, ScannerImpl, ScannerModeSwitcher};

/// An iterator over all non-overlapping matches.
///
/// The iterator yields [`Match`] values until no more matches could be found.
///
/// * `'h` represents the lifetime of the haystack being searched.
///
/// This iterator can be created with the [`crate::Scanner::find_iter`] method.
#[derive(Debug)]
pub struct FindMatches<'h> {
    inner: FindMatchesImpl<'h>,
}

impl<'h> FindMatches<'h> {
    /// Creates a new `FindMatches` iterator.
    pub(crate) fn new(scanner_impl: ScannerImpl, input: &'h str) -> Self {
        Self {
            inner: FindMatchesImpl::new(scanner_impl, input),
        }
    }

    /// Set the offset in the haystack to the given position relative to the start of the haystack.
    /// If a parser resets the scanner to a certain position, it can use this method.
    /// A use case is a parser that backtracks to a previous position in the input or a parser that
    /// switches between different scanner modes on its own.
    /// If the offset is greater than the length of the haystack, the offset is set to the length of
    /// the haystack.
    pub fn with_offset(self, offset: usize) -> Self {
        Self {
            inner: self.inner.with_offset(offset),
        }
    }

    /// Set the offset in the haystack to the given position relative to the start of the haystack.
    /// The function is used to set the position in the haystack to the given position.
    /// It provides the same functionality as the `with_offset` method, but it mutates the object
    /// in place.
    pub fn set_offset(&mut self, position: usize) {
        self.inner.set_offset(position);
    }

    /// Retrieve the current byte offset from the start of the haystack.
    /// This is the end offset of the last match found by the iterator.
    #[inline]
    pub fn offset(&self) -> usize {
        self.inner.offset()
    }

    /// Returns the next match in the haystack.
    ///
    /// If a match is found, the function advances the iterator to the end of the match and returns
    /// the match.
    ///
    /// If no match is found, the function repeatedly advances the haystack by one and tries again
    /// until a match is found or the iterator is exhausted.
    ///
    /// If the iterator is exhausted and no match is found, `None` is returned.
    ///
    /// This method is also used in the implementation of the `Iterator` trait for the `FindMatches`.
    #[inline]
    pub fn next_match(&mut self) -> Option<Match> {
        self.inner.next_match()
    }

    /// Peeks n matches ahead without consuming the matches.
    /// The function returns [PeekResult].
    ///
    /// The peek operation always stops at the end of the haystack or when a mode switch is
    /// triggered by the last match. The mode switch is not conducted by the peek operation to not
    /// change the state of the scanner as well as to avoid a mix of tokens from different modes
    /// being returned.
    pub fn peek_n(&mut self, n: usize) -> PeekResult {
        self.inner.peek_n(n)
    }

    /// Advance the haystack to the given position.
    /// The function is used to skip a given number of characters in the haystack.
    /// It can be used after a peek operation to skip the characters of the peeked matches.
    /// The function returns the new position in the haystack.
    /// If the new position is greater than the length of the haystack, the function returns the
    /// length of the haystack.
    /// If the new position is less than the current position in the haystack, the
    /// function returns the current position in the haystack, i.e. it does not allow to move
    /// backwards in the haystack.
    /// The current position in the haystack is the end index of the last match found
    /// by the iterator, such that the next call to `next_match` will start searching for matches
    /// at the following position.
    pub fn advance_to(&mut self, position: usize) -> usize {
        self.inner.advance_to(position)
    }
}

impl Iterator for FindMatches<'_> {
    type Item = Match;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_match()
    }
}

impl PositionProvider for FindMatches<'_> {
    /// Returns the line and column numbers of the given offset.
    /// The line number is the index of the line offset in the vector plus one.
    /// The column number is the offset minus the line offset.
    /// If the offset is greater than the length of the haystack, the function returns the last
    /// recorded line and the column number is calculated from the last recorded position.
    fn position(&self, offset: usize) -> Position {
        self.inner.position(offset)
    }

    /// Sets the offset of the haystack to the given position.
    fn set_offset(&mut self, offset: usize) {
        self.inner.set_offset(offset);
    }
}

impl ScannerModeSwitcher for FindMatches<'_> {
    /// Sets the current scanner mode of the scanner implementation.
    ///
    /// A parser can explicitly set the scanner mode to switch to a different set of DFAs.
    /// Usually, the scanner mode is changed by the scanner itself based on the transitions defined
    /// in the active scanner mode.
    fn set_mode(&mut self, mode: usize) {
        trace!("Set scanner mode to {}", mode);
        self.inner.set_mode(mode);
    }

    /// Returns the current scanner mode. Used for tests and debugging purposes.
    #[allow(dead_code)]
    #[inline]
    fn current_mode(&self) -> usize {
        self.inner.current_mode()
    }

    fn mode_name(&self, index: usize) -> Option<&str> {
        self.inner.mode_name(index)
    }
}

#[cfg(test)]
mod tests {
    use scnr_generate::Pattern;

    use super::*;
    use crate::{MatchExt, MatchExtIterator, ScannerBuilder, ScannerMode};

    static MODES: std::sync::LazyLock<[ScannerMode; 2]> = std::sync::LazyLock::new(|| {
        [
            ScannerMode::new(
                "INITIAL",
                vec![
                    Pattern::new(r"\r\n|\r|\n".to_string(), 0),  // Newline
                    Pattern::new(r"[\s--\r\n]+".to_string(), 1), // Whitespace
                    Pattern::new(r"//.*(\r\n|\r|\n)".to_string(), 2), // Line comment
                    Pattern::new(r"/\*([^*]|\*[^/])*\*/".to_string(), 3), // Block comment
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

    static INIT: std::sync::Once = std::sync::Once::new();

    const TARGET_FOLDER: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../target/testout/test_find_matches_impl"
    );

    fn init() {
        INIT.call_once(|| {
            let _ = env_logger::builder().is_test(true).try_init();
            // Delete all previously generated dot files.
            let _ = std::fs::remove_dir_all(TARGET_FOLDER);
            // Create the target folder.
            std::fs::create_dir_all(TARGET_FOLDER).unwrap();
        });
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_find_matches_impl() {
        use crate::ScannerBuilder;

        init();
        println!("{}", serde_json::to_string(&*MODES).unwrap());
        let scanner = ScannerBuilder::new()
            .add_scanner_modes(&*MODES)
            .build()
            .unwrap();

        scanner
            .generate_compiled_automata_as_dot("String", std::path::Path::new(TARGET_FOLDER))
            .expect("Failed to generate compiled automata as dot");

        let find_matches = scanner.find_iter(INPUT);
        let matches: Vec<Match> = find_matches.collect();
        trace!("Matches:");
        matches.iter().for_each(|m| {
            trace!("{:?}", m);
        });
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
}
