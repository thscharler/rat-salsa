# Rat-Widgets

This crate tries to provide an extended set of widgets with

- Eventhandling (currently crosstem, but not limited)
- Focus
- Scrolling
- Composability

It combines different aspects that have all been published as
separate crates:

* [rat-event](https://docs.rs/rat-event/latest/rat_event/): Define a generalized event-handling trait.
* [rat-focus](https://docs.rs/rat-focus/latest/rat_focus/): Focus handling for widgets.
* [rat-scrolled](https://docs.rs/rat-scrolled/latest/rat_scrolled/): Widgets for scrolling.
* [rat-input](https://docs.rs/rat-input/latest/rat_input/): Collection of widgets, but kept at a baseline level.
* [rat-ftable](https://docs.rs/rat-ftable/latest/rat_ftable/): Table widget for large data-sets.

## Widget list

* Button

Basic button.

* TextInput

Plain text input.

* DateInput

Date values.

* MaskedInput

Complex input masks.

* TextArea

Textarea with text styling. Uses [ropey](https://docs.rs/ropey/latest/ropey/) backend,
so should be good for long text too.

* FTable

FTable with backing TableData trait for large tables.
Renders in O(1) in regard of the data size.

* List

Wrapper around ratatui::List with custom selection models.

* MenuLine

Basic menu.

* MsgDialog

Basic message dialog.

* StatusLine

Statusline with multiple sections.

* calender::Month

Calender display.