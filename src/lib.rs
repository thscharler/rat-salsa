pub mod adapter;
pub mod scrolled;
mod util;
pub mod view;
pub mod viewport;

use ratatui::layout::Rect;
use std::cmp::max;

pub trait ScrollingWidget<State> {
    /// Widget wants a (horizontal, vertical) scrollbar.
    fn need_scroll(&self, area: Rect, state: &mut State) -> (bool, bool);
}

/// Trait for the widget-state of a scrollable widget.
///
/// This trait works purely in item-space, none of the values
/// correspond to screen coordinates.
///
/// The current visible page is represented as the pair (offset, page_len).
/// The limit for scrolling is given as max_offset, which is the maximum offset
/// where a full page can still be displayed. Note that the total length of
/// the widgets data is NOT max_offset + page_len. The page_len can be different for
/// every offset selected. Only if the offset is set to max_offset and after
/// the next round of rendering len == max_offset + page_len will hold true.
///
/// The offset can be set to any value possible for usize. It's the widgets job
/// to limit the value if necessary. Of course, it's possible for a user of this
/// trait to set their own limits.
pub trait ScrollingState {
    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    fn vertical_max_offset(&self) -> usize;
    /// Current vertical offset.
    fn vertical_offset(&self) -> usize;
    /// Vertical page-size at the current offset.
    fn vertical_page(&self) -> usize;
    /// Suggested scroll per scroll-event.
    fn vertical_scroll(&self) -> usize {
        max(self.vertical_page() / 10, 1)
    }

    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    fn horizontal_max_offset(&self) -> usize;
    /// Current horizontal offset.
    fn horizontal_offset(&self) -> usize;
    /// Horizontal page-size at the current offset.
    fn horizontal_page(&self) -> usize;
    /// Suggested scroll per scroll-event.
    fn horizontal_scroll(&self) -> usize {
        max(self.horizontal_page() / 10, 1)
    }

    /// Change the vertical offset.
    ///
    /// Due to overscroll it's possible that this is an invalid offset for the widget.
    /// The widget must deal with this situation.
    ///
    /// The widget returns true if the offset changed at all.
    fn set_vertical_offset(&mut self, offset: usize) -> bool;

    /// Change the horizontal offset.
    ///
    /// Due to overscroll it's possible that this is an invalid offset for the widget.
    /// The widget must deal with this situation.
    ///
    /// The widget returns true if the offset changed at all.
    fn set_horizontal_offset(&mut self, offset: usize) -> bool;

    /// Scroll up by n items.
    /// The widget returns true if the offset changed at all.
    fn scroll_up(&mut self, n: usize) -> bool {
        self.set_vertical_offset(self.vertical_offset().saturating_sub(n))
    }

    /// Scroll down by n items.
    /// The widget returns true if the offset changed at all.
    fn scroll_down(&mut self, n: usize) -> bool {
        self.set_vertical_offset(self.vertical_offset() + n)
    }

    /// Scroll up by n items.
    /// The widget returns true if the offset changed at all.
    fn scroll_left(&mut self, n: usize) -> bool {
        self.set_horizontal_offset(self.horizontal_offset().saturating_sub(n))
    }

    /// Scroll down by n items.
    /// The widget returns true if the offset changed at all.
    fn scroll_right(&mut self, n: usize) -> bool {
        self.set_horizontal_offset(self.horizontal_offset() + n)
    }
}

// /// A widget that can differentiate between these states can use this as a flag.
// /// It's the job of the widget to implement the difference.
// ///
// /// But in the end this is probably to many choices for most widgets, so this is
// /// pretty useless. A widget will better signal its capabilities in its
// /// own terminology.
// ///
// #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum ScrollingPolicy {
//     Selection,
//     #[default]
//     ItemOffset,
//     LineOffset,
// }

pub mod event {
    use rat_event::UsedEvent;
    pub use rat_event::{FocusKeys, HandleEvent, MouseOnly};

    /// Result value for event-handling. Used widgets in this crate.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Outcome<R> {
        /// Outcome of the inner widget.
        Inner(R),
        /// The given event was not handled at all.
        NotUsed,
        /// The event was handled, no repaint necessary.
        Unchanged,
        /// The event was handled, repaint necessary.
        Changed,
    }

    impl<R> UsedEvent for Outcome<R>
    where
        R: UsedEvent,
    {
        fn used_event(&self) -> bool {
            match self {
                Outcome::Inner(v) => v.used_event(),
                Outcome::NotUsed => false,
                Outcome::Unchanged => true,
                Outcome::Changed => true,
            }
        }
    }
}

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
