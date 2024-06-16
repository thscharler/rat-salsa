//!
//! The thread-pool for the background tasks.
//!

use crate::Control;
use crossbeam::channel::{bounded, unbounded, Receiver, SendError, Sender, TryRecvError};
use log::debug;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{mem, thread};

/// Type for a background task.
type BoxTask<Action, Error> = Box<
    dyn FnOnce(Cancel, &Sender<Result<Control<Action>, Error>>) -> Result<Control<Action>, Error>
        + Send,
>;

/// Type for cancellation.
pub type Cancel = Arc<Mutex<bool>>;

/// Basic thread-pool.
///
///
///
#[derive(Debug)]
pub(crate) struct ThreadPool<Action, Error>
where
    Action: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    send: Sender<(Cancel, BoxTask<Action, Error>)>,
    recv: Receiver<Result<Control<Action>, Error>>,
    handles: Vec<JoinHandle<()>>,
}

impl<Action, Error> ThreadPool<Action, Error>
where
    Action: 'static + Send + Debug,
    Error: 'static + Send + Debug,
{
    /// New thread-pool with the given task executor.
    pub(crate) fn new(n_worker: usize) -> Self {
        let (send, t_recv) = unbounded::<(Cancel, BoxTask<Action, Error>)>();
        let (t_send, recv) = unbounded::<Result<Control<Action>, Error>>();

        let mut handles = Vec::new();

        for _ in 0..n_worker {
            let t_recv = t_recv.clone();
            let t_send = t_send.clone();

            let handle = thread::spawn(move || {
                let t_recv = t_recv;

                'l: loop {
                    match t_recv.recv() {
                        Ok((cancel, task)) => {
                            let flow = task(cancel, &t_send);
                            if flow.is_ok() {
                                debug!("send back {:?}", flow);
                            }
                            if let Err(err) = t_send.send(flow) {
                                debug!("{:?}", err);
                                break 'l;
                            }
                        }
                        Err(err) => {
                            debug!("{:?}", err);
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
    pub(crate) fn send(
        &self,
        task: impl FnOnce(
                Cancel,
                &Sender<Result<Control<Action>, Error>>,
            ) -> Result<Control<Action>, Error>
            + Send
            + 'static,
    ) -> Result<Cancel, SendError<()>> {
        self._send(Box::new(task))
    }

    fn _send(&self, task: BoxTask<Action, Error>) -> Result<Cancel, SendError<()>> {
        if self.handles.is_empty() {
            return Err(SendError(()));
        }

        let cancel = Arc::new(Mutex::new(false));
        match self.send.send((Arc::clone(&cancel), task)) {
            Ok(_) => Ok(cancel),
            Err(_) => Err(SendError(())),
        }
    }

    /// Check the workers for liveness.
    pub(crate) fn check_liveness(&self) -> bool {
        for h in &self.handles {
            if h.is_finished() {
                return false;
            }
        }
        true
    }

    /// Is the receive-channel empty?
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.recv.is_empty()
    }

    /// Receive a result.
    pub(crate) fn try_recv(&self) -> Result<Control<Action>, Error>
    where
        Error: From<TryRecvError>,
    {
        if !self.recv.is_empty() {
            debug!("recv {:?}", self.recv);
        }
        match self.recv.try_recv() {
            Ok(v) => v,
            Err(TryRecvError::Empty) => Ok(Control::Continue),
            Err(e) => Err(e.into()),
        }
    }
}

impl<Action, Error> Drop for ThreadPool<Action, Error>
where
    Action: 'static + Send + Debug,
    Error: 'static + Send + Debug,
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
