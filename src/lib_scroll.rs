//!
//! Scrolling behaviour.
//!

/// Trait for a widget that can scroll vertically.
pub trait HasVerticalScroll {
    /// State needs scrolling?
    fn need_vscroll(&self) -> bool {
        true
    }

    /// Vertical length.
    fn vlen(&self) -> usize;

    /// Vertical offset.
    fn voffset(&self) -> usize;

    /// Change the offset
    fn set_voffset(&mut self, offset: usize);

    /// Vertical page size.
    fn vpage(&self) -> usize;

    /// Scroll up by n.
    fn vscroll_up(&mut self, n: usize) {
        let offset = self.voffset();
        if offset > n {
            self.set_voffset(offset - n);
        } else {
            self.set_voffset(0);
        }
    }

    /// Scroll down by n.
    fn vscroll_down(&mut self, n: usize) {
        let offset = self.voffset();
        let len = self.vlen();
        if offset + n < len {
            self.set_voffset(offset + n);
        } else {
            self.set_voffset(len);
        }
    }
}
