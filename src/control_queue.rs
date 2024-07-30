//!
//! Queue for all the results from event-handling.
//!

use crate::timer::TimeOut;
use crate::Control;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::Debug;

#[derive(Debug)]
pub(crate) struct QueueEntry<Message, Error>
where
    Message: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    pub(crate) ctrl: Result<Control<Message>, Error>,
    pub(crate) timeout: Option<TimeOut>,
}

/// Queue for event-handling results.
#[derive(Debug)]
pub(crate) struct ControlQueue<Message, Error>
where
    Message: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    queue: RefCell<VecDeque<QueueEntry<Message, Error>>>,
}

impl<Message, Error> Default for ControlQueue<Message, Error>
where
    Message: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    fn default() -> Self {
        Self {
            queue: RefCell::new(VecDeque::default()),
        }
    }
}

impl<Message, Error> ControlQueue<Message, Error>
where
    Message: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    /// is empty
    pub(crate) fn is_empty(&self) -> bool {
        self.queue.borrow().is_empty()
    }

    /// take the first result.
    pub(crate) fn take(&self) -> Option<QueueEntry<Message, Error>> {
        self.queue.borrow_mut().pop_front()
    }

    /// push a new result to the queue.
    pub(crate) fn push(&self, ctrl: Result<Control<Message>, Error>, timeout: Option<TimeOut>) {
        self.queue
            .borrow_mut()
            .push_back(QueueEntry { ctrl, timeout });
    }
}
