# 2.0.2

fix: reexport types used in macros. they will not be available if
the user imports ratatui instead of ratatui_core. 

# 2.0.1

the real deal ...

# 2.0.0

ratatui 0.30

# 1.6.1

* fix dependencies
* make examples work with rat-theme4

# 1.6.0

* feature: add flip_focus() to switch between menubar and
  some selected widget.
* feature: add HasFocus::build_nav(). This is called when the
  default Navigation is overridden when adding the widget to FocusBuilder.

# 1.5.1

* feature: add flip_focus() to switch between menubar and
  some selected widget.

# 1.5.0

These changes are related to the introduction of on_gained()
and on_lost() callbacks:

* break: FocusFlag::name() changed from &str to Box<str>.
* feature: FocusFlag::set_name() now available.
* feature: FocusFlag::with_name() adds a name later.

# 1.4.0

* feature: allow calling future() for container flags.

# 1.3.0

* feature: add callbacks for on_gained/on_lost. These are meant for
  widget implementors to get reliable notifications for these.
  Of course, it requires that you put part of your state behind Rc's ...

  These are also called for container-flags for completeness.

* feature: add enable_panic() and disable_panic() to panic when called
  with widgets that are not part of the focus. A debugging aid.
* feature: FocusBuilder: add log_build_for and log_rebuild_for.

# 1.2.0

* docs
* fix: don't reexport rat-event.
* fix: when setting the focus to a container, set to the first
  *navigable* widget instead of just the first.
* fix: remove FocusTraversal. Not useful enough.

* feature: add Focus::future() to set the focus to a widget
  not yet in the widget list.

# 1.1.1

* fix: impl_has_focus! should use full path for Rect.

# 1.1.0

* break: match_focus doesn't work any more with changes in macro_rules!
  changed fallback branch from '_' to 'else'

* fix: Focus::handle() event shouldn't return 'Unchanged' but 'Continue'
  for unprocessed events.
* feature: add HasFocus::id() as a unique identifier for a widget.
  Returns a unique usize value. Allow focusing by this id. This is
  for interop with other subsystems where using the FocusFlag feels
  strange.
* Add a qualifier FocusTraversal for event-handling.
  This is used by TextArea to provide alternative navigation keys instead
  of tab. Uses Esc to focus the next widget after a TextArea.

# 1.0.2

* update dependencies

# 1.0.1

* add some inlines

# 1.0.0

# 0.33.1

* feature: Focus: add next_force() and prev_force() to overcome
  navigation blocking.

# 0.33.0

* break: rename append_leaf() to leaf_widget()
* break: rename append_flags() to widget_with_flags()
* break: remove focus_flag_no_lost() no longer necessary.
* break: rename first_container() to first_in()
* break: start_with_flags() no longer accepts an optional FocusFlag. Useless feature.

* feature: first_in() now accepts plain widgets.

# 0.32.1

* fix: some docs

# 0.32.0

* BREAK: renames & refactors
    * FocusBuilder::for_container -> build_for
    * FocusBuilder::rebuild -> rebuild_for
    * FocusBuilder::add_widget -> append_flags
    * FocusBuilder::start_container -> start_with_flags
    * refactor: move functionality from HasFocus::build() to FocusBuilder::append_leaf()
    * refactor: remove impl for HasFocus::build(). Implementing this
      for simple widgets is not the default use case irl.
      These changes give better code language and better defaults.

* feature: Add focus_id() to FocusFlag to get a basic ID for a widget.
    - Alternative to storing the FocusFlag itself.
    - Doesn't persist, runtime only.
* feature: add impl_has_focus! macro.
    - ```impl_has_focus!(container_flag:area: widget1, widget2, widget3 for SomeComposit)```
    - ```impl_has_focus!(container_flag: widget1, widget2, widget3 for SomeComposit)```
    - ```impl_has_focus!(widget1, widget2, widget3 for SomeComposit)```
* fix: Better log messages.

# 0.31.0

* BREAK: Remove ContainerFlag and relatives.
  Since the addition of HasFocus::build() this separate trait
  has become more and more useless. And is now removed completely.
* break: remove focus_container(). focus() does all the work.

# 0.30.2

* moved all rat-crates to one repo

# 0.30.1

* feature: add widgets() which can take an array of `dyn HasFocus` and
  add all to the focus. convenience.
* fix: mouse focus reported a change even if the focus stays on the same
  widget after a mouse click.

# 0.30.0

* Add Hash to FocusFlag and ContainerFlag. With this addition those
  two now can act as unique id to reference a widget from other subsystems.

* perf: add an internal hashset to improve 'contains' checks.
  This removes the last O(n) when adding a widget. There is still a
  loop when adding a container, but that one corresponds with the
  depth of the widget tree, so it should be fine.

# 0.29.0

* break: remove ZRect. This was insufficient at the end, and the perf was not so good too.

  Replaced HasFocus::z_areas() with HasFocus::area_z() which returns a single z-value
  for the area. Now the same FocusFlag can now be added for a further areas as long
  as it only uses Navigation::Mouse for these. This is good enough for popups.

  Adds FocusContainer::area_z(). The z_area for a widget is now calculated starting
  from the base-z value of the surrounding container. And containers within containers
  stack one upon the other. When container and widget areas are encountered with
  the same z-value, widgets get prioritized.

  This gives a clean stacking now, and can satisfy window like structures.

* Focus::clone_destruct() gives a clone of the internal structures for debugging.

# 0.28.0

** upgrade to ratatui 0.29 **

* feature: add support for rat-reloc. adds relocate_z_area() and relocate_z_areas()

# 0.27.1

* feature: add ZRect::union_all(), ZRect::union()

# 0.27.0

Final renames.

* HasFocusFlag changes to HasFocus.
* current HasFocus changes to FocusContainer.

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
