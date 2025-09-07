# Control flow

## Everything goes through event()

rat-salsa models everything that happens as some kind of event, that is
send to your application. Your `event()` function is responsible to distribute 
those events through your application modules/component-tree and out to each 
widget-leaf. 

The result can be an unhandled error or a Control flag that signals to the
event-loop what to do next. 

## Widgets are similar

rat-widgets follow the same logic, you let them handle() some event, and they 
return an Outcome what happened at some level of abstraction. 

Most of the time it's enough to immediately react to the outcome and make
some follow-up change to your state. If you need some non-local effect you 
translate this to an application level event and let it run through the 
event() distribution to find some point where it can be handled. 

## Conclusion

The advantages I see with this are

- there is one source of truth what happens in your application. Events 
may be intercepted on their way, but you have to find that place in only
one call-tree. 
- most effects are localized, what happens due to a key-press can be
found on the following line of code. 
- long range effects can be tracked by 'find usage' of the event enum. 

# Details, details

[Control](https://docs.rs/rat-salsa/latest/rat_salsa/enum.Control.html) enum
and
[Outcome](https://docs.rs/rat-event/latest/rat_event/enum.Outcome.html) enum.

And the pain of combining two of those

[ConsumedEvent](https://docs.rs/rat-event/latest/rat_event/trait.ConsumedEvent.html)
[try_flow!](https://docs.rs/rat-event/latest/rat_event/macro.try_flow.html)

# Determinism

The followup events caused by event() will always be processed before requesting
fresh events from any event-source. This mostly ensures that there is a 
well defined sequence of followup events that come out from one original
event. Processing the aftermath of one key-press will not be interfered by
the next key-press that thrashes your expectations. 
