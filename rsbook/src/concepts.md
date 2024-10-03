
# Reinvent the wheel


## Widgets

The `StatefulWidget` trait works good enough for building
widgets, it's well known and my own ideas where not sufficiently
better so I kept that one.

All the widgets work just as plain StatefulWidgets. This effort
lead to the [rat-widget][refRatWidget] crate.

Or see the introduction in [widget chapter](./widgets.md).

## Application code

For the application code `StatefulWidget` is clearly missing, but
I kept the split-widget concept and there are two traits

* [AppWidget][refAppWidget]
  
  - Keeps the structure of StatefulWidget, just adds a 
    [RenderContext][refRenderContext].
* [AppState][refAppState] The state is the persistent half of
  every widget, so this one gets all the event-handling.
  
  Every event-type has its own function here, and the event types
  are used as provided.
  
  - It uses crossterm as it seems most complete at this time.

## run_tui

[run_tui][refRunTui] implements the event-loop and drives the
application.

- Polls all event-sources and ensures fairness for all events.
- Renders on demand.
- Maintains the background worker threads.
- Maintains the timers.
- Distributes application messages.
- Initializes the terminal and ensure clean shutdown even when
  panics occur.

All of this is orchestrated with the [Control enum][refControl].


[refRenderContext]: https://docs.rs/rat-salsa/latest/rat_salsa/struct.RenderContext.html

[refAppWidget]: https://docs.rs/rat-salsa/latest/rat_salsa/trait.AppWidget.html

[refRunTui]: https://docs.rs/rat-salsa/latest/rat_salsa/fn.run_tui.html

[refControl]: https://docs.rs/rat-salsa/latest/rat_salsa/enum.Control.html

[refRatWidget]: https://docs.rs/rat-widget/latest/rat_widget/

[refAppState]: https://docs.rs/rat-salsa/latest/rat_salsa/trait.AppState.html 

[refAppWidget]: https://docs.rs/rat-salsa/latest/rat_salsa/trait.AppWidget.html

[refAppState]: https://docs.rs/rat-salsa/latest/rat_salsa/trait.AppState.html

[refRenderContext]: https://docs.rs/rat-salsa/latest/rat_salsa/struct.RenderContext.html

[refRunTui]: https://docs.rs/rat-salsa/latest/rat_salsa/fn.run_tui.html

[refControl]: https://docs.rs/rat-salsa/latest/rat_salsa/enum.Control.html
