
# Walkthrough for `minimal.rs?

A walkthrough for examples/minimal.rs, a starting point for a
new application.

## main

```rust
fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = MinimalConfig::default();
    let theme = DarkTheme::new("Imperial".into(), IMPERIAL);
    let mut global = GlobalState::new(config, theme);

    let app = Scenery;
    let mut state = SceneryState::default();

    run_tui(
        app,
        &mut global,
        &mut state,
        RunConfig::default()?
            .poll(PollCrossterm)
            .poll(PollTimers::default())
            .poll(PollTasks::default())
            .poll(PollRendered),
    )?;

    Ok(())
}
```

run_tui runs the event loop. it gets

- app: The main AppWidget Scenery. It provides the scenery for
  the application, adds a status bar, displays error messages,
  and forwards the real application Minimal.
  
- global: whatever global state is necessary. This global state
  is useable across all app-widgets. Otherwise, the app-widgets
  only see their own state.

- state: the state-struct SceneryState.

- [RunConfig][refRunConfig]: configures the event-loop.

  - Uses the default Terminal which is CrosstermTerminal
  - Adds all the needed event sources
    - PollCrossterm for crossterm events.
    - PollTimers for timers.
    - PollTasks for tasks running in a worker thread.
    - PollRendered for a notification after rendering.
  
## mod global

```rust
    #[derive(Debug)]
    pub struct GlobalState {
        pub cfg: MinimalConfig,
        pub theme: DarkTheme,
    }
```

GlobalState contains everything that needs to be accessible
application wide.

## mod config

```rust
    pub struct MinimalConfig {}
```

Data from a config file or from start parameters. 

## mod event

```
#[derive(Debug)]
pub enum MinimalEvent {
    Timer(TimeOut),
    Event(crossterm::event::Event),
    Rendered,
    Message(String),
    Status(usize, String),
}
```

This is the only Event type the application uses. There are From
conversions from every event type any of the Pollxxx provides to
this application event type. I prefer to just wrap them in one
enum variant.

And I add an event for any long-range notifications I need.


## mod scenery

```rust
#[derive(Debug)]
pub struct Scenery;

#[derive(Debug, Default)]
pub struct SceneryState {
    pub minimal: MinimalState,
    pub status: StatusLineState,
    pub error_dlg: MsgDialogState,    
}
```

The AppWidget and AppState for the scenery.

```rust
impl AppWidget<GlobalState, MinimalEvent, Error> for Scenery {
    type State = SceneryState;

    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State,
        ctx: &mut RenderContext<'_>,
    ) -> Result<(), Error> {
        let t0 = SystemTime::now();

        let layout = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).split(area);

        Minimal.render(area, buf, &mut state.minimal, ctx)?;

        if state.error_dlg.active() {
            let err = MsgDialog::new().styles(ctx.g.theme.msg_dialog_style());
            err.render(layout[0], buf, &mut state.error_dlg);
        }

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        state.status.status(1, format!("R {:.0?}", el).to_string());

        let status_layout =
            Layout::horizontal([Constraint::Fill(61), Constraint::Fill(39)]).split(layout[1]);
        let status = StatusLine::new()
            .layout([
                Constraint::Fill(1),
                Constraint::Length(8),
                Constraint::Length(8),
            ])
            .styles(ctx.g.theme.statusline_style());
        status.render(status_layout[1], buf, &mut state.status);

        Ok(())
    }
}
```

The AppWidget implementation for Scenery renders some parts and forwards
other parts to the Minimal AppWidget. 


```rust
    impl AppState<GlobalState, MinimalEvent, Error> for SceneryState {
```

```rust
fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
    ctx.focus = Some(FocusBuilder::build_for(&self.minimal));
    self.minimal.init(ctx)?;
    Ok(())
}    
```

Init is called for the startup initialisation of the application.
It runs before the first render so you can setup your initial
state as needed.

Here it constructs the initial Focus and then forwards to minimal.

```rust
fn event(
    &mut self,
    event: &MinimalEvent,
    ctx: &mut rat_salsa::AppContext<'_, GlobalState, MinimalEvent, Error>,
) -> Result<Control<MinimalEvent>, Error> {
    let t0 = SystemTime::now();

    let mut r = match event {
        MinimalEvent::Event(event) => {
            let mut r = match &event {
                ct_event!(resized) => Control::Changed,
                ct_event!(key press CONTROL-'q') => Control::Quit,
                _ => Control::Continue,
            };

            r = r.or_else(|| {
                if self.error_dlg.active() {
                    self.error_dlg.handle(event, Dialog).into()
                } else {
                    Control::Continue
                }
            });

            let f = ctx.focus_mut().handle(event, Regular);
            ctx.queue(f);

            r
        }
        MinimalEvent::Rendered => {
            ctx.focus = Some(FocusBuilder::rebuild_for(&self.minimal, ctx.focus.take()));
            Control::Continue
        }
        MinimalEvent::Message(s) => {
            self.error_dlg.append(s.as_str());
            Control::Changed
        }
        MinimalEvent::Status(n, s) => {
            self.status.status(*n, s);
            Control::Changed
        }
        _ => Control::Continue,
    };

    r = r.or_else_try(|| self.minimal.event(event, ctx))?;

    let el = t0.elapsed()?;
    self.status.status(2, format!("E {:.0?}", el).to_string());

    Ok(r)
}
```

All events run through this function. 

In detail:

```rust
MinimalEvent::Event(event) => {
    let mut r = match &event {
        ct_event!(resized) => Control::Changed,
        ct_event!(key press CONTROL-'q') => Control::Quit,
        _ => Control::Continue,
    };
}
```

MinmalEvent::Event contains the crossterm events. This matches
on the events and returns the Control enum for the event loop.
Control::Changed requests a repaint of the ui, Control::Quit ends 
the event loop and Control::Continue means do nothing, continue
event handling and finally get a new event. 

```rust
    r = r.or_else(|| {
        if ctx.g.error_dlg.active() {
            ctx.g.error_dlg.handle(event, Dialog).into()
        } else {
            Control::Continue
        }
    });
```

Control implements [ConsumedEvent][refConsumedEvent] which allows 
short curcuiting the event handling if we reached a part of the
event handler that knows what to do with the event. Any value
other than Control::Continue means 'knows what to do'. 

There is the or_else combinator that allows to bypass any steps
the need not be bothered anymore. 

The error dialog uses this mechanism too. It consumes __all
__ events if it is active, thus preventing the rest of the
application from doing anything.

```rust
    let f = ctx.focus_mut().handle(event, Regular);
    ctx.queue(f);
```

Handle events for focus changes. 

Focus event handling is a bit special. While it acts as a normal
event handler, the _same_ event that triggers the focus change
might be used by the widget too. Click on a choice box and the
choice box gets the focus __and__ opens up the popup. That leads
to the case where the event handler can have __two__ results
instead of one. This is accomplished by adding the extra outcome
to the same queue where the return value of the event handler
function will end too. This gives a clear ordering for the main
event loop. It will first process anything in the return queue in
the order it was added, and go fetch the next event afterwards.
This leads to nice deterministic behaviour.

```rust
    MinimalEvent::Message(s) => {
        self.error_dlg.append(s.as_str());
        Control::Changed
    }
    MinimalEvent::Status(n, s) => {
        self.status.status(*n, s);
        Control::Changed
    }    
```

A simple application level event. It shows the string in the
message in the error/message dialog. And another one that 
sets one the status flags.

```rust
     MinimalEvent::Rendered => {
        ctx.focus = Some(FocusBuilder::rebuild_for(&self.minimal, ctx.focus.take()));
        Control::Continue
    }
```

The application layout might have changed after rendering, so
rebuilding the Focus to have all the correct areas is a good
idea.

```rust
    r = r.or_else_try(|| self.minimal.event(event, ctx))?;
```

Forward any unused events to the minimal AppState.

```rust
    Ok(r)
```

The final result is returned to the main loop which reacts to the
Control flag.


```rust
    fn error(
        &self,
        event: Error,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MinimalEvent>, Error> {
        ctx.g.error_dlg.append(format!("{:?}", &*event).as_str());
        Ok(Control::Changed)
    }
```

All errors coming from rendering or the event-loop are forwarded
here for logging/display. 

## mod minimal

This is the actual application. This example just adds a MenuLine widget and
lets you quit the application via menu.


```rust
#[derive(Debug)]
pub(crate) struct Minimal;

#[derive(Debug)]
pub struct MinimalState {
    pub menu: MenuLineState,
}
```

Define the necessary structs and any data/state.


```rust
impl AppWidget<GlobalState, MinimalMsg, Error> for Minimal {
    type State = MinimalState;

    fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State,
        ctx: &mut RenderContext<'_>,
    ) -> Result<(), Error> {
        // TODO: repaint_mask

        let r = Layout::new(
            Direction::Vertical,
            [
                Constraint::Fill(1), //
                Constraint::Length(1),
            ],
        )
        .split(area);

        let menu = MenuLine::new()
            .styles(ctx.g.theme.menu_style())
            .item_parsed("_Quit");
        menu.render(r[1], buf, &mut state.menu);

        Ok(())
    }
}
```

Render the menu.

```rust
impl HasFocus for MinimalState {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.widget(&self.menu);
    }

    fn focus(&self) -> FocusFlag {
        unimplemented!("not in use")
    }

    fn area(&self) -> Rect {
        unimplemented!("not in use")
    }
}
```

Implements the trait [HasFocus][refHasFocus] which is the trait
for widgets used by [Focus][refFocus]. This adds its widgets in
traversal order.

```rust
    impl AppState<GlobalState, MinimalMsg, Error> for MinimalState {
```    

Implements AppState...

```rust
    fn init(
        &mut self,
        ctx: &mut rat_salsa::AppContext<'_, GlobalState, MinimalEvent, Error>,
    ) -> Result<(), Error> {
        ctx.focus().first();
        Ok(())
    }
```    

Init sets the focus to the first widget. And does other init work.

```rust
    fn event(
        &mut self,
        event: &MinimalEvent,
        ctx: &mut rat_salsa::AppContext<'_, GlobalState, MinimalEvent, Error>,
    ) -> Result<Control<MinimalEvent>, Error> {
        let r = match event {
            MinimalEvent::Event(event) => {
                match self.menu.handle(event, Regular) {
                    MenuOutcome::Activated(0) => Control::Quit,
                    v => v.into(),
                }
            },
            _ => Control::Continue,
        };

        Ok(r)
    }
```


Calls the `Regular` event handler for the menu. MenuLine has its
own return type `MenuOutcome` to signal anything interesting.
What interests here is that the 'Quit' menu item has been
activated. Return the according Control::Quit to end the
application.

All other values are converted to some predefined Control value.


[refRunConfig]: https://docs.rs/rat-salsa/latest/rat_salsa/struct.RunConfig.html

[refLife]: https://github.com/thscharler/rat-salsa/blob/master/examples/life.life

[refCtEvent]: https://docs.rs/rat-event/latest/rat_event/macro.ct_event.html

[refConsumedEvent]: https://docs.rs/rat-event/latest/rat_event/trait.ConsumedEvent.html

[refHasFocus]: https://docs.rs/rat-focus/latest/rat_focus/trait.HasFocus.html

[refFocus]: https://docs.rs/rat-focus/latest/rat_focus/struct.Focus.html

[refSalsaTerminal]: https://docs.rs/rat-salsa/latest/rat_salsa/terminal/trait.Terminal.html

[refPollEvents]: https://docs.rs/rat-salsa/latest/rat_salsa/poll/trait.PollEvents.html
