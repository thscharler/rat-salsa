//!
//! Some widgets and extensions to ratatui-widgets.
//!

use std::time::{Duration, SystemTime};

pub mod basic;
pub mod button;
pub mod calendar;
pub mod date_input;
pub mod input;
pub mod list;
pub mod mask_input;
pub mod menuline;
pub mod message;
pub mod paragraph;
pub mod scrolled;
pub mod table;
pub mod text_area;
pub mod tree;

/// Small helper for handling mouse-events.
///
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MouseFlags {
    pub armed: Option<SystemTime>,
    pub drag: bool,
}

impl MouseFlags {
    /// Handling mouse drag events for this widget is enabled.
    /// It may make sense for a component to track mouse events outside its area.
    /// But usually with some limitations. This flag signals that those limits
    /// have been met, and drag event should be processed.
    pub fn do_drag(&self) -> bool {
        self.drag
    }

    /// Enable handling mouse drag events for the widget.
    pub fn set_drag(&mut self) {
        self.drag = true;
    }

    /// Clear the do-drag flag.
    pub fn clear_drag(&mut self) {
        self.drag = false;
    }

    /// Reset the double-click trigger.
    pub fn reset_trigger(&mut self) {
        self.armed = None;
    }

    /// Unconditionally set a new time for the trigger.
    pub fn arm_trigger(&mut self) {
        self.armed = Some(SystemTime::now());
    }

    /// Pull the trigger, returns true if the action is triggered.
    pub fn pull_trigger(&mut self, time_out: u64) -> bool {
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
