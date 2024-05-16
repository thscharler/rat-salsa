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
