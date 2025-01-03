
# Widget events 1

The widgets for [rat-widget][refRatWidget] use the trait
HandleEvent defined in [rat-event][refRatEvent].

```rust
    try_flow!(match self.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => {
            Control::Quit
        }
        v => v.into(),
    });
```

`self.menu` is the state struct for the menu widget. 
It can have multiple HandleEvent implementations, typical are
`Regular` and `MouseOnly`. The second parameter selects the
event-handler.

- Regular: Does all the expected event-handling and returns an
  Outcome value that details what has happened.
  
- MouseOnly: Only uses mouse events. Generally not very useful
  except when you want to write your own keybindings for a
  widget. Then you can forward that part to the MouseOnly handler
  and be done with the mousey part.
  
  See [mdedit][refMdEditMarkdown]. It overrides only part of the
  keybindings with its own implementation and forwards the rest
  to the Regular handler.
  
- Readonly: Text widgets have a Regular handler and a ReadOnly
  handler. The latter only moves and allows selections.
  
- DoubleClick: Some widgets add one. Double clicks are a bit
  rarer and often require special attention, so this behaviour is
  split off from Regular handling.
  
- Dialog and Popup: These are the regular handlers for dialog and
  popup widgets. They have some irregular behaviour, so it's good
  to see this immediately.
  
The handle functions return an outcome value that describes what
has happened. This value usually is widget specific.

And there is the try_flow! macro that surrounds it all. It returns
early, if the event has been consumed by the handler.
  
  
  
[refRatEvent]: https://docs.rs/rat-event/latest/rat_event/

[refControl]: https://docs.rs/rat-salsa/latest/rat_salsa/enum.Control.html

[refRatWidget]: https://docs.rs/rat-widget/latest/rat_widget/

[refAppContext]: https://docs.rs/rat-salsa/latest/rat_salsa/struct.AppContext.html

[refConsumedEvent]: https://docs.rs/rat-event/latest/rat_event/trait.ConsumedEvent.html

[refMdEditMarkdown]: https://github.com/thscharler/rat-salsa/blob/master/examples/mdedit_parts/mod.rs


