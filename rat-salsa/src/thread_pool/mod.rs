//! Thread pool.
use crate::Control;
use crate::tasks::{Cancel, Liveness};
use crossbeam::channel::{Receiver, SendError, Sender, TryRecvError, bounded, unbounded};
use log::error;
use std::fmt::{Debug, Formatter};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::thread::JoinHandle;
use std::{mem, thread};

/// Type for a background task.
type BoxTask<Event, Error> = Box<
    dyn FnOnce(Cancel, &Sender<Result<Control<Event>, Error>>) -> Result<Control<Event>, Error>
        + Send,
>;

/// Basic thread-pool.
pub(crate) struct ThreadPool<Event, Error>
where
    Event: 'static,
    Error: 'static,
{
    send: Sender<(Cancel, Liveness, BoxTask<Event, Error>)>,
    recv: Receiver<Result<Control<Event>, Error>>,
    handles: Vec<JoinHandle<()>>,
}

impl<Event, Error> Debug for ThreadPool<Event, Error>
where
    Event: 'static,
    Error: 'static,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThreadPool")
            .field("send", &self.send)
            .field("recv", &self.recv)
            .field("handles", &self.handles)
            .finish()
    }
}

impl<Event, Error> ThreadPool<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    /// New thread-pool with the given task executor.
    pub(crate) fn new(n_worker: usize) -> Self {
        let (send, t_recv) = unbounded::<(Cancel, Liveness, BoxTask<Event, Error>)>();
        let (t_send, recv) = unbounded::<Result<Control<Event>, Error>>();

        let mut handles = Vec::new();

        for _ in 0..n_worker {
            let t_recv = t_recv.clone();
            let t_send = t_send.clone();

            let handle = thread::spawn(move || {
                let t_recv = t_recv;

                'l: loop {
                    match t_recv.recv() {
                        Ok((cancel, liveness, task)) => {
                            liveness.born();
                            let flow = match catch_unwind(AssertUnwindSafe(|| {
                                task(cancel, &t_send) //
                            })) {
                                Ok(v) => v,
                                Err(e) => {
                                    error!("{:?}", e);
                                    continue;
                                }
                            };
                            liveness.dead();
                            if let Err(err) = t_send.send(flow) {
                                error!("{:?}", err);
                                break 'l;
                            }
                        }
                        Err(err) => {
                            error!("{:?}", err);
                            break 'l;
                        }
                    }
                }
            });

            handles.push(handle);
        }

        Self {
            send,
            recv,
            handles,
        }
    }

    /// Start a background task.
    ///
    /// The background task gets a `Arc<Mutex<bool>>` for cancellation support,
    /// and a channel to communicate back to the event loop.
    ///
    /// A clone of the `Arc<Mutex<bool>>` is returned by this function.
    ///
    /// If you need more, create an extra channel for communication to the background task.
    #[inline]
    pub(crate) fn spawn(
        &self,
        task: BoxTask<Event, Error>,
    ) -> Result<(Cancel, Liveness), SendError<()>> {
        if self.handles.is_empty() {
            return Err(SendError(()));
        }

        let cancel = Cancel::new();
        let liveness = Liveness::new();
        match self.send.send((cancel.clone(), liveness.clone(), task)) {
            Ok(_) => Ok((cancel, liveness)),
            Err(_) => Err(SendError(())),
        }
    }

    /// Is the receive-channel empty?
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.recv.is_empty()
    }

    /// Receive a result.
    pub(crate) fn try_recv(&self) -> Result<Control<Event>, Error>
    where
        Error: From<TryRecvError>,
    {
        match self.recv.try_recv() {
            Ok(v) => v,
            Err(TryRecvError::Empty) => Ok(Control::Continue),
            Err(e) => Err(e.into()),
        }
    }
}

impl<Event, Error> ThreadPool<Event, Error>
where
    Event: 'static,
    Error: 'static,
{
    /// Check the workers for liveness.
    pub(crate) fn check_liveness(&self) -> bool {
        for h in &self.handles {
            if h.is_finished() {
                return false;
            }
        }
        true
    }
}

impl<Event, Error> Drop for ThreadPool<Event, Error>
where
    Event: 'static,
    Error: 'static,
{
    fn drop(&mut self) {
        // dropping the channel will be noticed by the threads running the
        // background tasks.
        drop(mem::replace(&mut self.send, bounded(0).0));
        for h in self.handles.drain(..) {
            _ = h.join();
        }
    }
}
