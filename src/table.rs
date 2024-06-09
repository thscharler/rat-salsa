use crate::_private::NonExhaustive;
use crate::event::{FocusKeys, HandleEvent, MouseOnly, Outcome};
#[allow(unused_imports)]
use log::debug;
use rat_focus::{FocusFlag, HasFocusFlag};
use rat_ftable::selection::{CellSelection, NoSelection, RowSelection, RowSetSelection};
use rat_ftable::textdata::Row;
use rat_scrolled::{ScrollingState, ScrollingWidget};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Position, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, StatefulWidget};
use std::collections::HashSet;

use rat_ftable::event::{DoubleClick, DoubleClickOutcome, EditKeys, EditOutcome};
pub use rat_ftable::{FTableContext, FTableStyle, TableData, TableDataIter, TableSelection};

pub mod selection {
    pub use rat_ftable::selection::{CellSelection, NoSelection, RowSelection, RowSetSelection};
}

pub mod textdata {
    pub use rat_ftable::textdata::{Cell, Row};
}

#[derive(Debug, Default)]
pub struct RTable<'a, Selection> {
    widget: rat_ftable::FTable<'a, Selection>,
}

#[derive(Debug, Clone)]
pub struct RTableState<Selection> {
    pub widget: rat_ftable::FTableState<Selection>,
    pub focus: FocusFlag,

    pub non_exhaustive: NonExhaustive,
}

impl<'a, Selection> RTable<'a, Selection> {
    /// New, empty Table.
    pub fn new() -> Self
    where
        Selection: Default,
    {
        Self::default()
    }

    /// Create a new FTable with preformatted data. For compatibility
    /// with ratatui.
    ///
    /// Use of [RTable::data] is preferred.
    pub fn new_ratatui<R, C>(rows: R, widths: C) -> Self
    where
        R: IntoIterator,
        R::Item: Into<Row<'a>>,
        C: IntoIterator,
        C::Item: Into<Constraint>,
        Selection: Default,
    {
        Self {
            widget: rat_ftable::FTable::new_ratatui(rows, widths),
        }
    }

    /// Set preformatted row-data. For compatibility with ratatui.
    ///
    /// Use of [RTable::data] is preferred.
    pub fn rows<T>(mut self, rows: T) -> Self
    where
        T: IntoIterator<Item = Row<'a>>,
    {
        self.widget = self.widget.rows(rows);
        self
    }

    /// Set a reference to the TableData facade to your data.
    ///
    /// The way to go is to define a small struct that contains just a
    /// reference to your data. Then implement TableData for this struct.
    ///
    /// ```rust ignore
    /// use ratatui::buffer::Buffer;
    /// use ratatui::layout::Rect;
    /// use ratatui::prelude::Style;
    /// use ratatui::text::Span;
    /// use ratatui::widgets::Widget;
    /// use rat_ftable::{FTable, FTableState, TableData};
    ///
    /// # struct SampleRow;
    /// # let area = Rect::default();
    /// # let mut buf = Buffer::empty(area);
    /// # let buf = &mut buf;
    ///
    /// struct Data1<'a>(&'a [SampleRow]);
    ///
    /// impl<'a> TableData<'a> for Data1<'a> {
    ///     // returns (cols, rows)
    ///     fn size(&self) -> (usize, usize) {
    ///         (5, self.0.len())
    ///     }
    ///
    ///     fn row_height(&self, row: usize) -> u16 {
    ///         // to some calculations ...
    ///         1
    ///     }
    ///
    ///     fn row_style(&self, row: usize) -> Style {
    ///         // to some calculations ...
    ///         Style::default()
    ///     }
    ///
    ///     fn render_cell(&self, column: usize, row: usize, area: Rect, buf: &mut Buffer) {
    ///         if let Some(data) = self.0.get(row) {
    ///             let rend = match column {
    ///                 0 => Span::from("column1"),
    ///                 1 => Span::from("column2"),
    ///                 2 => Span::from("column3"),
    ///                 _ => return
    ///             };
    ///             rend.render(area, buf);
    ///         }
    ///     }
    /// }
    ///
    /// // When you are creating the table widget you hand over a reference
    /// // to the facade struct.
    ///
    /// let my_data_somewhere_else = vec![SampleRow;999999];
    /// let mut table_state_somewhere_else = FTableState::default();
    ///
    /// let tabledata1 = Data1(&my_data_somewhere_else);
    /// let table1 = FTable::default().data(&tabledata1);
    /// table1.render(area, buf, &mut table_state_somewhere_else);
    /// ```
    #[inline]
    pub fn data(mut self, data: &'a dyn TableData<'a>) -> Self {
        self.widget = self.widget.data(data);
        self
    }

    ///
    /// Alternative representation for the data is as an Iterator that yields a TableRowData.
    ///
    /// Caution: If you can't give the number of rows, the table will iterate over all
    /// the data.
    ///
    #[inline]
    pub fn iter(mut self, data: &'a mut dyn TableDataIter<'a>) -> Self {
        self.widget = self.widget.iter(data);
        self
    }

    /// Set the table-header.
    #[inline]
    pub fn header(mut self, header: Row<'a>) -> Self {
        self.widget = self.widget.header(header);
        self
    }

    /// Set the table-footer.
    #[inline]
    pub fn footer(mut self, footer: Row<'a>) -> Self {
        self.widget = self.widget.footer(footer);
        self
    }

    /// Column widths as Constraints.
    pub fn widths<I>(mut self, widths: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        self.widget = self.widget.widths(widths);
        self
    }

    /// Flex for layout.
    #[inline]
    pub fn flex(mut self, flex: Flex) -> Self {
        self.widget = self.widget.flex(flex);
        self
    }

    /// Spacing between columns.
    #[inline]
    pub fn column_spacing(mut self, spacing: u16) -> Self {
        self.widget = self.widget.column_spacing(spacing);
        self
    }

    /// Overrides the width of the rendering area for layout purposes.
    /// Layout uses this width, even if it means that some columns are
    /// not visible.
    #[inline]
    pub fn layout_width(mut self, width: u16) -> Self {
        self.widget = self.widget.layout_width(width);
        self
    }

    /// Draws a block around the table widget.
    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.widget = self.widget.block(block);
        self
    }

    /// Set all styles as a bundle.
    #[inline]
    pub fn styles(mut self, styles: FTableStyle) -> Self {
        self.widget = self.widget.styles(styles);
        self
    }

    /// Base style for the table.
    #[inline]
    pub fn style(mut self, style: Style) -> Self {
        self.widget = self.widget.style(style);
        self
    }

    /// Base style for the table.
    #[inline]
    pub fn header_style(mut self, style: Option<Style>) -> Self {
        self.widget = self.widget.header_style(style);
        self
    }

    /// Base style for the table.
    #[inline]
    pub fn footer_style(mut self, style: Option<Style>) -> Self {
        self.widget = self.widget.footer_style(style);
        self
    }

    /// Style for a selected row. The chosen selection must support
    /// row-selection for this to take effect.
    #[inline]
    pub fn select_row_style(mut self, select_style: Option<Style>) -> Self {
        self.widget = self.widget.select_row_style(select_style);
        self
    }

    #[inline]
    pub fn show_row_focus(mut self, show: bool) -> Self {
        self.widget = self.widget.show_row_focus(show);
        self
    }

    /// Style for a selected column. The chosen selection must support
    /// column-selection for this to take effect.
    #[inline]
    pub fn select_column_style(mut self, select_style: Option<Style>) -> Self {
        self.widget = self.widget.select_column_style(select_style);
        self
    }

    #[inline]
    pub fn show_column_focus(mut self, show: bool) -> Self {
        self.widget = self.widget.show_column_focus(show);
        self
    }

    /// Style for a selected cell. The chosen selection must support
    /// cell-selection for this to take effect.
    #[inline]
    pub fn select_cell_style(mut self, select_style: Option<Style>) -> Self {
        self.widget = self.widget.select_cell_style(select_style);
        self
    }

    #[inline]
    pub fn show_cell_focus(mut self, show: bool) -> Self {
        self.widget = self.widget.show_cell_focus(show);
        self
    }

    /// Style for a selected header cell. The chosen selection must
    /// support column-selection for this to take effect.
    #[inline]
    pub fn select_header_style(mut self, select_style: Option<Style>) -> Self {
        self.widget = self.widget.select_header_style(select_style);
        self
    }

    #[inline]
    pub fn show_header_focus(mut self, show: bool) -> Self {
        self.widget = self.widget.show_header_focus(show);
        self
    }

    /// Style for a selected footer cell. The chosen selection must
    /// support column-selection for this to take effect.
    #[inline]
    pub fn select_footer_style(mut self, select_style: Option<Style>) -> Self {
        self.widget = self.widget.select_footer_style(select_style);
        self
    }

    #[inline]
    pub fn show_footer_focus(mut self, show: bool) -> Self {
        self.widget = self.widget.show_footer_focus(show);
        self
    }

    /// This style will be patched onto the selection to indicate that
    /// the widget has the input focus.
    ///
    /// The selection must support some kind of selection for this to
    /// be effective.
    #[inline]
    pub fn focus_style(mut self, focus_style: Option<Style>) -> Self {
        self.widget = self.widget.focus_style(focus_style);
        self
    }

    #[inline]
    pub fn debug(mut self, debug: bool) -> Self {
        self.widget = self.widget.debug(debug);
        self
    }
}

impl<'a, Selection> ScrollingWidget<RTableState<Selection>> for RTable<'a, Selection> {
    #[inline]
    fn need_scroll(&self, area: Rect, _state: &mut RTableState<Selection>) -> (bool, bool) {
        self.widget.need_scroll(area)
    }
}

impl<'a, Selection> StatefulWidget for RTable<'a, Selection>
where
    Selection: TableSelection,
{
    type State = RTableState<Selection>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget
            .focus(state.is_focused())
            .render(area, buf, &mut state.widget);
    }
}

impl<Selection: Default> Default for RTableState<Selection> {
    fn default() -> Self {
        Self {
            widget: rat_ftable::FTableState::default(),
            focus: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Selection> RTableState<Selection> {
    /// Number of rows.
    #[inline]
    pub fn rows(&self) -> usize {
        self.widget.rows()
    }

    /// Number of columns.
    #[inline]
    pub fn columns(&self) -> usize {
        self.widget.columns()
    }

    /// Returns the column-areas for the given row, if it is visible.
    ///
    /// Attention: These areas might be 0-length if the column is scrolled
    /// beyond the table-area.
    ///
    /// See: [rat_ftable::FTableState::scroll_to]
    #[inline]
    pub fn row_cells(&self, row: usize) -> Option<(Rect, Vec<Rect>)> {
        self.widget.row_cells(row)
    }

    /// Cell at given position.
    #[inline]
    pub fn cell_at_clicked(&self, pos: Position) -> Option<(usize, usize)> {
        self.widget.cell_at_clicked(pos)
    }

    /// Column at given position.
    #[inline]
    pub fn column_at_clicked(&self, pos: Position) -> Option<usize> {
        self.widget.column_at_clicked(pos)
    }

    /// Row at given position.
    #[inline]
    pub fn row_at_clicked(&self, pos: Position) -> Option<usize> {
        self.widget.row_at_clicked(pos)
    }

    /// Cell when dragging. Can go outside the area.
    #[inline]
    pub fn cell_at_drag(&self, pos: Position) -> (usize, usize) {
        self.widget.cell_at_drag(pos)
    }

    /// Row when dragging. Can go outside the area.
    #[inline]
    pub fn row_at_drag(&self, pos: Position) -> usize {
        self.widget.row_at_drag(pos)
    }

    /// Column when dragging. Can go outside the area.
    #[inline]
    pub fn column_at_drag(&self, pos: Position) -> usize {
        self.widget.column_at_drag(pos)
    }

    /// Sets both offsets to 0.
    #[inline]
    pub fn clear_offset(&mut self) {
        self.widget.clear_offset();
    }
}

impl<Selection: TableSelection> RTableState<Selection> {
    /// Scroll to selected.
    #[inline]
    pub fn scroll_to_selected(&mut self) {
        self.widget.scroll_to_selected()
    }

    /// Scroll to position.
    #[inline]
    pub fn scroll_to(&mut self, pos: (usize, usize)) {
        self.widget.scroll_to(pos);
    }
}

impl<Selection: TableSelection> ScrollingState for RTableState<Selection> {
    #[inline]
    fn vertical_max_offset(&self) -> usize {
        self.widget.vertical_max_offset()
    }

    #[inline]
    fn vertical_offset(&self) -> usize {
        self.widget.vertical_offset()
    }

    #[inline]
    fn vertical_page(&self) -> usize {
        self.widget.vertical_page()
    }

    #[inline]
    fn vertical_scroll(&self) -> usize {
        self.widget.vertical_scroll()
    }

    #[inline]
    fn horizontal_max_offset(&self) -> usize {
        self.widget.horizontal_max_offset()
    }

    #[inline]
    fn horizontal_offset(&self) -> usize {
        self.widget.horizontal_offset()
    }

    #[inline]
    fn horizontal_page(&self) -> usize {
        self.widget.horizontal_page()
    }

    #[inline]
    fn horizontal_scroll(&self) -> usize {
        self.widget.horizontal_scroll()
    }

    #[inline]
    fn set_vertical_offset(&mut self, offset: usize) -> bool {
        self.widget.set_vertical_offset(offset)
    }

    #[inline]
    fn set_horizontal_offset(&mut self, offset: usize) -> bool {
        self.widget.set_horizontal_offset(offset)
    }

    #[inline]
    fn scroll_up(&mut self, n: usize) -> bool {
        self.widget.scroll_up(n)
    }

    #[inline]
    fn scroll_down(&mut self, n: usize) -> bool {
        self.widget.scroll_down(n)
    }

    #[inline]
    fn scroll_left(&mut self, n: usize) -> bool {
        self.widget.scroll_left(n)
    }

    #[inline]
    fn scroll_right(&mut self, n: usize) -> bool {
        self.widget.scroll_right(n)
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for RTableState<NoSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
        if self.is_focused() {
            self.widget.handle(event, FocusKeys)
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for RTableState<NoSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        self.widget.handle(event, MouseOnly)
    }
}

impl RTableState<RowSelection> {
    /// Scroll selection instead of offset.
    #[inline]
    pub fn set_scroll_selection(&mut self, scroll: bool) {
        self.widget.set_scroll_selection(scroll);
    }

    /// Scroll selection instead of offset.
    #[inline]
    pub fn scroll_selection(&self) -> bool {
        self.widget.scroll_selection()
    }

    /// Clear offsets and selection.
    #[inline]
    pub fn clear(&mut self) {
        self.widget.clear();
    }

    #[inline]
    pub fn clear_selection(&mut self) {
        self.widget.clear_selection();
    }

    #[inline]
    pub fn has_selection(&mut self) -> bool {
        self.widget.has_selection()
    }

    #[inline]
    pub fn selected(&self) -> Option<usize> {
        self.widget.selected()
    }

    #[inline]
    pub fn select(&mut self, row: Option<usize>) {
        self.widget.select(row);
    }

    /// Select a row, clamp between 0 and maximum.
    #[inline]
    pub fn select_clamped(&mut self, select: usize, maximum: usize) {
        self.widget.select_clamped(select, maximum);
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for RTableState<RowSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
        if self.is_focused() {
            self.widget.handle(event, FocusKeys)
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for RTableState<RowSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        self.widget.handle(event, MouseOnly)
    }
}

impl RTableState<RowSetSelection> {
    /// Clear offsets and selection.
    #[inline]
    pub fn clear(&mut self) {
        self.widget.clear();
    }

    /// Clear the selection.
    #[inline]
    pub fn clear_selection(&mut self) {
        self.widget.clear_selection();
    }

    #[inline]
    pub fn has_selection(&mut self) -> bool {
        self.widget.has_selection()
    }

    #[inline]
    pub fn selected(&self) -> HashSet<usize> {
        self.widget.selected()
    }

    #[inline]
    pub fn set_lead(&mut self, lead: Option<usize>, extend: bool) {
        self.widget.set_lead(lead, extend);
    }

    /// Set a new lead, at the same time limit the lead to max.
    #[inline]
    pub fn set_lead_clamped(&mut self, lead: usize, max: usize, extend: bool) {
        self.widget.set_lead_clamped(lead, max, extend);
    }

    /// Current lead.
    #[inline]
    pub fn lead(&self) -> Option<usize> {
        self.widget.lead()
    }

    /// Current anchor.
    #[inline]
    pub fn anchor(&self) -> Option<usize> {
        self.widget.anchor()
    }

    /// Add to selection.
    #[inline]
    pub fn add_selected(&mut self, idx: usize) {
        self.widget.add_selected(idx);
    }

    /// Remove from selection. Only works for retired selections, not for the
    /// active anchor-lead range.
    #[inline]
    pub fn remove_selected(&mut self, idx: usize) {
        self.widget.remove_selected(idx);
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for RTableState<RowSetSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _: FocusKeys) -> Outcome {
        if self.is_focused() {
            self.widget.handle(event, FocusKeys)
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for RTableState<RowSetSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _: MouseOnly) -> Outcome {
        self.widget.handle(event, MouseOnly)
    }
}

impl RTableState<CellSelection> {
    /// Clear offsets and selection.
    #[inline]
    pub fn clear(&mut self) {
        self.widget.clear();
    }

    #[inline]
    pub fn clear_selection(&mut self) {
        self.widget.clear_selection();
    }

    /// Selected cell.
    #[inline]
    pub fn selected(&self) -> Option<(usize, usize)> {
        self.widget.selected()
    }

    #[inline]
    pub fn has_selection(&mut self) -> bool {
        self.widget.has_selection()
    }

    /// Select a cell.
    #[inline]
    pub fn select_cell(&mut self, select: Option<(usize, usize)>) {
        self.widget.select_cell(select);
    }

    /// Select a row. Column stays the same.
    #[inline]
    pub fn select_row(&mut self, select: Option<usize>) {
        self.widget.select_row(select);
    }

    /// Select a column, row stays the same.
    #[inline]
    pub fn select_column(&mut self, select: Option<usize>) {
        self.widget.select_column(select);
    }

    /// Select a cell, clamp between 0 and maximum.
    #[inline]
    pub fn select_clamped(&mut self, select: (usize, usize), maximum: (usize, usize)) {
        self.widget.select_clamped(select, maximum);
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for RTableState<CellSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
        if self.is_focused() {
            self.widget.handle(event, FocusKeys)
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for RTableState<CellSelection> {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        self.widget.handle(event, MouseOnly)
    }
}

impl<Selection> HandleEvent<crossterm::event::Event, DoubleClick, DoubleClickOutcome>
    for RTableState<Selection>
where
    rat_ftable::FTableState<Selection>:
        HandleEvent<crossterm::event::Event, DoubleClick, DoubleClickOutcome>,
{
    fn handle(
        &mut self,
        event: &crossterm::event::Event,
        _keymap: DoubleClick,
    ) -> DoubleClickOutcome {
        self.widget.handle(event, DoubleClick)
    }
}

impl<Selection> HandleEvent<crossterm::event::Event, EditKeys, EditOutcome>
    for RTableState<Selection>
where
    rat_ftable::FTableState<Selection>: HandleEvent<crossterm::event::Event, MouseOnly, Outcome>,
    rat_ftable::FTableState<Selection>: HandleEvent<crossterm::event::Event, FocusKeys, Outcome>,
    rat_ftable::FTableState<Selection>: HandleEvent<crossterm::event::Event, EditKeys, EditOutcome>,
{
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: EditKeys) -> EditOutcome {
        if self.is_focused() {
            self.widget.handle(event, EditKeys)
        } else {
            self.widget.handle(event, MouseOnly).into()
        }
    }
}

impl<Selection> HasFocusFlag for RTableState<Selection> {
    #[inline]
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    #[inline]
    fn area(&self) -> Rect {
        self.widget.area
    }
}
