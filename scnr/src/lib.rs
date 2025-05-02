#![forbid(missing_docs)]
//! # `scnr`
//! The `scnr` crate is a library that provides lexical scanner for programming languages.
//! It is designed to be used in a parser of a compiler or interpreter for a programming language
//! or in similar tools that require lexical analysis, e.g. in a language server.
//! It provides multiple scanner modes out of the box, which can be switched at runtime depending
//! on the context of the input.
//! A parser can use different modes for different parts of the input, e.g. to scan comments in one
//! mode and code in another.
//! The scanner is designed to be fast and efficient, and it is implemented with the help of
//! finite state machines.
//! To parse the given regular expressions, the crate uses the `regex-syntax` crate.
//!
//! # Example with a simple pattern list
//! ```rust
//! use scnr::{ScannerBuilder, ScannerTrait};
//!
//! static PATTERNS: &[&str] = &[
//!     r";",                    // Semicolon
//!     r"0|[1-9][0-9]*",        // Number
//!     r"//.*(\r\n|\r|\n)",     // Line comment
//!     r"/\*([^*]|\*[^/])*\*/", // Block comment
//!     r"[a-zA-Z_]\w*",         // Identifier
//!     r"=",                    // Assignment
//! ];
//!
//! const INPUT: &str = r#"
//! // This is a comment
//! a = 10;
//! b = 20;
//! /* This is a block comment
//!    that spans multiple lines */
//! c = a;
//! "#;
//!
//! fn main() {
//!     let scanner = ScannerBuilder::new()
//!         .add_patterns(PATTERNS)
//!         .build()
//!         .expect("ScannerBuilder error");
//!     let find_iter = scanner.find_iter(INPUT);
//!     for ma in find_iter {
//!         println!("Match: {:?}: '{}'", ma, &INPUT[ma.span().range()]);
//!     }
//! }
//! ```
//! The output of the example is:
//! ```text
//! Match: Match { token_type: 2, span: Span { start: 1, end: 22 } }: '// This is a comment
//! '
//! Match: Match { token_type: 4, span: Span { start: 22, end: 23 } }: 'a'
//! Match: Match { token_type: 5, span: Span { start: 24, end: 25 } }: '='
//! Match: Match { token_type: 1, span: Span { start: 26, end: 28 } }: '10'
//! Match: Match { token_type: 0, span: Span { start: 28, end: 29 } }: ';'
//! Match: Match { token_type: 4, span: Span { start: 30, end: 31 } }: 'b'
//! Match: Match { token_type: 5, span: Span { start: 32, end: 33 } }: '='
//! Match: Match { token_type: 1, span: Span { start: 34, end: 36 } }: '20'
//! Match: Match { token_type: 0, span: Span { start: 36, end: 37 } }: ';'
//! Match: Match { token_type: 3, span: Span { start: 38, end: 96 } }: '/* This is a block comment
//!    that spans multiple lines */'
//! Match: Match { token_type: 4, span: Span { start: 97, end: 98 } }: 'c'
//! Match: Match { token_type: 5, span: Span { start: 99, end: 100 } }: '='
//! Match: Match { token_type: 4, span: Span { start: 101, end: 102 } }: 'a'
//! Match: Match { token_type: 0, span: Span { start: 102, end: 103 } }: ';'
//! ```
//!
//! # Example with scanner modes and position information
//! ```rust
//! use std::sync::LazyLock;
//!
//! use scnr::{MatchExtIterator, ScannerBuilder, ScannerMode, ScannerTrait};
//! use scnr_generate::Pattern;
//!
//! static SCANNER_MODES: LazyLock<Vec<ScannerMode>> = LazyLock::new(|| {
//!     vec![
//!         ScannerMode::new(
//!             "INITIAL",
//!             vec![
//!                 Pattern::new(r"\r\n|\r|\n".to_string(), 0),   // Newline
//!                 Pattern::new(r"[a-zA-Z_]\w*".to_string(), 4), // Identifier
//!                 Pattern::new(r#"""#.to_string(), 6),          // String delimiter
//!             ],
//!             vec![
//!                 (6, 1), // Token "String delimiter" -> Mode "STRING"
//!             ],
//!         ),
//!         ScannerMode::new(
//!             "STRING",
//!             vec![
//!                 Pattern::new(r#"""#.to_string(), 6),     // String delimiter
//!                 Pattern::new(r#"[^"]+"#.to_string(), 5), // String content
//!             ],
//!             vec![
//!                 (6, 0), // Token "String delimiter" -> Mode "INITIAL"
//!             ],
//!         ),
//!     ]
//! });
//!
//! const INPUT: &str = r#"Id1 "1. String" "2. String""#;
//!
//! fn main() {
//!     let scanner = ScannerBuilder::new()
//!         .add_scanner_modes(&SCANNER_MODES)
//!         .build()
//!         .expect("ScannerBuilder error");
//!     let find_iter = scanner.find_iter(INPUT).with_positions();
//!     for ma in find_iter {
//!         println!("{:?}: '{}'", ma, &INPUT[ma.span().range()]);
//!     }
//! }
//! ```
//!
//! The output of this example is:
//! ```text
//! MatchExt { token_type: 4, span: Span { start: 0, end: 3 }, start_position: Position { line: 1, column: 1 }, end_position: Position { line: 1, column: 4 } }: 'Id1'
//! MatchExt { token_type: 6, span: Span { start: 4, end: 5 }, start_position: Position { line: 1, column: 5 }, end_position: Position { line: 1, column: 6 } }: '"'
//! MatchExt { token_type: 5, span: Span { start: 5, end: 14 }, start_position: Position { line: 1, column: 6 }, end_position: Position { line: 1, column: 15 } }: '1. String'
//! MatchExt { token_type: 6, span: Span { start: 14, end: 15 }, start_position: Position { line: 1, column: 15 }, end_position: Position { line: 1, column: 16 } }: '"'
//! MatchExt { token_type: 6, span: Span { start: 16, end: 17 }, start_position: Position { line: 1, column: 17 }, end_position: Position { line: 1, column: 18 } }: '"'
//! MatchExt { token_type: 5, span: Span { start: 17, end: 26 }, start_position: Position { line: 1, column: 18 }, end_position: Position { line: 1, column: 27 } }: '2. String'
//! MatchExt { token_type: 6, span: Span { start: 26, end: 27 }, start_position: Position { line: 1, column: 27 }, end_position: Position { line: 1, column: 28 } }: '"'
//! ```
//!
//! # Crate features
//! The crate has the following features:
//! - `default`: This is the default feature set. When it is enabled it uses the `scnr` crate's own
//!   regex engine.
//!
//! - `regex_automata`: This feature is not enabled by default. It instructs the lib to use the
//!   crate `regex_automata` as regex engine.
//!
//! Both features are mutually exclusive. You can enable one of them, but not both at the same time.
//!
//! Enabling the default feature usually results in a slower scanner, but it is faster at compiling
//! the regexes. The `regex_automata` feature is faster at scanning the input, but it is possibly
//! slower at compiling the regexes. This depends on the size of your scanner modes, i.e. the number
//! of regexes you use.

/// Module that provides a FindMatches type
mod find_matches;
pub use find_matches::FindMatches;

/// The module with the scanner.
mod scanner;
pub use scanner::{Scanner, ScannerTrait};

/// The module with the scanner builder.
mod scanner_builder;
pub use scanner_builder::ScannerBuilder;

/// Module that provides a scanner cache type
mod scanner_cache;
pub(crate) use scanner_cache::SCANNER_CACHE;

/// Module that provides functions and types related to scanner implementations.
pub(crate) use scnr_generate::ScannerImpl;

/// The result type for the `scrn` crate.
pub type Result<T> = std::result::Result<T, scnr_generate::ScnrError>;

/// Re-imports from the `scnr_generate` crate.
pub use scnr_generate::{
    Match, MatchExt, MatchExtIterator, Position, PositionProvider, ScannerMode,
    ScannerModeSwitcher, Span,
};
