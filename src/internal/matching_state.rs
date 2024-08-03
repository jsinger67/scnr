use crate::Span;

/// The state of the DFA during matching.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct MatchingState<S>
where
    S: std::fmt::Debug + Default + Clone + Copy + PartialEq + Eq,
{
    // The current state number of the DFA during matching
    current_state: S,
    // The current state of the DFA during matching
    state: InnerMatchingState,
    // The start position of the current match
    start_position: Option<usize>,
    // The end position of the current match
    end_position: Option<usize>,
}

impl<S> MatchingState<S>
where
    S: std::fmt::Debug + Default + Clone + Copy + PartialEq + Eq,
{
    /// Create a new matching state.
    #[inline]
    pub(crate) fn new() -> Self {
        MatchingState::default()
    }

    /// Set the current state of the DFA during matching.
    #[inline]
    pub(crate) fn set_current_state(&mut self, state: S) {
        self.current_state = state;
    }

    /// Get the current state of the DFA during matching.
    #[inline]
    pub(crate) fn current_state(&self) -> S {
        self.current_state
    }

    /// No transition was found.
    /// See matching_state.dot for the state diagram
    pub(crate) fn no_transition(&mut self) {
        match self.state {
            InnerMatchingState::None => {
                // We had no match, continue search
            }
            InnerMatchingState::Start => *self = MatchingState::default(),
            InnerMatchingState::Accepting => {
                // We had a recorded match, return to it
                *self = MatchingState {
                    state: InnerMatchingState::Longest,
                    ..self.clone()
                }
            }
            InnerMatchingState::Longest => {
                // We had the longest match, keep it
            }
        };
    }

    /// Transition to a non-accepting state.
    /// See matching_state.dot for the state diagram
    pub(crate) fn transition_to_non_accepting(&mut self, i: usize) {
        match self.state {
            InnerMatchingState::None => {
                *self = MatchingState {
                    state: InnerMatchingState::Start,
                    start_position: Some(i),
                    ..self.clone()
                }
            }
            InnerMatchingState::Start => {
                // Continue search for an accepting state
            }
            InnerMatchingState::Accepting => {
                // We had a match, keep it and continue search for a longer match
            }
            InnerMatchingState::Longest => {
                // We had the longest match, keep it
            }
        }
    }

    /// Transition to an accepting state.
    /// See matching_state.dot for the state diagram
    pub(crate) fn transition_to_accepting(&mut self, i: usize, c: char) {
        match self.state {
            InnerMatchingState::None => {
                *self = MatchingState {
                    current_state: S::default(),
                    state: InnerMatchingState::Accepting,
                    start_position: Some(i),
                    end_position: Some(i + c.len_utf8()),
                }
            }
            InnerMatchingState::Start => {
                *self = MatchingState {
                    state: InnerMatchingState::Accepting,
                    end_position: Some(i + c.len_utf8()),
                    ..self.clone()
                }
            }
            InnerMatchingState::Accepting => {
                *self = MatchingState {
                    end_position: Some(i + c.len_utf8()),
                    ..self.clone()
                }
            }
            InnerMatchingState::Longest => {
                // We had the longest match, keep it
            }
        }
    }

    /// Returns true if the current state is no match.
    #[inline]
    pub(crate) fn is_no_match(&self) -> bool {
        self.state == InnerMatchingState::None
    }

    /// Returns true if the current state is the longest match.
    #[inline]

    pub(crate) fn is_longest_match(&self) -> bool {
        self.state == InnerMatchingState::Longest
    }

    /// Returns the last match found.

    pub(crate) fn last_match(&self) -> Option<Span> {
        if let (Some(start), Some(end)) = (self.start_position, self.end_position) {
            Some(Span { start, end })
        } else {
            None
        }
    }

    /// Returns the current state of the DFA during matching.
    #[allow(dead_code)]
    pub(crate) fn inner_state(&self) -> InnerMatchingState {
        self.state
    }
}

/// The state enumeration of the DFA during matching.
/// See matching_state.dot for the state diagram
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InnerMatchingState {
    /// No match recorded so far.
    /// Continue search on the next character.
    ///
    /// Current state is not an accepting state.
    ///
    /// If a transition to a non-accepting state is found, record the start of the match and switch
    /// to StartMatch.
    /// If a transition to an accepting state is found, record the match and switch to AcceptingMatch.
    /// If no transition is found stay in NoMatch.
    #[default]
    None,

    /// Start of a match has been recorded.
    /// Continue search for an accepting state.
    ///
    /// Current state is not an accepting state.
    ///
    /// If a transition is found, record the match and switch to AcceptingMatch.
    /// If no transition is found, reset the match and switch to NoMatch.
    Start,

    /// Match has been recorded before, continue search for a longer match.
    ///
    /// State is an accepting state.
    ///
    /// If no transition is found, switch to LongestMatch.
    /// If a transition to a non-accepting state is found stay in AcceptingMatch.
    /// If a transition to an accepting state is found, record the match and stay in AcceptingMatch.
    Accepting,

    /// Match has been recorded before.
    /// The match is the longest match found, no longer match is possible.
    ///
    /// State is an accepting state.
    ///
    /// This state can't be left.
    Longest,
}
