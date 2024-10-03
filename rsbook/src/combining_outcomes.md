## Event-Handling

There are a few complications, though:

```rust ignore
fn focus(state: &State) -> Focus {
    Focus::new(&[
        &state.widget1,
        &state.widget2,
        &start.widget3
    ])
}

fn handle_input(event: &Event, state: &mut State) -> Result<Outcome, anyhow::Error> {
    // does all the keyboard and mouse navigation, and 
    // returns Outcome::Changed if something noteworthy changed. 
    let f = focus(state).handle(event, Regular);

    // flow_ok! is a macro from rat-event. It returns early if 
    // the result is *not* Outcome::NotUsed. 
    //
    // But: If the result here is the third variant Outcome::Unchanged,
    //      this would return early with Outcome::Unchanged and 
    //      would not know that there was something else too. 
    //      If you trigger the repaint with Outcome::Changed you 
    //      would never see the focus-change.
    //
    // So:  Use the other way to combine the results of the
    //      eventhandlers.
    
    let mut r = state.widget1.handle(event, Regular)
        .or_else(|| state.widget2.handle(event, Regular))
        .or_else(|| state.widget3.handle(event, Regular));
    
    // Combine the two results. This returns max(f,r) which works.
    Ok(max(f,r))
}
```

This solution is a bit sub-par, admittedly.

If you are using [rat-salsa](https://crates.io/crates/rat-salsa)
as your application framework, there is a queue for extra results
from event-handling. You can add the result of `focus().handle()`
directly to the queue, and everything's well.

If you already have your own framework, you might want something
similar.
