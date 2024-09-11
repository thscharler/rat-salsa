# 0.12.1

* fix docs

# 0.12.0

* update ratatui to 0.28

# 0.11.0

* break: rename Focus::init() to Focus::initial()
* break: remove the lifetime from Focus.
  FocusFlags now contain a Rc<> of the flags, and when constructing
  the Focus a clone of the Rc<> is used inside Focus. This makes Focus
  more generally usable.
* break: Containers get their own ContainerFlag which works the same
  as FocusFlag for widgets. Avoids confusion of the two.
* break: trait HasFocus has been extended for better container support.
* break: replace the HasFocus::navigable() result with it's own
  enum for fine grain control of widget/focus interaction.
* break: change the name in FocusFlag to Box<str> to allow non-static names.

* feature: FocusFlags can now be compared. It uses Rc::ptr_eq for
  comparison.
* feature: add functions to manipulate the focus-list after construction.
  Allows adding/removing/replacing widgets and containers.
*

# 0.10.5

* change FocusFlag::set() to take a bool
* fix: areas for sub-containers must be checked before the area of the container.
  otherwise they probably never get a hit via mouse.

# 0.10.4

* feature: Add ZRect, a Rect with z-order. HasFocusFlag can not only return
  one Rect as the focusable area, but multiple ZRects for component-area +
  popup area.
* feature: Add HasFocusFlag::navigable(), denotes if a field can be reached
  with normal keyboard focus.
* refactor: Rename Focus::new_accu to new_container.

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
