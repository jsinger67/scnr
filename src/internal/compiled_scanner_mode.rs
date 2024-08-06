use super::{CompiledDfa, TerminalID};

/// A compiled scanner mode that can be used to scan a string.
#[derive(Debug)]
pub(crate) struct CompiledScannerMode {
    /// The name of the scanner mode.
    name: String,
    /// The regular expressions that are valid token types in this mode, bundled with their token
    /// type numbers.
    /// The priorities of the patterns are determined by their order in the vector. Lower indices
    /// have higher priority if multiple patterns match the input and have the same length.
    patterns: Vec<(CompiledDfa, TerminalID)>,
}
