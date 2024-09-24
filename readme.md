# Rat-Cursor

## Rationale

This crate defines just the trait [HasScreenCursor] for use in
other crates. This aims to overcome the shortcomings of ratatui
to handle cursor positioning by widgets.

In the long run I hope there will be a solution within ratatui
which will make this obsolete, but I need some solution now.

```rust
pub trait HasScreenCursor {
    fn screen_cursor(&self) -> Option<(u16, u16)>;
}
```

This trait is implemented for the widget-state struct.

> It's implemented for the state struct because the widget
> might need to run the full layout process to know the cursor
> position. Which would approximately double the rendering
> process.

Instead of setting the cursor position during rendering somehow,
the rendering process stores the cursor position in the state
struct, where it can be retrieved later on.

The trait returns a screen-position, but only in the case that it
actually needs the cursor to be displayed:

* The cursor is not scrolled off-screen.
* The widget has some kind of input-focus.

In the case of a container widget it can cascade down to its
components:

```rust ignore
    fn screen_cursor(&self) -> Option<(u16, u16)> {
    self.widget1.screen_cursor()
        .or(self.widget2.screen_cursor())
        .or(self.widget3.screen_cursor())
}
```
