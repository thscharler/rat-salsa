use std::cell::RefCell;
use std::collections::VecDeque;

/// Queue for which EventPoll wants to be read.
#[derive(Debug, Default)]
pub(crate) struct PollQueue {
    queue: RefCell<VecDeque<usize>>,
}

impl PollQueue {
    /// Empty
    pub(crate) fn is_empty(&self) -> bool {
        self.queue.borrow().is_empty()
    }

    /// Take the next handle.
    pub(crate) fn take(&self) -> Option<usize> {
        self.queue.borrow_mut().pop_front()
    }

    /// Push a handle to the queue.
    pub(crate) fn push(&self, poll: usize) {
        self.queue.borrow_mut().push_back(poll);
    }
}
