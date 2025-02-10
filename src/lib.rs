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
//! # Example
//! ```rust
//! use scnr::ScannerBuilder;
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
//! # Crate features
//! The crate has the following features:
//! - `default`: This is the default feature set. It uses the `scnr` crate's own regex engine.
//!
//! - `regex_automata`: This feature is not enabled by default. It uses the `regex_automata` crate
//!   as regex engine.
//!
//! Both features are mutually exclusive. You can enable one of them, but not both at the same time.
//!
//! Enabling the default feature usually results in a slower scanner, but it is faster at compiling
//! the regexes. The `regex_automata` feature is faster at scanning the input, but it is possibly
//! slower at compiling the regexes. This depends on the size of your scanner modes, i.e. the number
//! of regexes you use.

/// Module with error definitions
mod errors;
pub use errors::{Result, ScnrError, ScnrErrorKind};

/// Module that provides a FindMatches type
mod find_matches;
pub use find_matches::{FindMatches, PeekResult};

/// The module with internal implementation details.
mod internal;

/// Module that provides a Match type
mod match_type;
pub use match_type::{Match, MatchExt};

/// Module that provides a Pattern type and a Lookahead type
mod pattern;
pub use pattern::{Lookahead, Pattern};

/// Module that provides a position type
mod position;
pub use position::{Position, PositionProvider};

/// The module with the scanner.
mod scanner;
pub use scanner::{Scanner, ScannerModeSwitcher};

/// The module with the scanner builder.
mod scanner_builder;
pub use scanner_builder::ScannerBuilder;

/// The module with the scanner mode.
mod scanner_mode;
pub use scanner_mode::ScannerMode;

/// Module that provides a Span type
mod span;
pub use span::Span;

/// Module that provides a WithPositions type
mod with_positions;
pub use with_positions::{MatchExtIterator, WithPositions};
