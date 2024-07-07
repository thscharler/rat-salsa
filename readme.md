# Focus handling

This crate works by adding a [FocusFlag](crate::FocusFlag) to each widget's state.

[Focus](crate::Focus) is used to collect the list of FocusFlags.
It only holds references to the FocusFlags in the order the widgets
are navigated. It must be constructed freshly after each render,
as it holds copies of the areas of all the widgets.

Focus::next()/Focus::prev() do the actual navigation. They change the
active FocusFlag. Additionally, there are fields for focus-lost and
focus-gained that are changed too.

The widget should implement [HasFocusFlag](crate::HasFocusFlag) for it's state, which
makes coding easier.

## Event-Handling

Event-handling is implemented for crossterm and uses only Tab/Backtab.

Focus implements [HandleEvent](https://docs.rs/rat-event/latest/rat_event/trait.HandleEvent.html)
for event-handling.

You can use also the function `handle_focus()` if you prefer.

These return Outcome::Changed whenever something focus related changes.

## Mouse

Mouse support exists of course.

The trait HasFocusFlag has two methods to support this:

* area() - Returns the area of the widget that should react to mouse-clicks.
* z_areas() - Extends area(). Widgets can return more than one ZRect
  for mouse interaction, and each of those has an extra z-index to
  handle overlapping areas. Those can occur whenever the widget wants
  to render a popup/overlay on top of other widgets.

  This method is defaulted to return nothing which is good enough
  for most widgets.

  If z_areas is used, area must return the union of all Rects.
  Area is used as fast filter, z_areas are used for the details.

## Macros

There are the macros [on_lost](crate::on_lost!), [on_gained](crate::on_gained!)
and [match_focus](crate::match_focus!) that ease the use of the focus-flags,
providing a match like syntax.

## Composition

There is support for composite widgets too.

Use `Focus::new_container()` to create the list of widgets in a container.
This takes one extra FocusFlag which creates a summary of the individual
FocusFlags of each widget. This way the container widget has an answer to
'Does any of my widgets have the focus?'. The container can also have
its own area. If the container area is clicked and not some specific widget,
the first widget in the container gets the focus.

Lost/Gained also work for the whole container.

The trait [HasFocus](crate::HasFocus) indicates the existence of this behaviour.
Focus has a method `add_container()` for this too.

        There is a lighter version of a container too. 
        `Focus::new_grp()` creates a list of widgets, but without a container
        area.

Focus can handle recursive containers too.

## FocusFlag and Focus

The FocusFlag is constructed as `Rc<Cell<>>` of its flags, so it can
be cloned and held for some time without interfering with borrowing
from the state-struct.

With the use of Cell the Focus struct can work as a plain borrow after
construction.

## HasFocusFlag

In addition to the before-mentioned methods there are

* navigable() - The widget can indicate that it is not reachable with
  key navigation.
* is_focused(), lost_focus(), gained_focus() - These are useful when
  writing the application. 

  