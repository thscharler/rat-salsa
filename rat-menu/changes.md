# 3.0.0

ratatui 0.30

# 2.5.0

* feat: add MenuLine::width()

# 2.4.1

* fix dependencies

# 2.4.0

* refactor: when rendering the menu-popup the given area is ignored
  and the area is taken from the state.
* fix: Menubar had some problems with relocations.

# 2.3.0

* allow direct rendering of Menubar

# 2.2.0

* refactor/fix: All widgets now use the same logic when
  using a Block from some xxxStyle struct. And all xxxStyle structs
  now have the abilitiy to just set the border_style and the
  title_style without defining a full Block.

* break (minor): styling for the popup-menu is now separated from
  styling for the main-menu. If you use MenuStyle you need to define the
  colors for the popup-menu. It will fall back to defaults, so your
  application will still run, just rendering will be off.

* feature: add separate style for menu separators.

# 2.1.0

* break: PopupMenu: width() -> menu_width() and width_opt() -> menu_width_opt()
  at the same time adding width() and height() that return the dimensions.

* feature: add Block to MenuLine
* fix: minor things to conform to the api guidelines.

# 2.0.0

* breaking changes in rat-popup
* break: remove deprecated

# 1.1.1

* fix: display of 2 wide glyphs.

# 1.1.0

* fix: remove pub use of rat-event.
* refactor: fix upstream deprecated from rat-popup.
* refactor: move Block into PopupMenu.
* feature: better styling

# 1.0.4

* update dependencies

# 1.0.3

* fix: there are some subtle nuances with select_at().
  Sometimes we need it to report on any match, sometimes only on a changed match.
  Thus split select_at() into select_at() which keeps the current behaviour
  of only reporting actual changes, and select_at_always() which only
  checks for an area hit.

# 1.0.2

* fix: MenuLine+PopupMenu: Fix behaviour of selected.
  Fallback to first item if None is selected and there are items.
  Change to None if there are no items.
* fix: Some event handling still returned Outcome::Unchanged.
  This is discouraged for most cases.
* fix: Spurious panics with missing disabled data.
  Missing disabled data is a good hint for a method call before the
  first render. Bail out and do nothing for most cases.

# 1.0.1

* fix: ensure that the select_style is always patched onto the
  base-style. This makes a select-style `Style::new().underlined()`
  work fine.

# 1.0.0

... jump ...

# 0.33.0

* break: change menu item syntax to use a \\ prefix for separators.

# 0.32.0

* break: Menubar: remove Regular event-handling and do everything
  in the Popup event handler. Simplifies usage.

# 0.31.4

* feature: Menubar: submenus open with Enter too.

# 0.31.3

* update rat-focus

# 0.31.2

* moved all rat-crates to one repo

# 0.31.1

* clippy fixes

# 0.31.0

* remove uses of ZRect.
  Uses HasFocus::build() to add both the main menu and the popup menu areas
  as focus areas. Allows to set the z-value for the popup, which allows
  the popup-menus to be always on top of the application.

# 0.30.0

** upgrade to ratatui 0.29 **

* fix: use mouse_trap() to capture events for popup menus.
* feat: provide usable fallbacks when no style is set.

# 0.29.0

* break: MenuStructure requires Debug
* break: MenuStyle uses PopupStyle

# 0.28.0

* break: replace SubmenuPlacement with Placement

* fix: Menubar and PopupMenu quirks

* feature: add Menubar::right_style()
* feature: add MenuLine::xxx_opt() where useful.
* feature: add PopupMenu::xxx_opt() where useful.

# 0.27.0

* break: final renames in rat-focus.

# 0.26.0

* break: split-off crate rat-popup from PopupMenu and
  reimplemented it from there. This break Placement, which is
  now considerable larger. And it breaks PopupMenu::render()
  as that now expects the Rect of the popup instead of the
  related widget.
  As that was a major strangeness factor, I'm happy to accept the break.
* break: renamed `Menu_B_arState` to `Menu_b_arState` to fit in.

fix: select_at reported changes even if there were none. Lead to
a lot of unnecessary renders.

fix: update dependencies

# 0.25.0

Sync version for beta.

* fix: popup stays reactive event when not displayed.

# 0.10.0

Move from rat-widget.

* feature: allow disabled items
* refactor: add MenuItem as first class concept.
    * better raw string syntax
    * support for all widgets
* feature: add MenuBuilder and use it for MenuStructure trait.
* fix: diverse rendering quirks