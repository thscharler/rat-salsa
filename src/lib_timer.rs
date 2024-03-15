#[allow(unused_imports)]
use log::debug;
use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

/// Holds all the timers.
#[derive(Debug, Default)]
pub struct Timers {
    tags: Cell<usize>,
    timers: RefCell<Vec<TimerImpl>>,
}

#[derive(Debug)]
struct TimerImpl {
    tag: usize,
    repaint: bool,
    count: usize,
    repeat: Option<usize>,
    next: Instant,
    timer: Duration,
}

impl Timers {
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns the next sleep time.
    pub fn sleep_time(&self) -> Option<Duration> {
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
    /// Removes/recalculates the event and reorders the queue.
    pub fn poll(&self) -> bool {
        let timers = self.timers.borrow();
        if let Some(timer) = timers.last() {
            let now = Instant::now();
            if now < timer.next {
                false
            } else {
                true
            }
        } else {
            false
        }
    }

    /// Polls for the next timer event.
    /// Removes/recalculates the event and reorders the queue.
    pub fn read(&self) -> Option<TimerEvent> {
        let timer = self.timers.borrow_mut().pop();
        if let Some(mut timer) = timer {
            let now = Instant::now();
            if now < timer.next {
                self.timers.borrow_mut().push(timer);
                None
            } else {
                let evt = TimerEvent {
                    tag: timer.tag,
                    counter: timer.count,
                    repaint: timer.repaint,
                };

                // reschedule
                if let Some(repeat) = timer.repeat {
                    timer.count += 1;
                    if timer.count < repeat {
                        timer.next = timer.next + timer.timer;
                        self.add_impl(timer);
                    }
                }

                Some(evt)
            }
        } else {
            None
        }
    }

    fn add_impl(&self, t: TimerImpl) {
        let mut timers = self.timers.borrow_mut();
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
    pub fn add(&self, t: Timer) -> usize {
        let tag = self.tags.get() + 1;
        self.tags.set(tag);

        let t = TimerImpl {
            tag,
            count: 0,
            repaint: t.repaint,
            repeat: t.repeat,
            next: Instant::now() + t.timer,
            timer: t.timer,
        };
        self.add_impl(t);

        tag
    }

    /// Remove a timer.
    pub fn remove(&self, tag: usize) {
        let mut timer = self.timers.borrow_mut();
        for i in 0..timer.len() {
            if timer[i].tag == tag {
                timer.remove(i);
                break;
            }
        }
    }
}

/// Timer event.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TimerEvent {
    /// The tag identifies the timer.
    pub tag: usize,
    /// Current counter.
    pub counter: usize,
    /// Repaint timer or application timer.
    pub repaint: bool,
}

/// Holds the information to start a timer.
#[derive(Debug)]
pub struct Timer {
    /// Triggers a RepaintEvent.
    pub repaint: bool,
    /// Optional repeat.
    pub repeat: Option<usize>,
    /// Duration
    pub timer: Duration,
    /// Specific time.
    pub next: Option<Instant>,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            repaint: false,
            repeat: None,
            timer: Default::default(),
            next: None,
        }
    }
}

impl Timer {
    pub fn new() -> Self {
        Default::default()
    }

    /// Does this timer trigger a repaint or is it an application timer.
    pub fn repaint(mut self, repaint: bool) -> Self {
        self.repaint = repaint;
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
    /// or as a oneshot event for a given instant.
    pub fn next(mut self, next: Instant) -> Self {
        self.next = Some(next);
        self
    }
}
