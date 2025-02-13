
# Guidelines

# Widgets / rat-widget

The mode how ratatui operates is nice to work with. With its
Frame/ Buffer it has a good layer that hides all the nitty gritty
of terminal escape sequences. Those are good as a wire protocol,
not as way to write an application.

The Widget/StatefulWidget traits are good for the rendering
parts too. I couldn't come up with any ideas that were worth a
break with the existing ones.  Others have tried it too, I'm not
convinced of what I've seen. Most of them try to copy concepts
from more dynamic languages and they are not a good fit for rust.

## Layout

One thing that comes up often is a split between layout/
rendering. I think this just is a broken idea to start with. Very
few widgets have any intrinsic size to work with, and even those
must live with the fact 'there is not enough space' in the end.
So the approach taken by Widget is sufficient.

## Widget vs State

Widget as a term is a bit misleading. It sounds like Widget is
the main part and the State is an afterthought.

I now see this as the split between retained and
render-only properties of a widget, two sides of one coin.
StatefulWidget::State is the retained half, and StatefulWidget
brings all things needed for rendering together. And Widget is
just an alias for StatefulWidget<State=()>.

Seen this way the lifetime parameter of many widgets is very much
needed for temporary borrowing render parameters and not a 
hindrance.

## Event handling

There is no such thing defined by ratatui, so I added my 10 ct
with [rat-event](https://docs.rs/rat-event/latest/rat_event/).

The trait defined there can deal with different event-types and
can define as many modes of operation as are needed.

If one of those modes happens to be a configurable key-map it is
fine, but hardcoding this mapping in the event-handler is fine
too.

And it has a return type that can at least communicate the
information 'this event has been used'. With that one can already
build combinators and short curcuit the event-loop. Every widget
can have its own return type and there is a baseline `Outcome`
 for interoperability.

## Event handling 2

With this trait, event-handling for widgets tend to be straight
matches from key-event to a function defined for the widget
state. All of which want to be public to

- replace keybindings
- allow some automation.

# Layered rendering and other tricks

Layers in the UI are not part of ratatui, but that can be
simulated by rendering the top layers last and processing any
events for them first.

Container widgets have a similar solution, the container renders
it's containery bits and gives back the areas where the caller
can render the content. That avoids storing any contained widget
information and the user of the container widget can use any of
the myriad datastructures available to manage the content.

Starting from that viewpoint there can be widgets that split
into parts before each part is being rendered. Or multistate
rendering where a widget gets defined with its render parameters,
transforms into a buffer where the user of the widget can render
its content before finally turning into the final widget which
shifts and clips the buffer before copying it to the main output
buffer. 

# Generally

I prefer to have all the wiring, buttons, jacks on the front
panel, not hidden behind some automagic layer.

# Application

An application is not a widget. The latter needs some degree of
isolation to be reusable in many contexts. The former is where
all the context, config, themeing, locale, i18n, databases,
business logic etc come together.

For this to work I want some shared overall context that is
available everywhere, some degree of isolation for the parts of
the application to not go insane from the exponentially growing
interactions, while still having a uniform means of communication
between all parts if necessary. All of which working to change
a shared data model, but without micromanaging every bit
of information, which is not necessary as we always render
everything anyway.
