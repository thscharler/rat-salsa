# Focus handling

This crate works by adding a [FocusFlag](crate::FocusFlag) to each
widgets state.

[Focus](crate::Focus) is list of references to all relevant focus-flags.
It has methods next() and prev() that can change the focus. This way
each widget has its focused-state nearby and the list of focusable
widget can be constructed flexibly.

The trait [HasFocusFlag](crate::HasFocusFlag) mediates between the
two sides.

## Macros

There are the macros [on_lost](crate::on_lost!), [on_gained](crate::on_gained!)
and [match_focus](crate::match_focus!) that ease the use of the focus-flags,
providing a match like syntax.
