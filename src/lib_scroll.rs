//!
//! Scrolling behaviour.
//!

use ratatui::layout::Rect;

/// Trait for a widget that can scroll.
pub trait ScrolledWidget {
    /// Get the scrolling behaviour of the widget.
    /// The area given is the initial space including possible scrollbars.
    fn need_scroll(&self, area: Rect) -> ScrollParam;
}

/// Widget scrolling information.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ScrollParam {
    pub has_hscroll: bool,
    pub has_vscroll: bool,
}

/// Trait for a widget-state that can scroll.
pub trait HasScrolling {
    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is probably shorter than the length of the content by whatever fills a page.
    /// This is the base for the scrollbar content_length.
    fn max_v_offset(&self) -> usize;

    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is probably shorter than the length of the content by whatever fills a page.
    /// This is the base for the scrollbar content_length.
    fn max_h_offset(&self) -> usize;

    /// Current vertical offset.
    fn v_offset(&self) -> usize;

    /// Current horizontal offset.
    fn h_offset(&self) -> usize;

    /// Change the offset.
    ///
    /// Due to overscroll it's possible that this is an invalid offset for the widget.
    /// The widget must deal with this situation.
    fn set_v_offset(&mut self, offset: usize);

    /// Change the offset.
    ///
    /// Due to overscroll it's possible that this is an invalid offset for the widget.
    /// The widget must deal with this situation.
    fn set_h_offset(&mut self, offset: usize);

    /// Scroll up by n.
    fn scroll_up(&mut self, n: usize) {
        self.set_v_offset(self.v_offset().saturating_sub(n));
    }

    /// Scroll down by n.
    fn scroll_down(&mut self, n: usize) {
        self.set_v_offset(self.v_offset() + n);
    }

    /// Scroll up by n.
    fn scroll_left(&mut self, n: usize) {
        self.set_h_offset(self.h_offset().saturating_sub(n));
    }

    /// Scroll down by n.
    fn scroll_right(&mut self, n: usize) {
        self.set_h_offset(self.h_offset() + n);
    }
}
