!! This is out of date !!

# Focus handling for ratatui

This crate works by adding a [FocusFlag] to each widget's state.

[Focus] is used to build a list of all relevant FocusFlags.
Focus holds references to all FocusFlags, in the order the widgets should
be navigated. Additionally, it has the area of the widget for mouse interaction.

The focus list is constructed freshly after each render to account
for any changed areas and other changes in the set of widgets.

The functions `Focus::next()`/`Focus::prev()` do the actual navigation.
They change the active FocusFlag and set flags for focus-lost and
focus-gained too.

A widget should implement [HasFocusFlag] for it's
state, but this is not strictly necessary.

## Event-Handling

Event-handling is implemented for crossterm and uses only Tab/Backtab
to navigate. Focus implements
[HandleEvent](https://docs.rs/rat-event/latest/rat_event/trait.HandleEvent.html),
and there are the functions handle_focus() and handle_mouse_focus() too.
All of these return `Outcome::Changed` whenever something interesting happened.

There are a few complications, though:

```rust ignore
fn focus(state: &State) -> Focus {
    Focus::new(&[
        &state.widget1,
        &state.widget2,
        &start.widget3
    ])
}

fn handle_input(event: &Event, state: &mut State) -> Result<Outcome, anyhow::Error> {
    // does all the keyboard and mouse navigation, and 
    // returns Outcome::Changed if something noteworthy changed. 
    let f = focus(state).handle(event, Regular);

    // flow_ok! is a macro from rat-event. It returns early if 
    // the result is *not* Outcome::NotUsed. 
    //
    // But: If the result here is the third variant Outcome::Unchanged,
    //      this would return early with Outcome::Unchanged and 
    //      would not know that there was something else too. 
    //      If you trigger the repaint with Outcome::Changed you 
    //      would never see the focus-change.
    //      
    //      One solution is to give `f` to the macro too, 
    //      using the `consider f` syntax. 
    flow_ok!(state.widget1.handle(event, Regular), consider f);
    flow_ok!(state.widget2.handle(event, Regular), consider f);
    flow_ok!(state.widget3.handle(event, Regular), consider f);

    // no widget used the event, the only possible outcome is
    // the result of focus.
    Ok(f)
}
```

This solution is a bit sub-par, admittedly.

If you are using [rat-salsa](https://crates.io/crates/rat-salsa) as your
application framework, there is a queue for extra results from event-handling.
You can add the result of focus().handle() directly to the queue, and
everything's well.

If you already have your own framework, you might want something similar.

## Mouse

Mouse support exists of course.

The trait HasFocusFlag has two methods to support this:

* area() - Returns the area of the widget that should react to mouse-clicks.
  If you want to prevent mouse-focus for your widget, just return Rect::default().
* z_areas() - Extends area(). Widgets can return a list of ZRect
  for mouse interaction. A ZRect is a Rect with an added z-index
  to handle overlapping areas. Those can occur whenever the widget
  renders a popup/overlay on top of other widgets.

  If z_areas is used, area must return the union of all Rects.
  Area is used as fast filter, z_areas are used for the details.

  This method is defaulted to return nothing which is good enough
  for most widgets.

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

The trait [HasFocus] indicates the existence of this behaviour.
Focus has a method `add_container()` for this too.

> There is a lighter version of a container too.
> [xxFocus::new_grp()xx] creates a list of widgets, but without a
> container area.

Focus can handle recursive containers too.

## FocusFlag and Focus

The FocusFlag internally uses Rc<Cell<>> for all the flags,
so it can be cheaply cloned and used separately from its origin
state struct. That way it is possible to hand out a mutable reference
to the state struct and a `Focus` if necessary. Which is all the
time, as you want to split up the event-handling function sooner
rather than later.

By cloning the FocusFlags, Focus needs no lifetime and
could be stored for a longer period. It still would need and
update for all involved areas and conditionally disabled widgets.
As there is no widget tree or similar in ratatui there is no
way to automagically do this, the sensible way is to drop the
Focus at the end of event-handling and rebuild it anew next time.

## HasFocusFlag

In addition to the before-mentioned methods there are

* `navigable()` - The widget can indicate that it is not reachable with
  key navigation.
* `primary_keys()` - Focus has a secret second key to leave a widget
  via keyboard: `<Esc>`. There area a few widgets (textarea) that want
  to use the normal Tab/Shift-Tab themselves. Such a widget can signal this
  divergent behaviour with this function.
* `is_focused()`, `lost_focus()`, `gained_focus()` - These are useful when
  writing the application. 

  