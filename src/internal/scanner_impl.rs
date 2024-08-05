use crate::{Result, ScannerMode};

use super::CharacterClassRegistry;

#[derive(Debug)]
pub(crate) struct ScannerImpl {
    character_classes: CharacterClassRegistry,
}

impl TryFrom<Vec<ScannerMode>> for ScannerImpl {
    type Error = crate::ScnrError;
    fn try_from(_scanner_modes: Vec<ScannerMode>) -> Result<Self> {
        todo!()
    }
}
