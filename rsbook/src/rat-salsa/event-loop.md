
# Event loop

The event loop is the reason for rat-salsa.

## Running the loop

```
pub fn run_tui<Widget, Global, Event, Error>(
    app: Widget,
    global: &mut Global,
    state: &mut Widget::State,
    mut cfg: RunConfig<Event, Error>,
) -> Result<(), Error>
where
    Widget: AppWidget<Global, Event, Error> + 'static,
    Global: 'static,
    Event: Send + 'static,
    Error: Send + 'static + From<io::Error>,
```

This runs the event-loop for the given AppWidget+State. It also
gets the initialized global state and a runtime-configuration
struct.

The runtime-configuration looks somewhat like this

```
RunConfig::default()?
            .poll(PollCrossterm)
            .poll(PollTimers::default())
            .poll(PollTasks::default())
            .poll(PollRendered)
```

It uses its own dyn wrapper around ratatui::Terminal to avoid
one more type-variable that is just annoying to carry around
everywhere. There is a default for crossterm.

Next it gets initialized with all the event-sources that are of
interest for the application.

> The list is open ended, its not too complicated to add your own
> here. The [PollEvents][refPollEvents] works on a poll()/read()
> scheme that is already present in crossterm and is very easy to
> implement on top of crossbeam channels.

The event-loop polls all sources and notes which have pending events.

This secondary list is processed to get an event from the
concerned source.

> This two stage approach ensures fairness between all the sources. 

## Application Event

The event-loop doesn't have its own Event type, instead the 
application must provide one. Each event source added can put 
additional requirements onto this type. Primarily each event source
requires a `From` conversion from its event-type to the application 
event type. 

The resulting event is then given over to the `AppState::event()`
function, which can do whatever.

## Control


The result of the `AppState::event()` function is a Control enum that defines
what happens next. 

- Control::Continue: That's it. Poll the next event.
- Control::Unchanged: The same from rat-salsa's position.
  The difference is that Continue should run through all the
  applications event-handling and Unchanged can short-curcuit.
- Control::Changed: Some state changed that requires a render.
- Control::Event: Event handling triggered a followup event.
- Control::Quit: Quit the application.

## Control::Event

Control::Event can return _any_ application Event, which will
be looped back to `AppState::event()`. It is meant to distribute
application level events, and while it could be used to cycle
back a `crossterm::event::Event` this way, that's probably a bad
idea.

> There are rare cases where the result is more than one
> application event. There exists `AppContext::queue()` for this.
