#[allow(unused_imports)]
use log::debug;
use std::time::{Duration, SystemTime};

/// Provides the trigger for mouse double-click.
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
