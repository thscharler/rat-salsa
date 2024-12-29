![beta 3](https://img.shields.io/badge/stability-β--3-850101)
[![crates.io](https://img.shields.io/crates/v/rat-widget.svg)](https://crates.io/crates/rat-widget)
[![Documentation](https://docs.rs/rat-widget/badge.svg)](https://docs.rs/rat-widget)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-widget)

This crate is a part of [rat-salsa][refRatSalsa].

For examples see [rat-widget GitHub][refGitHubWidget]

* [Changes](https://github.com/thscharler/rat-widget/blob/master/changes.md)

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

* [Button](button/index.html)
* [Choice](choice/index.html)
* [Clipper](clipper/index.html)
* [Calendar](calendar/index.html)
* [DateInput](date_input/index.html) (using chrono)
* [EditList](list/edit/index.html)
* [EditTable](table/edit/index.html)
* [FileDialog](file_dialog/index.html)
* [TextInput](input/index.html)
* [MaskedInput](masked_input/index.html)
* [Menubar](menubar/index.html)
* [MenuLine](menuline/index.html)
* [MsgDialog](msgdialog/index.html)
* [NumberInput](number_input/index.html) (using format_num_pattern)
* [SinglePager and DualPager](pager/index.html)
* [PopupMenu](popup_menu/index.html)
* [Split](splitter/index.html)
* [StatusLine](statusline/index.html)
* [Tabbed](tabbed/index.html)
* [Table](table/index.html)
* [TextArea](textarea/index.html)
* [View](view/index.html)

and some adapters for ratatui widgets

* [List](list/index.html)
* [Paragraph](paragraph/index.html)

[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/

[refRatEvent]: https://docs.rs/rat-event

[refRatFocus]: https://docs.rs/rat-focus

[refRatFocusFlag]: https://docs.rs/rat-focus/latest/rat_focus/struct.FocusFlag.html

[refScroll]: https://docs.rs/rat-scrolled/latest/rat_scrolled/struct.Scroll.html

[refRatScrolled]: https://docs.rs/rat-scrolled

[refRatTable]: https://docs.rs/rat-ftable

[refRatTextArea]: textarea/index.html

[refGitHubWidget]: https://github.com/thscharler/rat-widget/tree/master/examples

