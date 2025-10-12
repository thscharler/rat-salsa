use std::cell::Cell;
use std::fmt::Debug;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

/// Result of the coroutine.
pub enum Resume<T> {
    /// Yielded without a result.
    Pending,
    /// Yielded with a result.
    Yield(T),
    /// Returned with a result.
    Done(T),
}

/// Implements a coroutine using a future.
///
/// This type of coroutine uses the same yield and return type.
/// It has the specialty that it can yield without producing a
/// result too.
pub struct Coroutine<P, T> {
    init: Option<Box<dyn FnOnce(P, YieldPoint<P, T>) -> Box<dyn Future<Output = T> + 'static>>>,

    yield_point: YieldPoint<P, T>,
    coroutine: Pin<Box<dyn Future<Output = T>>>,
}

impl<P: Debug, T: Debug> Coroutine<P, T> {
    /// Creates a new coroutine.
    ///
    /// ```
    /// use rat_vim::Coroutine;
    ///
    /// let mut co = Coroutine::new(|arg, yp| {
    ///     Box::new(async move {
    ///         let arg = yp.yield_(42*arg);
    ///         _ = yp.yield_(84*arg);
    ///         999
    ///     })
    /// });
    ///
    /// if co.start_pending() {
    ///     let v1 = co.start(1);
    ///     assert_eq!(v1, 42);
    /// }
    /// let v2 = co.resume(1);
    /// assert_eq!(v2, 84);
    /// let v3 = co.resume(1);
    /// assert_eq!(v3, 999);
    ///
    /// ```
    pub fn new<C>(construct: C) -> Coroutine<P, T>
    where
        C: FnOnce(P, YieldPoint<P, T>) -> Box<dyn Future<Output = T> + 'static> + 'static,
    {
        let yp = YieldPoint::new();
        Coroutine {
            yield_point: yp.clone(),
            init: Some(Box::new(construct)),
            coroutine: Box::pin(async { panic!("coroutine not started") }),
        }
    }

    /// Starts or resumes the coroutine.
    pub fn resume(&mut self, arg: P) -> Resume<T> {
        if let Some(construct) = self.init.take() {
            self.coroutine = Box::into_pin(construct(arg, self.yield_point.clone()));
        } else {
            self.yield_point.resuming(arg);
        }

        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        match self.coroutine.as_mut().poll(&mut cx) {
            Poll::Ready(v) => {
                Resume::Done(v) //
            }
            Poll::Pending => match self.yield_point.yielded() {
                Some(v) => {
                    Resume::Yield(v) // 
                }
                None => {
                    Resume::Pending //
                }
            },
        }
    }
}

#[macro_export]
macro_rules! yield_ {
    ($yp:expr) => {
        $yp.yield_pending().await
    };
    ($v:expr, $yp:expr) => {
        $yp.yield_($v).await
    };
}

/// Struct that implements the coroutine yield.
pub struct YieldPoint<P, T> {
    arg: Rc<Cell<Option<P>>>,
    yield_: Rc<Cell<Option<T>>>,
}

impl<P: Debug, T: Debug> YieldPoint<P, T> {
    pub fn new() -> Self {
        Self {
            arg: Default::default(),
            yield_: Default::default(),
        }
    }

    /// Set the resume parameter.
    pub fn resuming(&self, arg: P) {
        self.arg.set(Some(arg));
    }

    /// Yielded value.
    pub fn yielded(&self) -> Option<T> {
        self.yield_.take()
    }

    /// Yield without result.
    /// Results in a `Resume::Pending`.
    pub async fn yield_pending(&self) -> P {
        self.yield_.set(None);
        YieldNow(false).await;
        self.arg.take().expect("arg")
    }

    /// Yield with result.
    /// Results in a `Resume::Yield`
    pub async fn yield_(&self, value: T) -> P {
        self.yield_.set(Some(value));
        YieldNow(false).await;
        self.arg.take().expect("arg")
    }
}

impl<P, T> Clone for YieldPoint<P, T> {
    fn clone(&self) -> Self {
        Self {
            arg: Rc::clone(&self.arg),
            yield_: Rc::clone(&self.yield_),
        }
    }
}

struct YieldNow(bool);

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.0 {
            self.0 = true;
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
