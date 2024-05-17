use rat_event::UsedEvent;

pub mod list;
pub mod paragraph;
pub mod table;
pub mod tree;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The given event was not handled at all.
    NotUsed,
    /// The event was handled, no repaint necessary.
    Unchanged,
    /// The event was handled, repaint necessary.
    Changed,
}

impl UsedEvent for Outcome {
    fn used_event(&self) -> bool {
        *self != Outcome::NotUsed
    }
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
