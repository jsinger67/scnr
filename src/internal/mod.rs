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

/// Module that provides functions and types related to comparable ASTs.
pub(crate) mod comparable_ast;
pub(crate) use comparable_ast::ComparableAst;

/// Module with conversion to graphviz dot format
pub(crate) mod dot;

/// Module that provides functions and types related to the `find_matches` function.
pub(crate) mod find_matches_impl;

/// Module for sevearl ID types.
mod ids;
pub(crate) use ids::{
    CharClassID, ScannerModeID, StateID, StateIDBase, TerminalID, TerminalIDBase,
};

/// Module that provides functions and types related to match functions.
pub(crate) mod match_function;
pub(crate) use match_function::MatchFunction;

/// Module that provides functions and types related to DFA minimization.
pub(crate) mod minimizer;

/// Module that provides functions and types related to the multi pattern NFA.
pub(crate) mod multi_pattern_nfa;
pub(crate) use multi_pattern_nfa::MultiPatternNfa;

/// The nfa module contains the NFA implementation.
mod nfa;
pub(crate) use nfa::Nfa;

/// The parser module contains the regex syntax parser.
mod parser;
pub(crate) use parser::parse_regex_syntax;

mod scanner_cache;
pub(crate) use scanner_cache::SCANNER_CACHE;

/// Module that provides functions and types related to NFA scanner implementations.
pub(crate) mod scanner_impl;
pub(crate) use scanner_impl::ScannerImpl;
