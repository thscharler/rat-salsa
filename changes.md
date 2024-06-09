# 0.12.2

* Prefix all widgets with 'R' to disambiguate from their rat-input cousins.

* add REditTable
* add RNumberInput
* add HasFocus for container widgets.

* fix: screen_cursor() should only return a value if the widget
  is focused. not correct everywhere.
* hack: Block event processing for TextInputState and MaskedInputState when gained_focus().
  This avoids thrashing the selection with the focus-click.

# 0.12.1

* add various functions to FTable
* add Fill widget

* fix missing event-handling for FTable
* fix missing re-exports
* fix broken event-handling. is_focused() is essential here.

# 0.12.0

* update FTable

# 0.11.0

* migrated after upstream change in rat-event

# 0.10.0

* Added all available widgets now.

# 0.9.0

Initial version from test area.