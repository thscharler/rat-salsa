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
    pub fn returned(self) -> R {
        if let Resume::Return(v) = self {
            v
        } else {
            panic!("not_return")
        }
    }

    pub fn yielded(self) -> Y {
        if let Resume::Yield(v) = self {
            v
        } else {
            panic!("not_yield")
        }
    }
}

/// Implements a coroutine using a future.
///
/// This type of coroutine uses the same yield and return type.
/// It has the specialty that it can yield without producing a
/// result too.
pub struct Coroutine<'a, A, Y, R> {
    init: Option<Box<dyn FnOnce(A, Yield<A, Y>) -> Box<dyn Future<Output = R> + 'a> + 'a>>,

    yield_point: Yield<A, Y>,
    coroutine: Pin<Box<dyn Future<Output = R> + 'a>>,
}

impl<'a, A: Debug, Y: Debug, R: Debug> Coroutine<'a, A, Y, R> {
    /// Creates a new coroutine.
    ///
    /// Run in sync code.
    /// ```
    /// use rat_vim::coroutine::Coroutine;
    ///
    /// let mut co = Coroutine::new(|arg, yp| {
    ///     Box::new(async move {
    ///         let arg = yp.yield_(42).await;
    ///         let arg = yp.yield_(43).await;
    ///         44
    ///     })
    /// });
    ///
    /// let v1 = co.resume(1).yielded();
    /// assert_eq!(v1, 42);
    /// let v2 = co.resume(1).yielded();
    /// assert_eq!(v2, 43);
    /// let v3 = co.resume(1).returned();
    /// assert_eq!(v3, 44);
    ///
    /// ```
    ///
    /// Or in async code
    ///
    /// ```
    /// use rat_vim::coroutine::Coroutine;
    ///
    /// let mut co = Coroutine::new(|arg, yp| {
    ///     Box::new(async move {
    ///         let _ = yp.yield_(42).await;
    ///         let _ = yp.yield_(43).await;
    ///         44
    ///     })
    /// });
    ///
    /// let mut sample1 = pin!(async {
    ///     let v1 = co.resume_with(1).await.yielded();
    ///     assert_eq!(v1, 42);
    ///     let v2 = co.resume_with(2).await.yielded();
    ///     assert_eq!(v2, 43);
    ///     let v3 = co.resume_with(3).await.returned();
    ///     assert_eq!(v3, 44);
    /// });
    ///
    /// # use std::pin::pin;
    /// # use std::task::{Context, Poll};
    /// # let mut cx = Context::from_waker(futures::task::noop_waker_ref());
    /// # loop {
    /// #     match sample1.as_mut().poll(&mut cx) {
    /// #         Poll::Ready(_) => break,
    /// #         Poll::Pending => {}
    /// #     };
    /// # }
    /// ```
    ///
    pub fn new<C>(construct: C) -> Coroutine<'a, A, Y, R>
    where
        C: FnOnce(A, Yield<A, Y>) -> Box<dyn Future<Output = R> + 'a> + 'a,
    {
        let yp = Yield::new();
        Coroutine {
            init: Some(Box::new(construct)),
            yield_point: yp.clone(),
            coroutine: Box::pin(async { panic!("coroutine not started") }),
        }
    }

    /// Starts or resumes the coroutine.
    ///
    /// Call this in synchronous code.
    pub fn resume(&mut self, arg: A) -> Resume<Y, R> {
        if let Some(construct) = self.init.take() {
            self.coroutine = Box::into_pin(construct(arg, self.yield_point.clone()));
        } else {
            self.yield_point.resume_with(arg);
        }

        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        match self.coroutine.as_mut().poll(&mut cx) {
            Poll::Ready(v) => {
                Resume::Return(v) //
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

/// Result when using the coroutine in async code.
#[derive(Debug)]
pub enum CoResult<Y, R> {
    /// Yielded with a result.
    Yield(Y),
    /// Returned with a result.
    Return(R),
}

impl<Y, R> CoResult<Y, R> {
    pub fn returned(self) -> R {
        if let CoResult::Return(v) = self {
            v
        } else {
            panic!("not_return")
        }
    }

    pub fn yielded(self) -> Y {
        if let CoResult::Yield(v) = self {
            v
        } else {
            panic!("not_yield")
        }
    }
}

impl<'a, A, Y, R> Coroutine<'a, A, Y, R> {
    /// Set the resume parameter in an async context.
    /// Then call await on the result.
    pub fn resume_with(&mut self, arg: A) -> &mut Self {
        self.yield_point.resume_with(arg);
        self
    }
}

impl<'a, A, Y, R> Future for Coroutine<'a, A, Y, R> {
    type Output = CoResult<Y, R>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(construct) = self.init.take() {
            let arg = self.yield_point.arg.take().expect("arg");
            self.coroutine = Box::into_pin(construct(arg, self.yield_point.clone()));
        }

        match self.coroutine.as_mut().poll(cx) {
            Poll::Ready(v) => Poll::Ready(CoResult::Return(v)),
            Poll::Pending => match self.yield_point.yielded() {
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

impl<P, T> Yield<P, T> {
    pub fn new() -> Self {
        Self {
            arg: Default::default(),
            yielded: Default::default(),
        }
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
            CoResult::Return(v) => {
                unreachable!()
            }
        };
        match co.resume_with('a').await {
            CoResult::Yield(v) => {
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
            Resume::Return(v) => {
                unreachable!()
            }
        }
        match co.resume('a') {
            Resume::Pending => {}
            Resume::Yield(v) => {
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
