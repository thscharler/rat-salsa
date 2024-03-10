# rat-salsa

Application event-loop with ratatui and crossterm.

## run-tui

This function runs the event-loop.

It takes a [TuiApp] trait, which collects all the involved types, and provides
the basic operations. There is a separation of the data-model, ui-state and
actions upon the data. Actions can also be executed on a background worker thread
and communicate back to the event-loop via a channel.

The event-loop is steered with the ControlUI enum. This is a bit of amalgamation
of [core::ops::ControlFlow] and a [Result] with several specialized Ok variants.

* Continue - continue operations, eventually wait for a new event.
* Unchanged / Changed - an event has been processed by some part of the
  event-handling and can break early. Depending on the value the ui is
  repainted or not, and the loop gets the next event.
* Action - execute an action on the data-model.
* Spawn - execute an action in the worker thread.
* Break - break the event-loop and stop the application.
* Err - error has occured, invoke the error handler.

Each operation generates a ControlUI of its own and such can communicate
to the other parts of the app. 

### Error handling






