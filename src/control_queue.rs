//!
//! Queue for all the results from event-handling.
//!

use crate::Control;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::Debug;

/// Queue for event-handling results.
#[derive(Debug)]
pub(crate) struct ControlQueue<Action, Error>
where
    Action: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    queue: RefCell<VecDeque<Result<Control<Action>, Error>>>,
}

impl<Action, Error> Default for ControlQueue<Action, Error>
where
    Action: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    fn default() -> Self {
        Self {
            queue: RefCell::new(VecDeque::default()),
        }
    }
}

impl<Action, Error> ControlQueue<Action, Error>
where
    Action: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    /// is empty
    pub(crate) fn is_empty(&self) -> bool {
        self.queue.borrow().is_empty()
    }

    /// take the first result.
    pub(crate) fn take(&self) -> Option<Result<Control<Action>, Error>> {
        self.queue.borrow_mut().pop_front()
    }

    /// push a new result to the queue.
    pub(crate) fn push(&self, ctrl: Result<Control<Action>, Error>) {
        self.queue.borrow_mut().push_back(ctrl);
    }
}
