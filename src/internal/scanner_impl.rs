use crate::{Result, ScannerMode};

use super::{CharacterClassRegistry, CompiledScannerMode};

#[derive(Debug)]
pub(crate) struct ScannerImpl {
    character_classes: CharacterClassRegistry,
    scanner_modes: Vec<CompiledScannerMode>,
}

impl TryFrom<Vec<ScannerMode>> for ScannerImpl {
    type Error = crate::ScnrError;
    fn try_from(_scanner_modes: Vec<ScannerMode>) -> Result<Self> {
        todo!()
    }
}
