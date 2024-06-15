# 0.10.3

feature: add name for Focus debugging.
fix: lost&gained shouldn't be set if the focus stays the same. there might
be an exception for single field focus lists, but I think this should not be covered here.

# 0.10.2

* feature: add HasFocus trait for container widgets.
    * adds Focus::add_flag(), Focus::add_container() and
    * renames Focus::append() to Focus::add_focus()

* add Focus::new_grp() for groups of widgets.
* add FocusFlag::name field for debugging.
* add Focus::enable_log() for debugging.
* add Focus::clear()
* add Focus::focused(), Focus::lost_focus(), Focus::gained_focus()
  to get the current state.

# 0.10.1

* add focus_widget_no_lost and focus_widget which use the HasFocusFlag trait. Ease of use.
* add init() to set the initial focus and clear the rest.

# 0.10.0

* upstream change in rat-event.

# 0.9.1

* Fix versions

# 0.9.0

* Initial copy from the test area.
