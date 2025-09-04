use crate::Control;
use log::error;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::{AbortHandle, JoinHandle};

pub(crate) struct TokioTasks<Event, Error> {
    rt: Runtime,
    pending: RefCell<Vec<JoinHandle<Result<Control<Event>, Error>>>>,
    send_queue: Sender<Result<Control<Event>, Error>>,
}

impl<Event, Error> Debug for TokioTasks<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokioTasks")
            .field("rt", &self.rt)
            .field("pending.len", &self.pending.borrow().len())
            .field("send_queue.is_closed", &self.send_queue.is_closed())
            .finish()
    }
}

impl<Event, Error> TokioTasks<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    pub fn new(rt: Runtime) -> (Self, Receiver<Result<Control<Event>, Error>>) {
        let (send, recv) = channel(100);
        (
            Self {
                rt,
                pending: Default::default(),
                send_queue: send,
            },
            recv,
        )
    }

    pub(crate) fn spawn(
        &self,
        future: Box<dyn Future<Output = Result<Control<Event>, Error>> + Send>,
    ) -> AbortHandle {
        let h = self.rt.spawn(Box::into_pin(future));
        let ah = h.abort_handle();
        self.pending.borrow_mut().push(h);
        ah
    }

    pub(crate) fn sender(&self) -> Sender<Result<Control<Event>, Error>> {
        self.send_queue.clone()
    }

    pub(crate) fn poll_finished(&self) -> Result<(), Error> {
        self.pending.borrow_mut().retain_mut(|v| {
            if v.is_finished() {
                match self.rt.block_on(v) {
                    Ok(r) => {
                        if let Err(e) = self.send_queue.try_send(r) {
                            error!("{:?}", e);
                        }
                    }
                    Err(e) => error!("{:?}", e),
                }
                false
            } else {
                true
            }
        });
        Ok(())
    }
}
