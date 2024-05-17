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

    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length of the content by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    fn horizontal_max_offset(&self) -> usize;
    /// Current horizontal offset.
    fn horizontal_offset(&self) -> usize;
    /// Horizontal page-size at the current offset.
    fn horizontal_page(&self) -> usize;

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
}
```

## Widget [Scrolled](crate::scrolled::Scrolled)

Does the scrolling and display of the ScrollBars.

## Widget [View](crate::view::View) and [Viewport](crate::viewport::Viewport)

Create a separate buffer, render the widget to this temp buffer.
Then apply a col/row based offset and copy the temp buffer to
the actual one.

The reason for two of those is: I can't impl the StatefulWidget
trait in a way that it works with both Widget and StatefulWidget.
So View works for Widget and Viewport for StatefulWidget.

There are convenience methods in Scrolled to add a View/Viewport.






