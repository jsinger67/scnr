/// Module that provides functions and types related to character classes.
#[cfg(not(feature = "regex_automata"))]
mod character_class;
#[cfg(not(feature = "regex_automata"))]
pub(crate) use character_class::CharacterClass;

/// Module that provides the type CharacterClassRegistry.
#[cfg(not(feature = "regex_automata"))]
mod character_class_registry;
#[cfg(not(feature = "regex_automata"))]
pub(crate) use character_class_registry::CharacterClassRegistry;

/// Module that provides functions and types related to compiled Lookahead.
#[cfg(not(feature = "regex_automata"))]
pub(crate) mod compiled_lookahead;
// pub(crate) use compiled_lookahead::{CompiledLookahead, CompiledDfaLookahead};
#[cfg(not(feature = "regex_automata"))]
pub(crate) use compiled_lookahead::CompiledLookahead;

/// Module that provides functions and types related to compiled NFA.
#[cfg(not(feature = "regex_automata"))]
pub(crate) mod compiled_dfa;
// pub(crate) use compiled_dfa::CompiledDfa;

/// Module that provides functions and types related to compiled ScannerModes.
#[cfg(not(feature = "regex_automata"))]
pub(crate) mod compiled_scanner_mode;

/// Module that provides functions and types related to comparable ASTs.
#[cfg(not(feature = "regex_automata"))]
pub(crate) mod comparable_ast;
#[cfg(not(feature = "regex_automata"))]
pub(crate) use comparable_ast::ComparableAst;

/// Module with conversion to graphviz dot format
#[cfg(not(feature = "regex_automata"))]
pub(crate) mod dot;

/// Module that provides functions and types related to the `find_matches` function.
pub(crate) mod find_matches_impl;

/// Module for several ID types.
mod ids;
pub(crate) use ids::{ScannerModeID, TerminalID, TerminalIDBase};

#[cfg(not(feature = "regex_automata"))]
pub(crate) use ids::{CharClassID, StateID, StateIDBase};

/// Module that provides functions and types related to match functions.
#[cfg(not(feature = "regex_automata"))]
pub(crate) mod match_function;
#[cfg(not(feature = "regex_automata"))]
pub(crate) use match_function::MatchFunction;

/// Module that provides functions and types related to DFA minimization.
#[cfg(not(feature = "regex_automata"))]
pub(crate) mod minimizer;

/// Module that provides functions and types related to the multi pattern NFA.
#[cfg(not(feature = "regex_automata"))]
pub(crate) mod multi_pattern_nfa;
#[cfg(not(feature = "regex_automata"))]
pub(crate) use multi_pattern_nfa::MultiPatternNfa;

/// The nfa module contains the NFA implementation.
#[cfg(not(feature = "regex_automata"))]
mod nfa;
#[cfg(not(feature = "regex_automata"))]
pub(crate) use nfa::Nfa;

/// The parser module contains the regex syntax parser.
#[cfg(not(feature = "regex_automata"))]
mod parser;
#[cfg(not(feature = "regex_automata"))]
pub(crate) use parser::parse_regex_syntax;

mod scanner_cache;
pub(crate) use scanner_cache::SCANNER_CACHE;

/// Module that provides functions and types related to NFA scanner implementations.
#[cfg(not(feature = "regex_automata"))]
pub(crate) mod scanner_impl;
#[cfg(not(feature = "regex_automata"))]
pub(crate) use scanner_impl::ScannerImpl;

#[cfg(feature = "regex_automata")]
pub(crate) mod scanner_impl_rx;
#[cfg(feature = "regex_automata")]
pub(crate) use scanner_impl_rx::ScannerImpl;
