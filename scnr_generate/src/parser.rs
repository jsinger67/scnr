//! This module contains the parser for the regex syntax.
//! The parser is used to parse the regex syntax into an abstract syntax tree (AST).
//! We use the `regex_syntax` crate to parse the regex syntax, although we will only support a
//! subset of the regex syntax.

use crate::Result;
use log::trace;
use std::time::Instant;

/// Parse the regex syntax into an high-level intermediate representation (HIR).
/// The function returns an error if the regex syntax is invalid.
/// # Arguments
/// * `input` - A string slice that holds the regex syntax.
/// # Returns
/// An `Hir` that represents the high-level intermediate representation of the regex syntax.
/// # Errors
/// An error is returned if the regex syntax is invalid.
pub(crate) fn parse_regex_syntax(input: &str) -> Result<regex_syntax::hir::Hir> {
    let now = Instant::now();
    match regex_syntax::parse(input) {
        Ok(hir) => {
            let elapsed_time = now.elapsed();
            trace!("Parsing took {} milliseconds.", elapsed_time.as_millis());
            Ok(hir)
        }
        Err(e) => Err(e.into()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_regex_syntax_valid() {
        // Valid regex syntax
        let input = r"\d";
        let hir = parse_regex_syntax(input).unwrap();
        // Add assertions here to validate the AST
        eprintln!("Parsed HIR: {:?}", hir.clone().into_kind());
        assert!(matches!(
            hir.into_kind(),
            regex_syntax::hir::HirKind::Class(_)
        ));
    }

    #[test]
    #[should_panic(expected = "RegexSyntaxError(Parse(Error { kind: ClassUnclosed")]
    fn test_parse_regex_syntax_invalid() {
        // Invalid regex syntax
        let input = r"^\d{4}-\d{2}-\d{2}$[";
        let _ = parse_regex_syntax(input).unwrap();
    }

    #[test]
    fn test_parse_regex_syntax_empty() {
        // Empty regex syntax
        let input = "";
        let result = parse_regex_syntax(input);
        assert!(result.is_ok());
    }

    // This may hinder the use of the regex_syntax crate because it does not support lookaround
    // assertions. We'll have to evaluate if we can live with this limitation.
    #[test]
    #[should_panic(expected = "RegexSyntaxError(Parse(Error { kind: UnsupportedLookAround")]
    fn test_a_only_if_followed_by_b() {
        // Scanner syntax that matches 'a' only if it is followed by 'b'
        let input = r"a(?=b)";
        let _ = parse_regex_syntax(input).unwrap();
    }
}
