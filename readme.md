[![crates.io](https://img.shields.io/crates/v/rat-event.svg)](https://crates.io/crates/rat-event)
[![Documentation](https://docs.rs/rat-event/badge.svg)](https://docs.rs/rat-event)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License](https://img.shields.io/badge/license-APACHE-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
![](https://tokei.rs/b1/github/thscharler/rat-event)

This crate is a part of [rat-salsa][refRatSalsa].

* [Changes](https://github.com/thscharler/rat-event/blob/master/changes.md)

# Rat-Event

## Why?

This crate defines the trait [HandleEvent](HandleEvent) to help with
composability of event-handling for ratatui widgets.

Objectives are

- work for all event-types.
- allow for multiple handlers per widget
    - to override the key-bindings
    - to have different key-bindings for certain scenarios.
- have a return type to indicate what state change occured.

```rust ignore
    pub trait HandleEvent<Event, Qualifier, Return>
where
    Return: ConsumedEvent
{
    fn handle(
        &mut self,
        event: &Event,
        qualifier: Qualifier
    ) -> Return;
}
```

## Event

Can be anything.

## Qualifier

There are predefined qualifiers

* [Regular](Regular) - Do what is considered 'normal' behaviour.
  Can vary depending on the actual state of the widget
  (e.g. focus)

* [MouseOnly](MouseOnly) - Splitting off mouse interaction helps when
  you only want to redefine the key bindings. And handling
  mouse events is usually more involved/complicated/specific.

* [DoubleClick](DoubleClick) - Double clicks are a bit special for widgets,
  often it requires a distinct return type and it's not
  as generally needed as other mouse behaviour.

* [Popup](Popup), [Dialog](Dialog) - Specialized event-handlers, but they
  tend to popup again and again.

## Return

The return type can be anything at all.

To be useful it is required to implement
[ConsumedEvent](ConsumedEvent) to indicate if the event has been
handled by the widget and further event-handling can stop.

To set a baseline for the return type this crate defines the enum
[Outcome](Outcome) which can indicate if a render is necessary or not.

> For interop all return types in rat-salsa are convertible
> to/from Outcome.


[refRatSalsa]: https://docs.rs/rat-salsa/latest/rat_salsa/
