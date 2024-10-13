# 0.26.0

Missed a few things:

break: The functions is_focused(), lost_focus() and gained_focus() of HasFocus
can clash with the same in HasFocusFlag. Renamed the HasFocus functions to
is_container_focused(), container_lost_focus() and container_gained_focus().
break: FocusAdapter gets a const type param and adds z_areas. Can now emulate
a full widget.

feature: add Focus::none() to reset all focus flags.
feature: add ContainerAdapter analogous to FocusAdapter.
feature: add Focus::expel_focus() and Focus::expel_focus_container().
The expel the focus from the given widget/container and place it elsewhere.

fix: focus_container() should always focus regardless of navigation flags.
add first_container() that respects navigations-flags.
fix: update_container and replace_container used an outdated method to
build a container.

# 0.25.0

Sync version for beta.

Last big changes ...

* break: Reimagined focus init.

    - Focus looses most functions to modify the widget
      structure. And it's constructors. Only the container
      rebuild fn update_container(), replace_container()
      and remove_container() remain. All construction goes
      to FocusBuilder.

    - FocusBuilder. Classic builder for Focus, only
      a widget() and a container() fn remain for this.
      But those are fluent fn's so it's ok.

    - HasFocus gained a build() method which takes
      a FocusBuilder. This saves a few Vecs, and
      builder style is quite nice for focus construction.

      container() and area() have solid fallbacks.

    - add FocusBuilder::for_container() and FocusBuilder::rebuild() that
      can work with a &dyn HasFocus.

* Add Focus::focus_container() to focus the first widget.
* Add Navigation::Lock. Lock the focus with the current
  widget.
* FocusFlag now implements HasFocusFlag.
* HasFocus::area() now works without HasFocus::container() existing.

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
