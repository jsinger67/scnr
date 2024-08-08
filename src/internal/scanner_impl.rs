use crate::{Result, ScannerMode, ScnrError};

use super::{CharClassID, CharacterClassRegistry, CompiledScannerMode, MatchFunction};

pub(crate) struct ScannerImpl {
    character_classes: CharacterClassRegistry,
    scanner_modes: Vec<CompiledScannerMode>,
    match_char_class: Box<dyn Fn(CharClassID, char) -> bool + 'static>,
}

impl ScannerImpl {
    pub(crate) fn character_classes(&self) -> &CharacterClassRegistry {
        &self.character_classes
    }

    pub(crate) fn scanner_modes(&self) -> &[CompiledScannerMode] {
        &self.scanner_modes
    }

    pub(crate) fn match_char_class(&self, char_class: CharClassID, c: char) -> bool {
        (self.match_char_class)(char_class, c)
    }

    fn create_match_char_class(&mut self) -> Result<()> {
        let match_functions =
            self.character_classes
                .iter()
                .try_fold(Vec::new(), |mut acc, cc| {
                    let match_function: MatchFunction = cc.ast().try_into()?;
                    acc.push(match_function);
                    Ok::<Vec<MatchFunction>, ScnrError>(acc)
                })?;
        self.match_char_class =
            Box::new(move |char_class, c| match_functions[char_class.as_usize()].call(c));
        Ok(())
    }
}

impl TryFrom<Vec<ScannerMode>> for ScannerImpl {
    type Error = crate::ScnrError;
    fn try_from(scanner_modes: Vec<ScannerMode>) -> Result<Self> {
        let mut character_classes = CharacterClassRegistry::new();
        let scanner_modes =
            scanner_modes
                .into_iter()
                .try_fold(Vec::new(), |mut acc, scanner_mode| {
                    acc.push(CompiledScannerMode::try_from_scanner_mode(
                        scanner_mode,
                        &mut character_classes,
                    )?);
                    Ok::<Vec<CompiledScannerMode>, ScnrError>(acc)
                })?;
        let mut me = Self {
            character_classes,
            scanner_modes,
            match_char_class: Box::new(|_, _| false),
        };
        me.create_match_char_class()?;
        Ok(me)
    }
}

impl std::fmt::Debug for ScannerImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScannerImpl")
            .field("character_classes", &self.character_classes)
            .field("scanner_modes", &self.scanner_modes)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScannerMode;
    use std::convert::TryInto;

    #[test]
    fn test_try_from() {
        let scanner_modes = vec![
            ScannerMode::new("mode1", vec![("a".to_string(), 0)], vec![]),
            ScannerMode::new("mode2", vec![("b".to_string(), 1)], vec![]),
        ];
        let scanner_impl: ScannerImpl = scanner_modes.try_into().unwrap();
        assert_eq!(scanner_impl.character_classes.len(), 2);
        assert_eq!(scanner_impl.scanner_modes.len(), 2);
    }

    #[test]
    fn test_match_char_class() {
        let scanner_modes = vec![
            ScannerMode::new("mode1", vec![("a".to_string(), 0)], vec![]),
            ScannerMode::new("mode2", vec![("b".to_string(), 1)], vec![]),
        ];
        let scanner_impl: ScannerImpl = scanner_modes.try_into().unwrap();
        assert!(scanner_impl.match_char_class(0.into(), 'a'));
        assert!(!scanner_impl.match_char_class(0.into(), 'b'));
        assert!(!scanner_impl.match_char_class(0.into(), 'c'));
        assert!(!scanner_impl.match_char_class(1.into(), 'a'));
        assert!(scanner_impl.match_char_class(1.into(), 'b'));
        assert!(!scanner_impl.match_char_class(1.into(), 'c'));
    }
}
