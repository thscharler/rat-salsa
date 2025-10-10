use std::error::Error;
use std::fmt::{Display, Formatter};

mod coroutine;
mod vi_state;

pub use coroutine::{Coroutine, Resume, YieldPoint};
pub use vi_state::VICmd;

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum VIMode {
    #[default]
    Normal,
    Insert,
    Visual,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MoveDirection {
    Forward,
    Backward,
}

impl MoveDirection {
    pub fn mul(self, d: MoveDirection) -> MoveDirection {
        match (self, d) {
            (MoveDirection::Forward, MoveDirection::Forward) => MoveDirection::Forward,
            (MoveDirection::Forward, MoveDirection::Backward) => MoveDirection::Backward,
            (MoveDirection::Backward, MoveDirection::Forward) => MoveDirection::Backward,
            (MoveDirection::Backward, MoveDirection::Backward) => MoveDirection::Forward,
        }
    }
}

#[derive(Debug)]
pub struct SearchError;

impl Error for SearchError {}

impl Display for SearchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<regex_cursor::regex_automata::dfa::dense::BuildError> for SearchError {
    fn from(_value: regex_cursor::regex_automata::dfa::dense::BuildError) -> Self {
        Self
    }
}

impl From<regex_cursor::regex_automata::MatchError> for SearchError {
    fn from(_value: regex_cursor::regex_automata::MatchError) -> Self {
        Self
    }
}

impl From<regex_cursor::regex_automata::hybrid::BuildError> for SearchError {
    fn from(_value: regex_cursor::regex_automata::hybrid::BuildError) -> Self {
        Self
    }
}
