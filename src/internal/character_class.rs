use regex_syntax::ast::Ast;

use super::{CharClassID, ComparableAst};

/// A character class that can match a character.
#[derive(Default, Clone)]
pub(crate) struct CharacterClass {
    pub(crate) id: CharClassID,
    pub(crate) ast: ComparableAst,
}

impl CharacterClass {
    pub(crate) fn new(id: CharClassID, ast: Ast) -> Self {
        CharacterClass {
            id,
            ast: ComparableAst(ast),
        }
    }

    #[inline]
    pub(crate) fn id(&self) -> CharClassID {
        self.id
    }

    #[inline]
    pub(crate) fn ast(&self) -> &Ast {
        &self.ast.0
    }
}

impl std::fmt::Debug for CharacterClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CharacterClass {{ id: {:?}, ast: {:?} }}",
            self.id, self.ast
        )
    }
}

impl std::hash::Hash for CharacterClass {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.ast.hash(state);
        // Do not hash the match function, because it is not relevant for equality.
        // Actually it is calculated from the AST, so it would be redundant.
    }
}

impl PartialEq for CharacterClass {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.ast == other.ast
    }
}

impl Eq for CharacterClass {}

impl PartialOrd for CharacterClass {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl Ord for CharacterClass {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}
