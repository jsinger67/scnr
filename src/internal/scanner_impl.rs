use crate::{Result, ScannerMode};

#[derive(Debug)]
pub(crate) struct ScannerImpl;

impl TryFrom<Vec<ScannerMode>> for ScannerImpl {
    type Error = crate::ScnrError;
    fn try_from(_scanner_modes: Vec<ScannerMode>) -> Result<Self> {
        todo!()
    }
}
