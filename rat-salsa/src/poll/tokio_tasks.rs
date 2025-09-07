use crate::poll::PollEvents;
use crate::tokio_tasks::TokioTasks;
use crate::Control;
use std::any::Any;
use std::rc::Rc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Receiver;

/// Add PollTokio to the configuration to enable spawning
/// async operations from the application.
///
/// You cannot work with `tokio-main` but need to initialize
/// the runtime manually.
///
#[derive(Debug)]
pub struct PollTokio<Event, Error>
where
    Event: 'static,
    Error: 'static,
{
    tasks: Rc<TokioTasks<Event, Error>>,
    recv_queue: Receiver<Result<Control<Event>, Error>>,
}

impl<Event, Error> PollTokio<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    pub fn new(rt: Runtime) -> Self {
        let (tokio, recv) = TokioTasks::new(rt);
        Self {
            tasks: Rc::new(tokio),
            recv_queue: recv,
        }
    }
}

impl<Event, Error> PollTokio<Event, Error>
where
    Event: 'static,
    Error: 'static,
{
    pub(crate) fn get_tasks(&self) -> Rc<TokioTasks<Event, Error>> {
        self.tasks.clone()
    }
}

impl<Event, Error> PollEvents<Event, Error> for PollTokio<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn poll(&mut self) -> Result<bool, Error> {
        self.tasks.poll_finished()?;
        Ok(!self.recv_queue.is_empty())
    }

    fn read(&mut self) -> Result<Control<Event>, Error> {
        self.recv_queue
            .blocking_recv()
            .unwrap_or_else(|| Ok(Control::Continue))
    }
}
