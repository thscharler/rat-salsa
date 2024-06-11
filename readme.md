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
* [rat-input](https://docs.rs/rat-input): Collection of widgets, but kept at a baseline level.
* [rat-ftable](https://docs.rs/rat-ftable): Table widget for large data-sets.

This crate is part of [rat-salsa](https://docs.rs/rat-salsa).

## Widget list

* RButton
* RTextInput
* RDateInput
* RNumberInput
* RMaskedInput - Base for DateInput and NumberInput. Allows detailed control
  what text can be input at each position of the field.
* RTextArea - text-area with text styling. Uses [ropey](https://docs.rs/ropey/latest/ropey/)
  as backend, so should be good for long text too.
* RTable - Table for big lists. The traits TableData and TableDataIter provide
  facades, so the table can directly access your data.
* REditTable - Support for inline editing in a table.
* RList - Wrapper around ratatui::List with custom selection models.
* RMenuLine - Simple menu-line.
* RMsgDialog
* RStatusLine
* calender::RMonth - Month display. Uses chrono's I18N.