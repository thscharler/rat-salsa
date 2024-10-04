
# Event handling

The widgets for [rat-widget][refRatWidget] use the trait
HandleEvent defined in [rat-event][refRatEvent].


## General

```rust
        fn crossterm(
            &mut self,
            event: &Event,
            ctx: &mut AppContext<'_>,
        ) -> Result<Control<TurboMsg>, Error>
```

rat-salsa distributes the events with the plain functions in
AppState. It doesn't do any routing to specific widgets or such.
Any further distribution of events is up to the application.

All handling functions get the extra [ctx][refAppContext] for
access to application global data.

As result of each event it gets a
`Result<Control<Action>, Error>`, that tells it how to proceed.

- [Control::Continue][refControl]: continue with the next event.
  
- [Control::Unchanged][refControl]: event has been used, but
  requires no rendering. Just continues with the next event.
  
  Inside the application this is used to cut off eventhandling.
  
- [Control::Changed][refControl]: event has been used, and
  a render is necessary. Continues with the next event after
  rendering.
  
- [Control::Message(m)][refControl]: Distributes the message
  throughout the application. This works as just another event
  type with its own function responsible for distribution.
  
  The individual AppWidgets making up the application are quite
  isolated from other parts and just have access to their own
  state and some global application state.
  
  All information that crosses those lines uses messages with
  some payload.
  
- [Control::Quit][refControl]: Ends the event-loop and resets the
  terminal. This returns from run_tui() so whatever shutdown is
  needed can be done there.
  
## Control flow

There are the following constructs to help with control flow in
handler functions.

- Trait [ConsumedEvent][refConsumedEvent].
  
  `or_else` and `or_else_try` run a closure if the return value
  is Control::Continue; `and` and `and_try` run a closure if the
  return value is anything else.
  
- Macros `flow!` and `try_flow!`. These run the codeblock and
  return early if the result is anything but Control::Continue.
  `try_flow!` Ok-wraps the result, both do `.into()` conversion.
  
- Extensive use of `From<>`.
  
  - Widgets use the `Outcome` enum as a result, or have their
    derived outcome type if it is not sufficient. All extra
    outcome types are convertible to the base Outcome.
    
  - On the rat-salsa side is Control which is modeled after
    `Outcome` too but with its own extensions. It has a
    `From<T: Into<Outcome>` implementation. That means everything
    that is convertible to Outcome can in turn be converted to
    Control.
    
    This leads to
    
    - widgets don't need to know about rat-salsa.
    - rat-salsa doesn't need to know about every last widget.
    
  - Widgets often have action-functions that return bool to
    indicate changed/not changed. There is a conversion for Outcome
    that maps true/false to Changed/Unchanged. So those results
    are integrated too. 
    
- Ord for Outcome/Control

  Both implement Ord; for Outcome that's straightforward, Control 
  ignores the Message(m) payload for this purpose. 
  
  Now it's possible to combine results
  
  ```rust
    
    max(r1, r2)
    
  ```
  
  The enum values are ordered in a way that this gives a sensible result.
  
## Extended control flow

AppContext has functions that help with application control flow.

- add_timer(): Sets a timer event. This returns a TimerHandle to
  identify a specific timer.
  
- add_idle(): Adds a timer that triggers with a delay after the
  idle state has been detected.
  
- queue() and queue_err(): These functions add additional items
  to the list that will be processed after the event handler returns.
  The result of the event handler will be added at the end of this 
  list too. 
  
- spawn(): Run a closure as a background task. Such a closure
  gets a cancel-token and a back-channel to report its findings.
  
  ```rust
    let cancel = ctx.spawn(move |cancel, send| {
        let mut data = Data::new(config);
    
        loop {
            
            // ... long task ...
            
            // report partial results
            send.send(Ok(Control::Message(AppMsg::Partial)));
            
            if cancel.is_canceled() {
                break;
            }
        }
        
        Ok(Control::Message(AppMsg::Final))
    });
  ```  
  Spawns a background task. This is a move closure to capture
  the parameters for the closure. It returns a cancel token to
  interrupt the task if necessary.
  
  ```rust
    let cancel = ctx.spawn(move |cancel, send| {
  ```  
  Captures its parameters.
  
  ```rust
    let mut data = Data::new(config);
  ```  
  Goes into the extended calculation. This uses send to report a partial
  result as a message. At a point where canceling is sensible it checks
  the cancel state. 
  
  ```rust
    loop {
            
        // ... long task ...
        
        // report partial results
        send.send(Ok(Control::Message(AppMsg::Partial)));
        
        if cancel.is_canceled() {
            break;
        }
    }
    ```
  
  Finishes with some result.
  
  ```rust
    Ok(Control::Message(AppMsg::Final))
  ```
  
  
  
  
[refRatEvent]: https://docs.rs/rat-event/latest/rat_event/

[refControl]: https://docs.rs/rat-salsa/latest/rat_salsa/enum.Control.html

[refRatWidget]: https://docs.rs/rat-widget/latest/rat_widget/

[refAppContext]: https://docs.rs/rat-salsa/latest/rat_salsa/struct.AppContext.html

[refConsumedEvent]: https://docs.rs/rat-event/latest/rat_event/trait.ConsumedEvent.html
