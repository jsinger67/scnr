#![forbid(missing_docs)]
//! # `scnr`
//! The 'scnr' crate is a library that provides lexical scanner for programming languages.
//! It is designed to be used in a parser of a compiler or interpreter for a programming language
//! or in similar tools that require lexical analysis, e.g. in a language server.
//! It provides multiple scanner modes out of the box, which can be switched at runtime depending
//! on the context of the input.
//! A parser can use different modes for different parts of the input, e.g. to scan comments in one
//! mode and code in another.
//! The scanner is designed to be fast and efficient, and it is implemented with the help of
//! state machines.
//! To parse the given regular expressions, the crate uses the `regex-syntax` crate.

/// The module with internal implementation details.
mod internal;
pub(crate) use internal::find_matches_impl::FindMatchesImpl;
pub(crate) use internal::scanner_impl::ScannerImpl;

/// Module with error definitions
mod errors;
pub use errors::{Result, ScnrError, ScnrErrorKind};

/// The module with the scanner builder.
pub mod scanner_builder;

/// The module with the scanner mode.
pub mod scanner_mode;

/// The module with the scanner.
pub mod scanner;

/// Module that provides a Match type
mod match_type;
pub use match_type::Match;

/// Module that provides a Span type
mod span;
pub use span::Span;

/// Module that provides a FindMatches type
mod find_matches;
pub use find_matches::{FindMatches, PeekResult};
