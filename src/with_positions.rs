use crate::{scanner::ScannerModeSwitcher, Match, MatchExt, Position, PositionProvider};

/// An iterator over all non-overlapping matches with positions.
#[derive(Debug)]
pub struct WithPositions<I> {
    iter: I,
}

impl<I> WithPositions<I>
where
    I: Iterator<Item = Match> + PositionProvider + Sized,
{
    /// Create a new `WithPositions` iterator.
    pub(crate) fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<I> Iterator for WithPositions<I>
where
    I: Iterator<Item = Match> + PositionProvider + Sized,
{
    type Item = MatchExt;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|m| {
            let start_positon = self.iter.position(m.start());
            let end_position = self.iter.position(m.end());
            MatchExt::new(m.token_type(), m.span(), start_positon, end_position)
        })
    }
}

impl<I> ScannerModeSwitcher for WithPositions<I>
where
    I: ScannerModeSwitcher,
{
    fn set_mode(&mut self, mode: usize) {
        self.iter.set_mode(mode);
    }

    fn current_mode(&self) -> usize {
        self.iter.current_mode()
    }

    fn mode_name(&self, index: usize) -> Option<&str> {
        self.iter.mode_name(index)
    }
}

/// An extension trait for iterators over matches.
pub trait MatchExtIterator: Iterator<Item = Match> + PositionProvider + Sized {
    /// An iterator that yields matches with positions.
    fn with_positions(self) -> WithPositions<Self> {
        WithPositions::new(self)
    }
}

// Implement the trait for all types that implement the required traits.
impl<I: Iterator<Item = Match> + PositionProvider + Sized> MatchExtIterator for I {}

impl<I> PositionProvider for WithPositions<I>
where
    I: Iterator<Item = Match> + PositionProvider,
{
    fn position(&self, offset: usize) -> Position {
        self.iter.position(offset)
    }

    fn set_offset(&mut self, offset: usize) {
        self.iter.set_offset(offset);
    }
}
