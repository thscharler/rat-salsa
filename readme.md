[![crates.io](https://img.shields.io/crates/v/rat-widget.svg)](https://crates.io/crates/rat-widget)
[![Documentation](https://docs.rs/rat-widget/badge.svg)](https://docs.rs/rat-widget)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-widget)

This crate is a part of [rat-salsa][refRatSalsa].

# Rat Widgets

This crate tries to provide an extended set of widgets with

- *Event handling*
- *Focus handling*
- *Builtin scrolling*
- *Speed*

## Event handling

Uses the trait defined in [rat-event][refRatEvent] to implement
event-handling for crossterm. All widgets are designed with other
event-handlers in mind.

## Focus handling

Uses [FocusFlag][refRatFocusFlag] defined by 
[rat-focus][refRatFocus] internally, to mark the focused widget. 
This is just a passive flag, that probably can be used with other 
focus systems. Or you use [rat-focus][refRatFocus].

## Scrolling

Where it makes sense the widgets implement internal scrolling.
They use [Scroll][refScroll] from [rat-scrolled][refRatScrolled].

## Speed

Rendering all the widgets tries hard not to need allocations and
extensive copying during rendering.

Special mentions:
- [rat-ftable::Table][refRatTable]: It uses an adapter for the data
  for rendering instead of creating Row/Cell structs.

# Widgets

All the widgets are plain ratatui widgets, and implement StatefulWidget and
the (experimental) StatefulWidgetRef traits.

Event handling uses [rat-event::HandleEvent][refRatEvent].
Currently, crossterm events are implemented.

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


[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/
[refRatEvent]: https://docs.rs/rat-event
[refRatFocus]: https://docs.rs/rat-focus
[refRatFocusFlag]: https://docs.rs/rat-focus/latest/rat_focus/struct.FocusFlag.html
[refScroll]: https://docs.rs/rat-scrolled/latest/rat_scrolled/struct.Scroll.html
[refRatScrolled]: https://docs.rs/rat-scrolled
[refRatTable]: https://docs.rs/rat-ftable
[refRatTextArea]: https://docs.rs/rat-widget/latest/rat_widget/textarea/index.html

