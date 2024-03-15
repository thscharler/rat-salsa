//!
//! Some widgets and extensions to ratatui-widgets.
//!

use std::time::{Duration, SystemTime};

pub mod basic;
pub mod button;
pub mod calendar;
pub mod input;
pub mod mask_input;
pub mod menuline;
pub mod message;
pub mod table;

/// Small helper that provides a trigger for mouse double-click.
///
/// It uses a timeout to filter out the second click.
#[derive(Debug)]
pub struct ActionTrigger {
    pub armed: Option<SystemTime>,
}

impl Default for ActionTrigger {
    fn default() -> Self {
        Self { armed: None }
    }
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
