use crate::_private::NonExhaustive;
use crate::textdata::{Row, TextTableData};
use crate::util::MouseFlags;
use crate::{TableData, TableSelection};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Position, Rect};
use ratatui::style::Style;
use ratatui::widgets::{StatefulWidget, StatefulWidgetRef, WidgetRef};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Debug, Default, Clone)]
pub struct FTable<'a, Selection> {
    data: DataRepr<'a>,

    header: Option<Row<'a>>,
    footer: Option<Row<'a>>,

    widths: Vec<Constraint>,
    flex: Flex,
    column_spacing: u16,
    layout_width: Option<u16>,

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

#[derive(Debug, Clone)]
pub struct FTableState<Selection> {
    pub area: Rect,

    /// Total header area.
    pub header_area: Rect,
    /// Total table area.
    pub table_area: Rect,
    /// Area per visible row.
    pub row_areas: Vec<Rect>,
    /// Total footer area.
    pub footer_area: Rect,

    /// Rects for the columns. Only x and width are actually set.
    pub column_areas: Vec<Rect>,

    pub rows: usize,
    pub columns: usize,

    pub row_offset: usize,
    pub col_offset: usize,

    pub row_page_len: usize,
    pub col_page_len: usize,

    pub max_row_offset: usize,
    pub max_col_offset: usize,

    pub selection: Selection,

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
            ..Default::default()
        };
        Self {
            data: DataRepr::Text(data),
            widths,
            ..Default::default()
        }
    }

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

    #[inline]
    pub fn data(mut self, data: &'a dyn TableData<'a>) -> Self {
        self.data = DataRepr::Ref(data);
        self
    }

    #[inline]
    pub fn header(mut self, header: Row<'a>) -> Self {
        self.header = Some(header);
        self
    }

    #[inline]
    pub fn footer(mut self, footer: Row<'a>) -> Self {
        self.footer = Some(footer);
        self
    }

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

    #[inline]
    pub fn flex(mut self, flex: Flex) -> Self {
        self.flex = flex;
        self
    }

    #[inline]
    pub fn column_spacing(mut self, spacing: u16) -> Self {
        self.column_spacing = spacing;
        self
    }

    #[inline]
    pub fn layout_width(mut self, width: u16) -> Self {
        self.layout_width = Some(width);
        self
    }

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

    #[inline]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    #[inline]
    pub fn select_row_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.select_row_style = select_style.into();
        self
    }

    #[inline]
    pub fn select_column_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.select_column_style = select_style.into();
        self
    }

    #[inline]
    pub fn select_cell_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.select_cell_style = select_style.into();
        self
    }

    #[inline]
    pub fn select_header_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.select_header_style = select_style.into();
        self
    }

    #[inline]
    pub fn select_footer_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.select_footer_style = select_style.into();
        self
    }

    #[inline]
    pub fn focus_style<S: Into<Style>>(mut self, focus_style: S) -> Self {
        self.focus_style = focus_style.into();
        self
    }

    #[inline]
    pub fn focus(mut self, focus: bool) -> Self {
        self.focus = focus;
        self
    }
}

impl<'a, Selection> FTable<'a, Selection> {
    #[inline]
    fn data_ref(&self) -> &dyn TableData<'a> {
        match &self.data {
            DataRepr::Text(v) => &*v,
            DataRepr::Ref(v) => *v,
        }
    }

    fn total_width(&self, area_width: u16) -> u16 {
        if let Some(layout_width) = self.layout_width {
            layout_width
        } else {
            area_width
        }
    }

    #[inline]
    fn layout_columns(&self, columns: usize, width: u16) -> (Rc<[Rect]>, Rc<[Rect]>) {
        let widths;
        let widths = if self.widths.is_empty() {
            widths = vec![Constraint::Fill(1); columns];
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

impl<'a, Selection> StatefulWidget for FTable<'a, Selection>
where
    Selection: TableSelection,
{
    type State = FTableState<Selection>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_ref(area, buf, state);
    }
}

impl<'a, Selection> StatefulWidgetRef for FTable<'a, Selection>
where
    Selection: TableSelection,
{
    type State = FTableState<Selection>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
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
        let l_rows = self.layout_areas(area);
        state.header_area = l_rows[0];
        state.table_area = l_rows[1];
        state.footer_area = l_rows[2];

        // horizontal layout
        let (l_columns, l_spacers) = self.layout_columns(columns, state.area.width);

        // maximum offsets
        'f: {
            let mut page_height = 0;
            for r in (0..rows).rev() {
                let row_height = data.row_height(r);
                if page_height + row_height >= state.area.height {
                    state.max_row_offset = r + 1;
                    break 'f;
                }
                page_height += row_height;
            }
            state.max_row_offset = 0;
        }
        'f: {
            let total_width = self.total_width(state.area.width);
            let total_right = state.area.x + total_width;
            for (c, rect) in l_columns.iter().rev().enumerate() {
                if total_right - rect.left() < state.area.width {
                    state.max_col_offset = c;
                    break 'f;
                }
            }
            state.max_col_offset = 0;
        }

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
                state.col_page_len = 0; // reset here, a bit weird though

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

                    state.col_page_len += 1;

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

impl<Selection> FTableState<Selection> {
    /// Row at given position.
    pub fn row_at_clicked(&self, pos: Position) -> Option<usize> {
        for (i, r) in self.row_areas.iter().enumerate() {
            if r.contains(pos) {
                return Some(self.row_offset + i);
            }
        }
        None
    }

    /// Row when dragging. Can go outside the area.
    pub fn row_at_drag(&self, pos: Position) -> usize {
        let offset = self.row_offset;
        for (i, r) in self.row_areas.iter().enumerate() {
            if pos.y >= r.y && pos.y < r.y + r.height {
                return offset + i;
            }
        }

        let offset = self.row_offset as isize;
        let rr = if pos.y < self.table_area.y {
            // assume row-height=1 for outside the box.
            let min_row = self.table_area.y as isize;
            offset - (min_row - pos.y as isize)
        } else if pos.y >= self.table_area.y + self.table_area.height {
            let max_row = self.table_area.y as isize + self.table_area.height as isize;
            let vis_rows = self.row_areas.len() as isize;
            offset + vis_rows + (pos.y as isize - max_row)
        } else {
            if let Some(last) = self.row_areas.last() {
                // count from last row.
                let min_row = last.y as isize + last.height as isize;
                let vis_rows = self.row_areas.len() as isize;
                offset + vis_rows + (pos.y as isize - min_row)
            } else {
                // empty table, count from header
                let min_row = self.table_area.y as isize + self.table_area.height as isize;
                offset + (pos.y as isize - min_row)
            }
        };
        if rr < 0 {
            0
        } else {
            rr as usize
        }
    }
}

impl<Selection: TableSelection> FTableState<Selection> {
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
        self.set_row_offset(self.row_offset + n)
    }

    /// Scroll up by n items.
    pub fn scroll_left(&mut self, n: usize) -> bool {
        self.set_column_offset(self.col_offset.saturating_sub(n))
    }

    /// Scroll down by n items.
    pub fn scroll_right(&mut self, n: usize) -> bool {
        self.set_column_offset(self.col_offset + n)
    }
}

impl<Selection: TableSelection> FTableState<Selection> {
    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    pub fn selection_mut(&mut self) -> &mut Selection {
        &mut self.selection
    }

    /// Scroll to selected.
    pub fn scroll_to_selected(&mut self) {
        if let Some((sel_col, sel_row)) = self.selection.lead_selection() {
            self.row_offset = sel_row;
            self.col_offset = sel_col;
        }
    }
}
