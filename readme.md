!! this is out of date !!

[![crates.io](https://img.shields.io/crates/v/rat-widget.svg)](https://crates.io/crates/rat-widget)
[![Documentation](https://docs.rs/rat-widget/badge.svg)](https://docs.rs/rat-widget)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-widget)

# Rat-Widget

This crate tries to provide an extended set of widgets with

- *Event handling*
- *Focus handling*
- *Scrolling*
- *Speed*

## Event handling

Uses the trait defined in [rat-event](https://docs.rs/rat-event) to
implement event-handling for crossterm. All widgets are designed with
other event-handlers in mind. They provide single entry-functions
that map 1:1 with events most of the time.

## Focus handling

Uses FocusFlag defined by [rat-focus](https://docs.rs/rat-focus)
internally, to mark the focused widget.

This is just a passive flag, that probably can be used with other
focus systems. Or you use rat-focus, it is independent of any
frameworks and works by collecting references to the FocusFlags
that are part of the focus and navigating with this information.

## Scrolling

Where it makes sense the widgets implement an internal offset
for scrolling. They can display the scrollbar themselves by
using `Scroll` from [rat-scrolled](https://docs.rs/rat-scrolled).
This is a utility that works much like the ratatui `Block` and
can be plugged into the widget.

## Speed

Rendering all the widgets tries hard not to need allocations and
extensive copying of the underlying data.

Special mention here are [rat_ftable::Table](https://docs.rs/rat-ftable),
which uses an adapter for the data instead of creating Row/Cell structs.
The adapter is only called for the visible cells when rendering.
This way there is no real limit to the size of your data. Anything
that can provide a slice look-alike or an iterator is fine.

Second special mention is [rat_widget::TextArea](https://docs.rs/rat-widget/latest/rat_widget/textarea/index.html)
which uses [Ropey](https://docs.rs/ropey/latest/ropey/) for the underlying
storage. It also has range based styling builtin.

# Rat-Salsa

This crate is part of [rat-salsa](https://docs.rs/rat-salsa).

# Widgets

All the widgets are plain ratatui widgets, and implement StatefulWidget and
the (experimental) StatefulWidgetRef traits.

Event handling uses [rat-event::HandleEvent](https://docs.rs/rat-event/latest/rat_event/trait.HandleEvent.html)
for uniformity, but provide plain functions too. Currently, crossterm events
are implemented.

* [Button](https://docs.rs/rat-widget/latest/rat_widget/button/index.html)
* [Calendar](https://docs.rs/rat-widget/latest/rat_widget/calendar/index.html)
* [DateInput](https://docs.rs/rat-widget/latest/rat_widget/date_input/index.html) (using chrono)
* [EditList](https://docs.rs/rat-widget/latest/rat_widget/list/edit/index.html)
* [EditTable](https://docs.rs/rat-widget/latest/rat_widget/table/edit/index.html)
* [FileDialog](https://docs.rs/biosys/rat-widget/latest/rat_widget/file_dialog/index.html)
* [TextInput](https://docs.rs/rat-widget/latest/rat_widget/input/index.html)
* [MaskedInput](https://docs.rs/rat-widget/latest/rat_widget/masked_input/index.html)
* [Menubar](https://docs.rs/rat-widget/latest/rat_widget/menubar/index.html)
* [MenuLine](https://docs.rs/rat-widget/latest/rat_widget/menuline/index.html)
* [MsgDialog](https://docs.rs/rat-widget/latest/rat_widget/msgdialog/index.html)
* [NumberInput](https://docs.rs/rat-widget/latest/rat_widget/number_input/index.html) (using format_num_pattern)
* [PopupMenu](https://docs.rs/rat-widget/latest/rat_widget/popup_menu/index.html)
* [Split](https://docs.rs/rat-widget/latest/rat_widget/splitter/index.html)
* [StatusLine](https://docs.rs/rat-widget/latest/rat_widget/statusline/index.html)
* [Table](https://docs.rs/rat-widget/latest/rat_widget/table/index.html)
* [TextArea](https://docs.rs/rat-widget/latest/rat_widget/textarea/index.html)
* [View](https://docs.rs/rat-widget/latest/rat_widget/view/index.html)
* [Viewport](https://docs.rs/rat-widget/latest/rat_widget/viewport/index.html)

and some adapters for ratatui widgets

* [List](https://docs.rs/rat-widget/latest/rat_widget/list/index.html)
* [Paragraph](https://docs.rs/rat-widget/latest/rat_widget/paragraph/index.html)