//!
//! Queue for all the results from event-handling.
//!

use crate::Control;
use std::cell::RefCell;
use std::collections::VecDeque;

/// Queue for event-handling results.
#[derive(Debug)]
pub(crate) struct ControlQueue<Message, Error>
where
    Message: 'static + Send,
    Error: 'static + Send,
{
    queue: RefCell<VecDeque<Result<Control<Message>, Error>>>,
}

impl<Message, Error> Default for ControlQueue<Message, Error>
where
    Message: 'static + Send,
    Error: 'static + Send,
{
    fn default() -> Self {
        Self {
            queue: RefCell::new(VecDeque::default()),
        }
    }
}

impl<Message, Error> ControlQueue<Message, Error>
where
    Message: 'static + Send,
    Error: 'static + Send,
{
    /// is empty
    pub(crate) fn is_empty(&self) -> bool {
        self.queue.borrow().is_empty()
    }

    /// take the first result.
    pub(crate) fn take(&self) -> Option<Result<Control<Message>, Error>> {
        self.queue.borrow_mut().pop_front()
    }

    /// push a new result to the queue.
    pub(crate) fn push(&self, ctrl: Result<Control<Message>, Error>) {
        self.queue.borrow_mut().push_back(ctrl);
    }
}
