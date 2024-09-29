[![crates.io](https://img.shields.io/crates/v/rat-cursor.svg)](https://crates.io/crates/rat-cursor)
[![Documentation](https://docs.rs/rat-cursor/badge.svg)](https://docs.rs/rat-cursor)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-cursor)

This crate is a part of [rat-salsa][refRatSalsa].

# Rat-Cursor

## Why?

This crate defines just the trait [HasScreenCursor]() for use in
other crates. This aims to overcome the shortcomings of ratatui
to handle cursor positioning by widgets.

> In the long run I hope there will be a solution within ratatui
> which will make this obsolete, but for now ...

```rust
pub trait HasScreenCursor {
    fn screen_cursor(&self) -> Option<(u16, u16)>;
}
```

## Use

### Widget

This trait is implemented for the widget-state struct.

> It's implemented for the state struct because the widget
> might need to run the full layout process to know the cursor
> position. Which would approximately double the rendering
> process.

Instead of setting the cursor position during rendering somehow,
the rendering process stores the cursor position in the state
struct, where it can be retrieved later on.

The trait returns a screen-position, but only if it actually
needs the cursor to be displayed:

* The cursor is not scrolled off-screen.
* The widget has some kind of input-focus.

### Container widget

A container widget can cascade down to its components.

```rust ignore
    fn screen_cursor(&self) -> Option<(u16, u16)> {
    self.widget1.screen_cursor()
        .or(self.widget2.screen_cursor())
        .or(self.widget3.screen_cursor())
}
```

[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/

