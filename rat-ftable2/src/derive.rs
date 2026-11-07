/// Implements forward calls to core for a Table widget.
///
/// __Only useful if you want to implement a new Table widget__
macro_rules! derive_table_widget {
    () => {
        /// If your table uses varying row-heights for each row
        /// all page calculations will fail (somewhat).
        /// If you enable this flag, the row-heights for
        /// each row will be stored in the state to allow
        /// accurate page-calculations.
        pub fn varying_heights(mut self, varying: bool) -> Self {
            self.core = self.core.varying_heights(varying);
            self
        }

        /// Set the table-header.
        #[inline]
        pub fn header<I>(mut self, header: I) -> Self
        where
            I: IntoIterator,
            I::Item: Into<ratatui_core::text::Text<'a>>,
        {
            self.core = self.core.header(header);
            self
        }

        /// Set the table-footer.
        #[inline]
        pub fn footer<I>(mut self, footer: I) -> Self
        where
            I: IntoIterator,
            I::Item: Into<ratatui_core::text::Text<'a>>,
        {
            self.core = self.core.footer(footer);
            self
        }

        /// Column widths as Constraints.
        ///
        /// The table-width that is used for these constraints
        /// is influenced by [layout_width][Self::layout_width].
        #[inline]
        pub fn widths<I>(mut self, widths: I) -> Self
        where
            I: IntoIterator,
            I::Item: Into<ratatui_core::layout::Constraint>,
        {
            self.core = self.core.widths(widths);
            self
        }

        /// Flex-value for layout calculation.
        #[inline]
        pub fn flex(mut self, flex: ratatui_core::layout::Flex) -> Self {
            self.core = self.core.flex(flex);
            self
        }

        /// Spacing between columns.
        #[inline]
        pub fn column_spacing(mut self, spacing: u16) -> Self {
            self.core = self.core.column_spacing(spacing);
            self
        }

        /// Set the width used for the table-layout.
        ///
        /// If the width is wider than the area, horizontal scrolling
        /// is used.
        #[inline]
        pub fn layout_width(mut self, width: $crate::LayoutWidth) -> Self {
            self.core = self.core.layout_width(width);
            self
        }

        /// Draws a block around the table widget.
        #[inline]
        pub fn block(mut self, block: ratatui_widgets::block::Block<'a>) -> Self {
            self.core = self.core.block(block);
            self
        }

        /// Sets the border-style for the Block, if any.
        #[inline]
        pub fn border_style(mut self, style: ratatui_core::style::Style) -> Self {
            self.core = self.core.border_style(style);
            self
        }

        /// Sets the title-style for the Block, if any.
        #[inline]
        pub fn title_style(mut self, style: ratatui_core::style::Style) -> Self {
            self.core = self.core.title_style(style);
            self
        }

        /// Scrollbars
        #[inline]
        pub fn scroll(mut self, scroll: rat_scrolled::Scroll<'a>) -> Self {
            self.core = self.core.scroll(scroll);
            self
        }

        /// Scrollbars
        #[inline]
        pub fn hscroll(mut self, scroll: rat_scrolled::Scroll<'a>) -> Self {
            self.core = self.core.hscroll(scroll);
            self
        }

        /// Scrollbars
        #[inline]
        pub fn vscroll(mut self, scroll: rat_scrolled::Scroll<'a>) -> Self {
            self.core = self.core.vscroll(scroll);
            self
        }

        /// Set all styles as a bundle.
        #[inline]
        pub fn styles(mut self, styles: $crate::style::TableStyle) -> Self {
            self.core = self.core.styles(styles);
            self
        }

        /// Base style for the table.
        #[inline]
        pub fn style(mut self, style: ratatui_core::style::Style) -> Self {
            self.core = self.core.style(style);
            self
        }

        /// Base style for the table.
        #[inline]
        pub fn header_style(mut self, style: Option<ratatui_core::style::Style>) -> Self {
            self.core = self.core.header_style(style);
            self
        }

        /// Base style for the table.
        #[inline]
        pub fn footer_style(mut self, style: Option<ratatui_core::style::Style>) -> Self {
            self.core = self.core.footer_style(style);
            self
        }

        /// Style for a selected row. The chosen selection must support
        /// row-selection for this to take effect.
        #[inline]
        pub fn select_row_style(
            mut self,
            select_style: Option<ratatui_core::style::Style>,
        ) -> Self {
            self.core = self.core.select_row_style(select_style);
            self
        }

        /// Add the focus-style to the row-style if the table is focused.
        #[inline]
        pub fn show_row_focus(mut self, show: bool) -> Self {
            self.core = self.core.show_row_focus(show);
            self
        }

        /// Style for a selected column. The chosen selection must support
        /// column-selection for this to take effect.
        #[inline]
        pub fn select_column_style(
            mut self,
            select_style: Option<ratatui_core::style::Style>,
        ) -> Self {
            self.core = self.core.select_column_style(select_style);
            self
        }

        /// Add the focus-style to the column-style if the table is focused.
        #[inline]
        pub fn show_column_focus(mut self, show: bool) -> Self {
            self.core = self.core.show_column_focus(show);
            self
        }

        /// Style for a selected cell. The chosen selection must support
        /// cell-selection for this to take effect.
        #[inline]
        pub fn select_cell_style(
            mut self,
            select_style: Option<ratatui_core::style::Style>,
        ) -> Self {
            self.core = self.core.select_cell_style(select_style);
            self
        }

        /// Add the focus-style to the cell-style if the table is focused.
        #[inline]
        pub fn show_cell_focus(mut self, show: bool) -> Self {
            self.core = self.core.show_cell_focus(show);
            self
        }

        /// Style for a selected header cell. The chosen selection must
        /// support column-selection for this to take effect.
        #[inline]
        pub fn select_header_style(
            mut self,
            select_style: Option<ratatui_core::style::Style>,
        ) -> Self {
            self.core = self.core.select_header_style(select_style);
            self
        }

        /// Add the focus-style to the header-style if the table is focused.
        #[inline]
        pub fn show_header_focus(mut self, show: bool) -> Self {
            self.core = self.core.show_header_focus(show);
            self
        }

        /// Style for a selected footer cell. The chosen selection must
        /// support column-selection for this to take effect.
        #[inline]
        pub fn select_footer_style(
            mut self,
            select_style: Option<ratatui_core::style::Style>,
        ) -> Self {
            self.core = self.core.select_footer_style(select_style);
            self
        }

        /// Add the footer-style to the table-style if the table is focused.
        #[inline]
        pub fn show_footer_focus(mut self, show: bool) -> Self {
            self.core = self.core.show_footer_focus(show);
            self
        }

        /// This style will be patched onto the selection to indicate that
        /// the widget has the input focus.
        ///
        /// The selection must support some kind of selection for this to
        /// be effective.
        #[inline]
        pub fn focus_style(mut self, focus_style: Option<ratatui_core::style::Style>) -> Self {
            self.core = self.core.focus_style(focus_style);
            self
        }

        /// Inherent width.
        #[inline]
        pub fn width(&self) -> u16 {
            self.core.width()
        }

        /// Inherent height.
        #[inline]
        pub fn height(&self) -> u16 {
            self.core.height()
        }
    };
    (T) => {
        /// Calculate the row-height from the data.
        #[inline]
        pub fn row_height(mut self, row_height_fn: fn(T, &$crate::RenderContext) -> u16) -> Self {
            self.core = self.core.row_height(row_height_fn);
            self
        }

        /// Render the full row-area.
        /// The closure returns true if individual cells should be rendered.
        #[inline]
        pub fn render_row(
            mut self,
            render_row_fn: impl FnMut(T, Rect, &mut Buffer, &$crate::RenderContext) -> bool + 'a,
        ) -> Self {
            self.core = self.core.render_row(render_row_fn);
            self
        }

        /// Render the table cells.
        #[inline]
        pub fn render_cell(
            mut self,
            column: usize,
            render_cell_fn: impl FnMut(T, Rect, &mut Buffer, &$crate::RenderContext) + 'a,
        ) -> Self {
            self.core = self.core.render_cell(column, render_cell_fn);
            self
        }
    };
    (REF_T) => {
        /// Calculate the row-height from the data.
        #[inline]
        pub fn row_height(mut self, row_height_fn: fn(&T, &$crate::RenderContext) -> u16) -> Self {
            self.core = self.core.row_height(row_height_fn);
            self
        }

        /// Render the full row-area.
        /// The closure returns true if individual cells should be rendered.
        #[inline]
        pub fn render_row(
            mut self,
            render_row_fn: impl FnMut(&T, Rect, &mut Buffer, &$crate::RenderContext) -> bool + 'a,
        ) -> Self {
            self.core = self.core.render_row(render_row_fn);
            self
        }

        /// Render the table cells.
        #[inline]
        pub fn render_cell(
            mut self,
            column: usize,
            render_cell_fn: impl FnMut(&T, Rect, &mut Buffer, &$crate::RenderContext) + 'a,
        ) -> Self {
            self.core = self.core.render_cell(column, render_cell_fn);
            self
        }
    };
}

/// Implements forward calls to core for a Table widget state struct.
/// The functions come in groups. See [TableState][crate::TableState]
/// for usage.
///
/// __Only useful if you want to implement a new Table widget__
macro_rules! derive_table_widget_state {
    (BASE) => {
        /// Number of rows.
        #[inline]
        pub fn rows(&self) -> usize {
            self.core.rows()
        }

        /// Number of columns.
        #[inline]
        pub fn columns(&self) -> usize {
            self.core.columns()
        }
    };
    (AREAS) => {
        /// Returns the whole row-area and the cell-areas for the
        /// given row, if it is visible.
        ///
        /// Attention: These areas might be 0-length if the column is scrolled
        /// beyond the table-area.
        #[inline]
        pub fn row_cells(&self, row: usize) -> Option<(Rect, Vec<Rect>)> {
            self.core.row_cells(row)
        }

        /// Cell at given position.
        #[inline]
        pub fn cell_at_clicked(&self, pos: (u16, u16)) -> Option<(usize, usize)> {
            self.core.cell_at_clicked(pos)
        }

        /// Column at given position.
        #[inline]
        pub fn column_at_clicked(&self, pos: (u16, u16)) -> Option<usize> {
            self.core.column_at_clicked(pos)
        }

        /// Row at given position.
        #[inline]
        pub fn row_at_clicked(&self, pos: (u16, u16)) -> Option<usize> {
            self.core.row_at_clicked(pos)
        }

        /// Cell when dragging. Position can be outside the table area.
        /// See [row_at_drag](Self::row_at_drag), [col_at_drag](Self::column_at_drag)
        #[inline]
        pub fn cell_at_drag(&self, pos: (u16, u16)) -> (usize, usize) {
            self.core.cell_at_drag(pos)
        }

        /// Row when dragging. Position can be outside the table area.
        /// If the position is above the table-area this returns offset - #rows.
        /// If the position is below the table-area this returns offset + page_len + #rows.
        ///
        /// This doesn't account for the row-height of the actual rows outside
        /// the table area, just assumes '1'.
        #[inline]
        pub fn row_at_drag(&self, pos: (u16, u16)) -> usize {
            self.core.row_at_drag(pos)
        }

        /// Column when dragging. Position can be outside the table area.
        /// If the position is left of the table area this returns offset - 1.
        /// If the position is right of the table area this returns offset + page_width + 1.
        #[inline]
        pub fn column_at_drag(&self, pos: (u16, u16)) -> usize {
            self.core.column_at_drag(pos)
        }
    };
    (OFFSET) => {
        /// Sets both offsets to 0.
        #[inline]
        pub fn clear_offset(&mut self) {
            self.core.clear_offset()
        }

        /// Maximum offset that is accessible with scrolling.
        ///
        /// This is shorter than the length by whatever fills the last page.
        /// This is the base for the scrollbar content_length.
        #[inline]
        pub fn row_max_offset(&self) -> usize {
            self.core.row_max_offset()
        }

        /// Current vertical offset.
        #[inline]
        pub fn row_offset(&self) -> usize {
            self.core.row_offset()
        }

        /// Change the vertical offset.
        /// The offset may be beyond the number of rows.
        ///
        /// The widget returns true if the offset changed at all.
        #[inline]
        pub fn set_row_offset(&mut self, offset: usize) -> bool {
            self.core.set_row_offset(offset)
        }

        /// Vertical page-size at the current offset.
        #[inline]
        pub fn page_len(&self) -> usize {
            self.core.page_len()
        }

        /// Suggested scroll per scroll-event.
        #[inline]
        pub fn row_scroll_by(&self) -> usize {
            self.core.row_scroll_by()
        }

        /// Maximum offset that is accessible with scrolling.
        ///
        /// This is shorter than the length of the content by whatever fills the last page.
        /// This is the base for the scrollbar content_length.
        #[inline]
        pub fn x_max_offset(&self) -> usize {
            self.core.x_max_offset()
        }

        /// Current horizontal offset.
        #[inline]
        pub fn x_offset(&self) -> usize {
            self.core.x_offset()
        }

        /// Change the horizontal offset.
        /// The offset may be beyond the table-width.
        ///
        /// Returns true if the offset changed at all.
        #[inline]
        pub fn set_x_offset(&mut self, offset: usize) -> bool {
            self.core.set_x_offset(offset)
        }

        /// Horizontal page-size at the current offset.
        #[inline]
        pub fn page_width(&self) -> usize {
            self.core.page_width()
        }

        /// Suggested scroll per scroll-event.
        #[inline]
        pub fn x_scroll_by(&self) -> usize {
            self.core.x_scroll_by()
        }

        /// Ensures that the selected item is visible.
        /// Caveat: This doesn't work nicely if you have varying row-heights.
        /// Caveat: Number of rows needs to be correct.
        #[inline]
        pub fn scroll_to_selected(&mut self) -> bool {
            self.core.scroll_to_selected()
        }

        /// Ensures that the given row is visible.
        /// Caveat: This doesn't work nicely if you have varying row-heights.
        /// Caveat: Number of rows needs to be correct.
        #[inline]
        pub fn scroll_to_row(&mut self, pos: usize) -> bool {
            self.core.scroll_to_row(pos)
        }

        /// Ensures that the given column is completely visible.
        #[inline]
        pub fn scroll_to_col(&mut self, pos: usize) -> bool {
            self.core.scroll_to_col(pos)
        }

        /// Ensures that the given position is visible.
        #[inline]
        pub fn scroll_to_x(&mut self, pos: usize) -> bool {
            self.core.scroll_to_x(pos)
        }

        /// Reduce the row-offset by n.
        #[inline]
        pub fn scroll_up(&mut self, n: usize) -> bool {
            self.core.scroll_up(n)
        }

        /// Increase the row-offset by n.
        #[inline]
        pub fn scroll_down(&mut self, n: usize) -> bool {
            self.core.scroll_down(n)
        }

        /// Reduce the col-offset by n.
        #[inline]
        pub fn scroll_left(&mut self, n: usize) -> bool {
            self.core.scroll_left(n)
        }

        /// Increase the col-offset by n.
        #[inline]
        pub fn scroll_right(&mut self, n: usize) -> bool {
            self.core.scroll_right(n)
        }
    };
    (SELECTION) => {
        /// Clear the selection.
        #[inline]
        pub fn clear_selection(&mut self) {
            self.core.clear_selection()
        }
    };
    (ROWSELECTION) => {
        /// When scrolling the table, change the selection instead of the offset.
        #[inline]
        pub fn set_scroll_selection(&mut self, scroll: bool) {
            self.core.set_scroll_selection(scroll)
        }

        /// Anything selected?
        #[inline]
        pub fn has_selection(&mut self) -> bool {
            self.core.has_selection()
        }

        /// Selected row.
        /// The selected row is not constrained by the row-count.
        #[inline]
        pub fn selected(&self) -> Option<usize> {
            self.core.selected()
        }

        /// Return the selected row and ensure it is in the
        /// range `0..rows`.
        #[inline]
        pub fn selected_checked(&self) -> Option<usize> {
            self.core.selected_checked()
        }

        /// Select the row.
        /// The selection is not constrained by the row-count.
        #[inline]
        pub fn select(&mut self, row: Option<usize>) -> bool {
            self.core.select(row)
        }

        /// Set the selection to None and set the offset to 0
        #[inline]
        pub fn move_deselect(&mut self) -> bool {
            self.core.move_deselect()
        }

        /// Move the selection to the given row.
        /// Ensures the row is visible afterward.
        #[inline]
        pub fn move_to(&mut self, row: usize) -> bool {
            self.core.move_to(row)
        }

        /// Move the selection up n rows.
        /// Ensures the row is visible afterward.
        #[inline]
        pub fn move_up(&mut self, n: usize) -> bool {
            self.core.move_up(n)
        }

        /// Move the selection down n rows.
        /// Ensures the row is visible afterward.
        #[inline]
        pub fn move_down(&mut self, n: usize) -> bool {
            self.core.move_down(n)
        }
    };
    (ROWSETSELECTION) => {
        /// Anything selected?
        #[inline]
        pub fn has_selection(&mut self) -> bool {
            self.core.has_selection()
        }

        /// Selected rows.
        #[inline]
        pub fn selected(&self) -> HashSet<usize> {
            self.core.selected()
        }

        /// Change the lead-selection. Limits the value to the number of rows.
        /// If extend is false the current selection is cleared and both lead and
        /// anchor are set to the given value.
        /// If extend is true, the anchor is kept where it is and lead is changed.
        /// Everything in the range `anchor..lead` is selected. It doesn't matter
        /// if anchor < lead.
        #[inline]
        pub fn set_lead(&mut self, row: Option<usize>, extend: bool) -> bool {
            self.core.set_lead(row, extend)
        }

        /// Current lead.
        #[inline]
        pub fn lead(&self) -> Option<usize> {
            self.core.lead()
        }

        /// Current anchor.
        #[inline]
        pub fn anchor(&self) -> Option<usize> {
            self.core.anchor()
        }

        /// Retire the current anchor/lead selection to the set of selected rows.
        /// Resets lead and anchor and starts a new selection round.
        #[inline]
        pub fn retire_selection(&mut self) {
            self.core.retire_selection()
        }

        /// Add to selection. Only works for retired selections, not for the
        /// active anchor-lead range.
        ///
        /// To be sure call [retire_selection][Self::retire_selection] first.
        #[inline]
        pub fn add_selected(&mut self, idx: usize) {
            self.core.add_selected(idx)
        }

        /// Remove from selection. Only works for retired selections, not for the
        /// active anchor-lead range.
        ///
        /// To be sure call [retire_selection][Self::retire_selection] first.
        #[inline]
        pub fn remove_selected(&mut self, idx: usize) {
            self.core.remove_selected(idx)
        }

        /// Move the selection to the given row.
        /// Ensures the row is visible afterward.
        #[inline]
        pub fn move_to(&mut self, row: usize, extend: bool) -> bool {
            self.core.move_to(row, extend)
        }

        /// Move the selection up n rows.
        /// Ensures the row is visible afterward.
        #[inline]
        pub fn move_up(&mut self, n: usize, extend: bool) -> bool {
            self.core.move_up(n, extend)
        }

        /// Move the selection down n rows.
        /// Ensures the row is visible afterwards.
        #[inline]
        pub fn move_down(&mut self, n: usize, extend: bool) -> bool {
            self.core.move_down(n, extend)
        }
    };
    (CELLSELECTION) => {
        #[inline]
        pub fn has_selection(&mut self) -> bool {
            self.core.has_selection()
        }

        /// Selected cell.
        #[inline]
        pub fn selected(&self) -> Option<(usize, usize)> {
            self.core.selected()
        }

        /// Select a cell.
        #[inline]
        pub fn select_cell(&mut self, select: Option<(usize, usize)>) -> bool {
            self.core.select_cell(select)
        }

        /// Select a row. Column stays the same.
        #[inline]
        pub fn select_row(&mut self, row: Option<usize>) -> bool {
            self.core.select_row(row)
        }

        /// Select a column, row stays the same.
        #[inline]
        pub fn select_column(&mut self, column: Option<usize>) -> bool {
            self.core.select_column(column)
        }

        /// Select a cell, limit to maximum.
        #[inline]
        pub fn move_to(&mut self, select: (usize, usize)) -> bool {
            self.core.move_to(select)
        }

        /// Select a row, limit to maximum.
        #[inline]
        pub fn move_to_row(&mut self, row: usize) -> bool {
            self.core.move_to_row(row)
        }

        /// Select a cell, clamp between 0 and maximum.
        #[inline]
        pub fn move_to_col(&mut self, col: usize) -> bool {
            self.core.move_to_col(col)
        }

        /// Move the selection up n rows.
        /// Ensures the row is visible afterwards.
        #[inline]
        pub fn move_up(&mut self, n: usize) -> bool {
            self.core.move_up(n)
        }

        /// Move the selection down n rows.
        /// Ensures the row is visible afterwards.
        #[inline]
        pub fn move_down(&mut self, n: usize) -> bool {
            self.core.move_down(n)
        }

        /// Move the selection left n columns.
        /// Ensures the row is visible afterwards.
        #[inline]
        pub fn move_left(&mut self, n: usize) -> bool {
            self.core.move_left(n)
        }

        /// Move the selection right n columns.
        /// Ensures the row is visible afterwards.
        #[inline]
        pub fn move_right(&mut self, n: usize) -> bool {
            self.core.move_right(n)
        }
    };
    (ITEMS) => {
        /// Update the state to match adding items *during* event-handling.
        /// A lot of functions use the number of rows, so when you change the
        /// underlying data you might want to immediately reflect this in the
        /// table-state.
        ///
        /// This corrects the number of rows, offset and selection.
        #[inline]
        pub fn items_added(&mut self, pos: usize, n: usize) {
            self.core.items_added(pos, n)
        }

        /// Update the state to match removing items *during* event-handling.
        /// A lot of functions use the number of rows, so when you change the
        /// underlying data you might want to immediately reflect this in the
        /// table-state.
        ///
        /// This corrects the number of rows, offset and selection.
        #[inline]
        pub fn items_removed(&mut self, pos: usize, n: usize) {
            self.core.items_removed(pos, n)
        }

        /// Update the number of rows *during* event-handling.
        /// A lot of functions use the number of rows, so when you change the
        /// underlying data you might want to immediately reflect this in the
        /// table-state.
        ///
        /// This corrects the number of rows, offset and selection.
        #[inline]
        pub fn rows_changed(&mut self, rows: usize) {
            self.core.rows_changed(rows)
        }
    };
}

pub(crate) use derive_table_widget;
pub(crate) use derive_table_widget_state;
