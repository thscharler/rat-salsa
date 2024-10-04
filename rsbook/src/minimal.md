# minimal

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
        RunConfig::default()?.threads(1),
    )?;

    Ok(())
}
```

run_tui is fed with

- app: This is just the unit-struct Scenery. It provides the
  scenery for the application, adds a status bar, displays error
  messages, and forwards the real application Minimal.
  
- global: whatever global state is necessary. This global state
  is useable across all app-widgets. Otherwise, the app-widgets
  only see their own state.
  
- state: the state-struct SceneryState.
  
- [RunConfig][refRunConfig]: configures the event-loop
  
    - If you need some special terminal init/shutdown commands,
      implement the [rat-salsa::Terminal][refSalsaTerminal] trait
      and set it here.
      
    - Set the number of worker threads.
      
    - Add extra event-sources. Implement the  
      [PollEvents][refPollEvents] trait. This will need some
      extra trait for the appstate to distribute your events.
      
      See [examples/life.rs][refLife] for an example.
      
***

The rest is not very exciting. It defines a config-struct
which is just empty, loads a default theme for the application
and makes both accessible via the global state.

## mod global

Defines the global state...

```rust
    #[derive(Debug)]
    pub struct GlobalState {
        pub cfg: MinimalConfig,
        pub theme: DarkTheme,
        pub status: StatusLineState,
        pub error_dlg: MsgDialogState,
    }
```

## mod config

Defines the config...

```rust
    pub struct MinimalConfig {}
```

## mod message

This defines messages that can be sent between different parts of
the application. 

If you split the application into multiple AppWidget/AppState widgets,
the widgets have no easy way to communicate with each other. Or to know
of the others existence.

Which is good. But sometimes they still need to communicate.

The MinimalMsg enum defines all messages that can be exchanged.

> This is also the means to report back from a worker thread.

Of course every message value can have all the data it needs to convey.

## mod scenery

```rust
    #[derive(Debug)]
pub struct Scenery;

#[derive(Debug, Default)]
pub struct SceneryState {
    pub minimal: MinimalState,
}
```

Defines a unit struct for the scenery and a struct for any state.
Here it holds the state for the actual application.

### AppWidget

```rust
    impl AppWidget<GlobalState, MinimalMsg, Error> for Scenery {
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

        if ctx.g.error_dlg.active() {
            let err = MsgDialog::new().styles(ctx.g.theme.msg_dialog_style());
            err.render(layout[0], buf, &mut ctx.g.error_dlg);
        }

        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        ctx.g.status.status(1, format!("R {:.0?}", el).to_string());

        let status_layout =
            Layout::horizontal([Constraint::Fill(61), Constraint::Fill(39)]).split(layout[1]);
        let status = StatusLine::new()
            .layout([
                Constraint::Fill(1),
                Constraint::Length(8),
                Constraint::Length(8),
            ])
            .styles(ctx.g.theme.statusline_style());
        status.render(status_layout[1], buf, &mut ctx.g.status);

        Ok(())
    }
}
```

Implement the AppWidget trait. This forwards rendering to Minimal, and then
renders a MsgDialog if needed for error messages, and the status line.
The default displays some timings taken for rendering too.

### AppState

```rust
    impl AppState<GlobalState, MinimalMsg, Error> for SceneryState {
```

AppState has three type parameters that occur everywhere. I couldn't cut
back that number any further ...

```rust
    fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
        ctx.focus = Some(build_focus(&self.minimal));
        self.minimal.init(ctx)?;
        Ok(())
    }        
```

init is the first event for every application.

it sets up the initial [Focus](./focus) for the application and
forwards to MinimalState.

```rust
    fn timer(
        &mut self,
        event: &TimeOut,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MinimalMsg>, Error> {
        let t0 = SystemTime::now();
    
        ctx.focus = Some(FocusBuilder::rebuild(&self.minimal, ctx.focus.take()));
        let r = self.minimal.timer(event, ctx)?;
    
        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        ctx.g.status.status(2, format!("T {:.0?}", el).to_string());
    
        Ok(r)
    }
```

Timer handles TimeOut events. Does not much here, except rebuilding the
Focus and forwarding to MinimalState.

```rust
    fn crossterm(
        &mut self,
        event: &Event,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MinimalMsg>, Error> {
        let t0 = SystemTime::now();
    
        let mut r = match &event {
            ct_event!(resized) => Control::Changed,
            ct_event!(key press CONTROL-'q') => Control::Quit,
            _ => Control::Continue,
        };
    
        r = r.or_else(|| {
            if ctx.g.error_dlg.active() {
                ctx.g.error_dlg.handle(&event, Dialog).into()
            } else {
                Control::Continue
            }
        });
    
        r = r.or_else_try(|| {
            ctx.focus = Some(FocusBuilder::rebuild(&self.minimal, ctx.focus.take()));
            self.minimal.crossterm(&event, ctx)
        })?;
    
        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        ctx.g.status.status(2, format!("H {:.0?}", el).to_string());
    
        Ok(r)
    }
```

crossterm handles all crossterm events.

```rust
    let mut r = match & event {
        ct_event!(resized) => Control::Changed,
        ct_event!(key press CONTROL-'q') => Control::Quit,
        _ => Control::Continue,
    };
```

This reacts to specific crossterm events. Uses the [ct_event!][refCtEvent]
macro, which gives a nicer syntax for event patterns.

It matches a resized event and returns a Control::Changed result to
the event loop to indicate the need for repaint.

The second checks for `Ctrl+Q` and just quits the application without
further ado. This is ok while developing things, but maybe a bit crude
for actual use.

The last result Control::Continue is 'nothing happened, continue
with event handling'.

```rust
    r = r.or_else(| | {
        if ctx.g.error_dlg.active() {
            ctx.g.error_dlg.handle( & event, Dialog).into()
        } else {
            Control::Continue
        }
    });
```

> Control implements [ConsumedEvent][refConsumedEvent] which
> provides a few combinators.
>
> Event handling can/should stop, when an event is consumed
> by some part of the application. ConsumedEvent::is_consumed
> for Control returns false for Control::Continue and true for
> everything else. And that's what these combinators work with.

`or_else(..)` is only executed if r is Control::Continue. If the
error dialog is active, which is just some flag, it calls it's
event-handler for `Dialog` style event-handling. It does whatever
it does, the one thing special about it is that `Dialog` mode
consumes all events. This means, if an error dialog is displayed,
only it can react to events, everything else is shut out.

If the error dialog is not active it uses Control::Continue to
show event handling can continue.

```rust
    r = r.or_else_try(| | {
        
        ctx.focus = Some(FocusBuilder::rebuild(& self.minimal, ctx.focus.take()));
        
        self.minimal.crossterm( & event, ctx)
    
    }) ?;
```

One more or_else. This one refreshes/rebuilds the Focus and forwards
to MinimalState.

```rust
    Ok(r)
```

And finally the result of event handling is returned to the event loop,
which can work with the result. Depending on the result value it goes on
and calls other functions within the application. And depending on that
result value it goes on calling further functions in the application.
Only after every such result is processed the event loop will go looking
for new events.

```rust
    fn message(
        &mut self,
        event: &mut MinimalMsg,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MinimalMsg>, Error> {
        let t0 = SystemTime::now();
    
        #[allow(unreachable_patterns)]
        let r = match event {
            MinimalMsg::Message(s) => {
                ctx.g.status.status(0, &*s);
                Control::Changed
            }
            _ => {
                ctx.focus = Some(FocusBuilder::rebuild(&self.minimal, ctx.focus.take()));
                self.minimal.message(event, ctx)?
            }
        };
    
        let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
        ctx.g.status.status(2, format!("A {:.0?}", el).to_string());
    
        Ok(r)
    }
```

Processes a global message. Currently, there is only one such messages defined,
which sets some value in the status bar and repaints. All other messages
are forwarded to the MinimalStruct again.

And finally this again can result in further functions being called.

```rust
    fn error(
        &self,
        event: Error,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MinimalMsg>, Error> {
        ctx.g.error_dlg.append(format!("{:?}", &*event).as_str());
        Ok(Control::Changed)
    }
```

All errors that end in the event loop are forwarded here for processing.

This appends the message, which for error dialog sets the dialog
active too. So it will be rendered with the next render. Which is requested
by returning Control::Changed.

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
    impl Default for MinimalState {
        fn default() -> Self {
            let mut s = Self {
                menu: Default::default(),
            };
            s.menu.select(Some(0));
            s
        }
    }
```

Manual impl for Default to set the initial selection for the menu.

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
    }
```

Implements the trait [HasFocus][refHasFocus] which is the trait for container like widgets
used by [Focus][refFocus]. This adds its widgets in traversal order.

```rust
    impl AppState<GlobalState, MinimalMsg, Error> for MinimalState {
```    

Implements AppState...

```rust
    fn init(
        &mut self,
        ctx: &mut rat_salsa::AppContext<'_, GlobalState, MinimalMsg, Error>,
    ) -> Result<(), Error> {
        ctx.focus().first();
        Ok(())
    }
```    

Init sets the focus to the first widget.

```rust
    #[allow(unused_variables)]
    fn crossterm(
        &mut self,
        event: &Event,
        ctx: &mut AppContext<'_>,
    ) -> Result<Control<MinimalMsg>, Error> {
        let f = ctx.focus_mut().handle(event, Regular);
        ctx.queue(f);
    
        try_flow!(match self.menu.handle(event, Regular) {
                MenuOutcome::Activated(0) => {
                    Control::Quit
                }
                    v => v.into(),
        });
    
        Ok(Control::Continue)
    }
```

Handling events for Focus is a bit special.

```rust
    let f = ctx.focus_mut().handle(event, Regular);
    ctx.queue(f);
```

Focus implements an event handler for `Regular` events. Regular is similar
to `Dialog` seen before, and means bog-standard event handling whatever the
widget does. The speciality is that focus handling shouldn't consume the
recognized events. This is important for mouse events where the widget might
do something useful with the same click event that focused it.

Here `ctx.queue()` comes into play and adds an extra result. This way 
the focus change can initiate a render while the event handling function 
can still return whatever it wants.

```rust
    try_flow!(match self.menu.handle(event, Regular) {
        MenuOutcome::Activated(0) => {
            Control::Quit
        }
        v => v.into(),
    });
```

Calls the `Regular` event handler for the menu. MenuLine has its
own return type `MenuOutcome` to signal anything interesting.
What interests here is that the 'Quit' menu item has been
activated. Return the according Control::Quit to end the
application.

All other values are converted to some Control value.

## That's it

for a start :)


[refRunConfig]: https://docs.rs/rat-salsa/latest/rat_salsa/struct.RunConfig.html

[refLife]: https://github.com/thscharler/rat-salsa/blob/master/examples/life.life

[refCtEvent]: https://docs.rs/rat-event/latest/rat_event/macro.ct_event.html

[refConsumedEvent]: https://docs.rs/rat-event/latest/rat_event/trait.ConsumedEvent.html

[refHasFocus]: https://docs.rs/rat-focus/latest/rat_focus/trait.HasFocus.html

[refFocus]: https://docs.rs/rat-focus/latest/rat_focus/struct.Focus.html

[refSalsaTerminal]: https://docs.rs/rat-salsa/latest/rat_salsa/terminal/trait.Terminal.html

[refPollEvents]: https://docs.rs/rat-salsa/latest/rat_salsa/poll/trait.PollEvents.html
