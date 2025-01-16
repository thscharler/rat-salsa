//!
//! Tokio async execution.
//!
use crate::Control;
use std::cell::RefCell;
use std::future::Future;
use std::rc::Rc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Sender;
use tokio::task::{AbortHandle, JoinHandle};

#[derive(Debug)]
pub(crate) struct TokioSpawn<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    rt: Rc<RefCell<Runtime>>,
    pending: Rc<RefCell<Vec<JoinHandle<Result<Control<Event>, Error>>>>>,
    send_queue: Sender<Result<Control<Event>, Error>>,
}

impl<Event, Error> TokioSpawn<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    pub(crate) fn spawn(
        &self,
        future: Box<dyn Future<Output = Result<Control<Event>, Error>> + Send>,
    ) -> AbortHandle {
        let h = self.rt.borrow().spawn(Box::into_pin(future));
        let ah = h.abort_handle();
        self.pending.borrow_mut().push(h);
        ah
    }

    pub(crate) fn sender(&self) -> Sender<Result<Control<Event>, Error>> {
        self.send_queue.clone()
    }
}
