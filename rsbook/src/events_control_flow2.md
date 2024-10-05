
# Extended control flow

AppContext has functions that help with application control flow.

- `add_timer()`: Sets a timer event. This returns a TimerHandle
  to identify a specific timer.
  
- `queue()` and `queue_err()`: These functions add additional
  items to the list that will be processed after the event
  handler returns. The result of the event handler will be added
  at the end of this list too.
  
- `spawn()`: Run a closure as a background task. Such a closure
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
  Spawns a background task. This is a move closure to own the
  parameters for the 'static closure. It returns a clone of the
  cancel token to interrupt the task if necessary.
  
  ```rust
    let cancel = ctx.spawn(move |cancel, send| {
  ```  
  Captures its parameters.
  
  ```rust
    let mut data = Data::new(config);
  ```  
  Goes into the extended calculation. This uses `send` to report
  a partial result as a message. At a point where canceling is
  sensible it checks the cancel state.
  
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
