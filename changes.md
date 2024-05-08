# 0.10.0

* Fix handle() to take a &Event instead of an Event. This was so in the
  original, but I was too clever. In general event types are not necessarily
  Copy but read only, so `&` should be fine.

# 0.9.0

Initial release. 