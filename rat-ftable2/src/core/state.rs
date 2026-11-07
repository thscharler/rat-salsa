use crate::_private::NonExhaustive;
use crate::event::{DoubleClickOutcome, TableOutcome};
use crate::selection::TableSelection;
use crate::selection::cell::CellSelection;
use crate::selection::none::NoSelection;
use crate::selection::row::RowSelection;
use crate::selection::rowset::RowSetSelection;
use rat_event::crossterm::event::Event;
use rat_event::util::MouseFlags;
use rat_event::{DoubleClick, HandleEvent, MouseOnly, Regular, ct_event};
use rat_focus::{FocusFlag, HasFocus};
use rat_reloc::{RelocatableState, relocate_areas};
use rat_scrolled::ScrollState;
use ratatui_core::layout::Rect;
use std::cmp::min;
use std::collections::HashSet;

/// State for the core table.
#[derive(Debug)]
pub struct TableCoreState<Selection = RowSelection> {
    /// Current focus state.
    /// __read+write__
    pub focus: FocusFlag,

    /// Total area.
    /// __read only__ Renewed with each render.
    pub area: Rect,
    /// Area inside the border and scrollbars
    /// __read only__ Renewed with each render.
    pub inner: Rect,

    /// Total header area.
    /// __read only__ Renewed with each render.
    pub header_area: Rect,
    /// Total table area.
    /// __read only__ Renewed with each render.
    pub table_area: Rect,
    /// Area per visible row. The first element is at row_offset.
    /// __read only__ Renewed with each render.
    pub row_areas: Vec<Rect>,
    /// If your table uses varying row-heights the height
    /// for each row is stored here.
    pub row_heights: Vec<u16>,
    /// If your table doesn't use varying row-heights, the
    /// row-height is stored here.
    pub row_height: u16,
    /// Area for each column plus the following spacer if any.
    /// Invisible columns have width 0, height is the height of the table_area.
    /// __read only__ Renewed with each render.
    pub column_areas: Vec<Rect>,
    /// Layout areas for each column plus the following spacer if any.
    /// Positions are 0-based, y and height are 0.
    /// __read only__ Renewed with each render.
    pub column_layout: Vec<Rect>,
    /// Total footer area.
    /// __read only__ Renewed with each render.
    pub footer_area: Rect,

    /// Row count.
    /// __read+write__ Renewed with each render.
    pub rows: usize,
    /// Column count.
    /// __read only__ Renewed with each render.
    pub columns: usize,

    /// Row scrolling data.
    /// __read+write__ max_offset set with each render.
    pub vscroll: ScrollState,
    /// Column scrolling data.
    /// __read+write__ max_offset set with each render.
    pub hscroll: ScrollState,

    /// Selection data.
    /// __read+write__ selection model.
    /// selection is not bound by actual rows.
    pub selection: Selection,

    /// Helper for mouse interactions.
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl<Selection: Clone> Clone for TableCoreState<Selection> {
    fn clone(&self) -> Self {
        Self {
            focus: self.focus.new_instance(),
            area: self.area,
            inner: self.inner,
            header_area: self.header_area,
            table_area: self.table_area,
            row_areas: self.row_areas.clone(),
            row_heights: self.row_heights.clone(),
            row_height: self.row_height,
            column_areas: self.column_areas.clone(),
            column_layout: self.column_layout.clone(),
            footer_area: self.footer_area,
            rows: self.rows,
            columns: self.columns,
            vscroll: self.vscroll.clone(),
            hscroll: self.hscroll.clone(),
            selection: self.selection.clone(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Selection: Default> Default for TableCoreState<Selection> {
    fn default() -> Self {
        Self {
            focus: Default::default(),
            area: Default::default(),
            inner: Default::default(),
            header_area: Default::default(),
            table_area: Default::default(),
            row_areas: Default::default(),
            row_heights: Default::default(),
            row_height: Default::default(),
            column_areas: Default::default(),
            column_layout: Default::default(),
            footer_area: Default::default(),
            rows: Default::default(),
            columns: Default::default(),
            vscroll: Default::default(),
            hscroll: Default::default(),
            selection: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Selection> RelocatableState for TableCoreState<Selection> {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area.relocate(shift, clip);
        self.inner.relocate(shift, clip);
        self.header_area.relocate(shift, clip);
        self.table_area.relocate(shift, clip);
        relocate_areas(self.row_areas.as_mut_slice(), shift, clip);
        relocate_areas(self.column_areas.as_mut_slice(), shift, clip);
        relocate_areas(self.column_layout.as_mut_slice(), shift, clip);
        self.footer_area.relocate(shift, clip);
        self.hscroll.relocate(shift, clip);
        self.vscroll.relocate(shift, clip);
    }
}

impl<Selection> TableCoreState<Selection>
where
    Selection: Default,
{
    /// New widget.
    pub fn new() -> Self {
        Self::default()
    }

    /// New named widget.
    pub fn named(name: &str) -> Self {
        Self {
            focus: FocusFlag::new().with_name(name),
            ..TableCoreState::default()
        }
    }
}

// Baseline
impl<Selection> TableCoreState<Selection> {
    /// Number of rows.
    #[inline]
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Number of columns.
    #[inline]
    pub fn columns(&self) -> usize {
        self.columns
    }
}

// Table areas
impl<Selection> TableCoreState<Selection> {
    /// Returns the whole row-area and the cell-areas for the
    /// given row, if it is visible.
    ///
    /// Attention: These areas might be 0-length if the column is scrolled
    /// beyond the table-area.
    pub fn row_cells(&self, row: usize) -> Option<(Rect, Vec<Rect>)> {
        if row < self.vscroll.offset() || row >= self.vscroll.offset() + self.vscroll.page_len() {
            return None;
        }

        let mut areas = Vec::new();

        let r = self.row_areas[row - self.vscroll.offset()];
        for c in &self.column_areas {
            areas.push(Rect::new(c.x, r.y, c.width, r.height));
        }

        Some((r, areas))
    }

    /// Cell at given position.
    pub fn cell_at_clicked(&self, pos: (u16, u16)) -> Option<(usize, usize)> {
        let col = self.column_at_clicked(pos);
        let row = self.row_at_clicked(pos);

        match (col, row) {
            (Some(col), Some(row)) => Some((col, row)),
            _ => None,
        }
    }

    /// Column at given position.
    pub fn column_at_clicked(&self, pos: (u16, u16)) -> Option<usize> {
        self.mouse.column_at(&self.column_areas, pos.0)
    }

    /// Row at given position.
    pub fn row_at_clicked(&self, pos: (u16, u16)) -> Option<usize> {
        self.mouse
            .row_at(&self.row_areas, pos.1)
            .map(|v| self.vscroll.offset() + v)
    }

    /// Cell when dragging. Position can be outside the table area.
    /// See [row_at_drag](TableCoreState::row_at_drag), [col_at_drag](TableCoreState::column_at_drag)
    pub fn cell_at_drag(&self, pos: (u16, u16)) -> (usize, usize) {
        let col = self.column_at_drag(pos);
        let row = self.row_at_drag(pos);

        (col, row)
    }

    /// Row when dragging. Position can be outside the table area.
    /// If the position is above the table-area this returns offset - #rows.
    /// If the position is below the table-area this returns offset + page_len + #rows.
    ///
    /// This doesn't account for the row-height of the actual rows outside
    /// the table area, just assumes '1'.
    pub fn row_at_drag(&self, pos: (u16, u16)) -> usize {
        match self
            .mouse
            .row_at_drag(self.table_area, &self.row_areas, pos.1)
        {
            Ok(v) => self.vscroll.offset() + v,
            Err(v) if v <= 0 => self.vscroll.offset().saturating_sub((-v) as usize),
            Err(v) => self.vscroll.offset() + self.row_areas.len() + v as usize,
        }
    }

    /// Column when dragging. Position can be outside the table area.
    /// If the position is left of the table area this returns offset - 1.
    /// If the position is right of the table area this returns offset + page_width + 1.
    pub fn column_at_drag(&self, pos: (u16, u16)) -> usize {
        match self
            .mouse
            .column_at_drag(self.table_area, &self.column_areas, pos.0)
        {
            Ok(v) => v,
            Err(v) if v <= 0 => self.hscroll.offset().saturating_sub((-v) as usize),
            Err(v) => self.hscroll.offset() + self.hscroll.page_len() + v as usize,
        }
    }
}

// Offset related.
impl<Selection: TableSelection> TableCoreState<Selection> {
    /// Clear the selection.
    #[inline]
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Update the state to match adding items *during* event-handling.
    /// A lot of functions use the number of rows, so when you change the
    /// underlying data you might want to immediately reflect this in the
    /// table-state.
    ///
    /// This corrects the number of rows, offset and selection.
    pub fn items_added(&mut self, pos: usize, n: usize) {
        self.rows = self.rows.saturating_add(n);
        self.vscroll.items_added(pos, n);
        if pos <= self.row_heights.len() {
            for _ in 0..n {
                self.row_heights.insert(pos, 1);
            }
        }
        self.selection.items_added(pos, n);
    }

    /// Update the state to match adding items.
    /// Use this function if you have varying row-heights.
    ///
    /// This corrects the number of rows, offset and selection.
    /// This also corrects the stored row-heights with the
    /// given heights.
    pub fn items_added_with_heigths(&mut self, pos: usize, height: &[u16]) {
        self.rows = self.rows.saturating_add(height.len());
        self.vscroll.items_added(pos, height.len());
        if pos <= self.row_heights.len() {
            self.row_heights
                .splice(pos..pos, height.into_iter().copied());
        }
        self.selection.items_added(pos, height.len());
    }

    /// Update the state to match removing items *during* event-handling.
    /// A lot of functions use the number of rows, so when you change the
    /// underlying data you might want to immediately reflect this in the
    /// table-state.
    ///
    /// This corrects the number of rows, offset and selection.
    pub fn items_removed(&mut self, pos: usize, n: usize) {
        self.rows = self.rows.saturating_sub(n);
        self.vscroll.items_removed(pos, n);
        if !self.row_heights.is_empty() {
            if pos < self.row_heights.len() {
                if pos + n < self.row_heights.len() {
                    self.row_heights.drain(pos..pos + n);
                } else {
                    self.row_heights.drain(pos..);
                }
            }
        }
        self.selection.items_removed(pos, n, self.rows);
    }

    /// Update the number of rows *during* event-handling.
    /// A lot of functions use the number of rows, so when you change the
    /// underlying data you might want to immediately reflect this in the
    /// table-state.
    ///
    /// This corrects the number of rows, offset and selection.
    pub fn rows_changed(&mut self, rows: usize) {
        self.selection.validate_rows(rows);
        self.vscroll.rows_changed(rows);
        if !self.row_heights.is_empty() {
            // don't know about row-heights.
            self.row_heights.clear();
            self.vscroll.page_len = 0;
        }
        self.rows = rows;
    }

    /// Sets both offsets to 0.
    pub fn clear_offset(&mut self) {
        self.vscroll.set_offset(0);
        self.hscroll.set_offset(0);
    }

    /// Maximum offset that is accessible with scrolling.
    ///
    /// This is shorter than the length by whatever fills the last page.
    /// This is the base for the scrollbar content_length.
    pub fn row_max_offset(&self) -> usize {
        self.vscroll.max_offset()
    }

    /// Current vertical offset.
    pub fn row_offset(&self) -> usize {
        self.vscroll.offset()
    }

    /// Change the vertical offset.
    /// The vertical offset may be beyond the number of rows.
    ///
    /// The widget returns true if the offset changed at all.
    pub fn set_row_offset(&mut self, offset: usize) -> bool {
        self.vscroll.set_offset(offset)
    }

    /// Vertical page-size at the current offset.
    pub fn page_len(&self) -> usize {
        self.vscroll.page_len()
    }

    /// Suggested scroll per scroll-event.
    pub fn row_scroll_by(&self) -> usize {
        self.vscroll.scroll_by()
    }

    /// Maximum offset that is accessible with scrolling.
    /// This is the table-width minus the widget-width plus some overscroll.
    pub fn x_max_offset(&self) -> usize {
        self.hscroll.max_offset()
    }

    /// Current horizontal offset.
    pub fn x_offset(&self) -> usize {
        self.hscroll.offset()
    }

    /// Change the horizontal offset.
    /// The offset may be beyond the table-width.
    ///
    /// The widget returns true if the offset changed at all.
    pub fn set_x_offset(&mut self, offset: usize) -> bool {
        self.hscroll.set_offset(offset)
    }

    /// Horizontal page-size at the current offset.
    pub fn page_width(&self) -> usize {
        self.hscroll.page_len()
    }

    /// Suggested scroll per scroll-event.
    pub fn x_scroll_by(&self) -> usize {
        self.hscroll.scroll_by()
    }

    /// Ensures that the selected item is visible.
    /// Caveat: This doesn't work nicely if you have varying row-heights.
    /// Caveat: Number of rows needs to be correct.
    pub fn scroll_to_selected(&mut self) -> bool {
        if let Some(selected) = self.selection.lead_selection() {
            let c = self.scroll_to_col(selected.0);
            let r = self.scroll_to_row(selected.1);
            r || c
        } else {
            false
        }
    }

    /// Ensures that the given row is visible.
    /// Caveat: This doesn't work nicely if you have varying row-heights.
    /// Caveat: Number of rows needs to be correct.
    pub fn scroll_to_row(&mut self, mut pos: usize) -> bool {
        if pos >= self.rows {
            false
        } else if self.page_len() == 0 {
            self.set_row_offset(pos)
        } else if self.row_heights.is_empty() {
            // fixed row-heights. allows us to calculate the offset.
            if pos >= self.row_offset().saturating_add(self.page_len() - 1) {
                if self.page_len() as u16 * self.row_height > self.table_area.height {
                    self.set_row_offset(pos.saturating_sub(self.page_len() - 2))
                } else {
                    self.set_row_offset(pos.saturating_sub(self.page_len() - 1))
                }
            } else if pos < self.row_offset() {
                self.set_row_offset(pos)
            } else {
                false
            }
        } else {
            // varying row-height. use the stored values.
            if pos >= self.row_offset().saturating_add(self.page_len() - 1) {
                if pos >= self.row_heights.len() {
                    self.set_row_offset(pos)
                } else {
                    let mut sum_height = self.row_heights[pos];
                    if sum_height < self.table_area.height {
                        loop {
                            if pos < 1 {
                                break;
                            }
                            sum_height += self.row_heights[pos - 1];
                            if sum_height > self.table_area.height {
                                break;
                            }
                            pos -= 1;
                        }
                    }
                    self.set_row_offset(pos)
                }
            } else if pos < self.row_offset() {
                self.set_row_offset(pos)
            } else {
                false
            }
        }
    }

    /// Ensures that the given column is completely visible.
    pub fn scroll_to_col(&mut self, pos: usize) -> bool {
        if let Some(col) = self.column_layout.get(pos) {
            self.hscroll
                .scroll_to_range(col.x as usize..col.right() as usize)
        } else {
            false
        }
    }

    /// Ensures that the given position is visible.
    pub fn scroll_to_x(&mut self, pos: usize) -> bool {
        self.hscroll.scroll_to_pos(pos)
    }

    /// Reduce the row-offset by n.
    pub fn scroll_up(&mut self, n: usize) -> bool {
        self.vscroll.scroll_up(n)
    }

    /// Increase the row-offset by n.
    pub fn scroll_down(&mut self, n: usize) -> bool {
        self.vscroll.scroll_down(n)
    }

    /// Reduce the col-offset by n.
    pub fn scroll_left(&mut self, n: usize) -> bool {
        self.hscroll.scroll_left(n)
    }

    /// Increase the col-offset by n.
    pub fn scroll_right(&mut self, n: usize) -> bool {
        self.hscroll.scroll_right(n)
    }
}

impl TableCoreState<RowSelection> {
    /// When scrolling the table, change the selection instead of the offset.
    #[inline]
    pub fn set_scroll_selection(&mut self, scroll: bool) {
        self.selection.set_scroll_selected(scroll);
    }

    /// Anything selected?
    #[inline]
    pub fn has_selection(&mut self) -> bool {
        self.selection.has_selection()
    }

    /// Selected row.
    /// The selected row is not constrained by the row-count.
    #[inline]
    pub fn selected(&self) -> Option<usize> {
        self.selection.selected()
    }

    /// Return the selected row and ensure it is in the
    /// range `0..rows`.
    #[inline]
    #[allow(clippy::manual_filter)]
    pub fn selected_checked(&self) -> Option<usize> {
        if let Some(selected) = self.selection.selected() {
            if selected < self.rows {
                Some(selected)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Select the row.
    /// The selection is not constrained by the row-count.
    #[inline]
    pub fn select(&mut self, row: Option<usize>) -> bool {
        self.selection.select(row)
    }

    /// Scroll delivers a value between 0 and max_offset as offset.
    /// This remaps the ratio to the selection with a range 0..row_len.
    /// Info: This is used when scroll_selected is active.
    pub(crate) fn remap_offset_selection(&self, offset: usize) -> usize {
        if self.vscroll.max_offset() > 0 {
            (self.rows * offset) / self.vscroll.max_offset()
        } else {
            0
        }
    }

    /// Set the selection to None and set the offset to 0
    #[inline]
    pub fn move_deselect(&mut self) -> bool {
        let r = self.selection.select(None);
        let s = self.set_row_offset(0);
        r || s
    }

    /// Move the selection to the given row.
    /// Ensures the row is visible afterward.
    #[inline]
    pub fn move_to(&mut self, row: usize) -> bool {
        let r = self.selection.move_to(row, self.rows.saturating_sub(1));
        let s = self.scroll_to_row(self.selection.selected().expect("row"));
        r || s
    }

    /// Move the selection up n rows.
    /// Ensures the row is visible afterward.
    #[inline]
    pub fn move_up(&mut self, n: usize) -> bool {
        let r = self.selection.move_up(n, self.rows.saturating_sub(1));
        let s = self.scroll_to_row(self.selection.selected().expect("row"));
        r || s
    }

    /// Move the selection down n rows.
    /// Ensures the row is visible afterward.
    #[inline]
    pub fn move_down(&mut self, n: usize) -> bool {
        let r = self.selection.move_down(n, self.rows.saturating_sub(1));
        let s = self.scroll_to_row(self.selection.selected().expect("row"));
        r || s
    }
}

impl TableCoreState<RowSetSelection> {
    /// Anything selected?
    #[inline]
    pub fn has_selection(&mut self) -> bool {
        self.selection.has_selection()
    }

    /// Selected rows.
    #[inline]
    pub fn selected(&self) -> HashSet<usize> {
        self.selection.selected()
    }

    /// Change the lead-selection. Limits the value to the number of rows.
    /// If extend is false the current selection is cleared and both lead and
    /// anchor are set to the given value.
    /// If extend is true, the anchor is kept where it is and lead is changed.
    /// Everything in the range `anchor..lead` is selected. It doesn't matter
    /// if anchor < lead.
    #[inline]
    pub fn set_lead(&mut self, row: Option<usize>, extend: bool) -> bool {
        self.selection.set_lead(row, extend)
    }

    /// Current lead.
    #[inline]
    pub fn lead(&self) -> Option<usize> {
        self.selection.lead()
    }

    /// Current anchor.
    #[inline]
    pub fn anchor(&self) -> Option<usize> {
        self.selection.anchor()
    }

    /// Retire the current anchor/lead selection to the set of selected rows.
    /// Resets lead and anchor and starts a new selection round.
    #[inline]
    pub fn retire_selection(&mut self) {
        self.selection.retire_selection();
    }

    /// Add to selection. Only works for retired selections, not for the
    /// active anchor-lead range.
    ///
    /// To be sure call [retire_selection][Self::retire_selection] first.
    #[inline]
    pub fn add_selected(&mut self, idx: usize) {
        self.selection.add(idx);
    }

    /// Remove from selection. Only works for retired selections, not for the
    /// active anchor-lead range.
    ///
    /// To be sure call [retire_selection][Self::retire_selection] first.
    #[inline]
    pub fn remove_selected(&mut self, idx: usize) {
        self.selection.remove(idx);
    }

    /// Move the selection to the given row.
    /// Ensures the row is visible afterward.
    #[inline]
    pub fn move_to(&mut self, row: usize, extend: bool) -> bool {
        let r = self
            .selection
            .move_to(row, self.rows.saturating_sub(1), extend);
        let s = self.scroll_to_row(self.selection.lead().expect("row"));
        r || s
    }

    /// Move the selection up n rows.
    /// Ensures the row is visible afterward.
    #[inline]
    pub fn move_up(&mut self, n: usize, extend: bool) -> bool {
        let r = self
            .selection
            .move_up(n, self.rows.saturating_sub(1), extend);
        let s = self.scroll_to_row(self.selection.lead().expect("row"));
        r || s
    }

    /// Move the selection down n rows.
    /// Ensures the row is visible afterwards.
    #[inline]
    pub fn move_down(&mut self, n: usize, extend: bool) -> bool {
        let r = self
            .selection
            .move_down(n, self.rows.saturating_sub(1), extend);
        let s = self.scroll_to_row(self.selection.lead().expect("row"));
        r || s
    }
}

impl TableCoreState<CellSelection> {
    #[inline]
    pub fn has_selection(&mut self) -> bool {
        self.selection.has_selection()
    }

    /// Selected cell.
    #[inline]
    pub fn selected(&self) -> Option<(usize, usize)> {
        self.selection.selected()
    }

    /// Select a cell.
    #[inline]
    pub fn select_cell(&mut self, select: Option<(usize, usize)>) -> bool {
        self.selection.select_cell(select)
    }

    /// Select a row. Column stays the same.
    #[inline]
    pub fn select_row(&mut self, row: Option<usize>) -> bool {
        if let Some(row) = row {
            self.selection
                .select_row(Some(min(row, self.rows.saturating_sub(1))))
        } else {
            self.selection.select_row(None)
        }
    }

    /// Select a column, row stays the same.
    #[inline]
    pub fn select_column(&mut self, column: Option<usize>) -> bool {
        if let Some(column) = column {
            self.selection
                .select_column(Some(min(column, self.columns.saturating_sub(1))))
        } else {
            self.selection.select_column(None)
        }
    }

    /// Select a cell, limit to maximum.
    #[inline]
    pub fn move_to(&mut self, select: (usize, usize)) -> bool {
        let r = self.selection.move_to(
            select,
            (self.columns.saturating_sub(1), self.rows.saturating_sub(1)),
        );
        let s = self.scroll_to_selected();
        r || s
    }

    /// Select a row, limit to maximum.
    #[inline]
    pub fn move_to_row(&mut self, row: usize) -> bool {
        let r = self.selection.move_to_row(row, self.rows.saturating_sub(1));
        let s = self.scroll_to_selected();
        r || s
    }

    /// Select a cell, clamp between 0 and maximum.
    #[inline]
    pub fn move_to_col(&mut self, col: usize) -> bool {
        let r = self
            .selection
            .move_to_col(col, self.columns.saturating_sub(1));
        let s = self.scroll_to_selected();
        r || s
    }

    /// Move the selection up n rows.
    /// Ensures the row is visible afterwards.
    #[inline]
    pub fn move_up(&mut self, n: usize) -> bool {
        let r = self.selection.move_up(n, self.rows.saturating_sub(1));
        let s = self.scroll_to_selected();
        r || s
    }

    /// Move the selection down n rows.
    /// Ensures the row is visible afterwards.
    #[inline]
    pub fn move_down(&mut self, n: usize) -> bool {
        let r = self.selection.move_down(n, self.rows.saturating_sub(1));
        let s = self.scroll_to_selected();
        r || s
    }

    /// Move the selection left n columns.
    /// Ensures the row is visible afterwards.
    #[inline]
    pub fn move_left(&mut self, n: usize) -> bool {
        let r = self.selection.move_left(n, self.columns.saturating_sub(1));
        let s = self.scroll_to_selected();
        r || s
    }

    /// Move the selection right n columns.
    /// Ensures the row is visible afterwards.
    #[inline]
    pub fn move_right(&mut self, n: usize) -> bool {
        let r = self.selection.move_right(n, self.columns.saturating_sub(1));
        let s = self.scroll_to_selected();
        r || s
    }
}

impl<Selection> HandleEvent<Event, DoubleClick, DoubleClickOutcome> for TableCoreState<Selection> {
    /// Handles double-click events on the table.
    fn handle(&mut self, event: &Event, _keymap: DoubleClick) -> DoubleClickOutcome {
        if self.focus.is_obscured() {
            return DoubleClickOutcome::Continue;
        }

        match event {
            ct_event!(mouse any for m) if self.mouse.doubleclick(self.table_area, m) => {
                if let Some((col, row)) = self.cell_at_clicked((m.column, m.row)) {
                    DoubleClickOutcome::ClickClick(col, row)
                } else {
                    DoubleClickOutcome::Continue
                }
            }
            _ => DoubleClickOutcome::Continue,
        }
    }
}

impl<Selection> TableCoreState<Selection> {
    /// Handle all events for recognizing double-clicks.
    pub fn handle_doubleclick_events(&mut self, event: &Event) -> DoubleClickOutcome {
        self.handle(event, DoubleClick)
    }
}

impl TableCoreState<RowSelection> {
    /// Handle all events.
    /// Table events are only processed if focus is true.
    /// Mouse events are processed if they are in range.
    pub fn handle_events(&mut self, focus: bool, event: &Event) -> TableOutcome {
        self.focus.set(focus);
        self.handle(event, Regular)
    }

    /// Handle only mouse-events.
    pub fn handle_mouse_events(&mut self, event: &Event) -> TableOutcome {
        self.handle(event, MouseOnly)
    }
}

impl TableCoreState<CellSelection> {
    /// Handle all events.
    /// Table events are only processed if focus is true.
    /// Mouse events are processed if they are in range.
    pub fn handle_events(&mut self, focus: bool, event: &Event) -> TableOutcome {
        self.focus.set(focus);
        self.handle(event, Regular)
    }

    /// Handle only mouse-events.
    pub fn handle_mouse_events(&mut self, event: &Event) -> TableOutcome {
        self.handle(event, MouseOnly)
    }
}

impl TableCoreState<NoSelection> {
    /// Handle all events.
    /// Table events are only processed if focus is true.
    /// Mouse events are processed if they are in range.
    pub fn handle_events(&mut self, focus: bool, event: &Event) -> TableOutcome {
        self.focus.set(focus);
        self.handle(event, Regular)
    }

    /// Handle only mouse-events.
    pub fn handle_mouse_events(&mut self, event: &Event) -> TableOutcome {
        self.handle(event, MouseOnly)
    }
}

impl TableCoreState<RowSetSelection> {
    /// Handle all events.
    /// Table events are only processed if focus is true.
    /// Mouse events are processed if they are in range.
    pub fn handle_events(&mut self, focus: bool, event: &Event) -> TableOutcome {
        self.focus.set(focus);
        self.handle(event, Regular)
    }

    /// Handle only mouse-events.
    pub fn handle_mouse_events(&mut self, event: &Event) -> TableOutcome {
        self.handle(event, MouseOnly)
    }
}
