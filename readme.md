# Rat-Widgets

This crate tries to provide an extended set of widgets with

- Eventhandling (currently crossterm, but not limited)
- Focus
- Scrolling
- Wrappers for other external widgets.

It combines different aspects that have all been published as
separate crates:

* [rat-event](https://docs.rs/rat-event): Define a generalized event-handling trait.
* [rat-focus](https://docs.rs/rat-focus): Focus handling for widgets.
* [rat-scrolled](https://docs.rs/rat-scrolled): Widgets for scrolling.
* [rat-ftable](https://docs.rs/rat-ftable): Table widget for large data-sets.

This crate is part of [rat-salsa](https://docs.rs/rat-salsa).

# Widgets

These widgets are ratatui widgets.

Eventhandling is currently crossterm only.
In practice event-handling is calling 1 or 2 functions on the state, so this
should be easy to map to other systems. (Contributions welcome :)

## [TextArea](crate::textarea)

Editable text area.

* Range based text styles.
* Text selection with keyboard + mouse
* Possible states as style: Focused
* Emoji supported.

![image](https://github.com/thscharler/rat-input/blob/master/textarea.gif?raw=true)

## [TextInput](crate::input)

Basic text input field.

* Text selection with keyboard + mouse
* Possible states as styles: Focused, Invalid

## [MaskedInput](crate::masked_input)

Text input with an input mask.

* Text selection with keyboard + mouse
* Possible states as styles: Focused, Invalid
* Pattern based input -> "##,###,##0.00"
    * number patterns: `09#-+.,`
    * numeric text: `HhOoDd`
    * text: `lac_`
    * arbitrary separators between sub-fields
* info-overlay for sub-fields without value
* Localization with [rat-input::NumberSymbols] based on [pure-rust-locales](pure-rust-locales)

## [Button](crate::button::Button)

Simple button widget.

## [DateInput](crate::date_input), [NumberInput](crate::number_input)

Date input with format strings parsed by [chrono](https://docs.rs/chrono/latest/chrono/).
Number input with format strings parsed
by [format_num_pattern](https://docs.rs/format_num_pattern/latest/format_num_pattern/)

## [Month](crate::calendar)

Widget for calender display.

## [MenuLine](crate::menuline), [PopupMenu](crate::popup_menu) and [MenuBar](crate::menubar)

Menu widgets.

## [StatusLine](crate::statusline)

Statusline with multiple segments.

## TODO

... more widgets 