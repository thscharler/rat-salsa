use std::cell::Cell;
use std::fmt::Debug;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

pub enum Resume<T> {
    Pending,
    Yield(T),
    Done(T),
}

pub struct Coroutine<P, T> {
    value: YieldPoint<P, T>,
    init: Option<Box<dyn FnOnce(P, YieldPoint<P, T>) -> Box<dyn Future<Output = T> + 'static>>>,
    inner: Pin<Box<dyn Future<Output = T>>>,
}

impl<P: Debug, T: Debug> Coroutine<P, T> {
    pub fn new(
        inner: impl FnOnce(P, YieldPoint<P, T>) -> Box<dyn Future<Output = T> + 'static> + 'static,
    ) -> Coroutine<P, T> {
        let yp = YieldPoint::new();
        Coroutine {
            value: yp.clone(),
            init: Some(Box::new(inner)),
            inner: Box::pin(async { panic!("coroutine not started") }),
        }
    }

    pub fn start_pending(&self) -> bool {
        self.init.is_some()
    }

    pub fn start(&mut self, arg: P) -> Resume<T> {
        let init = self.init.take().expect("init");
        let f = init(arg, self.value.clone());
        self.inner = Box::into_pin(f);

        self.poll()
    }

    pub fn resume(&mut self, arg: P) -> Resume<T> {
        self.value.resuming(arg);
        self.poll()
    }

    fn poll(&mut self) -> Resume<T> {
        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        match self.inner.as_mut().poll(&mut cx) {
            Poll::Ready(v) => Resume::Done(v),
            Poll::Pending => match self.value.yielded() {
                Some(v) => Resume::Yield(v),
                None => Resume::Pending,
            },
        }
    }
}

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

    pub fn resuming(&self, arg: P) {
        self.arg.set(Some(arg));
    }

    pub fn yielded(&self) -> Option<T> {
        self.yield_.take()
    }

    pub async fn yield0(&self) -> P {
        self.yield_.set(None);
        YieldNow(false).await;
        self.arg.take().expect("arg")
    }

    pub async fn yield1(&self, value: T) -> P {
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
