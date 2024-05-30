# Rat-Event

## Rationale

This crate defines a general event-handling trait,
that can be used to create ratatui widgets.

The main idea here is _not_ to try to unify the different event sources.

Instead, provide a trait that can be implemented once per event-type.
And as there seem to be widely diverging opinions on what the right
key-bindings should be, we add a marker type to allow for more than
one key-binding.

If the widget is designed in a way, that each key binding only needs to call
a function or two, the code duplication is negligible.

This accomplishes:

* Widget author can support different event types.
* Widget author can supply several key bindings.
* Application author can define ves own key bindings, and as the
  widget is designed this way it's possible to do this.
* As there is a trait for this, widgets can be composed generically.

For examples see [rat-input](https://docs.rs/rat-input/latest/rat_input/)

## Composition

To allow a minimal level of composition of different return types,
there is the trait ConsumedEvent. This allows for early returns,
even if the details of the return type are not known.

## Utilities

# ct_event!

A neat little thing, that generates pattern matches for mouse events.
Has a much terser syntax than composing struct patterns.

# selectors

The functions `row_at_clicked`, `column_at_clicked`, `row_at_drag` and
`column_at_drag` allow easier identification which of a slice of Rect
is actually meant.

# Outcome

Reference result type for event-handling. This is just a minimal
baseline. Feel free to define your own; maybe add a conversion to
Outcome.

# Mouseflags

Identifying double-clicks and mouse-drag is not trivial.
This struct helps, add it to your widget state.






