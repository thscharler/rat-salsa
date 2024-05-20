use crate::_private::NonExhaustive;
use rat_focus::{FocusFlag, HasFocusFlag};
use rat_ftable::textdata::Row;
use ratatui::layout::{Constraint, Flex, Position, Rect};
use ratatui::style::{Style, Styled};
use ratatui::widgets::{Block, StatefulWidget};
use std::cmp::max;

pub use rat_ftable::{FTableStyle, TableData, TableSelection};
use rat_scrolled::{ScrollingState, ScrollingWidget};
use ratatui::buffer::Buffer;

pub mod selection {
    pub use rat_ftable::selection::{CellSelection, NoSelection, RowSelection, RowSetSelection};
}

pub mod textdata {
    pub use rat_ftable::textdata::{Cell, Row};
}

#[derive(Debug, Default, Clone)]
pub struct FTable<'a, Selection> {
    widget: rat_ftable::FTable<'a, Selection>,
}

#[derive(Debug, Clone)]
pub struct FTableState<Selection> {
    pub widget: rat_ftable::FTableState<Selection>,
    pub focus: FocusFlag,

    pub non_exhaustive: NonExhaustive,
}

impl<'a, Selection> FTable<'a, Selection> {
    /// Create a new FTable with preformatted data. For compatibility
    /// with ratatui.
    ///
    /// Use of [FTable::data] is preferred.
    pub fn new<R, C>(rows: R, widths: C) -> Self
    where
        R: IntoIterator,
        R::Item: Into<Row<'a>>,
        C: IntoIterator,
        C::Item: Into<Constraint>,
        Selection: Default,
    {
        Self {
            widget: rat_ftable::FTable::new(rows, widths),
        }
    }

    /// Set preformatted row-data. For compatibility with ratatui.
    ///
    /// Use of [FTable::data] is preferred.
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
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.widget = self.widget.style(style);
        self
    }

    /// Style for a selected row. The chosen selection must support
    /// row-selection for this to take effect.
    #[inline]
    pub fn select_row_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.widget = self.widget.select_row_style(select_style);
        self
    }

    /// Style for a selected column. The chosen selection must support
    /// column-selection for this to take effect.
    #[inline]
    pub fn select_column_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.widget = self.widget.select_column_style(select_style);
        self
    }

    /// Style for a selected cell. The chosen selection must support
    /// cell-selection for this to take effect.
    #[inline]
    pub fn select_cell_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.widget = self.widget.select_cell_style(select_style);
        self
    }

    /// Style for a selected header cell. The chosen selection must
    /// support column-selection for this to take effect.
    #[inline]
    pub fn select_header_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.widget = self.widget.select_header_style(select_style);
        self
    }

    /// Style for a selected footer cell. The chosen selection must
    /// support column-selection for this to take effect.
    #[inline]
    pub fn select_footer_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.widget = self.widget.select_footer_style(select_style);
        self
    }

    /// This style will be patched onto the selection to indicate that
    /// the widget has the input focus.
    ///
    /// The selection must support some kind of selection for this to
    /// be effective.
    #[inline]
    pub fn focus_style<S: Into<Style>>(mut self, focus_style: S) -> Self {
        self.widget = self.widget.focus_style(focus_style);
        self
    }
}

impl<'a, Selection> Styled for FTable<'a, Selection> {
    type Item = Self;

    fn style(&self) -> Style {
        Styled::style(&self.widget)
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.widget = Styled::set_style(self.widget, style);
        self
    }
}

impl<'a, Selection> ScrollingWidget<FTableState<Selection>> for FTable<'a, Selection> {
    fn need_scroll(&self, area: Rect, _state: &mut FTableState<Selection>) -> (bool, bool) {
        self.widget.need_scroll(area)
    }
}

impl<'a, Selection> StatefulWidget for FTable<'a, Selection>
where
    Selection: TableSelection,
{
    type State = FTableState<Selection>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.widget
            .focus(state.is_focused())
            .render(area, buf, &mut state.widget);
    }
}

impl<Selection: Default> Default for FTableState<Selection> {
    fn default() -> Self {
        Self {
            widget: rat_ftable::FTableState::default(),
            focus: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Selection> FTableState<Selection> {
    /// Cell at given position.
    pub fn cell_at_clicked(&self, pos: Position) -> Option<(usize, usize)> {
        self.widget.cell_at_clicked(pos)
    }

    /// Column at given position.
    pub fn column_at_clicked(&self, pos: Position) -> Option<usize> {
        self.widget.column_at_clicked(pos)
    }

    /// Row at given position.
    pub fn row_at_clicked(&self, pos: Position) -> Option<usize> {
        self.widget.row_at_clicked(pos)
    }

    /// Cell when dragging. Can go outside the area.
    pub fn cell_at_drag(&self, pos: Position) -> (usize, usize) {
        self.widget.cell_at_drag(pos)
    }

    /// Row when dragging. Can go outside the area.
    pub fn row_at_drag(&self, pos: Position) -> usize {
        self.widget.row_at_drag(pos)
    }

    /// Column when dragging. Can go outside the area.
    pub fn column_at_drag(&self, pos: Position) -> usize {
        self.widget.column_at_drag(pos)
    }
}

impl<Selection: TableSelection> FTableState<Selection> {
    /// Scroll to selected.
    pub fn scroll_to_selected(&mut self) {
        self.widget.scroll_to_selected()
    }
}

impl<Selection: TableSelection> ScrollingState for FTableState<Selection> {
    fn vertical_max_offset(&self) -> usize {
        self.widget.max_row_offset
    }

    fn vertical_offset(&self) -> usize {
        self.widget.row_offset
    }

    fn vertical_page(&self) -> usize {
        self.widget.row_page_len
    }

    fn vertical_scroll(&self) -> usize {
        max(self.widget.row_page_len / 10, 1)
    }

    fn horizontal_max_offset(&self) -> usize {
        self.widget.max_col_offset
    }

    fn horizontal_offset(&self) -> usize {
        self.widget.col_offset
    }

    fn horizontal_page(&self) -> usize {
        self.widget.col_page_len
    }

    fn horizontal_scroll(&self) -> usize {
        1
    }

    fn set_vertical_offset(&mut self, offset: usize) -> bool {
        self.widget.set_row_offset(offset)
    }

    fn set_horizontal_offset(&mut self, offset: usize) -> bool {
        self.widget.set_column_offset(offset)
    }
}

impl<Selection> HasFocusFlag for FTableState<Selection> {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.widget.area
    }
}
