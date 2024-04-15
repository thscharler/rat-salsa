//!
//! Some widgets and extensions to ratatui-widgets.
//!

use std::cell::Cell;
use std::time::{Duration, SystemTime};

pub mod basic;
pub mod button;
pub mod calendar;
pub mod date_input;
pub mod input;
pub mod mask_input;
pub mod menuline;
pub mod message;
pub mod paragraph;
pub mod table;

/// Small helper that provides a trigger for mouse double-click.
///
/// It uses a timeout to filter out the second click.
#[derive(Debug, Default)]
pub struct ActionTrigger {
    pub armed: Option<SystemTime>,
}

impl ActionTrigger {
    /// Reset the trigger.
    pub fn reset(&mut self) {
        self.armed = None;
    }

    /// Pull the trigger, returns true if the action is triggered.
    pub fn pull(&mut self, time_out: u64) -> bool {
        match &self.armed {
            None => {
                self.armed = Some(SystemTime::now());
                false
            }
            Some(armed) => {
                let elapsed = armed.elapsed().expect("timeout");
                if elapsed > Duration::from_millis(time_out) {
                    self.armed = None;
                    false
                } else {
                    self.armed = None;
                    true
                }
            }
        }
    }
}

/// Scroll-state of a widget.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Scroll {
    /// Total content length
    pub len: Cell<usize>,
    /// Current offset
    pub offset: Cell<usize>,
    /// Page-size
    pub page: Cell<usize>,
    /// Mouse scrolling active
    pub mouse: Cell<bool>,
}

impl Scroll {
    pub fn set_len(&self, len: usize) {
        self.len.set(len);
    }

    pub fn len(&self) -> usize {
        self.len.get()
    }

    pub fn set_offset(&self, offset: usize) {
        self.offset.set(offset);
    }

    pub fn offset(&self) -> usize {
        self.offset.get()
    }

    pub fn set_page(&self, page: usize) {
        self.page.set(page);
    }

    pub fn page(&self) -> usize {
        self.page.get()
    }
}

/// Trait for a widget that can scroll vertically.
pub trait HasVerticalScroll {
    /// State needs scrolling?
    fn need_vscroll(&self) -> bool {
        true
    }

    /// Scroll data.
    fn vscroll(&self) -> &Scroll;

    /// Vertical length.
    fn vlen(&self) -> usize {
        self.vscroll().len.get()
    }

    /// Vertical offset.
    fn voffset(&self) -> usize {
        self.vscroll().offset.get()
    }

    /// Vertical page size.
    fn vpage(&self) -> usize {
        self.vscroll().page.get()
    }

    /// Scroll up by n.
    fn vscroll_up(&self, n: usize) {
        let offset = self.vscroll().offset.get();
        if offset > n {
            self.vscroll().offset.set(offset - n);
        } else {
            self.vscroll().offset.set(0);
        }
    }

    /// Scroll down by n.
    fn vscroll_down(&self, n: usize) {
        let offset = self.voffset();
        let len = self.vlen();
        if offset + n < len {
            self.vscroll().offset.set(offset + n);
        } else {
            self.vscroll().offset.set(len);
        }
    }
}
