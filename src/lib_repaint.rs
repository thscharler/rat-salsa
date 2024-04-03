use crate::lib_timer::Timed;
use std::cell::Cell;

/// Flags a repaint from event-handling code.
///
/// The standard way is to return [crate::ControlUI::Change] from event-handling. But this
/// consumes the event and returns early, which is not always what you want.
///
/// This flag provides an alternate way to trigger the repaint in that case.
#[derive(Debug, Default)]
pub struct Repaint {
    pub repaint: Cell<bool>,
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
}

/// Gives some extra information why a repaint was triggered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepaintEvent {
    /// There was a [ControlUI::Change](crate::ControlUI::Change) or the change flag has been set.
    Change,
    /// A timer triggered this.
    Timer(Timed),
}
