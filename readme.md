# Rat-Event

## Rationale

This crate defines a general event-handling trait,
that can be used along with ratatui widgets.

The main idea here is _not_ to try to unify the different event sources.

Instead, provide a trait that can be implemented per event-type.

And as there seem to be widely diverging opinions on what the right
key-bindings should be, we add a qualifier type to allow for more than
one key-binding. The widget should provide functions for all the
interactions anyway, so this mapping can be a one-liner most of the time.

This accomplishes:

* The widget creator can support different event types.
* The widget creator can supply several key bindings.
* The application writer can define ves own key bindings, and as the
  widget is designed this way it's possible to do this.

## Composition

The [HandleEvent::handle] has a generic return type, so each widget can
define its own result to convey any state changes.

To allow a minimal level of composition of different return types,
there is the trait ConsumedEvent. This allows for early returns,
even if the details of the return type are not known.

The [Outcome] enum gives a minimum of information that should be provided.
It is very helpful if any outcome from a widget allows conversion to and
from [Outcome].

## Known qualifiers

These are the predefined qualifiers

* [Regular] - Event-handlers of this kind process all events relevant
  for a widget. What happens exactly may depend on the internal state
  of the widget, primarily if it has the input-focus or not.

  See [rat-focus](https://docs.rs/rat-focus/) for one way to do focus-handling.

* [MouseOnly] - Event-handler for all interactions with a widget that
  doesn't have the input focus. Usually only mouse-events here, but
  hot-keys are possible too.

* [Popup] - Specialized event-handler for widgets that draw overlays/popups
  above other widgets.
* [Dialog] - Specialized even-handler for modal widgets. Such an event-handler
  consumes _all_ events if active, and prevents other widgets from reacting
  at all.

## Utilities

# ct_event!

A neat little thing, that generates pattern matches for mouse events.
Has a much terser syntax than composing struct patterns.

# select functions

The functions `row_at_clicked`, `column_at_clicked`, `row_at_drag` and
`column_at_drag` allow easier identification which of a slice of Rect
is actually meant.

# Mouseflags

Identifying double-clicks and mouse-drag is not trivial.
This struct helps, add it to your widget state.






