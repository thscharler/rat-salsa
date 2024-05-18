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

## Composition

There is support for composite widgets too. You can use `Focus::new_accu()`
to create the focus cycle. There you can give one extra FocusFlag
that will contain a summary of the focus-state for all contained
widgets.

If any of the contained widgets is focused, the summary will have the
focus flag too. Lost and Gained work that if any contained widget
gained the focus and no other contained widget lost it, only
then will the composite widget have a gained flag set. Lost works
vice versa.

There is the method [Focus::append], which can append another focus cycle.
This can stack to arbitrary depth.

There is a nice demo to illustrate this with `focus_recursive2`.





