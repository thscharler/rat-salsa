# Event handling

```rust
    fn event(
        &mut self,
        event: &MinimalEvent,
        ctx: &mut rat_salsa::AppContext<'_, GlobalState, MinimalEvent, Error>,
    ) -> Result<Control<MinimalEvent>, Error> {
```

rat-salsa requires the application to define its own event
type and provide conversions from every outside event
that the application is interested in. The details of the
conversion are left to the application, but mapping everything
to an application defined enum is a good start.

```rust
    #[derive(Debug)]
    pub enum MinimalEvent {
        Timer(TimeOut),
        Event(crossterm::event::Event),
        Rendered,
        Message(String),
    }
```

rat-salsa polls all event-sources and takes note which one
has an event to process. Then it takes this notes and
starts with the first event-source and asks it to send
its event.

The event-source converts its event-type to the application
event and sends it off to the `event()` function of the
main AppState.

This results either in a specific action like 'render' or
in a followup event. Followups are sent down to event() too,
and result in another followup.

At some point this ends with a result `Control::Continue`.
This is the point where the event-loop goes back to its
notes and asks the next event source to send its event.

> Note that every event-source with an outstanding event
> is processed before asking all event-sources if there
> are new events. This prevents starving event-sources
> further down the list.

There are no special cases, or any routing of events,
everything goes straight to the `event()` function.

The event() function gets the extra parameter
[ctx][refAppContext] for access to application global data.

## Result

The result is a `Result<Control<Event>, Error>`, that tells
rat-salsa how to proceed.

- [Control::Continue][refControl]: Continue with the next event.

- [Control::Unchanged][refControl]: Event has been used, but
  requires no rendering. Just continues with the next event.

  Within the application this is used to break early.

- [Control::Changed][refControl]: Event has been used, and
  a render is necessary. Continues with the next event after
  rendering. May send a RenderedEvent immediately after
  the render occurred before any other events.

- [Control::Event(m)][refControl]: This contains a followup
  event. It will be put on the current events queue and
  processed in order. But before polling for new events.

  The individual AppWidgets making up the application are quite
  isolated from other parts and just have access to their own
  state and some global application state.

  All communication across AppWidgets can use this mechanism
  to send special events/messages.

- [Control::Quit][refControl]: Ends the event-loop and resets the
  terminal. This returns from run_tui() and ends the application
  by running out of main.

[refRatEvent]: https://docs.rs/rat-event/latest/rat_event/

[refControl]: https://docs.rs/rat-salsa/latest/rat_salsa/enum.Control.html

[refRatWidget]: https://docs.rs/rat-widget/latest/rat_widget/

[refAppContext]: https://docs.rs/rat-salsa/latest/rat_salsa/struct.AppContext.html

[refConsumedEvent]: https://docs.rs/rat-event/latest/rat_event/trait.ConsumedEvent.html
