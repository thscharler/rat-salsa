# rat-salsa

Application event-loop with ratatui and crossterm.

## run-tui

This function runs the event-loop.

It takes a [TuiApp] trait, which collects all the involved types, and provides
the basic operations. There is a separation of the data-model, ui-state and
actions upon the data. Actions can also be executed on a background worker thread
and communicate back to the event-loop via a channel.

The event-loop is steered with the [ControlUI] enum. This is a bit of amalgamation
of [core::ops::ControlFlow] and a [Result] with several specialized Ok variants.

* Continue - continue operations, eventually wait for a new event.
* Unchanged / Changed - an event has been processed by some part of the
  event-handling and can break early. Depending on the value the ui is
  repainted or not, and the loop goes get the next event.
* Action - execute an action on the data-model.
* Spawn - execute an action in the worker thread.
* Break - break the event-loop and stop the application.
* Err - error has occured, invoke the error handler.

Each operation generates a ControlUI as result, which is evaluated before
waiting for a new event.

### Error handling and other macros

* [err!] - converts a [Result::Err] to a [ControlUI::Err] and returns early.
* [cut!] - returns early if the value is everything but a [ControlUI::Continue].
* [yeet!] - returns early if the value is a [ControlUI::Err], otherwise evaluates to the other values.

### Background worker

This functionality is split in two functions in [TuiApp]:

* [TuiApp::start_task()] gets invoked in the event loop thread and gets passed
  the complete context. It then creates a [TuiApp::Task] which gets send over
  to the worker threads.

  This allows passing on more than just the plain [TuiApp::Action]. I use it to send
  a copy of the configuration. This way my actions can work without synchronisation.

* [TuiApp::run_task()] is called in the worker thread and does the real work.
  The result is then sent back to the event loop. Additionally, it gets passed a
  [TaskSender] for any extra communication needs.

### Other functionality

The rest of the [TuiApp] functions are self-explanatory.

## Keyboard focus

The struct [focus::Focus] can be used to manage the focused widget. It works by embedding a
[focus::FocusFlag] in the state of each widget. Focus gets a list with all FocusFlags that are 
involved in the focus-cyle. This list is then used to switch between the widgets.

### Additions

* [focus::FocusFlag::tag] - Each participant in a focus cycle gets a unique tag, basically an u16.
  This can be used set the focus programmatically.
* [focus::FocusFlag::lost] - Is set if the widget just lost the focus. There is a [validate!] macro that
  uses this flag to conditionally validate the content of the widget.

## Extensions and traits

### FrameWidget

Setting the cursor position is only supported by [ratatui::Frame]. This trait introduces
a new widget type that takes the frame instead of the buffer. There is also an extension trait
for frame to support this case.

TuiApp is completely agnostic to this one.

### Event handling

There is a trait [HandleEvent] to encapsulate the concept. It works with crossterm events.

TuiApp doesn't use this trait, it's just a convenience for widgets.




