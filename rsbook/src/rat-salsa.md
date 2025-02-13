
# rat-salsa

rat-salsa takes the model `StatefulWidget/State` + `HandleEvent` and
extends them for the needs of an application.

| ratatui / rat-event    | rat-salsa                                |
|------------------------|------------------------------------------|
| StatefulWidget         | **[AppWidget][refAppWidget]**            |
|                        | Adds a [RenderContext<>][refRenderContext] to access app-wide state and the state contained in Frame. |
| StatefulWidget::State  | **[AppState][refAppState]**              |
|                        | Adds lifecycle functions and incorporates event-handling to reduce the number of traits needed. |
|                        | Event-handling replaces the qualifier with an [AppContext][refAppContext] and uses [Control][refControl] instead of [Outcome][refOutcome] for its result. |
|                        | Uses a unitary Event type defined by the application for **all** events going through. |
|                        |                                          |
| HandleEvent            | Gets incorporated in AppState as `event()` |
| Outcome                | Replaced with [Control][refControl] which adds `Event(event)` and `Quit` |


[refAppContext]: https://docs.rs/rat-salsa/latest/rat_salsa/struct.AppContext.html
[refRenderContext]: https://docs.rs/rat-salsa/latest/rat_salsa/struct.RenderContext.html
[refAppState]: https://docs.rs/rat-salsa/latest/rat_salsa/trait.AppState.html
[refAppWidget]: https://docs.rs/rat-salsa/latest/rat_salsa/trait.AppWidget.html
[refControl]: https://docs.rs/rat-salsa/latest/rat_salsa/enum.Control.html
[refOutcome]: https://docs.rs/rat-event/latest/rat_event/enum.Outcome.html
