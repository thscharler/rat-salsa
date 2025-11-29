#[macro_export]
macro_rules! derive_text_widget {
    ($state:ty) => {
        derive_text_widget!(BASE $state);
    };
    (BASE $state:ty) => {
        impl <'a> $state {
            /// Set the combined style.
            #[inline]
            pub fn styles(mut self, style: TextStyle) -> Self {
                self.widget = self.widget.styles(style);
                self
            }

            /// Base text style.
            #[inline]
            pub fn style(mut self, style: impl Into<Style>) -> Self {
                self.widget = self.widget.style(style);
                self
            }

            /// Style when focused.
            #[inline]
            pub fn focus_style(mut self, style: impl Into<Style>) -> Self {
                self.widget = self.widget.focus_style(style);
                self
            }

            /// Style for selection
            #[inline]
            pub fn select_style(mut self, style: impl Into<Style>) -> Self {
                self.widget = self.widget.select_style(style);
                self
            }

            /// Style for the invalid indicator.
            #[inline]
            pub fn invalid_style(mut self, style: impl Into<Style>) -> Self {
                self.widget = self.widget.invalid_style(style);
                self
            }

            /// Block
            #[inline]
            pub fn block(mut self, block: Block<'a>) -> Self {
                self.widget = self.widget.block(block);
                self
            }

            /// Focus behaviour
            #[inline]
            pub fn on_focus_gained(mut self, of: TextFocusGained) -> Self {
                self.widget = self.widget.on_focus_gained(of);
                self
            }

            /// Focus behaviour
            #[inline]
            pub fn on_focus_lost(mut self, of: TextFocusLost) -> Self {
                self.widget = self.widget.on_focus_lost(of);
                self
            }

            /// `Tab` behaviour
            #[inline]
            pub fn on_tab(mut self, of: TextTab) -> Self {
                self.widget = self.widget.on_tab(of);
                self
            }
        }
    }
}

#[macro_export]
macro_rules! derive_text_widget_state {
    ($state:ty) => {
        derive_text_widget_state!(BASE $state);
        derive_text_widget_state!(CLIPBOARD $state);
        derive_text_widget_state!(UNDO $state);
        derive_text_widget_state!(STYLE $state);
        derive_text_widget_state!(OFFSET $state);
        derive_text_widget_state!(EDIT $state);
        derive_text_widget_state!(MOVE $state);
        derive_text_widget_state!(FOCUS $state);
        derive_text_widget_state!(SCREENCURSOR $state);
        derive_text_widget_state!(RELOCATE $state);
    };
    (BASE $state:ty) => {
        impl $state {
            /// Empty
            #[inline]
            pub fn is_empty(&self) -> bool {
                self.widget.is_empty()
            }

            /// Length in grapheme count.
            #[inline]
            pub fn len(&self) -> $crate::upos_type {
                self.widget.len()
            }

            /// Length as grapheme count.
            #[inline]
            pub fn line_width(&self) -> $crate::upos_type {
                self.widget.line_width()
            }

            /// Renders the widget in invalid style.
            #[inline]
            pub fn set_invalid(&mut self, invalid: bool) {
                self.widget.invalid = invalid;
            }

            /// Renders the widget in invalid style.
            #[inline]
            pub fn get_invalid(&self) -> bool {
                self.widget.invalid
            }

            /// The next edit operation will overwrite the current content
            /// instead of adding text. Any move operations will cancel
            /// this overwrite.
            #[inline]
            pub fn set_overwrite(&mut self, overwrite: bool) {
                self.widget.set_overwrite(overwrite);
            }

            /// Will the next edit operation overwrite the content?
            #[inline]
            pub fn overwrite(&self) -> bool {
                self.widget.overwrite()
            }
        }
    };
    (CLIPBOARD $state:ty) => {
        impl $state {
            /// Clipboard used.
            /// Default is to use the [global_clipboard](crate::clipboard::global_clipboard).
            #[inline]
            pub fn set_clipboard(&mut self, clip: Option<impl $crate::clipboard::Clipboard + 'static>) {
                self.widget.set_clipboard(clip);
            }

            /// Clipboard used.
            /// Default is to use the [global_clipboard](crate::clipboard::global_clipboard).
            #[inline]
            pub fn clipboard(&self) -> Option<&dyn $crate::clipboard::Clipboard> {
                self.widget.clipboard()
            }

            /// Copy to clipboard
            #[inline]
            pub fn copy_to_clip(&mut self) -> bool {
                self.widget.copy_to_clip()
            }

            /// Cut to clipboard
            #[inline]
            pub fn cut_to_clip(&mut self) -> bool {
                self.widget.cut_to_clip()
            }

            /// Paste from clipboard.
            #[inline]
            pub fn paste_from_clip(&mut self) -> bool {
                self.widget.paste_from_clip()
            }
        }
    };
    (UNDO $state:ty) => {
        impl $state {
            /// Set undo buffer.
            #[inline]
            pub fn set_undo_buffer(&mut self, undo: Option<impl $crate::undo_buffer::UndoBuffer + 'static>) {
                self.widget.set_undo_buffer(undo);
            }

            /// Undo
            #[inline]
            pub fn undo_buffer(&self) -> Option<&dyn $crate::undo_buffer::UndoBuffer> {
                self.widget.undo_buffer()
            }

            /// Undo
            #[inline]
            pub fn undo_buffer_mut(&mut self) -> Option<&mut dyn $crate::undo_buffer::UndoBuffer> {
                self.widget.undo_buffer_mut()
            }

            /// Get all recent replay recordings.
            #[inline]
            pub fn recent_replay_log(&mut self) -> Vec<$crate::undo_buffer::UndoEntry> {
                self.widget.recent_replay_log()
            }

            /// Apply the replay recording.
            #[inline]
            pub fn replay_log(&mut self, replay: &[$crate::undo_buffer::UndoEntry]) {
                self.widget.replay_log(replay)
            }

            /// Undo operation
            #[inline]
            pub fn undo(&mut self) -> bool {
                self.widget.undo()
            }

            /// Redo operation
            #[inline]
            pub fn redo(&mut self) -> bool {
                self.widget.redo()
            }
        }
    };
    (STYLE $state:ty) => {
        impl $state {
            /// Set and replace all styles.
            #[inline]
            pub fn set_styles(&mut self, styles: Vec<(std::ops::Range<usize>, usize)>) {
                self.widget.set_styles(styles);
            }

            /// Add a style for a byte-range.
            #[inline]
            pub fn add_style(&mut self, range: std::ops::Range<usize>, style: usize) {
                self.widget.add_style(range, style);
            }

            /// Add a style for a `Range<upos_type>` .
            /// The style-nr refers to one of the styles set with the widget.
            #[inline]
            pub fn add_range_style(
                &mut self,
                range: std::ops::Range<$crate::upos_type>,
                style: usize,
            ) -> Result<(), $crate::TextError> {
                self.widget.add_range_style(range, style)
            }

            /// Remove the exact TextRange and style.
            #[inline]
            pub fn remove_style(&mut self, range: std::ops::Range<usize>, style: usize) {
                self.widget.remove_style(range, style);
            }

            /// Remove the exact `Range<upos_type>` and style.
            #[inline]
            pub fn remove_range_style(
                &mut self,
                range: std::ops::Range<$crate::upos_type>,
                style: usize,
            ) -> Result<(), $crate::TextError> {
                self.widget.remove_range_style(range, style)
            }

            /// Find all styles that touch the given range.
            pub fn styles_in(&self, range: std::ops::Range<usize>, buf: &mut Vec<(std::ops::Range<usize>, usize)>) {
                self.widget.styles_in(range, buf)
            }

            /// All styles active at the given position.
            #[inline]
            pub fn styles_at(&self, byte_pos: usize, buf: &mut Vec<(std::ops::Range<usize>, usize)>) {
                self.widget.styles_at(byte_pos, buf)
            }

            /// Check if the given style applies at the position and
            /// return the complete range for the style.
            #[inline]
            pub fn style_match(&self, byte_pos: usize, style: usize) -> Option<std::ops::Range<usize>> {
                self.widget.styles_at_match(byte_pos, style)
            }

            /// List of all styles.
            #[inline]
            pub fn styles(&self) -> Option<impl Iterator<Item = (std::ops::Range<usize>, usize)> + '_> {
                self.widget.styles()
            }
        }
    };
    (OFFSET $state:ty) => {
        impl $state {
            /// Offset shown.
            #[inline]
            pub fn offset(&self) -> $crate::upos_type {
                self.widget.offset()
            }

            /// Offset shown. This is corrected if the cursor wouldn't be visible.
            #[inline]
            pub fn set_offset(&mut self, offset: $crate::upos_type) {
                self.widget.set_offset(offset)
            }

            /// Cursor position
            #[inline]
            pub fn cursor(&self) -> $crate::upos_type {
                self.widget.cursor()
            }

            /// Set the cursor position, reset selection.
            #[inline]
            pub fn set_cursor(&mut self, cursor: $crate::upos_type, extend_selection: bool) -> bool {
                self.widget.set_cursor(cursor, extend_selection)
            }

            /// Place cursor at some sensible position according to the mask.
            #[inline]
            pub fn set_default_cursor(&mut self) {
                self.widget.set_default_cursor()
            }

            /// Selection anchor.
            #[inline]
            pub fn anchor(&self) -> $crate::upos_type {
                self.widget.anchor()
            }

            /// Selection
            #[inline]
            pub fn has_selection(&self) -> bool {
                self.widget.has_selection()
            }

            /// Selection
            #[inline]
            pub fn selection(&self) -> std::ops::Range<$crate::upos_type> {
                self.widget.selection()
            }

            /// Selection
            #[inline]
            pub fn set_selection(&mut self, anchor: $crate::upos_type, cursor: $crate::upos_type) -> bool {
                self.widget.set_selection(anchor, cursor)
            }

            /// Select all text.
            #[inline]
            pub fn select_all(&mut self) {
                self.widget.select_all();
            }

            /// Selection
            #[inline]
            pub fn selected_text(&self) -> &str {
                self.widget.selected_text()
            }
        }
    };
    (EDIT $state:ty) => {
        impl $state {
            /// Insert a char at the current position.
            #[inline]
            pub fn insert_char(&mut self, c: char) -> bool {
                self.widget.insert_char(c)
            }

            /// Remove the selected range. The text will be replaced with the default value
            /// as defined by the mask.
            #[inline]
            pub fn delete_range(&mut self, range: std::ops::Range<$crate::upos_type>) -> bool {
                self.widget.delete_range(range)
            }

            /// Remove the selected range. The text will be replaced with the default value
            /// as defined by the mask.
            #[inline]
            pub fn try_delete_range(&mut self, range: std::ops::Range<$crate::upos_type>) -> Result<bool, $crate::TextError> {
                self.widget.try_delete_range(range)
            }

            // Delete the char after the cursor.
            #[inline]
            pub fn delete_next_char(&mut self) -> bool {
                self.widget.delete_next_char()
            }

            /// Delete the char before the cursor.
            #[inline]
            pub fn delete_prev_char(&mut self) -> bool {
                self.widget.delete_prev_char()
            }
        }
    };
    (MOVE $state:ty) => {
        impl $state {
            /// Move to the next char.
            #[inline]
            pub fn move_right(&mut self, extend_selection: bool) -> bool {
                self.widget.move_right(extend_selection)
            }

            /// Move to the previous char.
            #[inline]
            pub fn move_left(&mut self, extend_selection: bool) -> bool {
                self.widget.move_left(extend_selection)
            }

            /// Start of line
            #[inline]
            pub fn move_to_line_start(&mut self, extend_selection: bool) -> bool {
                self.widget.move_to_line_start(extend_selection)
            }

            /// End of line
            #[inline]
            pub fn move_to_line_end(&mut self, extend_selection: bool) -> bool {
                self.widget.move_to_line_end(extend_selection)
            }
        }
    };
    (FOCUS $state:ty) => {
        impl rat_focus::HasFocus for $state {
            fn build(&self, builder: &mut rat_focus::FocusBuilder) {
                builder.leaf_widget(self);
            }

            #[inline]
            fn focus(&self) -> rat_focus::FocusFlag {
                self.widget.focus.clone()
            }

            #[inline]
            fn area(&self) -> Rect {
                self.widget.area
            }

            #[inline]
            fn navigable(&self) -> rat_focus::Navigation {
                self.widget.navigable()
            }
        }
    };
    (SCREENCURSOR $state:ty) => {
        impl $crate::HasScreenCursor for $state {
            /// The current text cursor as an absolute screen position.
            #[inline]
            fn screen_cursor(&self) -> Option<(u16, u16)> {
                self.widget.screen_cursor()
            }
        }

        impl $state {
            /// Converts a grapheme based position to a screen position
            /// relative to the widget area.
            #[inline]
            pub fn col_to_screen(&self, pos: $crate::upos_type) -> Option<u16> {
                self.widget.col_to_screen(pos)
            }

            /// Converts from a widget relative screen coordinate to a grapheme index.
            /// x is the relative screen position.
            #[inline]
            pub fn screen_to_col(&self, scx: i16) -> $crate::upos_type {
                self.widget.screen_to_col(scx)
            }

            /// Set the cursor position from a screen position relative to the origin
            /// of the widget. This value can be negative, which selects a currently
            /// not visible position and scrolls to it.
            #[inline]
            pub fn set_screen_cursor(&mut self, cursor: i16, extend_selection: bool) -> bool {
                self.widget.set_screen_cursor(cursor, extend_selection)
            }
        }
    };
    (RELOCATE $state:ty) => {
        impl rat_reloc::RelocatableState for $state {
            fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
                self.area.relocate(shift, clip);
                self.inner.relocate(shift, clip);
                self.widget.relocate(shift, clip);
            }
        }
    };
}
