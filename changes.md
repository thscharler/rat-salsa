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
