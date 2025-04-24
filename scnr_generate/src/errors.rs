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
    RegexSyntaxError(regex_syntax::Error, String),

    /// An error occurred during the parsing of the regex syntax.
    #[error("'{1}' {0}")]
    RegexSyntaxAstError(regex_syntax::ast::Error, String),

    /// An error occurred during conversion to the regex HIR (high-level intermediate representation).
    #[error("'{1}' {0}")]
    RegexSyntaxHirError(regex_syntax::hir::Error, String),

    /// A std::io error occurred.
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// Used regex features that are not supported (yet).
    #[error("Unsupported regex feature: {0}")]
    UnsupportedFeature(String),

    /// An error occurred during the conversion of a regex syntax to a regex HIR.
    #[error(transparent)]
    InvalidUtf8(#[from] core::str::Utf8Error),

    /// An empty tokens is matched. This leads to an infinite loop. Avoid regexes that match empty
    /// tokens.
    #[error("Empty tokens are not allowed.")]
    EmptyToken,
}

impl From<regex_syntax::Error> for ScnrError {
    fn from(error: regex_syntax::Error) -> Self {
        ScnrError::new(ScnrErrorKind::RegexSyntaxError(error, "!".to_string()))
    }
}

impl From<regex_syntax::ast::Error> for ScnrError {
    fn from(error: regex_syntax::ast::Error) -> Self {
        ScnrError::new(ScnrErrorKind::RegexSyntaxAstError(error, "!".to_string()))
    }
}

impl From<std::io::Error> for ScnrError {
    fn from(error: std::io::Error) -> Self {
        ScnrError::new(ScnrErrorKind::IoError(error))
    }
}
