
# Event handling

```rust
        fn crossterm(
            &mut self,
            event: &Event,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<TurboMsg>, Error>
```

rat-salsa distributes the events with the plain functions in
AppState. It doesn't do any routing to specific widgets or such.
Any further distribution of events is up to the application.

All handling functions get the extra [ctx][refAppContext] for
access to application global data.

The result is a `Result<Control<Action>, Error>`, that tells
rat-salsa how to proceed.

- [Control::Continue][refControl]: Continue with the next event.
  
- [Control::Unchanged][refControl]: Event has been used, but
  requires no rendering. Just continues with the next event.
  
  Within the application this is used to break early.
  
- [Control::Changed][refControl]: Event has been used, and
  a render is necessary. Continues with the next event after
  rendering.
  
- [Control::Message(m)][refControl]: Distributes the message
  throughout the application. This works as just another event
  type with its own function responsible for distribution.
  
  The individual AppWidgets making up the application are quite
  isolated from other parts and just have access to their own
  state and some global application state.
  
  All communication across AppWidgets uses messages with some
  some payload.
  
- [Control::Quit][refControl]: Ends the event-loop and resets the
  terminal. This returns from run_tui() and ends the application
  by running out of main.
  

  

  
  
  
[refRatEvent]: https://docs.rs/rat-event/latest/rat_event/

[refControl]: https://docs.rs/rat-salsa/latest/rat_salsa/enum.Control.html

[refRatWidget]: https://docs.rs/rat-widget/latest/rat_widget/

[refAppContext]: https://docs.rs/rat-salsa/latest/rat_salsa/struct.AppContext.html

[refConsumedEvent]: https://docs.rs/rat-event/latest/rat_event/trait.ConsumedEvent.html
