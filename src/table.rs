use crate::_private::NonExhaustive;
use crate::selection::{CellSelection, RowSelection, RowSetSelection};
use crate::textdata::{Row, TextTableData};
use crate::util::MouseFlags;
use crate::{TableData, TableSelection};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Position, Rect};
use ratatui::prelude::BlockExt;
use ratatui::style::{Style, Styled};
use ratatui::widgets::{Block, StatefulWidget, Widget, WidgetRef};
use std::cmp::min;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::rc::Rc;

/// FTable widget.
///
/// Can be used like a ratatui::Table, but the benefits only
/// show if you use [FTable::data] to set the table data.
///
/// See [FTable::data] for a sample.
#[derive(Debug, Default, Clone)]
pub struct FTable<'a, Selection> {
    data: DataRepr<'a>,

    header: Option<Row<'a>>,
    footer: Option<Row<'a>>,

    widths: Vec<Constraint>,
    flex: Flex,
    column_spacing: u16,
    layout_width: Option<u16>,

    block: Option<Block<'a>>,

    style: Style,

    select_row_style: Style,
    select_column_style: Style,
    select_cell_style: Style,
    select_header_style: Style,
    select_footer_style: Style,
    focus_style: Style,

    focus: bool,

    _phantom: PhantomData<Selection>,
}

#[derive(Clone)]
enum DataRepr<'a> {
    Text(TextTableData<'a>),
    Ref(&'a dyn TableData<'a>),
}

/// Combined style.
#[derive(Debug)]
pub struct FTableStyle {
    pub style: Style,
    pub select_row_style: Style,
    pub select_column_style: Style,
    pub select_cell_style: Style,
    pub select_header_style: Style,
    pub select_footer_style: Style,
    pub focus_style: Style,
    pub non_exhaustive: NonExhaustive,
}

/// FTable state.
#[derive(Debug, Clone)]
pub struct FTableState<Selection> {
    /// Total area.
    pub area: Rect,

    /// Total header area.
    pub header_area: Rect,
    /// Total table area.
    pub table_area: Rect,
    /// Area per visible row.
    pub row_areas: Vec<Rect>,
    /// Area per visible column, also contains the following spacer if any.
    pub column_areas: Vec<Rect>,
    /// Total footer area.
    pub footer_area: Rect,

    /// Row count.
    pub rows: usize,
    /// Column count.
    pub columns: usize,

    /// Current row offset. Automatically capped at rows-1.
    pub row_offset: usize,
    /// Current column offset. Automatically capped at columns-1.
    pub col_offset: usize,

    /// Current page len as items.
    pub row_page_len: usize,
    /// Current page width as columns.
    pub col_page_len: usize,

    /// Maximum offset for row-scrolling. Can be set higher, but this
    /// offset guarantees a full page display.
    pub max_row_offset: usize,
    /// Maximum offset for column-scrolling. Can be set higher, but this
    /// offset guarantees a full page display.
    pub max_col_offset: usize,

    /// Selection data.
    pub selection: Selection,

    /// Helper for mouse interactions.
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl<'a> Debug for DataRepr<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data").finish()
    }
}

impl<'a> Default for DataRepr<'a> {
    fn default() -> Self {
        Self::Text(TextTableData::default())
    }
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
        let widths = widths.into_iter().map(|v| v.into()).collect::<Vec<_>>();
        let data = TextTableData {
            columns: widths.len(),
            rows: rows.into_iter().map(|v| v.into()).collect(),
        };
        Self {
            data: DataRepr::Text(data),
            widths,
            ..Default::default()
        }
    }

    /// Set preformatted row-data. For compatibility with ratatui.
    ///
    /// Use of [FTable::data] is preferred.
    pub fn rows<T>(mut self, rows: T) -> Self
    where
        T: IntoIterator<Item = Row<'a>>,
    {
        let rows = rows.into_iter().collect();
        match &mut self.data {
            DataRepr::Text(d) => {
                d.rows = rows;
            }
            DataRepr::Ref(_) => {
                unimplemented!("doesn't work that way");
            }
        }
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
    /// // ...
    ///
    /// let tabledata1 = Data1(&my_data_somewhere_else);
    /// let table1 = FTable::default().data(&tabledata1);
    /// table1.render(area, buf, &mut table_state_somewhere_else);
    /// ```
    #[inline]
    pub fn data(mut self, data: &'a dyn TableData<'a>) -> Self {
        self.data = DataRepr::Ref(data);
        self
    }

    /// Set the table-header.
    #[inline]
    pub fn header(mut self, header: Row<'a>) -> Self {
        self.header = Some(header);
        self
    }

    /// Set the table-footer.
    #[inline]
    pub fn footer(mut self, footer: Row<'a>) -> Self {
        self.footer = Some(footer);
        self
    }

    /// Column widths as Constraints.
    pub fn widths<I>(mut self, widths: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        self.widths = widths.into_iter().map(|v| v.into()).collect();

        match &mut self.data {
            DataRepr::Text(v) => {
                v.columns = self.widths.len();
            }
            DataRepr::Ref(_) => {}
        }

        self
    }

    /// Flex for layout.
    #[inline]
    pub fn flex(mut self, flex: Flex) -> Self {
        self.flex = flex;
        self
    }

    /// Spacing between columns.
    #[inline]
    pub fn column_spacing(mut self, spacing: u16) -> Self {
        self.column_spacing = spacing;
        self
    }

    /// Overrides the width of the rendering area for layout purposes.
    /// Layout uses this width, even if it means that some columns are
    /// not visible.
    #[inline]
    pub fn layout_width(mut self, width: u16) -> Self {
        self.layout_width = Some(width);
        self
    }

    /// Draws a block around the table widget.
    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Set all styles as a bundle.
    #[inline]
    pub fn styles(mut self, styles: FTableStyle) -> Self {
        self.style = styles.style;
        self.select_row_style = styles.select_row_style;
        self.select_column_style = styles.select_column_style;
        self.select_cell_style = styles.select_cell_style;
        self.select_header_style = styles.select_header_style;
        self.select_footer_style = styles.select_footer_style;
        self.focus_style = styles.focus_style;
        self
    }

    /// Base style for the table.
    #[inline]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    /// Style for a selected row. The chosen selection must support
    /// row-selection for this to take effect.
    #[inline]
    pub fn select_row_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.select_row_style = select_style.into();
        self
    }

    /// Style for a selected column. The chosen selection must support
    /// column-selection for this to take effect.
    #[inline]
    pub fn select_column_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.select_column_style = select_style.into();
        self
    }

    /// Style for a selected cell. The chosen selection must support
    /// cell-selection for this to take effect.
    #[inline]
    pub fn select_cell_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.select_cell_style = select_style.into();
        self
    }

    /// Style for a selected header cell. The chosen selection must
    /// support column-selection for this to take effect.
    #[inline]
    pub fn select_header_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.select_header_style = select_style.into();
        self
    }

    /// Style for a selected footer cell. The chosen selection must
    /// support column-selection for this to take effect.
    #[inline]
    pub fn select_footer_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.select_footer_style = select_style.into();
        self
    }

    /// This style will be patched onto the selection to indicate that
    /// the widget has the input focus.
    ///
    /// The selection must support some kind of selection for this to
    /// be effective.
    #[inline]
    pub fn focus_style<S: Into<Style>>(mut self, focus_style: S) -> Self {
        self.focus_style = focus_style.into();
        self
    }

    /// Indicates that this widget has the input focus.
    #[inline]
    pub fn focus(mut self, focus: bool) -> Self {
        self.focus = focus;
        self
    }
}

impl<'a, Selection> FTable<'a, Selection> {
    /// Returns a reference to the `TableData`.
    #[inline]
    #[allow(clippy::borrow_deref_ref)]
    pub fn data_ref(&self) -> &dyn TableData<'a> {
        match &self.data {
            DataRepr::Text(v) => &*v,
            DataRepr::Ref(v) => *v,
        }
    }

    /// Does this table need scrollbars?
    /// Returns (horizontal, vertical)
    pub fn need_scroll(&self, area: Rect) -> (bool, bool) {
        //
        // Attention: This must be kept in sync with the actual rendering.
        //

        let data = self.data_ref();
        let (columns, rows) = data.size();

        // vertical layout
        let inner_area = self.block.inner_if_some(area);
        let l_rows = self.layout_areas(inner_area);
        let table_area = l_rows[1];

        // horizontal layout
        let (l_columns, _) = self.layout_columns(columns, table_area.width);

        // maximum offsets
        let vertical = 'f: {
            let mut page_height = 0;
            for r in 0..rows {
                let row_height = data.row_height(r);
                if page_height + row_height >= table_area.height {
                    break 'f true;
                }
                page_height += row_height;
            }
            false
        };
        let horizontal = 'f: {
            for c in 0..columns {
                if l_columns[c].right() >= table_area.width {
                    break 'f true;
                }
            }
            false
        };

        (horizontal, vertical)
    }

    // area_width or layout_width
    #[inline]
    fn total_width(&self, area_width: u16) -> u16 {
        if let Some(layout_width) = self.layout_width {
            layout_width
        } else {
            area_width
        }
    }

    // Do the column-layout. Fill in missing columns, if necessary.
    #[inline]
    fn layout_columns(&self, columns: usize, width: u16) -> (Rc<[Rect]>, Rc<[Rect]>) {
        let mut widths;
        let widths = if self.widths.is_empty() {
            widths = vec![Constraint::Fill(1); columns];
            widths.as_slice()
        } else if self.widths.len() != columns {
            widths = self.widths.clone();
            while widths.len() < columns {
                widths.push(Constraint::Fill(1));
            }
            widths.as_slice()
        } else {
            self.widths.as_slice()
        };

        let area = Rect::new(0, 0, self.total_width(width), 0);

        Layout::horizontal(widths)
            .flex(self.flex)
            .spacing(self.column_spacing)
            .split_with_spacers(area)
    }

    // Layout header/table/footer
    #[inline]
    fn layout_areas(&self, area: Rect) -> Rc<[Rect]> {
        let heights = vec![
            Constraint::Length(self.header.as_ref().map(|v| v.height).unwrap_or(0)),
            Constraint::Fill(1),
            Constraint::Length(self.footer.as_ref().map(|v| v.height).unwrap_or(0)),
        ];

        Layout::vertical(heights).split(area)
    }
}

impl<'a, Selection> Styled for FTable<'a, Selection> {
    type Item = Self;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style<S: Into<Style>>(mut self, style: S) -> Self::Item {
        self.style = style.into();
        self
    }
}

impl<'a, Selection> StatefulWidget for FTable<'a, Selection>
where
    Selection: TableSelection,
{
    type State = FTableState<Selection>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let data = self.data_ref();
        let (columns, rows) = data.size();

        // limits
        if state.row_offset >= rows {
            state.row_offset = rows.saturating_sub(1);
        }
        if state.col_offset >= columns {
            state.col_offset = columns.saturating_sub(1);
        }

        // state
        state.rows = rows;
        state.columns = columns;
        state.area = area;

        // vertical layout
        let inner_area = self.block.inner_if_some(area);
        let l_rows = self.layout_areas(inner_area);
        state.header_area = l_rows[0];
        state.table_area = l_rows[1];
        state.footer_area = l_rows[2];

        // horizontal layout
        let (l_columns, l_spacers) = self.layout_columns(columns, state.table_area.width);

        // maximum offsets
        {
            state.max_row_offset = 0;
            let mut page_height = 0;
            for r in (0..rows).rev() {
                let row_height = data.row_height(r);
                if page_height + row_height >= state.table_area.height {
                    state.max_row_offset = r;
                    break;
                }
                page_height += row_height;
            }
        }
        {
            state.max_col_offset = 0;
            let max_right = l_columns.last().map(|v| v.right()).unwrap_or(0);
            for c in (0..columns).rev() {
                if max_right - l_columns[c].left() >= state.table_area.width {
                    state.max_col_offset = c;
                    break;
                }
            }
        }

        // column areas
        {
            state.column_areas.clear();
            state.col_page_len = 0;

            let mut col = state.col_offset;
            loop {
                if col >= columns {
                    break;
                }

                let column_area = Rect::new(
                    state.table_area.x + l_columns[col].x - l_columns[state.col_offset].x,
                    state.table_area.y,
                    l_columns[col].width + l_spacers[col + 1].width,
                    state.table_area.height,
                )
                .intersection(state.table_area);

                // let space_area = Rect::new(
                //     state.table_area.x + l_spacers[col + 1].x - l_columns[state.col_offset].x,
                //     state.table_area.y,
                //     l_spacers[col + 1].width,
                //     state.table_area.height,
                // )
                // .intersection(state.table_area);

                state.column_areas.push(column_area);
                state.col_page_len += 1;

                if column_area.right() >= state.table_area.right() {
                    break;
                }

                col += 1;
            }
        }

        // render block
        self.block.render(area, buf);

        // render header
        if let Some(header) = &self.header {
            let header_style = if header.style != Style::default() {
                header.style
            } else {
                self.style
            };
            if header_style != Style::default() {
                buf.set_style(state.header_area, header_style);
            }

            let mut col = state.col_offset;
            loop {
                if col >= columns {
                    break;
                }

                let cell_area = Rect::new(
                    state.header_area.x + l_columns[col].x - l_columns[state.col_offset].x,
                    state.header_area.y,
                    l_columns[col].width,
                    state.header_area.height,
                )
                .intersection(state.header_area);

                let space_area = Rect::new(
                    state.header_area.x + l_spacers[col + 1].x - l_columns[state.col_offset].x,
                    state.header_area.y,
                    l_spacers[col + 1].width,
                    state.header_area.height,
                )
                .intersection(state.header_area);

                let mut selected_style = if state.selection.is_selected_column(col) {
                    self.select_header_style
                } else {
                    Style::default()
                };
                if self.focus {
                    selected_style = selected_style.patch(self.focus_style);
                }
                if selected_style != Style::default() {
                    buf.set_style(cell_area, selected_style);
                    buf.set_style(space_area, selected_style);
                }

                if let Some(cell) = header.cells.get(col) {
                    if cell.style != Style::default() {
                        buf.set_style(cell_area, cell.style);
                    }
                    cell.content.render_ref(cell_area, buf);
                }

                if cell_area.right() >= state.header_area.right() {
                    break;
                }

                col += 1;
            }
        }

        // render footer
        if let Some(footer) = &self.footer {
            let footer_style = if footer.style != Style::default() {
                footer.style
            } else {
                self.style
            };
            if footer_style != Style::default() {
                buf.set_style(state.footer_area, footer_style);
            }

            let mut col = state.col_offset;
            loop {
                if col >= columns {
                    break;
                }

                let cell_area = Rect::new(
                    state.footer_area.x + l_columns[col].x - l_columns[state.col_offset].x,
                    state.footer_area.y,
                    l_columns[col].width,
                    state.footer_area.height,
                )
                .intersection(state.footer_area);

                let space_area = Rect::new(
                    state.footer_area.x + l_spacers[col + 1].x - l_columns[state.col_offset].x,
                    state.footer_area.y,
                    l_spacers[col + 1].width,
                    state.footer_area.height,
                )
                .intersection(state.footer_area);

                let mut selected_style = if state.selection.is_selected_column(col) {
                    self.select_footer_style
                } else {
                    Style::default()
                };
                if self.focus {
                    selected_style = selected_style.patch(self.focus_style);
                }
                if selected_style != Style::default() {
                    buf.set_style(cell_area, selected_style);
                    buf.set_style(space_area, selected_style);
                }

                if let Some(cell) = footer.cells.get(col) {
                    if cell.style != Style::default() {
                        buf.set_style(cell_area, cell.style);
                    }
                    cell.content.render_ref(cell_area, buf);
                }

                if cell_area.right() >= state.footer_area.right() {
                    break;
                }

                col += 1;
            }
        }

        // render table
        {
            let table_style = self.style;
            buf.set_style(state.table_area, table_style);

            state.row_areas.clear();
            state.row_page_len = 0;

            let mut row = state.row_offset;
            let mut row_y = state.table_area.y;
            loop {
                if row >= rows {
                    break;
                }

                let row_area = Rect::new(
                    state.table_area.x,
                    row_y,
                    state.table_area.width,
                    data.row_height(row),
                )
                .intersection(state.table_area);

                if data.row_style(row) != Style::default() {
                    buf.set_style(row_area, data.row_style(row));
                }

                state.row_areas.push(row_area);
                state.row_page_len += 1;

                let mut col = state.col_offset;
                loop {
                    if col >= columns {
                        break;
                    }

                    let cell_area = Rect::new(
                        row_area.x + l_columns[col].x - l_columns[state.col_offset].x,
                        row_area.y,
                        l_columns[col].width,
                        row_area.height,
                    )
                    .intersection(state.table_area);

                    let space_area = Rect::new(
                        row_area.x + l_spacers[col + 1].x - l_columns[state.col_offset].x,
                        row_area.y,
                        l_spacers[col + 1].width,
                        row_area.height,
                    )
                    .intersection(state.table_area);

                    let mut select_style = if state.selection.is_selected_cell(col, row) {
                        self.select_cell_style
                    } else if state.selection.is_selected_row(row) {
                        self.select_row_style
                    } else if state.selection.is_selected_column(col) {
                        self.select_column_style
                    } else {
                        Style::default()
                    };
                    if self.focus {
                        select_style = select_style.patch(self.focus_style);
                    }
                    if select_style != Style::default() {
                        buf.set_style(cell_area, select_style);
                        buf.set_style(space_area, select_style);
                    }

                    data.render_cell(col, row, cell_area, buf);

                    if cell_area.right() >= state.table_area.right() {
                        break;
                    }
                    col += 1;
                }

                if row_area.bottom() >= state.table_area.bottom() {
                    break;
                }

                row += 1;
                row_y += row_area.height;
            }
        }
    }
}

impl Default for FTableStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            select_row_style: Default::default(),
            select_column_style: Default::default(),
            select_cell_style: Default::default(),
            select_header_style: Default::default(),
            select_footer_style: Default::default(),
            focus_style: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Selection: Default> Default for FTableState<Selection> {
    fn default() -> Self {
        Self {
            area: Default::default(),
            header_area: Default::default(),
            table_area: Default::default(),
            footer_area: Default::default(),
            row_areas: Default::default(),
            column_areas: Default::default(),
            rows: 0,
            columns: 0,
            row_offset: 0,
            col_offset: 0,
            row_page_len: 0,
            col_page_len: 0,
            max_row_offset: 0,
            max_col_offset: 0,
            selection: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Selection: TableSelection> FTableState<Selection> {
    /// Cell at given position.
    pub fn cell_at_clicked(&self, pos: Position) -> Option<(usize, usize)> {
        let col = self.column_at_clicked(pos);
        let row = self.row_at_clicked(pos);

        match (col, row) {
            (Some(col), Some(row)) => Some((col, row)),
            _ => None,
        }
    }

    /// Column at given position.
    pub fn column_at_clicked(&self, pos: Position) -> Option<usize> {
        rat_event::util::column_at_clicked(&self.column_areas, pos.x).map(|v| self.col_offset + v)
    }

    /// Row at given position.
    pub fn row_at_clicked(&self, pos: Position) -> Option<usize> {
        rat_event::util::row_at_clicked(&self.row_areas, pos.y).map(|v| self.row_offset + v)
    }

    /// Cell when dragging. Can go outside the area.
    pub fn cell_at_drag(&self, pos: Position) -> (usize, usize) {
        let col = self.column_at_drag(pos);
        let row = self.row_at_drag(pos);

        (col, row)
    }

    /// Row when dragging. Can go outside the area.
    pub fn row_at_drag(&self, pos: Position) -> usize {
        match rat_event::util::row_at_drag(self.table_area, &self.row_areas, pos.y) {
            Ok(v) => self.row_offset + v,
            Err(v) if v <= 0 => self.row_offset.saturating_sub((-v) as usize),
            Err(v) => self.row_offset + self.row_areas.len() + v as usize,
        }
    }

    /// Column when dragging. Can go outside the area.
    pub fn column_at_drag(&self, pos: Position) -> usize {
        match rat_event::util::column_at_drag(self.table_area, &self.column_areas, pos.x) {
            Ok(v) => self.col_offset + v,
            Err(v) if v <= 0 => self.col_offset.saturating_sub(1),
            Err(_v) => self.col_offset + self.column_areas.len() + 1,
        }
    }

    /// Change the vertical offset.
    /// Returns true, if there was some change to the offset, even
    /// if clipped.
    pub fn set_row_offset(&mut self, offset: usize) -> bool {
        let old_offset = self.row_offset;
        if offset >= self.rows {
            self.row_offset = self.rows;
        } else {
            self.row_offset = offset;
        }
        old_offset != self.row_offset
    }

    /// Change the horizontal offset.
    /// Returns true, if there was some change to the offset, even
    /// if clipped.
    pub fn set_column_offset(&mut self, offset: usize) -> bool {
        let old_offset = self.col_offset;
        if offset >= self.columns {
            self.col_offset = self.columns;
        } else {
            self.col_offset = offset;
        }
        old_offset != self.col_offset
    }

    /// Scroll up by n items.
    pub fn scroll_up(&mut self, n: usize) -> bool {
        self.set_row_offset(self.row_offset.saturating_sub(n))
    }

    /// Scroll down by n items.
    pub fn scroll_down(&mut self, n: usize) -> bool {
        self.set_row_offset(min(self.row_offset + n, self.max_row_offset))
    }

    /// Scroll up by n items.
    pub fn scroll_left(&mut self, n: usize) -> bool {
        self.set_column_offset(self.col_offset.saturating_sub(n))
    }

    /// Scroll down by n items.
    pub fn scroll_right(&mut self, n: usize) -> bool {
        self.set_column_offset(min(self.col_offset + n, self.max_col_offset))
    }

    /// Scroll to selected.
    pub fn scroll_to_selected(&mut self) {
        if let Some((selected_col, selected_row)) = self.selection.lead_selection() {
            if self.row_offset + self.row_page_len <= selected_row {
                self.set_row_offset(selected_row - self.row_page_len + 1);
            }
            if self.row_offset > selected_row {
                self.set_row_offset(selected_row);
            }

            if self.col_offset + self.col_page_len <= selected_col {
                self.set_column_offset(selected_col - self.col_page_len + 1);
            }
            if self.col_offset > selected_col {
                self.set_column_offset(selected_col);
            }
        }
    }
}

impl FTableState<RowSelection> {
    #[inline]
    pub fn selected(&self) -> Option<usize> {
        self.selection.selected()
    }

    #[inline]
    pub fn select(&mut self, row: Option<usize>) {
        self.selection.select(row);
    }

    /// Select a row, clamp between 0 and maximum.
    #[inline]
    pub fn select_clamped(&mut self, select: usize, maximum: usize) {
        self.selection.select_clamped(select, maximum);
    }
}

impl FTableState<RowSetSelection> {
    #[inline]
    pub fn selected(&self) -> HashSet<usize> {
        self.selection.selected()
    }

    #[inline]
    pub fn set_lead(&mut self, lead: Option<usize>, extend: bool) {
        self.selection.set_lead(lead, extend);
    }

    /// Set a new lead, at the same time limit the lead to max.
    #[inline]
    pub fn set_lead_clamped(&mut self, lead: usize, max: usize, extend: bool) {
        self.selection.set_lead_clamped(lead, max, extend);
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

    /// Clear the selection.
    #[inline]
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Add to selection.
    #[inline]
    pub fn add_selected(&mut self, idx: usize) {
        self.selection.add(idx);
    }

    /// Remove from selection. Only works for retired selections, not for the
    /// active anchor-lead range.
    #[inline]
    pub fn remove_selected(&mut self, idx: usize) {
        self.selection.remove(idx);
    }
}

impl FTableState<CellSelection> {
    /// Selected cell.
    #[inline]
    pub fn selected(&self) -> Option<(usize, usize)> {
        self.selection.selected()
    }

    /// Select a cell.
    #[inline]
    pub fn select_cell(&mut self, select: Option<(usize, usize)>) {
        self.selection.select_cell(select);
    }

    /// Select a row. Column stays the same.
    #[inline]
    pub fn select_row(&mut self, select: Option<usize>) {
        self.selection.select_row(select);
    }

    /// Select a column, row stays the same.
    #[inline]
    pub fn select_column(&mut self, select: Option<usize>) {
        self.selection.select_column(select);
    }

    /// Select a cell, clamp between 0 and maximum.
    #[inline]
    pub fn select_clamped(&mut self, select: (usize, usize), maximum: (usize, usize)) {
        self.selection.select_clamped(select, maximum);
    }
}
