/// A scanner mode that can be used to scan specific parts of the input.
/// The contained data is used to create a scanner mode that can be used to scan the input.
#[derive(Debug, Clone)]
pub struct ScannerMode {
    /// The name of the scanner mode.
    pub name: String,
    /// The regular expressions that are valid token types in this mode, bundled with their token
    /// type numbers.
    /// The priorities of the patterns are determined by their order in the vector. Lower indices
    /// have higher priority if multiple patterns match the input and have the same length.
    pub patterns: Vec<(String, usize)>,
}

impl ScannerMode {
    /// Creates a new scanner mode with the given name and patterns.
    pub fn new(name: &str, patterns: &[(String, usize)]) -> Self {
        let patterns = patterns.to_vec();
        Self {
            name: name.to_string(),
            patterns,
        }
    }
}
