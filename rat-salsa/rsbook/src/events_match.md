
# Matching events

```rust
    try_flow!(match &event {
        ct_event!(resized) => Control::Changed,
        ct_event!(key press CONTROL-'q') => Control::Quit,
        _ => Control::Continue,
    });
```

If you want to match specific events during event-handling 
match is great. Less so is the struct pattern for crossterm
events.

That's why I started with [ct_event!][refCtEvent] ...

It provides a very readable syntax, and I think it now covers
all of crossterm::Event. 

> [!NOTE]: If you use `key press SHIFT-'q'` it will not work.
> It expects a capital 'Q' in that case. The same for any
> combination with SHIFT.



[refCtEvent]: https://docs.rs/rat-event/latest/rat_event/macro.ct_event.html
