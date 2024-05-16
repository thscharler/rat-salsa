# Scrolled and Viewport widgets

## Requirements for Scrolled

There are two traits for the widget that should be scrolled.

For the widget struct:

```rust
pub trait ScrollingWidget<State> {
    /// Widget wants a (horizontal, vertical) scrollbar.
    fn need_scroll(&self, area: Rect, state: &mut State) -> (bool, bool);
}
```

For the widget state:

```rust
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
```

## Widget [Scrolled](crate::Scrolled)

Does the scrolling and display of the ScrollBars.





