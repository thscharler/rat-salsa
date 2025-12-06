# 2.0.0

ratatui 0.30

# 1.4.0

* feature: add relocate_popup() and relocate_popup_hidden()
  widgets need to differentiate regular areas and popup areas,
  as they are not rendered at the same time.

# 1.3.0

* feature: add relocate_hidden to RelocatableState trait.

# 1.2.1

* docs: updates

# 1.2.0

* feature: impl RelocatableState for `()`.

# 1.1.2

* feature: impl RelocatableState for [Rect] too.

# 1.1.1

* feature: impl RelocatableState for Rect. This allows impl_relocatable_state!
  to be used with plain Rect members.

# 1.1.0

* add impl_relocatable_state! for quick impl

# 1.0.1

* moved all rat-crates to one repo

# 1.0.0

Seems good enough to stabilize.

# 0.2.0

** upgrade to ratatui 0.29 **

* add relocate_dark_offset(). calculates the extra offset induced
  by clipping. some widgets can use this.

# 0.1.0

Initial version. Moved from rat-widget.