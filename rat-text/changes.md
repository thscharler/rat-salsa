# 1.3.1

* fix: scrolling issue with wrapped text.
* refactor: remove some useless codepaths.
* docs: some updates

# 1.3.0

* fix: several small behaviour bugs with wrapped text.
* fix: cleaning up the GlyphIter2 code fixed a handful
  of bugs. not sure if they were ever hit in rl.
* fix: cleanup of the screen pos/text pos mapping functions.

* feature: when setting the cursor, the scroll_cursor_to_visible()
  logic differentiates between near and far moves. Near moves
  change the offset one line at a time. far moves place the
  cursor a bit off the middle of the rendered area.
  the limit is one page before/after the visible page.

* cargo: forward features cr_lines and unicode_lines from ropey.
  sync rat-text new-line handling with ropey features.

# 1.2.0

* break: remove HandleEvent with FocusTraversal. not useful.

* feature: allow setting TextWrap on the widget.
* fix: use focus_style for the TextArea too. styling has been
  synced across all text widgets.

# 1.1.0

* break: change move_col from upos_type to i16.
  this is a screen-position now.
* break: TextStore now uses an associated type for it's iterator and
  has a few other changes.

* feature: new glyph iteration.
    * Adds a word-breaking algorithm.
    * activate with text_area_state.set_text_wrap()
    * adds TextWrap::Shift for current behaviour, TextWrap::Hard for vi-style wrapping
      and TextWrap::Word for word-breaking.
    * Word-breaking supports soft-hyphen and zero-width-spaces in text.

* feature: TextAreaState::set_show_ctrl(true) shows all control-characters and introduced line-breaks.
* feature: TextAreaState::set_wrap_ctrl(true) shows all line-breaks and hidden characters
  as soft-hyphen and zero-width-space.
* feature: TextAreaState::set_focus_navigation() to change the default navigation.
  If set to regular Tab is used for navigation instead of editing.
* feature: TextArea::text_style_idx() added.

* feature: LineNumbers::with_textarea() can now take the row data
  from the textarea. This is very necessary when using text-wrapping and
  shows only the first occurrence of a line.

* add a caching layer. used for line-breaking and a few common but
  expensive functions (len_lines() and line_width()).
  Navigation functions now use this cache to speed up cursor
  positioning.
* speed up glyph iteration with SkipLine trait. This is implemented
  by the grapheme level iterators and allows to skip the rest of
  a text-line or to jump to a specific byte-index. This significantly
  speeds up rendering long lines too.

* deprecated old glyphs() function and made the new one only pub(crate).
* deprecated old screen to pos and pos to screen functions.
  they still work, but only if no text-wrapping is used.

* fix: word_end() at the end of a line gave a wrong offset.

# 1.0.5

* update dependencies
*
* fix #5: It was not possible to scroll the cursor to a sensible
  position before the first render.

  This also adds auto-scroll to set_cursor(), set_selection() and select_all()
  for TextArea, TextInput and MaskedInput.

# 1.0.4

* fix: set the default line-ending via compile time cfg.

# 1.0.3

* feature: TextArea: add text_style_map() as a HashMap instead of a Vec
* fix: TextArea: if the select_style has no fg or bg don't patch it onto
  the base style. This way Style::new().reversed() works nicely.

# 1.0.2

* fix: ensure that the select_style is always patched onto the
  base-style. This makes a select-style `Style::new().underlined()`
  work fine.

# 1.0.1

* feature: UndoBuffer: add open_undo() and open_redo().
  Those give back the remaining number of operations.
  Useful to mark a 'needs saving' flag.

# 1.0.0

... jump ...

# 0.30.4

* add TextRange::MAX

# 0.30.3

* fix: LineNumbers must render a background if there is no Block.

# 0.30.2

* feature: set_global_clipboard()

# 0.30.1

* feature: NumberInput::new_pattern() and NumberInput::new_loc_pattern() constructors.

# 0.30.0

* fix: reset cursor to default position with set_text()

* feature: add border_style to TextStyle. Sets the border_style
  for any pre-existing Block.
* feature: add on_focus_gained() and on_focus_lost() behaviour.
    - Fixed set of behaviours:
        - TextFocusGained::Overwrite - set the overwrite-flag.
          Any text-input overwrites all content, but if you use any
          navigation keys this flag is reset and changing the content
          is possible.
        - TextFocusGained::SelectAll - select all text.
        - TextFocusLost::Position0 - set the cursor to the default position,
          which is 0 for most widgets. MaskedInput may have a different default.
          This prevents clipped left text after an edit.
    - Behaviours can be set via TextStyle.
    - adds set_overwrite() to set this behaviour selectively.
* feature: MaskedInput: ',' and '.' are recognized as universal separator matches.
  If these characters are not allowed for the current field they match
  with the next separator whatever that separator is. This makes date-input
  with the num-pad only possible: '1' '.' '1' '.' '2025'
* feature: add global_clipboard() to set a application global clipboard.
    - used as default for all text-widgets.
    - enables copy&paste between all text widgets out of the box.

# 0.29.5

* update rat-focus

# 0.29.4

* moved all rat-crates to one repo

# 0.29.3

* feature: add TextInput::value() which auto-converts the text-field
  content to any target type if an Into conversion exists.
* feature: add NumberInput::value_opt() which returns None for an
  empty number field.

# 0.29.2

* fix: default pattern for number_input
* fix: remove select-all on focus-gained. not very useful, but looks ugly.
  todo: might add a flag for 'overwrite on focus-gained' which
  does the same without showing a selection.

# 0.29.1

* add doc-changes from #1 by nick42d.

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
