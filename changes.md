# 0.29.0

** upgrade to ratatui 0.29 **
** upgrade to unicode-width 0.2 **

* feat: make useable when no styles are set.
* fix: rendering of LineNumbers.
* feature: implement RelocatableState for TextInput, TextArea and MaskedInput.
  Those work correctly even when partially clipped.

# 0.28.0

* refactor: merge TextInputStyle and TextAreaStyle to TextStyle.

# 0.27.1

* update rat-scrolled

# 0.27.0

* break: names in xxStyle changed

# 0.26.0

break: final renames in rat-focus.

# 0.25.1

fix: update dependencies

# 0.25.0

Sync version for beta.

* fix: set a default format for number-input.

# 0.12.1

* feature: add auto_quote when inserting '"', '(', ...
* fix: replay didn't work with undo sequences

# 0.12.0

* Update changes in rat-scroll.

* feature: add begin_undo_seq() + end_undo_seq() to combine
  multiple changes into a single undo/redo. Quite useful for
  delete/insert combinations.

* feature: trait `HasScreenCursor` for general cursor display.

* feature: add styles_in(range)

* feature: add str_slice_byte(byte-range)

* feature: selection + tab now indents the selection.

* refactor: styles_at() now returns range+style

* fix: inserting \r \n as single characters panicked. rewrite
  glyph combination code.

* fix: bytes_to_range failed when the position was equal to
  len().

* fix: undo-count limits the number of changes. This counts
  grouped changes as 1 change now.

* fix: When the scrollbar/border don't paint the complete area of
  the widget it looks a bit broken. fill the complete area with
  the default-style.

* fix: invalidation of the style cache sometimes was broken.

# 0.11.0

* feature: add LineNumbers widget

* fix: text_input_mask cleanup section navigation.

# 0.10.0

Moved the text-widgets from rat-widgets to this crate. This was
not a simple migration, but a start from scratch with the goal to
use one backend for all text-widgets.

This introduces the TextStore trait which acts as backend for the
backend and does the text manipulation and mapping from graphemes
to bytes. There is a String based implementation which supports
only a single line of text and a rope based implementation for
the full textarea.

The api of the widgets stays more or less the same, but
everything is re-implemented based on text-store.
