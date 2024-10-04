//!
//! Support for timers.
//!

#[allow(unused_imports)]
use log::debug;
use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

/// Holds all the timers.
#[derive(Debug, Default)]
pub(crate) struct Timers {
    tags: Cell<usize>,
    timers: RefCell<Vec<TimerImpl>>,
}

/// Handle for a submitted timer.
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
pub struct TimerHandle(usize);

#[derive(Debug)]
struct TimerImpl {
    tag: usize,
    count: usize,
    repeat: Option<usize>,
    next: Instant,
    timer: Duration,
}

impl Timers {
    /// Returns the next sleep time.
    pub(crate) fn sleep_time(&self) -> Option<Duration> {
        let timers = self.timers.borrow();
        if let Some(timer) = timers.last() {
            let now = Instant::now();
            if now > timer.next {
                Some(Duration::from_nanos(0))
            } else {
                Some(timer.next.duration_since(now))
            }
        } else {
            None
        }
    }

    /// Polls for the next timer event.
    pub(crate) fn poll(&self) -> bool {
        let timers = self.timers.borrow();
        if let Some(timer) = timers.last() {
            Instant::now() >= timer.next
        } else {
            false
        }
    }

    /// Polls for the next timer event.
    /// Removes/recalculates the event and reorders the queue.
    pub(crate) fn read(&self) -> Option<TimerEvent> {
        let mut timers = self.timers.borrow_mut();

        let timer = timers.pop();
        if let Some(mut timer) = timer {
            if Instant::now() >= timer.next {
                let evt = TimerEvent(TimeOut {
                    handle: TimerHandle(timer.tag),
                    counter: timer.count,
                });

                // reschedule
                if let Some(repeat) = timer.repeat {
                    timer.count += 1;
                    if timer.count < repeat {
                        timer.next += timer.timer;
                        Self::add_impl(timers.as_mut(), timer);
                    }
                }

                Some(evt)
            } else {
                self.timers.borrow_mut().push(timer);
                None
            }
        } else {
            None
        }
    }

    fn add_impl(timers: &mut Vec<TimerImpl>, t: TimerImpl) {
        'f: {
            for i in 0..timers.len() {
                if timers[i].next <= t.next {
                    timers.insert(i, t);
                    break 'f;
                }
            }
            timers.push(t);
        }
    }

    /// Add a timer.
    #[must_use]
    pub(crate) fn add(&self, t: TimerDef) -> TimerHandle {
        let tag = self.tags.get() + 1;
        self.tags.set(tag);

        let t = TimerImpl {
            tag,
            count: 0,
            repeat: t.repeat,
            next: if let Some(next) = t.next {
                next
            } else {
                Instant::now() + t.timer
            },
            timer: t.timer,
        };

        let mut timers = self.timers.borrow_mut();
        Self::add_impl(timers.as_mut(), t);

        TimerHandle(tag)
    }

    /// Remove a timer.
    pub(crate) fn remove(&self, tag: TimerHandle) {
        let mut timer = self.timers.borrow_mut();
        for i in 0..timer.len() {
            if timer[i].tag == tag.0 {
                timer.remove(i);
                break;
            }
        }
    }
}

/// Timing event data. Used by [TimerEvent].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeOut {
    pub handle: TimerHandle,
    pub counter: usize,
}

/// Timer event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimerEvent(pub TimeOut);

/// Holds the information to start a timer.
#[derive(Debug, Default)]
pub struct TimerDef {
    /// Optional repeat.
    repeat: Option<usize>,
    /// Duration
    timer: Duration,
    /// Specific time.
    next: Option<Instant>,
}

impl TimerDef {
    pub fn new() -> Self {
        Default::default()
    }

    /// Repeat forever.
    pub fn repeat_forever(mut self) -> Self {
        self.repeat = Some(usize::MAX);
        self
    }

    /// Repeat count.
    pub fn repeat(mut self, repeat: usize) -> Self {
        self.repeat = Some(repeat);
        self
    }

    /// Timer interval.
    pub fn timer(mut self, timer: Duration) -> Self {
        self.timer = timer;
        self
    }

    /// Next time the timer is due. Can set a start delay for a repeating timer,
    /// or as an oneshot event for a given instant.
    pub fn next(mut self, next: Instant) -> Self {
        self.next = Some(next);
        self
    }
}
