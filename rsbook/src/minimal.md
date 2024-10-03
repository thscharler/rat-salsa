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

run_tui is feed with 

- app: This is just the unit-struct Scenery. 
  It provides the scenery for the application, adds
  a status bar, displays error messages, and forwards the 
  real application Minimal.
  
- global: whatever global state is necessary. This global
  state is useable across all app-widgets. Otherwise the
  app-widgets only see their own state.
  
- state: the state-struct SceneryState. 

- [RunConfig][refRunConfig]: configures the event-loop

  - If you need some special terminal init/shutdown
    commands, implement the rat-salsa::Terminal trait
    and set it here. 
  - Set the number of worker threads.
  - Add extra event-sources. Implement the PollEvents trait.
    This will need some extra trait for the appstate to
    distribute your events.
    
    See [examples/life.rs][refLife] for an example.
  
***

The rest is not very exciting. It defines a config-struct
which is just empty, loads a default theme for the application 
and makes both accessible via the global state. 

## mod global

Defines the global state...


## mod config

Defines the config...

## mod message

This defines messages that can be sent between different parts of
the application. If you structure the application using AppWidget/AppState
the different parts have no easy way to communicate or to even know
of each others existence. Which is good. But sometimes they still need
to communicate. 

The MinimalMsg enum defines all messages that can be interchanged.

> This is also the means to report back information from a worker thread. 

Of course every message value can have all the data it needs to convey. 

## mod scenery

```
    #[derive(Debug)]
    pub struct Scenery;

    #[derive(Debug, Default)]
    pub struct SceneryState {
        pub minimal: MinimalState,
    }
```

Defines a unit struct for the scenery and a struct for any state. 
This here holds the state for the actual application. 

```
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

```
    impl AppState<GlobalState, MinimalMsg, Error> for SceneryState {
```

AppState has three type parameters that occur everywhere. I couldn't cut
back that number any further ...

```
        fn init(&mut self, ctx: &mut AppContext<'_>) -> Result<(), Error> {
            ctx.focus = Some(build_focus(&self.minimal));
            self.minimal.init(ctx)?;
            Ok(())
        }        
```

init is the first event for every application. 

it sets up the initial [Focus](./focus) for the application and
forwards to MinimalState.

















[refRunConfig]: https://docs.rs/rat-salsa/latest/rat_salsa/struct.RunConfig.html

[refLife]: https://github.com/thscharler/rat-salsa/blob/master/examples/life.life
