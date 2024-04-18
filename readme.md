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

### Repainting the tui

Creating the layout and rendering is done in [TuiApp::repaint()](crate::TuiApp::repaint()).
It gets the frame, data and uistate to do this.

It is called once before the start of the event-loop and every time a `ControlUI::Change`
is indicated.

Additionally, there is [Repaint] or you can set a repaint timer. The differences
are communicated with a [RepaintEvent].

### Events

For now there are

* [TuiApp::handle_timer]
* [TuiApp::handle_event]

I kept them separate because I didn't want even more type variables or some kind of super
event-enum. And this scheme makes it easy to add new event types if wanted.

### Control-flow and error-handling

The activities of the event-loop are steered by returning a [ControlUI] result from every
involved function.

There is a macro [check_break!] to cut short the event-processing. Whenever a part of
the event-handler finds that it can handle an event it creates a response `ControlUI`.
`check_break!` simply returns early if the response is anything but `ControlUI::Continue`.

There is one more trick involved that helps writing in an and-then-style. The resulting
ControlUI is evaluated _before_ polling a new event. So an action can invoke another
action which invokes one more action which triggers a repaint. Only once the result
reaches `ControlUI::Continue` the next event is processed.

While there is the possibility to create an endless loop this way, this behaviour has not
been spotted in the wild so far.

Error handling is done with `ControlUI::Err` instead of a classic `Result`. On one hand this
makes involved types simpler and at the same time it causes friction when calling into
parts that use Result. There is the macro [tr!] to help. It converts a
[Result::Err] to a [ControlUI::Err] and returns early. (It would have been nice to
use the ? operator, but we are still waiting for that.)

[tr!] can work with [ControlUI::Err] too.

### Running Actions

Actions started with `ControlUI::Run()` are executed as part of the event-loop with a call to
[TuiApp::run_action()]. They can do whatever, but in order to keep the ui responsive they should do
it quick. If that's not possible use `ControlUI::Spawn()`. Alternatively `run_action()` gets a
`Sender<Self::Action>` so that an action can spawn one or more background tasks.

Actions started with `ControlUI::Spawn()` are sent to a worker thread, which then calls
[TuiApp::run_task()], where the real work is done. The result of that is sent back to
the event loop automatically. Additionally `run_task()` gets
a [Sender<ControlUI<Action, Error>>](crossbeam::channel::Sender) to
send more than one ControlUI command.

### Other functionality

#### [Repaint](crate::Repaint)

This is a side-channel to trigger a repaint, if the returned `ControlUI` is needed
for something else. It acts as one more source for a repaint event.

#### [Timers](crate::Timers)

There are timers too.

They are optional, so add a `Timers` to your uistate and make it available with [TuiApp::get_timers()]
if you need this functionality.

There are two kinds of timers, one that triggers a `repaint()` and another that gets handed to
the application with [TuiApp::handle_timer].

```rust ignore
if uistate.mask0.timer_1 == 0 {
uistate.mask0.timer_1 = uistate.timers.add(
TimerDef::new()
.repeat(usize::MAX)
.repaint(true)
.timer(Duration::from_millis(500)),
);
}

if let RepaintEvent::Timer(t) = event {
if t.tag == uistate.mask0.timer_1 {
uistate.mask0.roll = t.counter % 29;
}
}
```

## Keyboard focus

The struct [Focus] can be used to manage the focused widget. It works by adding
[FocusFlag] to the state of each widget.

Focus is constructed with a list of the focus-flags and optionally some area of the widget that
should react to a mouse-click.

After that Focus simply is one more step in event handling. It gets passed an event, decides
if something needs to be done, and that's it.

```rust ignore
fn focus0(mask0: &Mask0) -> Focus<'_> {
    Focus::new([
        (mask0.text.focus(), mask0.text.area()),
        (mask0.decimal.focus(), mask0.decimal.area()),
        (mask0.float.focus(), mask0.float.area()),
        (mask0.ipv4.focus(), mask0.ipv4.area()),
        (mask0.hexcolor.focus(), mask0.hexcolor.area()),
        (mask0.creditcard.focus(), mask0.creditcard.area()),
        (mask0.date.focus(), mask0.date.area()),
        (mask0.alpha.focus(), mask0.alpha.area()),
        (mask0.dec7_2.focus(), mask0.dec7_2.area()),
        (mask0.euro.focus(), mask0.euro.area()),
        (mask0.exp.focus(), mask0.exp.area()),
    ])
}

fn handle_mask0(event: &Event, data: &mut FormOneData, uistate: &mut FormOneState) -> Control {
    let mask0 = &mut uistate.mask0;

    focus0(mask0)
        .handle(event, DefaultKeys)
        .and_do(|_| uistate.g.repaint.set());

    // ...
}
```

One caveat is `.and_do(|_| uistate.g.repaint.set())`. As Focus doesn't want to interact with
normal control-flow it has no way to trigger the necessary repaint. With the help of the
[Repaint](crate::Repaint) mechanism this can be done reasonably.

And no, focus really doesn't want to interact with the control-flow. Especially when using
mouse events to set the focus. Many widgets want to react to the same click-event that changed
the focus. To set the cursor for example, or to select a row in a list or table.

Something similar could be achieved by remembering the ControlUI-result from Focus and combining
it with the regular one at the end. But that's ugly too.

### More on focus

* [FocusFlag::tag] - Each participant in a focus cycle gets a unique tag, basically an u16.
  This can be used set the focus programmatically.

* [FocusFlag::lost] - Is set if the widget just lost the focus.
* [FocusFlag::gained] - Is set if the widget just lost the focus.

  _Note_: The lost and gained flags are only valid for one run of the event-loop and are reset the
  next time `Focus::handle()` is called, regardless of the concrete event.

* [on_lost!] - Uses a match-like style to check for the flag.

```rust ignore
    on_lost!(
        mask0.decimal => {
            let v = mask0.decimal.value().parse::<i64>();
            if let Some(v) = mask0.decimal.set_valid_from(v) {
                mask0.decimal.set_value(format!("{}", v));
            }
        },
        mask0.float => {
            let v = mask0.float.value().parse::<f64>();
            if let Some(v) = mask0.float.set_valid_from(v) {
                mask0.float.set_value(format!("{}", v));
            }
        }
    );
```

* [on_gained!] - The same for gained.

```rust ignore
    on_gained!(
        mask0.decimal => {
            mask0.decimal.select_all();
        },
        mask0.float => {
            mask0.float.select_all();
        }
    );
```

* [match_focus!] - The same for the focus flag itself. This one can return a result and has
  a else branch too.

```rust ignore
    let r = match_focus!(
        uistate.mask0.ipv4 => Some(&uistate.mask0.ipv4),
        uistate.mask0.hexcolor => Some(&uistate.mask0.hexcolor),
        uistate.mask0.creditcard => Some(&uistate.mask0.creditcard),
        uistate.mask0.date => Some(&uistate.mask0.date),
        uistate.mask0.alpha =>Some( &uistate.mask0.alpha),
        uistate.mask0.dec7_2 => Some(&uistate.mask0.dec7_2),
        uistate.mask0.euro => Some(&uistate.mask0.euro),
        uistate.mask0.exp => Some(&uistate.mask0.exp),
        _ => None
    );
```

## Input validation

While input validation can be done reasonably well with `on_lost!` that's not all.

There is a [ValidFlag](crate::ValidFlag) which can be added to the state of a widget.
The widget can adjust its rendering according to this flag.

This allows the use of the [HasValidFlag](crate::HasValidFlag) trait to manage this state.

It can do something nice like `set_valid_from` too:

```rust ignore
let v = mask0.float.value().parse::<f64>();
if let Some(v) = mask0.float.set_valid_from(v) {
mask0.float.set_value(format ! ("{}", v));
}
```

A widget can validate its content each time it's rendered, or it can provide [CanValidate](crate::CanValidate).
With this trait it's possible to use the macro [validate!]. It only calls `validate()` for the widget that
has just lost the focus.

```rust ignore
validate!(
    mask0.date1, 
    mask0.number1
);
```

## Widget extensions

### FrameWidget

This trait introduces a new widget type that takes the frame instead of the buffer for rendering.
The only benefit of this is it allows to set the cursor position from within the widget.
The classic `Widget` and `StatefulWidget` are still fine for everything else.

```rust ignore
frame.render_frame_widget(w_some, area_some, & mut state.some_state);
```

### Event handling

There is a trait [HandleCrossterm] to encapsulate the concept for widgets.
It works with crossterm events, but it can easily be copied for other event-types.

The nice thing about this one is it allows multiple key-mappings for one widget. You could
probably make the key-mapping configurable too, but I haven't tried that.

```rust ignore
uistate.page1.table1.handle(evt, DefaultKeys)
```

vs

```rust ignore
uistate.page1.table1.handle(evt, VimMotions)
```

The design allows for widget creators to provide multiple key-bindings and for applications
to add their own for existing widgets without much fuzz.

```rust ignore
struct SpecialKeybinding;

impl<R> HandleCrossterm<R, SpecialKeybinding> for DateInputState
{
    fn handle(&mut self, event: &Event, keymap: SpecialKeybinding) -> ControlUI<A, E> {
        // ... 
    }
}
```

To simplify the process there is one more trait [Input](crate::Input) which enables widgets
to define the set of interactions. The key-binding maps to these interactions and the interactions
can do whatever complicated logic is needed.

[TuiApp] doesn't use this trait, it's just for widgets.

### Scrolling

See [HasVerticalScroll](crate::widget::HasVerticalScroll).

And also [scrolled](crate::widget::scrolled)

### Selection

See [selected](crate::widget::selected)