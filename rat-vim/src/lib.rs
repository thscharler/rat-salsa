use std::error::Error;
use std::fmt::{Display, Formatter};

mod coroutine;
mod ctrl;
mod vi;
mod vi_status_line;

pub use coroutine::{Coroutine, Resume, YieldPoint};
pub use vi::{Mode, VI};
pub use vi_status_line::VIStatusLine;

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
