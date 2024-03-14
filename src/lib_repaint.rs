use std::cell::{Cell, RefCell};
use std::time::{Duration, SystemTime};

/// Flags a mandatory repaint from event-handling code.
///
/// The standard way is to return [ControlUI::Changed] from event-handling. But this
/// consumes the event and returns early, which is not always what you want.
///
/// This flag provides an alternate way to trigger the repaint in that case.
#[derive(Debug, Default)]
pub struct Repaint {
    pub repaint: Cell<bool>,
    pub timer: RefCell<Vec<Timer>>,
}

impl Repaint {
    pub fn new() -> Self {
        Self::default()
    }

    /// Current repaint state.
    pub fn get(&self) -> bool {
        self.repaint.get()
    }

    /// Flag for repaint.
    pub fn set(&self) {
        self.repaint.set(true);
    }

    /// Reset the flag.
    pub fn reset(&self) {
        self.repaint.set(false)
    }

    /// Set a repaint timer
    pub fn set_timer(&self, t: Timer) {
        let mut timer = self.timer.borrow_mut();
        'l: {
            // insert in descending order. should avoid starvation of long timers.
            for i in 0..timer.len() {
                if timer[i].timeout > t.timeout {
                    timer.insert(i, t);
                    break 'l;
                }
            }
            timer.push(t);
        }
    }

    /// Has a specific timer?
    pub fn has_timer(&self, tag: usize) -> bool {
        self.timer.borrow().iter().find(|v| v.tag == tag).is_some()
    }

    /// Drop a repaint timer.
    pub fn drop_timer(&self, tag: usize) {
        let mut timer = self.timer.borrow_mut();
        for i in 0..timer.len() {
            if timer[i].tag == tag {
                timer.remove(i);
                break;
            }
        }
    }
}

/// Reason for a repaint.
#[derive(Debug, Clone, Copy)]
pub enum RepaintReason {
    Change,
    Timeout(Timeout),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Timeout {
    pub tag: usize,
    pub counter: usize,
}

#[derive(Debug)]
pub struct Timer {
    pub tag: usize,
    pub repeat: bool,
    pub count: Option<usize>,
    pub max: Option<usize>,
    pub start: SystemTime,
    pub timeout: Duration,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            tag: 0,
            repeat: false,
            count: None,
            max: None,
            start: SystemTime::now(),
            timeout: Default::default(),
        }
    }
}

impl Timer {
    pub fn new(tag: usize) -> Self {
        Self {
            tag,
            ..Default::default()
        }
    }

    pub fn tag(mut self, tag: usize) -> Self {
        self.tag = tag;
        self
    }

    pub fn repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }

    pub fn max(mut self, max: usize) -> Self {
        self.repeat = true;
        self.max = Some(max);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}
