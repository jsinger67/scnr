use thiserror::Error;

/// The result type for the `scrn` crate.
pub type Result<T> = std::result::Result<T, ScnrError>;

/// The error type for the `scrn` crate.
#[derive(Error, Debug)]
pub struct ScnrError {
    /// The source of the error.
    pub source: Box<ScnrErrorKind>,
}

impl ScnrError {
    /// Create a new `ScnrError`.
    pub fn new(kind: ScnrErrorKind) -> Self {
        ScnrError {
            source: Box::new(kind),
        }
    }
}

impl std::fmt::Display for ScnrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.source)
    }
}

/// The error kind type.
#[derive(Error, Debug)]
pub enum ScnrErrorKind {
    /// An error occurred during the parsing of the regex syntax.
    #[error("'{1}' {0}")]
    RegexSyntaxError(regex_syntax::ast::Error, String),

    /// An error occurred during the parsing of the regex syntax.
    #[cfg(feature = "regex_automata")]
    #[error(transparent)]
    RegexBuildError(#[from] regex_automata::dfa::dense::BuildError),

    /// A std::io error occurred.
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Used regex features that are not supported (yet).
    #[error("Unsupported regex feature: {0}")]
    UnsupportedFeature(String),

    /// An empty tokens is matched. This leads to an infinite loop. Avoid regexes that match empty
    /// tokens.
    #[error("Empty tokens are not allowed.")]
    EmptyToken,
}

impl From<regex_syntax::ast::Error> for ScnrError {
    fn from(error: regex_syntax::ast::Error) -> Self {
        ScnrError::new(ScnrErrorKind::RegexSyntaxError(error, "!".to_string()))
    }
}

impl From<std::io::Error> for ScnrError {
    fn from(error: std::io::Error) -> Self {
        ScnrError::new(ScnrErrorKind::IoError(error))
    }
}
