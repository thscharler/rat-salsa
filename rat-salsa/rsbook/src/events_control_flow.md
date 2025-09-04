
# Control flow

There are some constructs to help with control flow in
handler functions.

- Trait [ConsumedEvent][refConsumedEvent] is implemented for
  Control and all Outcome types.
  
  The fn `or_else` and `or_else_try` run a closure if the return
  value is Control::Continue; `and` and `and_try` run a closure
  if the return value is anything else.
  
- Macros [flow!][refFlow] and [try_flow!][refTryFlow]. These run
  the codeblock and return early if the result is anything but
  Control::Continue. `try_flow!` Ok-wraps the result, both do
  `.into()` conversion.
  
Both reach similar results, and there are situations where one
or the other is easier/clearer. 
  
- Extensive use of `From<>`.
  
  - Widgets use the `Outcome` enum as a result, or have their
    derived outcome type if it is not sufficient. All extra
    outcome types are convertible to the base Outcome.
    
  - On the rat-salsa side is `Control` which is modeled
    after `Outcome` with its own extensions. It has a
    `From<T: Into<Outcome>` implementation. That means everything
    that is convertible to Outcome can in turn be converted to
    Control.
    
    This leads to
    
    - widgets don't need to know about rat-salsa.
    - rat-salsa doesn't need to know about every last widget.
    
  - Widgets often have action-functions that return bool to
    indicate 'changed'/'not changed'. There is a conversion for
    Outcome that maps true/false to Changed/Continue. So those
    results are integrated too.
    
- Ord for Outcome/Control
  
  Both implement Ord; for Outcome that's straightforward, Control
  ignores the Message(m) payload for this purpose.
  
  Now it's possible to combine results
  
  ```rust
    
    max(r1, r2)
    
  ```  
  The enum values are ordered in a way that this gives a sensible
  result.
  
[refConsumedEvent]: https://docs.rs/rat-event/latest/rat_event/trait.ConsumedEvent.html

[refFlow]: https://docs.rs/rat-event/latest/rat_event/macro.flow.html

[refTryFlow]: https://docs.rs/rat-event/latest/rat_event/macro.try_flow.html

