[![crates.io](https://img.shields.io/crates/v/rat-scrolled.svg)](https://crates.io/crates/rat-scrolled)
[![Documentation](https://docs.rs/rat-scrolled/badge.svg)](https://docs.rs/rat-scrolled)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-scrolled)

This crate is a part of [rat-salsa][refRatSalsa].

# Scroll

[Scroll](Scroll) adds support for widgets that want to scroll their
content.

Scroll works analogous to Block, it can be set on the widget
struct where it is supported. The widget can decide which 
scrolling it can do, horizontal vertical or both.

# Adding scroll to a widget

- Add [Scroll](Scroll) to the widget struct.
- Use [ScrollArea](ScrollArea) for the layout and rendering of
  the combination of Block+Scroll+Scroll your widget supports.
- Add [ScrollState](ScrollState) to the widget state struct. 
- Create a [ScrollAreaState](ScrollAreaState) for event-handling.


[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/
