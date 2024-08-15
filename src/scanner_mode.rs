use crate::internal::{ScannerModeID, TerminalID, TerminalIDBase};
use serde::{Deserialize, Serialize};

/// A scanner mode that can be used to scan specific parts of the input.
/// The contained data is used to create a scanner mode that can be used to scan the input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScannerMode {
    /// The name of the scanner mode.
    pub(crate) name: String,
    /// The regular expressions that are valid token types in this mode, bundled with their token
    /// type numbers.
    /// The priorities of the patterns are determined by their order in the vector. Lower indices
    /// have higher priority if multiple patterns match the input and have the same length.
    pub(crate) patterns: Vec<(String, TerminalID)>,

    /// The transitions between the scanner modes triggered by a token type number.
    /// The entries are tuples of the token type numbers and the new scanner mode index and are
    /// sorted by token type number.
    pub(crate) transitions: Vec<(TerminalID, ScannerModeID)>,
}

impl ScannerMode {
    /// Creates a new scanner mode with the given name and patterns.
    /// # Arguments
    /// * `name` - The name of the scanner mode.
    /// * `patterns` - The regular expressions that are valid token types in this mode, bundled with
    ///     their token type numbers.
    /// * `mode_transitions` - The transitions between the scanner modes triggered by a token type
    ///     number. It is a vector of tuples of the token type numbers and the new scanner mode
    ///     index. The entries should be are sorted by token type number.
    ///     The scanner mode index is the index of the scanner mode in the scanner mode vector of
    ///     the scanner and is determined by the order of the insertions of scanner modes into the
    ///     scanner.
    /// # Returns
    /// The new scanner mode.
    pub fn new<P, S, T>(name: &str, patterns: P, mode_transitions: T) -> Self
    where
        P: IntoIterator<Item = (S, usize)>,
        S: AsRef<str>,
        T: IntoIterator<Item = (usize, usize)>,
    {
        let patterns = patterns
            .into_iter()
            .map(|(p, t)| (p.as_ref().to_string(), TerminalID::new(t as TerminalIDBase)))
            .collect();
        let transitions = mode_transitions
            .into_iter()
            .map(|(t, m)| (TerminalID::new(t as TerminalIDBase), ScannerModeID::new(m)))
            .collect::<Vec<_>>();
        debug_assert!(
            transitions.windows(2).all(|w| w[0].0 < w[1].0),
            "Transitions are not sorted by token type number."
        );
        Self {
            name: name.to_string(),
            patterns,
            transitions,
        }
    }

    /// Returns the name of the scanner mode.
    pub fn name(&self) -> &str {
        &self.name
    }
}
