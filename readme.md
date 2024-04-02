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
* NoChange / Change - an event has been processed by some part of the
  event-handling and can break early. Depending on the value the ui is
  repainted or not, and the loop goes get the next event.
* Run - execute an action on the data-model.
* Spawn - execute an action in the worker thread.
* Break - break the event-loop and stop the application.
* Err - error has occured, invoke the error handler.

There is one more trick involved that helps writing in an `and_then` style:
Every part of the event-loop generates a ControlUI as result, which is immediately run
through the event-loop again, etc. Only once the result reaches `ControlUI::Continue`
new events are polled. While there is the possibility to create an endless loop
this way this has not been spotted in the wild so far.

### Error handling and other macros

* [try_result!] - converts a [Result::Err] to a [ControlUI::Err] and returns early.
* [check_break!] - returns early if the value is everything but a [ControlUI::Continue].
* [try_ui!] - returns early if the value is a [ControlUI::Err], otherwise evaluates to the other values.

### Background worker

Any action marked with `ControlUI::Spawn()` is sent to a worker thread, which then calls
[TuiApp::run_task()], where the real work is done. The result of that is sent back to 
the event loop automatically. Additionally `run_task()` gets a [TaskSender] to send more
than one ControlUI command.

### Events

There are 3 functions at this point that handle events.

* [TuiApp::repaint]
* [TuiApp::handle_timer]
* [TuiApp::handle_event]

I kept them separate because I didn't want even more type variables or a super
enum. And this scheme makes it easy to add new event types if wanted. 

### Other functionality

#### [Repaint]

This is a side-channel to trigger a repaint, if the returned ControlUI is needed
for something else. It acts just as one more event-source. 

#### [Timer]

Generates timer events. Timers can auto-repeat. There are two kinds, one that
triggers a repaint and one that gets handed to the application with [TuiApp::handle_timer].

```rust ignore
    if uistate.timer_1 == 0 {
        uistate.timer_1 = uistate.timers.add(
            Timer::new()
                .repeat(usize::MAX)
                .repaint(true)
                .timer(Duration::from_millis(500)),
        );
    }
    if let RepaintEvent::Timer(t) = event {
        if t.tag == uistate.timer_1 {
            uistate.roll = t.counter % 29;
        }
    }
```

## Keyboard focus
 
The struct [Focus] can be used to manage the focused widget. It works by adding
[FocusFlag] to the state of each widget. Focus is constructed with a list of
the focus-flags that should be involved. Each widget stays separate otherwise and takes
its current state from this flag.

### Additions

* [FocusFlag::tag] - Each participant in a focus cycle gets a unique tag, basically an u16.
  This can be used set the focus programmatically.
* [FocusFlag::lost] - Is set if the widget just lost the focus. There is a [on_lost!] macro that
  uses this flag to conditionally validate the content of the widget.
* [FocusFlag::gained] - Is set if the widget just lost the focus. There is a [on_gained!] macro that
  uses this flag to conditionally validate the content of the widget.

Note: The lost and gained flags are only valid for one run of the event-loop and are reset the
next time `Focus::handle()` is called, regardless of the concrete event. 

## Extensions and traits

### FrameWidget

Setting the cursor position is only supported by [ratatui::Frame]. This trait introduces
a new widget type that takes the frame instead of the buffer. There is also an extension trait
for `Frame` to support this case.

TuiApp is completely agnostic to this one.

### Event handling

There is a trait [HandleCrossterm] to encapsulate the concept. It works with crossterm events,
but the basic concept can easily be copied for other types.

[TuiApp] doesn't use this trait, it's just for widgets.


