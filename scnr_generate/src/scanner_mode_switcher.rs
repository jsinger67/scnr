/// A trait to switch between scanner modes.
///
/// This trait is used to switch between different scanner modes from a parser's perspective.
/// The parser can set the current scanner mode to switch to a different set of Finite State
/// Machines, FSMs.
/// Usually, the scanner mode is changed by the scanner itself based on the transitions defined
/// in the active scanner mode.
///
/// It is discouraged to use a mix of scanner induced mode changes and parser induced mode changes.
/// This can lead to unexpected behavior and is not recommended.
///
/// Only several kinds of parsers are able to handle mode changes as part of the grammar.
/// Note that 'part of the grammar' means that the mode changes are part of the parser's state
/// machine and not anything implemented in semantic actions.
///
/// For example, an LL parser is able to handle scanner mode switches in the grammar because it
/// always 'knows' the next production to parse. If the production contains a scanner mode switch,
/// the parser can switch the scanner mode before scanning the next token.
///
/// A LR parser is not able to handle mode changes in the grammar because it does not know the next
/// production to parse. The parser has to decide which production to parse based on the lookahead
/// tokens. If the lookahead tokens contain a token that needed a scanner mode switch, the parser
/// is not able to switch the scanner mode before reading the next token.
///
/// An example of a parser induced mode changes is the `parol` parser generator. It is able to
/// handle mode changes in the grammar because it generates LL parsers. The parser is able to switch
/// the scanner mode before scanning the next token.
/// `parol` is also able to handle scanner induced mode changes stored as transitions in the scanner
/// modes. The scanner mode changes are then no part of the grammar but instead part of the scanner.
///
/// Furthermore `parol` can also generate LR parsers. In this case, the scanner mode changes are not
/// part of the grammar but instead part of the scanner. `parol` will prevent the LR grammar from
/// containing scanner mode changes.
///
/// See <https://github.com/jsinger67/parol> for more information about the `parol` parser
/// generator.
pub trait ScannerModeSwitcher {
    /// Sets the current scanner mode.
    fn set_mode(&mut self, mode: usize);
    /// Returns the current scanner mode.
    fn current_mode(&self) -> usize;
    /// Returns the name of the scanner mode with the given index.
    fn mode_name(&self, index: usize) -> Option<&str>;
}
