## Rat-Event

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