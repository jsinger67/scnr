use crate::{scanner::Scanner, scanner_mode::ScannerMode, Result, ScannerImpl};

/// A builder for creating a scanner.
#[derive(Debug, Clone, Default)]
pub struct ScannerBuilder {
    scanner_modes: Vec<ScannerMode>,
}

impl ScannerBuilder {
    /// Creates a new scanner builder.
    pub fn new() -> Self {
        Self {
            scanner_modes: Vec::new(),
        }
    }

    /// Adds a scanner mode to the scanner builder.
    pub fn add_scanner_mode(mut self, scanner_mode: ScannerMode) -> Self {
        self.scanner_modes.push(scanner_mode);
        self
    }

    /// Adds multiple scanner modes to the scanner builder.
    pub fn add_scanner_modes(mut self, scanner_modes: &[ScannerMode]) -> Self {
        self.scanner_modes.extend_from_slice(scanner_modes);
        self
    }

    /// Builds the scanner from the scanner builder.
    pub fn build(self) -> Result<Scanner> {
        Ok(Scanner {
            inner: ScannerImpl::try_from(self.scanner_modes)?,
        })
    }
}
