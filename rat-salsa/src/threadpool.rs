//!
//! The thread-pool for the background tasks.
//!

use crate::Control;
use crossbeam::channel::{bounded, unbounded, Receiver, SendError, Sender, TryRecvError};
use log::warn;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::{mem, thread};

/// Type for a background task.
type BoxTask<Event, Error> = Box<
    dyn FnOnce(Cancel, &Sender<Result<Control<Event>, Error>>) -> Result<Control<Event>, Error>
        + Send,
>;

/// Cancel background tasks.
#[derive(Debug, Default, Clone)]
pub struct Cancel(Arc<AtomicBool>);

impl Cancel {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    pub fn is_canceled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
}

/// Basic thread-pool.
///
///
///
#[derive(Debug)]
pub(crate) struct ThreadPool<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    send: Sender<(Cancel, BoxTask<Event, Error>)>,
    recv: Receiver<Result<Control<Event>, Error>>,
    handles: Vec<JoinHandle<()>>,
}

impl<Event, Error> ThreadPool<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
{
    /// New thread-pool with the given task executor.
    pub(crate) fn new(n_worker: usize) -> Self {
        let (send, t_recv) = unbounded::<(Cancel, BoxTask<Event, Error>)>();
        let (t_send, recv) = unbounded::<Result<Control<Event>, Error>>();

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
                            if let Err(err) = t_send.send(flow) {
                                warn!("{:?}", err);
                                break 'l;
                            }
                        }
                        Err(err) => {
                            warn!("{:?}", err);
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
    pub(crate) fn send(&self, task: BoxTask<Event, Error>) -> Result<Cancel, SendError<()>> {
        if self.handles.is_empty() {
            return Err(SendError(()));
        }

        let cancel = Cancel::new();
        match self.send.send((cancel.clone(), task)) {
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

impl<Event, Error> Drop for ThreadPool<Event, Error>
where
    Event: 'static + Send,
    Error: 'static + Send,
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
