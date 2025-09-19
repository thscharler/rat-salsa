# 1.2.5

* docs
* undo change in flow! macros.

# 1.2.4

* add break_flow! macro
* change flow! macro, accepts 'tt' now.

# 1.2.3

* fix: naming of and() and and_try() was wrong. add and_then() and
  and_then_try() to stay in line with Result and Option.
  deprecate the old variants.
* feature: add break_flow! macro, that doesn't return but does a labeled break
  instead.
* feature: flow!, try_flow! and break_flow! now accept a full :tt instead of an
  expression.

# 1.2.2

* fix: ConsumedEvent::and() and ConsumedEvent::and_try() must
  do the opposite of or() and or_try(). The current impl is
  pointless.

# 1.2.1

* fix: docs

# 1.2.0

* feature: add Default for Outcome::Continue

# 1.1.1

* moved all rat-crates to one repo

# 1.1.0

* Key Release events are not generally available.

  Add a static flag which can be queried with
  have_keyboard_enhancement() and set it during terminal
  initialization.

  There already exists a similar static for double-click delay,
  and this sits in the same niche.

* If KeyBoardEnhancement is set for a terminal it starts to
  differentiate between Key Press and Key Repeat. This is not
  very useful for most applications, and the ct_event! macro now
  covers both under the label 'Press'.

# 1.0.0

stabilization

# 0.26.1

* feature: add scroll up/scroll down without any bindings to
  ct_event!

# 0.26

** upgrade to ratatui 0.29 **

# feature: add mouse_trap() to capture most events for a given area.

# 0.25.2

* fix: inline some trait fn.

# 0.25.1

* docs

# 0.25

Sync version for beta.

* fix double-click recognition. reset after timeout lead to
  needing a triple click quite often.

# 0.16.1

* fix docs

# 0.16.0

* upgrade ratatui to 0.28

* break: remove consider syntext in flow!

* break: remove or_else!

* break: renamed flow_ok! to try_flow!

# 0.15.0

* break: rename `FocusKeys` to the more fitting `Regular`.

* break: unified naming of Outcome and rat_salsa::Control

    * rename Outcome::NotUsed -> Outcome::Continue
* break: rename utils: item_at_clicked->item_at,
  row_at_clicked->row_at column_at_clicked->column_at

* feature: add a hover flag

* feature: MouseFlags recognize double-click patterns

    * down-up-up seems to occur
    * down-up-down-up with a timeout is added.
* feature: add general qualifier DoubleClick.

* feature: flow! and flow_ok! get one more variant: flow!
  (regular, consider extra)

* feature: or_else! for another type of control-flow.

* feature: add ConsumedEvent::then() for chaining.

* feature: add catch all HandleEvent for the null state `()`.

* refactor: remove all BitOr behaviour. It's not worth it.

# 0.14.6

* add Dialog qualifier

# 0.14.5

* add item_at_clicked() helper
* add `ct_event!(key ANY-'k')` matches any modifier.
* add Popup event qualifier. Used to split the event-handling of
  a widget in baseline and special treatment for popup/overlays
  This helps with the 'what has been clicked' problem of popups.
  Do all popup handling first and regular handling later.

# 0.14.4

* feature: add or_else() to ConsumedEvent
* fix: rename focus-gained and focus-lost to focus_gained and
  focus_lost. rustfmt formatted those weirdly.

# 0.14.3

* refactor: Rename KeyMap type parameter to `Qualifier`. This
  parameter is worth more than just a simple key-map.

* Fix: flow!() had it's if the wrong way around.

* Fix: ct_event!() couldn't handle F-keys.

# 0.14.2

* Add default impl for some of TableDataIter's functions.
* rendering: switch the row counter to Option<usize>, which is
  fundamentally more correct. This helped fix some quirks if the
  provided row-count is wrong.
* Add focus-lost, focus-gained and paste for completeness to
  ct_event.
* Move flow! up here from rat-salsa as it can be commonly useful.
  Complement flow! with flow_ok! which Ok-wraps its result.

# 0.14.1

* Add conversion from bool for Outcome.

# 0.14.0

* Reorg module layout. Outcome should be at the toplevel,
  everything else is confusing for the dependend crates.

# 0.13.3

* Reset immediately after doubleclick.

# 0.13.2

* Switch MouseFlags to interior mutability.

# 0.13.1

* Further testing showed that MouseFlags can be simplified.

# 0.13.0

* Add MouseFlags for interactions like double-click and drag.
  Filtering those is non-trivial, this struct makes it easier.
* Rename UsedEvent to ConsumedEvent. Fits the terminology better.

# 0.12.5

* Remove ratatui-flag: unstable-widget-ref

# 0.12.4

* Add CONTROL_ALT

# 0.12.3

* Extend ct_event!

# 0.12.2

* Add general `Outcome` type as a baseline what can be expected
  from any widget.

# 0.12.1

* Impl UsedEvent for Result<T,E> and Option<T> where T:
  UsedEvent.

# 0.12.0

* Add trait UsedEvent to enable layering of widgets. Provides
  the information whether an event has been consumed by a inner
  layer.

# 0.11.0

* Add utils for row_at_clicked, column_at_clicked, row_at_drag,
  column_at_drag.

# 0.10.0

* Fix handle() to take a &Event instead of an Event. This was so
  in the original, but I was too clever. In general event types
  are not necessarily Copy but read only, so `&` should be fine.

# 0.9.0

Initial release.
