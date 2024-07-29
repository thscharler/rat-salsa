# rat-salsa

An application event-loop with ratatui and crossterm.

## companion crates

rat-salsa covers only the event-loop and application building.

There is more:

* [rat-widget](https://docs.rs/rat-widget)
  widget library
* [rat-scrolled](https://docs.rs/rat-scrolled)
  utilities for scrolling. Included in rat-widget.
* [rat-ftable](https://docs.rs/rat-ftable)
  table. uses traits to render your data, and renders only the visible cells.
  this makes rendering effectively O(1) in regard to the number of rows.
  Included in rat-widget.
* [rat-focus](https://docs.rs/rat-focus)
  Primitives for focus-handling as used by rat-widget. Included in rat-widget.
* [rat-event](https://docs.rs/rat-event)
  Defines the primitives for event-handling. Included in rat-widget.
* [rat-theme](https://docs.rs/rat-theme)
  Color-palettes and widget styles.
* [rat-salsa](https://docs.rs/rat-salsa)
  Implements the event-loop as a single function call to `run_tui()`.
    * Defines the traits AppWidget and AppEvents to echo Widget/StatefulWidget
      and HandleEvent, with added application context.
    * timer-support and background-tasks

## run-tui

This function runs the event-loop.

* app - The main AppWidget that handles the whole application.
* global - Global state stuff. Put your config, theme, logging, database connection
  and the like here.
* state - Initial state of the app widget.
* cfg - Some tweaks for the event loop.

Polls all event-sources and ensures an equal time-share for each source,
should one of them start flooding. The default sources are Timers, Crossterm and
Task-Results. You can add your own to cfg.

## Control

The result-type for event-handling:

* Continue - poll the next event.
* Unchanged - Does nothing for the main loop, but can be used with `flow_ok!`
  to break early from event-handling.
* Changed - Renders the application.
* Message - Calls `message` to distribute application level events.
* Quit - Quit the application.

The result of an event is processed immediately after the
function returns, before polling new events. This way an action
can trigger another action which triggers the repaint without
other events intervening.

If you ever need to return more than one result from event-handling,
you can hand it to AppContext/RenderContext::queue(). Events
in the queue are processed in order, and the return value of
the event-handler comes last. If an error is returned, everything
send to the queue will be executed nonetheless.

## AppWidget and AppEvents

AppWidget is styled after StatefulWidget.

Additionaly it gets

* ctx - RenderContext

AppEvents packs together the currently supported event-handlers.

* init - called at application startup before the first repaint.
* timer - application timers
* crossterm - crossterm events.
* message - application supplied messages/events.
* error - error handling

Each of them get some event and an AppContext.

## AppContext and RenderContext

AppContext and RenderContext are not the same, the latter
has rendering specific information not available in the
general case.

AppContext contains

* field `g` for the global state data.
* field `focus` to access the current valid Focus.
* add_timer(), remove_timer()
* spawn() - Runs the closure given and returns an `Arc<Mutex<bool>>`
  that is shared with the worker thread to support basic
  cancellation support.
* queue() - Queues additional results from event-handling.

> Remark: The main reason for this is focus-handling.
> When handling the click to focus a widget, the same
> click event should interact with the widget. This gives
> two results from event-handling. The focus change wants
> a Control::Repaint, and the widget might have its own
> ideas. So now you queue() the focus result and go on
> with event-handling.

RenderContext contains

* field `g` for the global state data.
* timeout - When the repaint was triggered by a repaint-timer this
  is the timeout that occurred.
* frame counter
* cursor position for displaying the cursor.

## Example

There is no example here, that would be too much.
The examples directory contains files.rs and mdedit.rs.
There is minimal.rs for a starter.



