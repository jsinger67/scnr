use super::ScannerImpl;

/// An iterator over all non-overlapping matches.
#[derive(Debug)]
pub(crate) struct FindMatchesImpl<'h> {
    // The scanner used to find matches.
    _scanner: ScannerImpl,
    // The input haystack.
    _char_indices: std::str::CharIndices<'h>,
}
