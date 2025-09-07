# examples/minimal.rs

## main()


```rust
fn main() -> Result<(), Error> {
    setup_logging()?;

    let config = Config::default();
    let theme = DarkTheme::new("Imperial".into(), IMPERIAL);
    let mut global = Global::new(config, theme);
    let mut state = Scenery::default();

    run_tui(
        init,
        render,
        event,
        error,
        &mut global,
        &mut state,
        RunConfig::default()?
            .poll(PollCrossterm) //
            .poll(PollRendered),
    )?;

    Ok(())
}

```

`run_tui` runs the event-loop and calls out to the 4 
functions `init`, `render`, `event` and `error`.

RunConfig contains the configuration for the terminal
and a list of event-sources that should be polled. You
can add your own too.

The application state is divided into

- Global: Everything that should be accessible
  throughout the application.
  
  - Config: I like to use this for program args and
    whatever permanent config I have.
  - Theme: A collection of widget-styles combined with
    a selection of color palettes.
- Scenery: Contains the stateful half of the
  widget-tree. Plus any extra state you need for some
  component of your application to work.
  
## Global    
  
Global state that is shared independend from the 
state tree.

```rust
/// Globally accessible data/state.
#[derive(Debug)]
pub struct Global {
    ctx: SalsaAppContext<AppEvent, Error>,
    pub cfg: Config,
    pub theme: DarkTheme,
}

impl SalsaContext<AppEvent, Error> for Global {
    fn set_salsa_ctx(&mut self, app_ctx: SalsaAppContext<AppEvent, Error>) {
        self.ctx = app_ctx;
    }

    #[inline(always)]
    fn salsa_ctx(&self) -> &SalsaAppContext<AppEvent, Error> {
        &self.ctx
    }
}

impl Global {
    pub fn new(cfg: Config, theme: DarkTheme) -> Self {
        Self {
            ctx: Default::default(),
            cfg,
            theme,
        }
    }
}
```

rat-salsa provides some infrastructure of its own. Global 
implements [SalsaContext][refSalsaContext] to give access to this infrastructure.
run_tui injects the concrete implementation via `set_salsa_ctx()`.
This gives seamless access to all global state. 

## Config

Configuration data. Either start parameters or from some config. 
I like to keep those separate from other things. 

```
/// Configuration.
#[derive(Debug, Default)]
pub struct Config {}
```

## Event

Instead of rat-salsa defining some event-type, the application
does it and provides conversions for every type one of the
event-sources can produce. 

You can also add any other messages you want to
distribute via event-handling. This can replace most
other forms of communication used, be it shared state
or your own queues etc.


```
/// Application wide messages.
#[derive(Debug)]
pub enum AppEvent {
    Event(crossterm::event::Event),
    Rendered,
    Message(String),
    Status(usize, String),
}

impl From<RenderedEvent> for AppEvent {
    fn from(_: RenderedEvent) -> Self {
        Self::Rendered
    }
}

impl From<crossterm::event::Event> for AppEvent {
    fn from(value: crossterm::event::Event) -> Self {
        Self::Event(value)
    }
}
```

## Application state

This state contains the states of any StatefulWidgets used. 
And everything else that is needed. 

```
#[derive(Debug, Default)]
pub struct Minimal {
    pub menu: MenuLineState,
    pub status: StatusLineState,
    pub error_dlg: MsgDialogState,
}
```

## Focus

Focus handling is sprinkled throughout the code. 
This macro defines which widgets in a container can get
the focus and in what order. 

```
impl_has_focus!(menu for Minimal);
```

## render()

This function is the equivalent to Widget::render(). 

> There is no trait or anything for this. If you need
> to structure your application just do so.

```
pub fn render(
    area: Rect,
    buf: &mut Buffer,
    state: &mut Minimal,
    ctx: &mut Global,
) -> Result<(), Error> {
    let t0 = SystemTime::now();

    let layout = Layout::vertical([
        Constraint::Fill(1), //
        Constraint::Length(1),
    ])
    .split(area);

    MenuLine::new()
        .styles(ctx.theme.menu_style())
        .item_parsed("_Quit")
        .render(layout[1], buf, &mut state.menu);

    if state.error_dlg.active() {
        MsgDialog::new()
            .styles(ctx.theme.msg_dialog_style())
            .render(layout[0], buf, &mut state.error_dlg);
    }

    let el = t0.elapsed().unwrap_or(Duration::from_nanos(0));
    state.status.status(1, format!("R {:.0?}", el).to_string());

    let status_layout = Layout::horizontal([
        Constraint::Fill(61), //
        Constraint::Fill(39),
    ])
    .split(layout[1]);

    StatusLine::new()
        .layout([
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(8),
        ])
        .styles(ctx.theme.statusline_style())
        .render(status_layout[1], buf, &mut state.status);

    Ok(())
}
```

## init()

Init is one of the functions given to run_tui(). It is
called once before the event-loop starts and after the
SalsaAppContext is initialized.

> This creates the Focus for the application and sets
> the focus to the first possible widget.

```
pub fn init(state: &mut Minimal, ctx: &mut Global) -> Result<(), Error> {
    ctx.set_focus(FocusBuilder::build_for(state));
    ctx.focus().first();
    Ok(())
}
```

## event()

The event function is called for every event that occurs. 

It returns a [Control][refControl] that determines what happens next on the
event-loop.

```
pub fn event(
    event: &AppEvent,
    state: &mut Minimal,
    ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
```

match is your friend here. 

```
    match event {
        AppEvent::Event(event) => {
```

The `Control` enum comes with perks. One is the [try_flow!][refTryFlow] 
macro, which breaks event-handling an returns early
when it finds that the event has been processed. Every value but
Control::Continue means the event has been processed.

> There is a second macro `ct_event!` for crossterm-event. It
> creates a pattern for crossterm events with a much nicer syntax
> compared to raw rust.

```                
            try_flow!(match &event {
                ct_event!(resized) => Control::Changed,
                ct_event!(key press CONTROL-'q') => Control::Quit,
                _ => Control::Continue,
            });

            try_flow!({
                if state.error_dlg.active() {
                    state.error_dlg.handle(event, Dialog).into()
                } else {
                    Control::Continue
                }
            });
```

Focus handling is so essential, it has its own place in
SalsaAppContext. And focus handling has some quirks that are
hidden behind this function.

```
            ctx.handle_focus(event);
            
```

The widgets in rat-widget all implement [HandleEvent][refHandleEvent]. 
It defines a `handle()` function that manages all event-handling
for the specific widget. The second parameter qualifies what kind
of event-handling should happen. `Regular` is what you normally
want, but there is `MouseOnly`, that only deals with mouse events,
and a few more.

`handle()` also allows for a widget-specific return type,
that can communicate at a high level what has happened.
Here we have the outcome that the first menu-item has
been activated, whatever that means we quit. With some From
magic all the other outcomes are converted to their
corresponding Control enum.

```            
            try_flow!(match state.menu.handle(event, Regular) {
                MenuOutcome::Activated(0) => Control::Quit,
                v => v.into(),
            });

            Ok(Control::Continue)
        }
```

Another event from a different event-source. This one is
generated by `run_tui()` and sent immediately after rendering
a frame.

> This is a good point to update the Focus. All rat-widgets
> store their areas when rendering. And as any of them might have
> changed, a renewed Focus with correct areas is a good thing.

```        
        AppEvent::Rendered => {
            ctx.set_focus(FocusBuilder::rebuild_for(state, ctx.take_focus()));
            Ok(Control::Continue)
        }
```

An application defined event. Instead of accessing the widget state
for the error-dialog or the statusbar you can send a message and
react at one point.

```        
        AppEvent::Message(s) => {
            state.error_dlg.append(s.as_str());
            Ok(Control::Changed)
        }
        AppEvent::Status(n, s) => {
            state.status.status(*n, s);
            Ok(Control::Changed)
        }
    }
}
```

## error() 

The last of the functions given to `run_tui()`.

At this point it can't do any better that logging and displaying
any error. 

```
pub fn error(
    event: Error,
    state: &mut Minimal,
    _ctx: &mut Global,
) -> Result<Control<AppEvent>, Error> {
    error!("{:?}", event);
    state.error_dlg.append(format!("{:?}", &*event).as_str());
    Ok(Control::Changed)
}
```

## References

[rat-event][https://docs.rs/rat-event/]
[rat-focus][https://docs.rs/rat-focus/]
[rat-widget][https://docs.rs/rat-widget]

This example

[minimal.rs][https://github.com/thscharler/rat-salsa/blob/master/rat-salsa2/examples/minimal.rs]

Another minimal example, with app-level components.

[nominal.rs][https://github.com/thscharler/rat-salsa/blob/master/rat-salsa2/examples/nominal.rs]



[refSalsaContext]: https://docs.rs/rat-salsa/latest/rat_salsa/trait.SalsaContext.html

[refControl]: https://docs.rs/rat-salsa/latest/rat_salsa/enum.Control.html

[refTryFlow]: https://docs.rs/rat-event/latest/rat_event/macro.try_flow.html

[refHandleEvent]: https://docs.rs/rat-event/latest/rat_event/trait.HandleEvent.html 
