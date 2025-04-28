use syn::braced;

use crate::{Pattern, TerminalID, TerminalIDBase};

macro_rules! parse_ident {
    ($input:ident, $name:ident) => {
        $input.parse().map_err(|e| {
            syn::Error::new(
                e.span(),
                concat!("expected identifier `", stringify!($name), "`"),
            )
        })?
    };
}

#[derive(Debug)]
pub(crate) struct ScannerModeWithNamedTransitions {
    /// The name of the scanner mode.
    pub(crate) name: String,
    /// The regular expressions that are valid token types in this mode, bundled with their token
    /// type numbers.
    /// The priorities of the patterns are determined by their order in the vector. Lower indices
    /// have higher priority if multiple patterns match the input and have the same length.
    pub(crate) patterns: Vec<Pattern>,

    /// The transitions between the scanner modes triggered by a token type number.
    /// The entries are tuples of the token type numbers and the new scanner mode name and are
    /// sorted by token type number.
    pub(crate) transitions: Vec<(TerminalID, String)>,
}

/// This is used to create a scanner mode from a part of a macro input.
/// The macro input looks like this:
/// ```text
/// mode INITIAL {
///     token r"\r\n|\r|\n" => 1;
///     token r"[\s--\r\n]+" => 2;
///     token r"//.*(\r\n|\r|\n)?" => 3;
///     token r"/\*([^*]|\*[^/])*\*/" => 4;
///     token r#"""# => 8;
///     token r"Hello" => 9;
///     token r"World" => 10;
///     token r"World" => 11 with lookahead positive r"!";
///     token r"!" => 12;
///     token r"[a-zA-Z_]\w*" => 13;
///     token r"." => 14;
///
///     transition 8 => STRING;
/// }
/// ```
/// where there must be at least one token entries which are parsed with the help of the `Pattern`
/// struct's `parse` method. Zero or more `transition` entries can exist.
/// The `transition` entries are tuples of the token type numbers and the new scanner mode name.
/// The scanner mode name is later converted to the scanner mode ID and the transitions are sorted
/// by token type number.
///
impl syn::parse::Parse for ScannerModeWithNamedTransitions {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mode: syn::Ident = parse_ident!(input, mode);
        let mode = mode.to_string();
        if mode != "mode" {
            return Err(input.error("expected 'mode'"));
        }
        let name: syn::Ident = parse_ident!(input, mode_name);
        let name = name.to_string();
        if name.is_empty() {
            return Err(input.error("expected a mode name"));
        }
        let content;
        braced!(content in input);
        let mut patterns = Vec::new();
        let mut transitions = Vec::new();
        while !content.is_empty() {
            let token_or_transition: syn::Ident = parse_ident!(content, token_or_transition);
            let token_or_transition = token_or_transition.to_string();
            if token_or_transition == "token" {
                let pattern: Pattern = content.parse()?;
                patterns.push(pattern);
            } else if token_or_transition == "transition" {
                let token_type: syn::LitInt = content.parse()?;
                let token_type = token_type.base10_parse::<TerminalIDBase>()?;
                let _: syn::Token![=>] = content.parse()?;
                let new_mode: syn::Ident = parse_ident!(content, new_mode);
                let new_mode = new_mode.to_string();
                if new_mode.is_empty() {
                    return Err(content.error("expected a mode name"));
                }
                transitions.push((token_type.into(), new_mode));
                // Parse the semicolon at the end of the transition.
                if content.peek(syn::Token![;]) {
                    content.parse::<syn::Token![;]>()?;
                } else {
                    return Err(content.error("expected ';'"));
                }
            } else {
                return Err(content.error("expected 'token' or 'transition'"));
            }
        }
        Ok(ScannerModeWithNamedTransitions {
            name,
            patterns,
            transitions,
        })
    }
}

#[derive(Debug)]
pub(crate) struct ScannerData {
    /// The scanner name.
    pub name: String,
    /// The scanner modes.
    pub modes: Vec<ScannerModeWithNamedTransitions>,
}

/// This is used to create a scanner from a part of a macro input.
/// The macro input looks like this:
/// ```text
/// scanner HelloWorld {
///     // One or more scanner modes
/// }
impl syn::parse::Parse for ScannerData {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let scanner: syn::Ident = parse_ident!(input, scanner);
        let scanner = scanner.to_string();
        if scanner != "scanner" {
            return Err(input.error("expected 'scanner'"));
        }
        let name: syn::Ident = parse_ident!(input, scanner_name);
        let name = name.to_string();
        if name.is_empty() {
            return Err(input.error("expected a scanner name"));
        }
        let content;
        braced!(content in input);
        let mut modes = Vec::new();
        // Parse at least one scanner mode
        let initial_mode: ScannerModeWithNamedTransitions = content.parse()?;
        modes.push(initial_mode);
        while !content.is_empty() {
            let mode: ScannerModeWithNamedTransitions = content.parse()?;
            modes.push(mode);
        }
        Ok(ScannerData { name, modes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scanner_data() {
        let input = quote::quote! {
            scanner HelloWorld {
                mode INITIAL {
                    token r"\r\n|\r|\n" => 1;
                    token r"[\s--\r\n]+" => 2;
                    token r"//.*(\r\n|\r|\n)?" => 3;
                    token r"/\*([^*]|\*[^/])*\*/" => 4;
                    token r#"""# => 8;
                    token r"Hello" => 9;
                    token r"World" => 10;
                    token r"World" => 11 with lookahead positive r"!";
                    token r"!" => 12;
                    token r"[a-zA-Z_]\w*" => 13;
                    token r"." => 14;

                    transition 8 => STRING;
                }
                mode STRING {
                    token r#"\\[\"\\bfnt]"# => 5;
                    token r"\\[\s--\n\r]*\r?\n" => 6;
                    token r#"[^\"\]+"# => 7;
                    token r#"""# => 8;
                    token r"." => 14;

                    transition 8 => INITIAL;
                }
            }
        };
        let scanner_data: ScannerData = syn::parse2(input).unwrap();
        assert_eq!(scanner_data.name, "HelloWorld");
        assert_eq!(scanner_data.modes.len(), 2);
        let mode_initial = &scanner_data.modes[0];
        assert_eq!(mode_initial.name, "INITIAL");
        assert_eq!(mode_initial.patterns.len(), 11);
        assert_eq!(mode_initial.transitions.len(), 1);
        assert_eq!(mode_initial.transitions[0].0, 8.into());
        assert_eq!(mode_initial.transitions[0].1, "STRING");
        let mode_initial_patterns = &mode_initial.patterns;
        assert_eq!(mode_initial_patterns[0].pattern(), r"\r\n|\r|\n");
        assert_eq!(mode_initial_patterns[1].pattern(), r"[\s--\r\n]+");
        assert_eq!(mode_initial_patterns[2].pattern(), r"//.*(\r\n|\r|\n)?");
        assert_eq!(mode_initial_patterns[3].pattern(), r"/\*([^*]|\*[^/])*\*/");
        assert_eq!(mode_initial_patterns[4].pattern(), r#"""#);
        assert_eq!(mode_initial_patterns[5].pattern(), r"Hello");
        assert_eq!(mode_initial_patterns[6].pattern(), r"World");
        assert_eq!(mode_initial_patterns[7].pattern(), r"World");
        assert_eq!(mode_initial_patterns[8].pattern(), r"!");
        assert_eq!(mode_initial_patterns[9].pattern(), r"[a-zA-Z_]\w*");
        assert_eq!(mode_initial_patterns[10].pattern(), r".");
        assert_eq!(mode_initial_patterns[0].terminal_id(), 1);
        assert_eq!(mode_initial_patterns[1].terminal_id(), 2);
        assert_eq!(mode_initial_patterns[2].terminal_id(), 3);
        assert_eq!(mode_initial_patterns[3].terminal_id(), 4);
        assert_eq!(mode_initial_patterns[4].terminal_id(), 8);
        assert_eq!(mode_initial_patterns[5].terminal_id(), 9);
        assert_eq!(mode_initial_patterns[6].terminal_id(), 10);
        assert_eq!(mode_initial_patterns[7].terminal_id(), 11);
        assert_eq!(mode_initial_patterns[8].terminal_id(), 12);
        assert_eq!(mode_initial_patterns[9].terminal_id(), 13);
        assert_eq!(mode_initial_patterns[10].terminal_id(), 14);
        assert_eq!(mode_initial_patterns[0].lookahead(), None);
        assert_eq!(mode_initial_patterns[1].lookahead(), None);
        assert_eq!(mode_initial_patterns[2].lookahead(), None);
        assert_eq!(mode_initial_patterns[3].lookahead(), None);
        assert_eq!(mode_initial_patterns[4].lookahead(), None);
        assert_eq!(mode_initial_patterns[5].lookahead(), None);
        assert_eq!(mode_initial_patterns[6].lookahead(), None);
        assert_eq!(
            mode_initial_patterns[7].lookahead(),
            Some(crate::Lookahead {
                is_positive: true,
                pattern: "!".to_string(),
            })
            .as_ref()
        );
        assert_eq!(mode_initial_patterns[8].lookahead(), None);
        assert_eq!(mode_initial_patterns[9].lookahead(), None);
        assert_eq!(mode_initial_patterns[10].lookahead(), None);

        let mode_string = &scanner_data.modes[1];
        assert_eq!(mode_string.name, "STRING");
        assert_eq!(mode_string.patterns.len(), 5);
        assert_eq!(mode_string.transitions.len(), 1);
        assert_eq!(mode_string.transitions[0].0, 8.into());
        assert_eq!(mode_string.transitions[0].1, "INITIAL");
        let mode_string_patterns = &mode_string.patterns;
        assert_eq!(mode_string_patterns[0].pattern(), r#"\\[\"\\bfnt]"#);
        assert_eq!(mode_string_patterns[1].pattern(), r"\\[\s--\n\r]*\r?\n");
        assert_eq!(mode_string_patterns[2].pattern(), r#"[^\"\]+"#);
        assert_eq!(mode_string_patterns[3].pattern(), r#"""#);
        assert_eq!(mode_string_patterns[4].pattern(), r".");
        assert_eq!(mode_string_patterns[0].terminal_id(), 5);
        assert_eq!(mode_string_patterns[1].terminal_id(), 6);
        assert_eq!(mode_string_patterns[2].terminal_id(), 7);
        assert_eq!(mode_string_patterns[3].terminal_id(), 8);
        assert_eq!(mode_string_patterns[4].terminal_id(), 14);
        assert_eq!(mode_string_patterns[0].lookahead(), None);
        assert_eq!(mode_string_patterns[1].lookahead(), None);
        assert_eq!(mode_string_patterns[2].lookahead(), None);
        assert_eq!(mode_string_patterns[3].lookahead(), None);
        assert_eq!(mode_string_patterns[4].lookahead(), None);
    }
}
