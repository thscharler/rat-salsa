use crate::tokio_tasks::TokioSpawn;
use crate::{AppContext, AppState, Control, PollEvents};
use log::error;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;

/// Add PollTokio to the configuration to enable spawning
/// async operations from the application.
///
/// You cannot work with `tokio-main` but need to initialize
/// the runtime manually.
///
#[derive(Debug)]
pub struct PollTokio<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    rt: Rc<RefCell<Runtime>>,
    pending: Rc<RefCell<Vec<JoinHandle<Result<Control<Event>, Error>>>>>,
    send_queue: Sender<Result<Control<Event>, Error>>,
    recv_queue: Receiver<Result<Control<Event>, Error>>,
}

impl<Event, Error> PollTokio<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    pub fn new(rt: Runtime) -> Self {
        let (send, recv) = channel(100);
        Self {
            rt: Rc::new(RefCell::new(rt)),
            pending: Default::default(),
            send_queue: send,
            recv_queue: recv,
        }
    }

    pub(crate) fn get_spawn(&self) -> TokioSpawn<Event, Error> {
        TokioSpawn {
            rt: self.rt.clone(),
            pending: self.pending.clone(),
            send_queue: self.send_queue.clone(),
        }
    }
}

impl<Global, State, Event, Error> PollEvents<Global, State, Event, Error>
    for PollTokio<Event, Error>
where
    State: AppState<Global, Event, Error> + ?Sized,
    Event: 'static + Send,
    Error: 'static + Send,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn poll(&mut self, _ctx: &mut AppContext<'_, Global, Event, Error>) -> Result<bool, Error> {
        let mut pending = Vec::new();

        for v in self.pending.borrow_mut().drain(..) {
            if v.is_finished() {
                let r = self.rt.borrow().block_on(v);
                match r {
                    Ok(r) => {
                        if let Err(e) = self.send_queue.blocking_send(r) {
                            error!("{:?}", e);
                        }
                    }
                    Err(e) => error!("{:?}", e),
                }
            } else {
                pending.push(v);
            }
        }
        self.pending.replace(pending);

        Ok(!self.recv_queue.is_empty())
    }

    fn read_exec(
        &mut self,
        _state: &mut State,
        _ctx: &mut AppContext<'_, Global, Event, Error>,
    ) -> Result<Control<Event>, Error> {
        self.recv_queue
            .blocking_recv()
            .unwrap_or_else(|| Ok(Control::Continue))
    }
}
