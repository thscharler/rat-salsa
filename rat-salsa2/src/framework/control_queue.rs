//!
//! Queue for all the results from event-handling.
//!

use crate::Control;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};

/// Queue for event-handling results.
pub(crate) struct ControlQueue<Event, Error>
where
    Event: 'static,
    Error: 'static,
{
    queue: RefCell<VecDeque<Result<Control<Event>, Error>>>,
}

impl<Event, Error> Debug for ControlQueue<Event, Error>
where
    Event: 'static,
    Error: 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ControlQueue")
            .field("queue.len", &self.queue.borrow().len())
            .finish()
    }
}

impl<Event, Error> Default for ControlQueue<Event, Error>
where
    Event: 'static,
    Error: 'static,
{
    fn default() -> Self {
        Self {
            queue: RefCell::new(VecDeque::default()),
        }
    }
}

impl<Event, Error> ControlQueue<Event, Error>
where
    Event: 'static,
    Error: 'static,
{
    /// is empty
    pub(crate) fn is_empty(&self) -> bool {
        self.queue.borrow().is_empty()
    }

    /// take the first result.
    pub(crate) fn take(&self) -> Option<Result<Control<Event>, Error>> {
        self.queue.borrow_mut().pop_front()
    }

    /// push a new result to the queue.
    pub(crate) fn push(&self, ctrl: Result<Control<Event>, Error>) {
        self.queue.borrow_mut().push_back(ctrl);
    }
}
