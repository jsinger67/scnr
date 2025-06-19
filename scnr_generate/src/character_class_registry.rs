use super::{ids::CharClassIDBase, CharClassID, CharacterClass, HirWithPattern};
use crate::{DisjointCharClassID, MatchFunction, Result, ScnrError};

/// CharacterClassRegistry is a registry of character classes.
#[derive(Debug, Clone, Default)]
pub(crate) struct CharacterClassRegistry {
    character_classes: Vec<CharacterClass>,
    elementary_intervals: Vec<(std::ops::RangeInclusive<char>, Vec<CharClassID>)>,
}

impl CharacterClassRegistry {
    /// Creates a new CharacterClassRegistry.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Returns a slice of the character classes in the registry.
    /// It is used for debugging purposes.
    #[allow(unused)]
    pub(crate) fn character_classes(&self) -> &[CharacterClass] {
        &self.character_classes
    }

    /// Adds a character class to the registry if it is not already present and returns its ID.
    pub(crate) fn add_character_class(&mut self, hir: &regex_syntax::hir::Hir) -> CharClassID {
        let hir_with_pattern = HirWithPattern::new(hir.clone());
        if let Some(id) = self
            .character_classes
            .iter()
            .position(|cc| cc.hir == hir_with_pattern.clone())
        {
            CharClassID::new(id as CharClassIDBase)
        } else {
            let id = CharClassID::new(self.character_classes.len() as CharClassIDBase);
            self.character_classes
                .push(CharacterClass::new(id, hir_with_pattern));
            self.elementary_intervals.clear(); // Clear elementary intervals to regenerate them
            id
        }
    }

    /// Returns the character class with the given ID.
    /// It is used for debugging purposes mostly in the [crate::dot] module.
    #[allow(unused)]
    pub(crate) fn get_character_class(&self, id: CharClassID) -> Option<&CharacterClass> {
        self.character_classes.get(id.as_usize())
    }

    /// Returns the character class with the given ID.
    /// It is used for debugging purposes mostly in the [crate::dot] module.
    #[allow(unused)]
    pub(crate) fn get_elementary_interval(
        &self,
        id: DisjointCharClassID,
    ) -> Option<&std::ops::RangeInclusive<char>> {
        debug_assert!(
            !self.elementary_intervals.is_empty(),
            "Elementary intervals should not be empty"
        );
        debug_assert!(
            id.as_usize() < self.elementary_intervals.len(),
            "Disjoint character class ID out of bounds"
        );
        self.elementary_intervals
            .get(id.as_usize())
            .map(|(interval, _)| interval)
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
    ) -> Result<Box<dyn (Fn(usize, char) -> bool) + 'static + Send + Sync>> {
        let match_functions =
            self.character_classes
                .iter()
                .try_fold(Vec::new(), |mut acc, cc| {
                    // trace!("Create match function for char class {:?}", cc);
                    let match_function: MatchFunction = (&cc.hir).try_into()?;
                    acc.push(match_function);
                    Ok::<Vec<MatchFunction>, ScnrError>(acc)
                })?;
        Ok(Box::new(move |char_class, c| {
            // trace!("Match char class #{} '{}' -> {:?}", char_class.id(), c, res);
            unsafe { match_functions.get_unchecked(char_class).call(c) }
        }))
    }

    pub(crate) fn generate(&self, name: &str) -> proc_macro2::TokenStream {
        let name = syn::Ident::new(name, proc_macro2::Span::call_site()); // Convert name to an Ident
        let mut match_functions = Vec::new();
        for cc in &self.character_classes {
            match_functions.push(cc.generate());
        }
        quote::quote! {
            #[allow(clippy::manual_is_ascii_check, dead_code)]
            pub(crate) fn #name(char_class: usize, c: char) -> bool {
                use std::cmp::Ordering;

                // Define a table of closures for each char_class
                static CHAR_CLASS_TABLE: &[&[std::ops::RangeInclusive<char>]] = &[
                                #(
                                    #match_functions,
                                )*
                ];

                // Check if char_class is within bounds
                if let Some(ranges) = CHAR_CLASS_TABLE.get(char_class) {
                    ranges.binary_search_by(|range| {
                        if c < *range.start() {
                            Ordering::Greater
                        } else if c > *range.end() {
                            Ordering::Less
                        } else {
                            Ordering::Equal
                        }
                    }).is_ok()
                } else {
                    false
                }
            }
        }
    }

    /// Creates disjoint character classes from the elementary intervals.
    /// This function generates elementary intervals from the character classes and
    /// associates them with the character classes that match them.
    /// It is called after all character classes are added to the registry.
    /// This means that the all patterns must be fully parsed and all character classes must be
    /// added to the registry before this function is called. The same applies to all
    /// possibly existing lookahead patterns.
    pub(crate) fn create_disjoint_character_classes(&mut self) {
        if !self.elementary_intervals.is_empty() {
            // If elementary intervals are already created, we should not recreate them.
            return;
        }
        // Step 1: Collect all boundary points
        // The boundaries are collected in a BTreeSet to ensure they are unique and sorted.
        let mut boundaries = std::collections::BTreeSet::new();
        for cc in &self.character_classes {
            match cc.hir.hir.kind() {
                regex_syntax::hir::HirKind::Literal(literal) => {
                    // Literals here are separated into single characters.
                    let bytes = literal.0.clone();
                    // We convert the first 4 bytes to a u32.
                    // If the literal is smaller than 4 bytes, take will ensure we only take the bytes
                    // that exist.
                    let lit: u32 = bytes
                        .iter()
                        .take(4)
                        .fold(0, |acc, &b| (acc << 8) | b as u32);
                    if let Some(c) = char::from_u32(lit) {
                        boundaries.insert(c);
                        // Add the character after the end as a boundary to create half-open
                        // intervals
                        boundaries.insert(char::from_u32(lit + 1).unwrap_or(char::MAX));
                    }
                }
                regex_syntax::hir::HirKind::Class(class) => match class {
                    regex_syntax::hir::Class::Unicode(unicode) => {
                        for range in unicode.ranges() {
                            boundaries.insert(range.start());
                            // Add the character after the end as a boundary to create half-open
                            // intervals
                            if let Some(next_char) = char::from_u32(range.end() as u32 + 1) {
                                boundaries.insert(next_char);
                            } else {
                                // Handle the case where end() is the last Unicode character
                                boundaries.insert(char::MAX);
                            }
                        }
                    }
                    regex_syntax::hir::Class::Bytes(bytes) => {
                        for range in bytes.ranges() {
                            boundaries.insert(range.start() as char);
                            // Add the character after the end as a boundary to create half-open
                            // intervals
                            if let Some(next_char) = char::from_u32(range.end() as u32 + 1) {
                                boundaries.insert(next_char);
                            } else {
                                // Handle the case where end() is the last byte
                                boundaries.insert(char::MAX);
                            }
                        }
                    }
                },
                _ => {
                    unreachable!(
                        "Only Literal and Class are expected in character classes, found: {:?}",
                        cc.hir.hir.kind()
                    );
                }
            }
        }
        let boundaries: Vec<char> = boundaries.into_iter().collect();

        // Step 2: Generate elementary intervals from the boundaries
        // Elementary intervals are ranges between consecutive boundaries.
        self.elementary_intervals.clear();
        for i in 0..boundaries.len() - 1 {
            let start = boundaries[i];
            // Get character before next boundary.
            // If the next boundary is out of range, use the current character.
            if let Some(end) = char::from_u32(boundaries[i + 1] as u32 - 1) {
                if start <= end {
                    // Create a closed interval [start, end] again
                    // Insert the interval into the elementary intervals only if any character class
                    // matches it.
                    if self
                        .character_classes
                        .iter()
                        .any(|cc| cc.contains(&(start..=end)))
                    {
                        // We use inclusive ranges to represent the intervals.
                        self.elementary_intervals.push((start..=end, Vec::new()));
                    }
                }
            } else {
                // If the next boundary is not a valid character, we use the current character
                // as the end of the interval.
                // Insert the interval into the elementary intervals only if any character class
                // matches it.
                if self
                    .character_classes
                    .iter()
                    .any(|cc| cc.contains(&(start..=start)))
                {
                    // We use inclusive ranges to represent the intervals.
                    self.elementary_intervals.push((start..=start, Vec::new()));
                }
            }
        }

        // Step 4: Add disjoint intervals to each character class
        for cc in self.character_classes.iter_mut() {
            for (idx, (interval, cc_ids)) in self.elementary_intervals.iter_mut().enumerate() {
                // Check if the character class matches the interval
                if cc.contains(interval) {
                    // Ensure that the disjoint interval is not already present.
                    debug_assert!(!cc_ids.contains(&cc.id));
                    cc_ids.push(cc.id);
                    cc.add_disjoint_interval(DisjointCharClassID::new(idx as CharClassIDBase));
                }
            }
        }
    }

    /// Returns the disjoint character classes for the given character class ID.
    pub(crate) fn get_disjoint_character_classes(&self, cc: CharClassID) -> &[DisjointCharClassID] {
        debug_assert!(
            cc.as_usize() < self.character_classes.len(),
            "Character class ID out of bounds"
        );
        // Get the character class by ID
        let character_class = &self.character_classes[cc.as_usize()];
        // Return the disjoint intervals of the character class
        &character_class.disjoint_intervals
    }

    /// Checks if the given character class ID matches the character.
    pub(crate) fn matches(&self, cc: DisjointCharClassID, c: char) -> bool {
        self.elementary_intervals
            .get(cc.as_usize())
            .is_some_and(|(interval, _)| interval.contains(&c))
    }
}

impl std::fmt::Display for CharacterClassRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.character_classes.is_empty() {
            write!(f, "\nElementary Intervals:")?;
            for (idx, (interval, cc_ids)) in self.elementary_intervals.iter().enumerate() {
                write!(
                    f,
                    "\n{}  {}..={} => {:?}",
                    idx,
                    interval.start(),
                    interval.end(),
                    cc_ids
                )?;
            }
        } else {
            write!(f, " (no elementary intervals)")?;
        }
        if self.character_classes.is_empty() {
            write!(f, " (no character classes)")?;
        } else {
            write!(f, "\nCharacter Classes:")?;
            for cc in &self.character_classes {
                write!(f, "\n{}", cc)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static INIT: std::sync::Once = std::sync::Once::new();
    fn init() {
        INIT.call_once(|| {
            let _ = env_logger::builder().is_test(true).try_init();
        });
    }

    #[rstest]
    #[case::c1(
        // regex
        r"[a-f][0-9a-f]",
        // elementary_intervals
        &[
            ('0'..='9', vec![CharClassID::new(1)]),
            ('a'..='f', vec![CharClassID::new(0), CharClassID::new(1)]),
        ])]
    #[case::c2(
        // regex
        r"[a-f]",
        // elementary_intervals
        &[
            ('a'..='f', vec![CharClassID::new(0)])
        ])]
    #[case::c3(
        // regex
        r"[0-9]+(_[0-9]+)*\.[0-9]+(_[0-9]+)*[eE][+-]?[0-9]+(_[0-9]+)*",
        // elementary_intervals
        &[
            ('+'..='+', vec![CharClassID::new(4)]),
            ('-'..='-', vec![CharClassID::new(4)]),
            ('.'..='.', vec![CharClassID::new(2)]),
            ('0'..='9', vec![CharClassID::new(0)]),
            ('E'..='E', vec![CharClassID::new(3)]),
            ('_'..='_', vec![CharClassID::new(1)]),
            ('e'..='e', vec![CharClassID::new(3)])
        ])]
    #[case::c4(
        // regex
        r"[\s--\r\n]+",
        // elementary_intervals
        &[
            ('\t'..='\t', vec![CharClassID::new(0)]),
            ('\u{b}'..='\u{c}', vec![CharClassID::new(0)]),
            (' '..=' ', vec![CharClassID::new(0)]),
            ('\u{85}'..='\u{85}', vec![CharClassID::new(0)]),
            ('\u{a0}'..='\u{a0}', vec![CharClassID::new(0)]),
            ('\u{1680}'..='\u{1680}', vec![CharClassID::new(0)]),
            ('\u{2000}'..='\u{200a}', vec![CharClassID::new(0)]),
            ('\u{2028}'..='\u{2029}', vec![CharClassID::new(0)]),
            ('\u{202f}'..='\u{202f}', vec![CharClassID::new(0)]),
            ('\u{205f}'..='\u{205f}', vec![CharClassID::new(0)]),
            ('\u{3000}'..='\u{3000}', vec![CharClassID::new(0)])
        ])]
    #[case::c5(
        // regex
        r"\+=|-=|\*=|/=|%=|&=|\\|=|\^=|<<=|>>=|<<<=|>>>=",
        // elementary_intervals
        &[
            ('%'..='%', vec![CharClassID::new(5)]),
            ('&'..='&', vec![CharClassID::new(6)]),
            ('*'..='*', vec![CharClassID::new(3)]),
            ('+'..='+', vec![CharClassID::new(0)]),
            ('-'..='-', vec![CharClassID::new(2)]),
            ('/'..='/', vec![CharClassID::new(4)]),
            ('<'..='<', vec![CharClassID::new(9)]),
            ('='..='=', vec![CharClassID::new(1)]),
            ('>'..='>', vec![CharClassID::new(10)]),
            ('\\'..='\\', vec![CharClassID::new(7)]),
            ('^'..='^', vec![CharClassID::new(8)])
        ])]
    fn test_create_disjoint_character_classes(
        #[case] regex: &'static str,
        #[case] elementary_intervals: &[(std::ops::RangeInclusive<char>, Vec<CharClassID>)],
    ) {
        init();

        let mut character_class_registry = crate::CharacterClassRegistry::new();
        let hir = crate::parse_regex_syntax(regex).unwrap();
        let _: crate::Nfa = crate::Nfa::try_from_hir(hir, &mut character_class_registry).unwrap();
        character_class_registry.create_disjoint_character_classes();

        eprintln!("==========================");
        eprintln!("Character Class Registry:\n{}", character_class_registry);

        assert_eq!(
            character_class_registry.elementary_intervals,
            elementary_intervals
        );
        assert_eq!(
            character_class_registry.elementary_intervals.len(),
            elementary_intervals.len()
        );
    }
}
