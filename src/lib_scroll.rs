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
    pub hlen: usize,
    pub vlen: usize,
    pub has_hscroll: bool,
    pub has_vscroll: bool,
}

/// Trait for a widget-state that can scroll.
pub trait HasScrolling {
    ///
    fn has_vscroll(&self) -> bool;
    ///
    fn has_hscroll(&self) -> bool;

    /// Vertical length.
    fn vlen(&self) -> usize;

    /// Horizontal length.
    fn hlen(&self) -> usize;

    /// Maximum offset.
    fn vmax(&self) -> usize;

    /// Maximum offset.
    fn hmax(&self) -> usize;

    /// Vertical offset.
    fn voffset(&self) -> usize;

    /// Horizontal offset.
    fn hoffset(&self) -> usize;

    /// Change the offset
    fn set_voffset(&mut self, offset: usize);

    /// Change the offset
    fn set_hoffset(&mut self, offset: usize);

    /// Scroll up by n.
    fn scroll_up(&mut self, n: usize) {
        self.set_voffset(self.voffset().saturating_sub(n));
    }

    /// Scroll down by n.
    fn scroll_down(&mut self, n: usize) {
        self.set_voffset(self.voffset() + n);
    }

    /// Scroll up by n.
    fn scroll_left(&mut self, n: usize) {
        self.set_hoffset(self.hoffset().saturating_sub(n));
    }

    /// Scroll down by n.
    fn scroll_right(&mut self, n: usize) {
        self.set_hoffset(self.hoffset() + n);
    }
}
