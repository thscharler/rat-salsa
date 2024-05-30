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

* Impl UsedEvent for Result<T,E> and Option<T> where T: UsedEvent.

# 0.12.0

* Add trait UsedEvent to enable layering of widgets. Provides the
  information whether an event has been consumed by a inner layer.

# 0.11.0

* Add utils for row_at_clicked, column_at_clicked, row_at_drag, column_at_drag.

# 0.10.0

* Fix handle() to take a &Event instead of an Event. This was so in the
  original, but I was too clever. In general event types are not necessarily
  Copy but read only, so `&` should be fine.

# 0.9.0

Initial release. 