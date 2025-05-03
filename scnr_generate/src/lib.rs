#[cfg(test)]
#[macro_use]
extern crate rstest;

/// Module with error definitions
mod errors;
pub use errors::{Result, ScnrError, ScnrErrorKind};

/// Module that provides functions and types related to character classes.
mod character_class;
pub(crate) use character_class::CharacterClass;

/// Module that provides the type CharacterClassRegistry.
mod character_class_registry;
pub(crate) use character_class_registry::CharacterClassRegistry;

/// Module that provides functions and types related to compiled Lookahead.
pub(crate) mod compiled_lookahead;
// pub(crate) use compiled_lookahead::{CompiledLookahead, CompiledDfaLookahead};
pub(crate) use compiled_lookahead::CompiledLookahead;

/// Module that provides functions and types related to compiled NFA.
pub(crate) mod compiled_dfa;
// pub(crate) use compiled_dfa::CompiledDfa;

/// Module that provides functions and types related to compiled ScannerModes.
pub(crate) mod compiled_scanner_mode;

/// Module with the type HirWithPattern.
pub(crate) mod hir_with_pattern;
pub(crate) use hir_with_pattern::HirWithPattern;

/// Module with conversion to graphviz dot format
#[cfg(feature = "dot_writer")]
pub(crate) mod dot;

/// Module that provides a FindMatches type
mod find_matches;
pub use find_matches::FindMatches;

/// Module that provides functions and types related to the `find_matches` function.
pub(crate) mod find_matches_impl;

/// Module that provides code generation for scanners.
mod generate;
pub use generate::generate;

/// Module for several ID types.
mod ids;
pub(crate) use ids::{ScannerModeID, TerminalID, TerminalIDBase};

pub(crate) use ids::{CharClassID, StateID, StateIDBase};

/// Module that provides functions and types related to match functions.
pub(crate) mod match_function;
pub(crate) use match_function::MatchFunction;

/// Module that provides a Match type
mod match_type;
pub use match_type::{Match, MatchExt};

/// Module that provides functions and types related to DFA minimization.
pub(crate) mod minimizer;

/// Module that provides a Pattern type and a Lookahead type
mod pattern;
pub use pattern::{Lookahead, Pattern};

/// Module that provides a peek result type
mod peek_result;
pub use peek_result::PeekResult;

/// Module that provides a position type
mod position;
pub use position::{Position, PositionProvider};

/// Module that provides functions and types related to the multi pattern NFA.
pub(crate) mod multi_pattern_nfa;
pub(crate) use multi_pattern_nfa::MultiPatternNfa;

/// The nfa module contains the NFA implementation.
mod nfa;
pub(crate) use nfa::Nfa;

/// The parser module contains the regex syntax parser.
mod parser;
pub(crate) use parser::parse_regex_syntax;

pub(crate) mod rust_code_formatter;

/// Module that provides functions and types related to NFA scanner implementations.
pub(crate) mod scanner_impl;
pub use scanner_impl::ScannerImpl;

/// Module that provides helper types to parse the scanner.
pub(crate) mod scanner_data;

/// The module with the scanner mode.
mod scanner_mode;
pub use scanner_mode::ScannerMode;

/// The module with the scanner mode switcher.
mod scanner_mode_switcher;
pub use scanner_mode_switcher::ScannerModeSwitcher;

/// Module that provides a Span type
mod span;
pub use span::Span;

/// Module that provides a WithPositions type
mod with_positions;
pub use with_positions::{MatchExtIterator, WithPositions};
