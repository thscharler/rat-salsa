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