use crate::poll::PollEvents;
use crate::thread_pool::ThreadPool;
use crate::Control;
use crossbeam::channel::TryRecvError;
use std::any::Any;
use std::rc::Rc;

/// Processes results from background tasks.
#[derive(Debug)]
pub struct PollTasks<Event, Error>
where
    Event: 'static,
    Error: 'static,
{
    tasks: Rc<ThreadPool<Event, Error>>,
}

impl<Event, Error> Default for PollTasks<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    fn default() -> Self {
        Self::new(1)
    }
}

impl<Event, Error> PollTasks<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    pub fn new(num_workers: usize) -> Self {
        Self {
            tasks: Rc::new(ThreadPool::new(num_workers)),
        }
    }
}

impl<Event, Error> PollTasks<Event, Error>
where
    Event: 'static,
    Error: 'static,
{
    pub(crate) fn get_tasks(&self) -> Rc<ThreadPool<Event, Error>> {
        self.tasks.clone()
    }
}

impl<Event, Error> PollEvents<Event, Error> for PollTasks<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send + From<TryRecvError>,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn poll(&mut self) -> Result<bool, Error> {
        Ok(!self.tasks.is_empty())
    }

    fn read(&mut self) -> Result<Control<Event>, Error> {
        self.tasks.try_recv()
    }
}
