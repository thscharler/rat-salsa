# 0.28.6

* fix: version bump for rat-focus

# 0.28.5

* fix: didn't behave correctly on overflow of boundary.

# 0.28.4

* moved all rat-crates to one repo

# 0.28.3

* add border_style to PopupStyle. Allows setting the style
  without providing a definite border. When applying the PopupStyle
  style+border_style override the settings of a previously set block.

# 0.28.2

* clippy fixes

# 0.28.1

* Allow setting the z-value for the popup.

# 0.28.0

** upgrade to ratatui 0.29 **

* feature: all values of PopupCore can be public.
* feature: add support for rat-reloc to change position after rendering.
* feature: enable StatefulWidgetRef for PopupCore.
* feat: provide usable fallbacks when no styles are set.

# 0.27.2

* feature: add Placement to styles.
* feature: add get_block_padding() and inner()

# 0.27.1

* fix: when using AboveOrBelow/BelowOrAbove the offset must be mirrored.
* upgrade: rat-scrolled

# 0.27.0

* break: split current Placement into Placement and PopupConstraint.  
  The first contains only the flags. Can be used by other widgets now.

* feature: Add new Placement variants: Left,Right,Above,Below - just synonyms.
  AboveOrBelow, BelowOrAbove: Switch places depending on available space.

* feature: add PopupStyle
* feature: add PopupCore::xxx_opt()
* feature: add Scroll support for PopupCore.

# 0.26.0

* break: final renames in rat-focus.

# 0.25.0

* feat: Initial release.

Took some inspiration from PopupMenu and generalized this.
Reimplemented PopupMenu with PopupCore afterwards . 
