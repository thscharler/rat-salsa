# rat-salsa

An application event-loop with ratatui and crossterm.

## companion crates

rat-salsa covers only the event-loop and application building.

There is more:

* [rat-widget](https://docs.rs/rat-widget)
  widget library. +focus-handling +scrolling +ratatui-wrappers
    * button
    * calender
    * date-input, number-input
    * text-input
    * text-input with masks
    * text-area
    * menuline
    * table
    * ... more to come ...
* [rat-scrolled](https://docs.rs/rat-scrolled)
  scrolling for widgets, stateful widgets. viewports.
    * Scrolled widget + support traits
    * View and Viewport widget for Widget/StatefulWidget
      -> reexported by rat-widget
* [rat-input](https://docs.rs/rat-input)
  baseline implementation of the widgets without strapped on focus-handling
  and without the scrolling traits. Should be compatible with any existing
  ratatui application. Can be hooked into your own focus-handling.
  Widgets have builtin scrolling where useful, just the trait impl from
  rat-scrolled are missing.
  -> wrapped up & reexported by rat-widget
* [rat-ftable](https://docs.rs/rat-ftable)
  table implementation mostly api-compatible with the ratatui table.
    * Adds TableData and TableDataIter traits which allow it
      to render only the visible cells. Rendering the individual cells
      is solely done by these traits, so you can render whatever.
      Have tried it with 1,000,000 rows and worked nicely.
      It also supports rendering endless iterators with some restrictions.
    * Pluggable selection-models. Builtin are NoSelection, RowSelection,
      RowSetSelection and CellSelection.
    * Currently, it has column-wise horizontal scrolling. Plans are to
      extend this to char-wise scrolling.
    * There is a FEditTable widget too, which supports inline editing
      of the table-data.
* [rat-focus](https://docs.rs/rat-focus)
  Defines the primitives for focus-handling as used by rat-widget.
    * Can collect data from sub-widgets/container like widgets.
    * Can support widget-groups with a collective focus-state.
    * Easy to add to existing widgets: Add FocusFlag to your state
      & impl the trait HasFocusFlag.
    * Lost & Gained flags for logic.
* [rat-event](https://docs.rs/rat-event)
  Defines the primitives for event-handling used by all of the above.
    * Build around `HandleEvent<EventType, Qualifier, Outcome>`.
        * open for any type of event
        * qualifier can be many things
            * Allows for a type-state pattern, predefined types for this
              are `FocusKey` and `MouseOnly`, but the other libraries
              define their own (DoubleClick, EditKeys, ReadOnly)
            * Applications can override the keybindings for every
              widget if needed.
            * Can be used as a Context-Parameter if needed.
        * open outcome of event-handling allows widgets to return
          whatever they need to.
    * There is a very basic type Outcome with
        * NotUsed - Event not recognized.
        * Unchanged - Event recognized, but no changes.
        * Changed - Event recognized, state has changed. Please repaint.
          It is encouraged for other outcome-types to provide conversions
          to and from this type. That makes life much easier for users,
          as everything is just one `.into()` away :)
    * A control-flow macro `flow!` which allows to break event-handling
      as soon as a responsible widget has been found.
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
* Break - Does nothing for the main loop, but can be used with `flow_ok!`
  to break early from event-handling.
* Repaint - Renders the application.
* Action - Calls into the action handlers.
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
* action - application supplied actions.
* error - error handling

Each of them get some event and an AppContext.

## AppContext and RenderContext

AppContext and RenderContext are not the same, the latter
has rendering specific information not available in the
general case.

AppContext contains

* field `g` for the global state data.
* add_timer(), remove_timer()
* spawn() - Runs the closure given and returns an `Arc<Mutex<bool>>`
  that is shared with the worker thread to support basic
  cancellation support.
* queue() - Queues additional results from event-handling.

        Remark: The main reason for this is focus-handling.
                When handling the click to focus a widget, the same
                click event should interact with the widget. This gives
                two results from event-handling. The focus change wants
                a Control::Repaint, and the widget might have its own 
                ideas. So now you queue() the focus result and go on
                with event-handling. 

RenderContext contains

* field `g` for the global state data.
* timeout - When the repaint was triggered by a repaint-timer this
  is the timeout that occurred.
* frame counter
* frame area
* cursor position for displaying the cursor.

## Example

There is no example here, that would be too much.
The examples directory contains minimal.rs and showcase.rs.
