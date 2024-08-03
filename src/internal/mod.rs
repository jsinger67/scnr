/// Module that provides functions and types related to character classes.
mod character_class;
pub(crate) use character_class::CharacterClass;

/// Module that provides functions and types related to comparable ASTs.
pub(crate) mod comparable_ast;
pub(crate) use comparable_ast::ComparableAst;

/// Module that provides functions and types related to DFAs.
pub(crate) mod dfa;
pub(crate) use dfa::Dfa;

/// Module that provides functions and types related to the `find_matches` function.
pub(crate) mod find_matches_impl;

/// Module for sevearl ID types.
mod ids;
pub(crate) use ids::{CharClassID, PatternID, StateID};

/// Module that provides functions and types related to match functions.
pub(crate) mod match_function;

/// Module that provides functions and types related to matching states.
pub(crate) mod matching_state;

/// The nfa module contains the NFA implementation.
mod nfa;
pub(crate) use nfa::Nfa;

/// The parser module contains the regex syntax parser.
mod parser;
pub(crate) use parser::parse_regex_syntax;

/// Module that provides functions and types related to scanner implementations.
pub(crate) mod scanner_impl;
pub(crate) use scanner_impl::ScannerImpl;
