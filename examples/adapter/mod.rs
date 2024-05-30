use rat_input::event::ConsumedEvent;

pub mod list;
pub mod paragraph;
pub mod table;
pub mod tree;

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
