use super::ScannerImpl;

#[derive(Debug)]
pub(crate) struct FindMatchesImpl<'h> {
    _scanner: ScannerImpl,
    _char_indices: std::str::CharIndices<'h>,
}
