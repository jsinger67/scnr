use regex_syntax::ast::Ast;

use super::{ids::CharClassIDBase, CharClassID, CharacterClass, ComparableAst};
use crate::{internal::MatchFunction, Result, ScnrError};

/// CharacterClassRegistry is a registry of character classes.
#[derive(Debug, Clone, Default)]
pub(crate) struct CharacterClassRegistry {
    character_classes: Vec<CharacterClass>,
}

impl CharacterClassRegistry {
    /// Creates a new CharacterClassRegistry.
    pub(crate) fn new() -> Self {
        Self {
            character_classes: Vec::new(),
        }
    }

    /// Returns a slice of the character classes in the registry.
    /// It is used for debugging purposes.
    #[allow(unused)]
    pub(crate) fn character_classes(&self) -> &[CharacterClass] {
        &self.character_classes
    }

    /// Adds a character class to the registry if it is not already present and returns its ID.
    pub(crate) fn add_character_class(&mut self, ast: &Ast) -> CharClassID {
        let character_class = ComparableAst(ast.clone());
        if let Some(id) = self
            .character_classes
            .iter()
            .position(|cc| cc.ast == character_class)
        {
            CharClassID::new(id as CharClassIDBase)
        } else {
            let id = CharClassID::new(self.character_classes.len() as CharClassIDBase);
            self.character_classes
                .push(CharacterClass::new(id, character_class.0));
            id
        }
    }

    /// Returns the character class with the given ID.
    /// It is used for debugging purposes mostly in the [crate::internal::dot] module.
    #[allow(unused)]
    pub(crate) fn get_character_class(&self, id: CharClassID) -> Option<&CharacterClass> {
        self.character_classes.get(id.as_usize())
    }

    /// Returns the number of character classes in the registry.
    /// It is used for debugging purposes.
    #[allow(unused)]
    pub(crate) fn len(&self) -> usize {
        self.character_classes.len()
    }

    /// Returns true if the registry is empty.
    /// It is used for debugging purposes.
    #[allow(unused)]
    pub(crate) fn is_empty(&self) -> bool {
        self.character_classes.is_empty()
    }

    /// Creates a match function for the character classes in the registry.
    ///
    /// Safety:
    ///     The callers ensure that the character classes in the registry are valid.
    ///     All character classes in the registry are valid which is guaranteed by the construction
    ///     of the registry.
    pub(crate) fn create_match_char_class(
        &self,
    ) -> Result<Box<dyn (Fn(CharClassID, char) -> bool) + 'static + Send + Sync>> {
        let match_functions =
            self.character_classes
                .iter()
                .try_fold(Vec::new(), |mut acc, cc| {
                    // trace!("Create match function for char class {:?}", cc);
                    let match_function: MatchFunction = cc.ast().try_into()?;
                    acc.push(match_function);
                    Ok::<Vec<MatchFunction>, ScnrError>(acc)
                })?;
        Ok(Box::new(move |char_class, c| {
            // trace!("Match char class #{} '{}' -> {:?}", char_class.id(), c, res);
            unsafe { match_functions.get_unchecked(char_class.as_usize()).call(c) }
        }))
    }
}

impl std::fmt::Display for CharacterClassRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CharacterClassRegistry {{ len: {} }}",
            self.character_classes.len()
        )?;
        for cc in &self.character_classes {
            write!(f, "\n  {}", cc)?;
        }
        Ok(())
    }
}
