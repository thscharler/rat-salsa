//!
//! Scrolling behaviour.
//!

use ratatui::layout::Rect;

/// Trait for a widget that can scroll.
pub trait ScrolledWidget {
    /// Get the scrolling behaviour of the widget.
    ///
    /// The area is the area for the scroll widget minus any block set on the [Scrolled] widget.
    /// It doesn't account for the scroll-bars.
    fn need_scroll(&self, area: Rect) -> ScrollParam;
}

/// Widget scrolling information.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ScrollParam {
    pub has_hscroll: bool,
    pub has_vscroll: bool,
}

/// Trait for the state of a widget that can scroll.
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
pub trait HasScrolling {
    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    fn max_v_offset(&self) -> usize;

    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    fn max_h_offset(&self) -> usize;

    /// Vertical page-size at the current offset.
    fn v_page_len(&self) -> usize;

    /// Horizontal page-size at the current offset.
    fn h_page_len(&self) -> usize;

    /// Current vertical offset.
    fn v_offset(&self) -> usize;

    /// Current horizontal offset.
    fn h_offset(&self) -> usize;

    /// Change the vertical offset.
    ///
    /// Due to overscroll it's possible that this is an invalid offset for the widget.
    /// The widget must deal with this situation.
    fn set_v_offset(&mut self, offset: usize);

    /// Change the horizontal offset.
    ///
    /// Due to overscroll it's possible that this is an invalid offset for the widget.
    /// The widget must deal with this situation.
    fn set_h_offset(&mut self, offset: usize);

    /// Scroll up by n items.
    fn scroll_up(&mut self, n: usize) {
        self.set_v_offset(self.v_offset().saturating_sub(n));
    }

    /// Scroll down by n items.
    fn scroll_down(&mut self, n: usize) {
        self.set_v_offset(self.v_offset() + n);
    }

    /// Scroll up by n items.
    fn scroll_left(&mut self, n: usize) {
        self.set_h_offset(self.h_offset().saturating_sub(n));
    }

    /// Scroll down by n items.
    fn scroll_right(&mut self, n: usize) {
        self.set_h_offset(self.h_offset() + n);
    }
}
