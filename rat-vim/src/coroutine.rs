//!
//! A sync+async stackless coroutine implementation.
//!
//! It uses a future to drive the inner workings of the coroutine.
//!
//! __Sync__
//!
//! [Coroutine::resume] is built like [Future::poll].
//! It uses [Resume] instead of [Poll] for its result.
//!
//! Run in sync code.
//! ```
//! use rat_vim::coroutine::Coroutine;
//! use rat_vim::yield_;
//!
//! let mut co = Coroutine::new(|arg, yy| {
//!     Box::new(async move {
//!         let arg = yield_!(42, yy);
//!         let arg = yield_!(43, yy);
//!         44
//!     })
//! });
//!
//! let v1 = co.resume(1).yielded();
//! assert_eq!(v1, 42);
//! let v2 = co.resume(1).yielded();
//! assert_eq!(v2, 43);
//! let v3 = co.resume(1).returned();
//! assert_eq!(v3, 44);
//!
//! ```
//!
//! __Async__
//!
//! [Coroutine::resume_with] sets the argument for the next resume.
//! It returns a future that will actually resume the coroutine
//! when awaited.
//!
//! ```
//! use rat_vim::coroutine::Coroutine;
//! use rat_vim::yield_;
//!
//! let mut co = Coroutine::new(|arg, yy| {
//!     Box::new(async move {
//!         let arg = yield_!(42, yy);
//!         let arg = yield_!(43, yy);
//!         44
//!     })
//! });
//!
//! let mut sample1 = pin!(async {
//!     let v1 = co.resume_with(1).await.yielded();
//!     assert_eq!(v1, 42);
//!     let v2 = co.resume_with(2).await.yielded();
//!     assert_eq!(v2, 43);
//!     let v3 = co.resume_with(3).await.returned();
//!     assert_eq!(v3, 44);
//! });
//!
//! # use std::pin::pin;
//! # use std::task::{Context, Poll};
//! # let mut cx = Context::from_waker(futures::task::noop_waker_ref());
//! # loop {
//! #     match sample1.as_mut().poll(&mut cx) {
//! #         Poll::Ready(_) => break,
//! #         Poll::Pending => {}
//! #     };
//! # }
//! ```

use std::cell::Cell;
use std::fmt::Debug;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

/// Result of the coroutine.
#[derive(Debug)]
pub enum Resume<Y, R> {
    /// Yielded without a result.
    Pending,
    /// Yielded with a result.
    Yield(Y),
    /// Returned with a result.
    Return(R),
}

impl<Y, R> Resume<Y, R> {
    /// Is the resume a [Resume::Pending]
    pub fn pending(self) -> bool {
        matches!(self, Resume::Pending)
    }

    /// Returned value.
    ///
    /// Panic
    ///
    /// Panics if this is not a [Resume::Return].
    pub fn returned(self) -> R {
        if let Resume::Return(v) = self {
            v
        } else {
            panic!("not_return")
        }
    }

    /// Returned value.
    pub fn try_returned(self) -> Option<R> {
        if let Resume::Return(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Yielded value.
    ///
    /// Panic
    ///
    /// Panics if this is not a [Resume::Yield].
    pub fn yielded(self) -> Y {
        if let Resume::Yield(v) = self {
            v
        } else {
            panic!("not_yield")
        }
    }

    /// Yielded value.
    pub fn try_yielded(self) -> Option<Y> {
        if let Resume::Yield(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl<Y> Resume<Y, Y> {
    /// Yielded/Returned value.
    ///
    /// Panic
    ///
    /// Panics if this is a [Resume::Pending].
    pub fn value(self) -> Y {
        if let Resume::Yield(v) = self {
            v
        } else if let Resume::Return(v) = self {
            v
        } else {
            panic!("not_yield")
        }
    }

    /// Yielded value.
    pub fn try_value(self) -> Option<Y> {
        if let Resume::Yield(v) = self {
            Some(v)
        } else if let Resume::Return(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

/// Implements a coroutine using a future.
///
/// It has the specialty that it can yield without producing a result.
#[allow(clippy::type_complexity)]
pub struct Coroutine<'a, A, Y, R> {
    init: Option<Box<dyn FnOnce(A, Yield<A, Y>) -> Box<dyn Future<Output = R> + 'a> + 'a>>,

    yield_: Yield<A, Y>,
    coroutine: Pin<Box<dyn Future<Output = R> + 'a>>,
}

impl<'a, A: Debug, Y: Debug, R: Debug> Coroutine<'a, A, Y, R> {
    /// Creates a new coroutine.
    ///
    /// This doesn't start the coroutine, instead it takes a
    /// closure that will be called by the first [resume].
    ///
    /// The first resume calls the closure with the first argument
    /// and [Yield] support. The closure returns the boxed future
    /// that runs the coroutine.
    pub fn new<C>(construct: C) -> Coroutine<'a, A, Y, R>
    where
        C: FnOnce(A, Yield<A, Y>) -> Box<dyn Future<Output = R> + 'a> + 'a,
    {
        let yp = Yield::new();
        Coroutine {
            init: Some(Box::new(construct)),
            yield_: yp.clone(),
            coroutine: Box::pin(async { panic!("coroutine not started") }),
        }
    }

    /// Starts or resumes the coroutine.
    ///
    /// Call this in synchronous code.
    pub fn resume(&mut self, arg: A) -> Resume<Y, R> {
        if let Some(construct) = self.init.take() {
            self.coroutine = Box::into_pin(construct(arg, self.yield_.clone()));
        } else {
            self.yield_.resume_with(arg);
        }

        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        match self.coroutine.as_mut().poll(&mut cx) {
            Poll::Ready(v) => {
                Resume::Return(v) //
            }
            Poll::Pending => match self.yield_.yielded() {
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

/// Result when using the coroutine in async code.
#[derive(Debug)]
pub enum CoResult<Y, R> {
    /// Yielded with a result.
    Yield(Y),
    /// Returned with a result.
    Return(R),
}

impl<Y, R> CoResult<Y, R> {
    /// Returned result.
    ///
    /// Panic
    ///
    /// Panics if this is not a [CoResult::Return]
    pub fn returned(self) -> R {
        if let CoResult::Return(v) = self {
            v
        } else {
            panic!("not_return")
        }
    }

    /// Returned result.
    pub fn try_returned(self) -> Option<R> {
        if let CoResult::Return(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Yielded result.
    ///
    /// Panic
    ///
    /// Panics if this is not a [CoResult::Yield]
    pub fn yielded(self) -> Y {
        if let CoResult::Yield(v) = self {
            v
        } else {
            panic!("not_yield")
        }
    }

    /// Yielded result.
    pub fn try_yielded(self) -> Option<Y> {
        if let CoResult::Yield(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl<Y> CoResult<Y, Y> {
    /// Yielded/Returned result.
    pub fn value(self) -> Y {
        match self {
            CoResult::Yield(v) => v,
            CoResult::Return(v) => v,
        }
    }
}

impl<'a, A, Y, R> Coroutine<'a, A, Y, R> {
    /// Set the resume parameter in an async context.
    /// Then call await on the result.
    pub fn resume_with<'b>(&'b mut self, arg: A) -> CoroutineArg<'a, 'b, A, Y, R> {
        self.yield_.resume_with(arg);
        CoroutineArg { co: self }
    }
}

/// Result type for [Coroutine::resume_with].
/// This can be awaited.
pub struct CoroutineArg<'a, 'b, A, Y, R> {
    co: &'b mut Coroutine<'a, A, Y, R>,
}

impl<'a, 'b, A, Y, R> Future for CoroutineArg<'a, 'b, A, Y, R> {
    type Output = CoResult<Y, R>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(construct) = self.co.init.take() {
            let arg = self.co.yield_.arg.take().expect("arg");
            self.co.coroutine = Box::into_pin(construct(arg, self.co.yield_.clone()));
        }

        match self.co.coroutine.as_mut().poll(cx) {
            Poll::Ready(v) => Poll::Ready(CoResult::Return(v)),
            Poll::Pending => match self.co.yield_.yielded() {
                Some(v) => Poll::Ready(CoResult::Yield(v)),
                None => Poll::Pending,
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
pub struct Yield<P, T> {
    arg: Rc<Cell<Option<P>>>,
    yielded: Rc<Cell<Option<T>>>,
}

impl<P, T> Default for Yield<P, T> {
    fn default() -> Self {
        Self {
            arg: Default::default(),
            yielded: Default::default(),
        }
    }
}

impl<P, T> Yield<P, T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the resume parameter.
    pub fn resume_with(&self, arg: P) {
        self.arg.set(Some(arg));
    }

    /// Yielded value.
    pub fn yielded(&self) -> Option<T> {
        self.yielded.take()
    }

    /// Yield without result.
    /// Results in a [Resume::Pending] or suspends the async call.
    pub async fn yield_pending(&self) -> P {
        self.yielded.set(None);
        yield_tech().await;
        self.arg.take().expect("arg")
    }

    /// Yield with result.
    /// Results in a [Resume::Yield] or a [CoResult::Yield]
    pub async fn yield_(&self, value: T) -> P {
        self.yielded.set(Some(value));
        yield_tech().await;
        self.arg.take().expect("arg")
    }
}

impl<P, T> Clone for Yield<P, T> {
    fn clone(&self) -> Self {
        Self {
            arg: Rc::clone(&self.arg),
            yielded: Rc::clone(&self.yielded),
        }
    }
}

fn yield_tech() -> impl Future<Output = ()> {
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

    YieldNow(false)
}

#[cfg(test)]
mod test {
    use crate::coroutine::{CoResult, Coroutine, Resume, Yield};
    use std::pin::pin;
    use std::task::{Context, Poll};

    #[test]
    fn test_async() {
        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        let mut f1 = pin!(sample1());
        loop {
            match f1.as_mut().poll(&mut cx) {
                Poll::Ready(_) => break,
                Poll::Pending => {}
            };
        }
    }

    async fn sample1() {
        let mut co = cr_co();

        match co.resume_with('a').await {
            CoResult::Yield(v) => {
                dbg!(v);
                assert_eq!(v, 'A');
            }
            CoResult::Return(_v) => {
                unreachable!()
            }
        };
        match co.resume_with('a').await {
            CoResult::Yield(_v) => {
                unreachable!()
            }
            CoResult::Return(v) => {
                dbg!(v);
                assert_eq!(v, 'B');
            }
        };
    }

    #[test]
    fn test_sync() {
        sample2();
    }

    fn sample2() {
        let mut co = cr_co();

        match co.resume('a') {
            Resume::Pending => {}
            Resume::Yield(v) => {
                dbg!(v);
                assert_eq!(v, 'A');
            }
            Resume::Return(_v) => {
                unreachable!()
            }
        }
        match co.resume('a') {
            Resume::Pending => {}
            Resume::Yield(_v) => {
                unreachable!()
            }
            Resume::Return(v) => {
                dbg!(v);
                assert_eq!(v, 'B');
            }
        }
    }

    fn cr_co() -> Coroutine<'static, char, char, char> {
        Coroutine::new(|cc: char, yp: Yield<char, char>| {
            Box::new(async move {
                let c = match cc {
                    'a' => 'A',
                    c => c,
                };
                let cc = yp.yield_(c).await;
                match cc {
                    'a' => 'B',
                    c => c,
                }
            })
        })
    }
}
