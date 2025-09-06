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
        scenery::init,
        scenery::render,
        scenery::event,
        scenery::error,
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

`run_tui` runs the event-loop and calls out to the 4 
functions `init`, `render`, `event` and `error`.
