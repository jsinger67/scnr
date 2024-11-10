/// Module that provides functions and types related to character classes.
mod character_class;
pub(crate) use character_class::CharacterClass;

/// Module that provides the type CharacterClassRegistry.
mod character_class_registry;
pub(crate) use character_class_registry::CharacterClassRegistry;

/// Module that provides functions and types related to compiled DFAs.
pub(crate) mod compiled_dfa;
pub(crate) use compiled_dfa::CompiledDfa;

/// Module that provides functions and types related to compiled Lookahead.
pub(crate) mod compiled_lookahead;
pub(crate) use compiled_lookahead::{CompiledLookahead, CompiledNfaLookahead};

/// Module that provides functions and types related to compiled NFA.
pub(crate) mod compiled_nfa;
// pub(crate) use compiled_nfa::CompiledNfa;

/// Module that provides functions and types related to compiled ScannerModes.
pub(crate) mod compiled_scanner_mode;
pub(crate) use compiled_scanner_mode::CompiledScannerMode;

/// Module that provides functions and types related to comparable ASTs.
pub(crate) mod comparable_ast;
pub(crate) use comparable_ast::ComparableAst;

/// Module that provides functions and types related to DFAs.
pub(crate) mod dfa;

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

/// Module that provides functions and types related to NFA scanner implementations.
pub(crate) mod scanner_nfa_impl;
pub(crate) use scanner_nfa_impl::ScannerNfaImpl;
