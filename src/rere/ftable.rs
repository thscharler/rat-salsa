use crate::_private::NonExhaustive;
use crate::rere::ftable::text::{TextRow, TextTableData};
use crate::widget::MouseFlags;
use crate::{
    ct_event, ControlUI, DefaultKeys, FocusFlag, HandleCrossterm, HasFocusFlag, HasScrolling,
    ListSelection, MouseOnly, NoSelection, ScrollOutcome, ScrollParam, ScrolledWidget,
    SingleSelection,
};
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Position, Rect};
use ratatui::prelude::BlockExt;
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::widgets::{Block, StatefulWidget};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::rc::Rc;

/// Trait for the table-data.
///
/// Implement this trait on a struct that holds a reference to your data.
pub trait TableData<'a> {
    fn size(&self) -> (usize, usize);

    fn row_height(&self, row: usize) -> u16;

    fn render_cell(&self, column: usize, row: usize, style: Style, area: Rect, buf: &mut Buffer);
}

/// Furious table.
///
///
///
///
pub struct FTable<'a, Selection> {
    data: DataRepr<'a>,

    widths: Vec<Constraint>,
    flex: Flex,
    column_spacing: u16,
    layout_width: Option<u16>,

    block: Option<Block<'a>>,

    /// Base style
    style: Style,
    /// Style for selected + not focused.
    select_style: Style,
    /// Style for selected + focused.
    focus_style: Style,

    _phantom: PhantomData<Selection>,
}

enum DataRepr<'a> {
    Text(TextTableData<'a>),
    Ref(&'a dyn TableData<'a>),
}

impl<'a, Selection: Debug> Debug for FTable<'a, Selection> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (columns, rows) = self.data_ref().size();
        f.debug_struct("FTable")
            .field("rows", &rows)
            .field("columns", &columns)
            .field("style", &self.style)
            .field("select_style", &self.select_style)
            .field("focus_style", &self.focus_style)
            .finish()
    }
}

impl<'a, Selection: Default> Default for FTable<'a, Selection> {
    fn default() -> Self {
        Self {
            data: DataRepr::Text(Default::default()),
            widths: Default::default(),
            flex: Default::default(),
            column_spacing: 0,
            layout_width: None,
            block: None,
            style: Default::default(),
            select_style: Style::default().add_modifier(Modifier::REVERSED),
            focus_style: Style::default().add_modifier(Modifier::REVERSED),
            _phantom: Default::default(),
        }
    }
}

impl<'a, Selection> FTable<'a, Selection> {
    pub fn new<R, C>(rows: R, widths: C) -> Self
    where
        R: IntoIterator,
        R::Item: Into<TextRow<'a>>,
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

    pub fn rows<T>(mut self, rows: T) -> Self
    where
        T: IntoIterator<Item = TextRow<'a>>,
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

    pub fn data(mut self, data: &'a dyn TableData<'a>) -> Self {
        self.data = DataRepr::Ref(data);
        self
    }

    pub fn widths<I>(mut self, widths: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        self.widths = widths.into_iter().map(|v| v.into()).collect();
        self
    }

    pub fn flex(mut self, flex: Flex) -> Self {
        self.flex = flex;
        self
    }

    pub fn column_spacing(mut self, spacing: u16) -> Self {
        self.column_spacing = spacing;
        self
    }

    pub fn layout_width(mut self, width: u16) -> Self {
        self.layout_width = Some(width);
        self
    }

    // todo: header
    // todo: footer

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn styles(mut self, styles: FTableStyle) -> Self {
        self.style = styles.style;
        self.select_style = styles.select_style;
        self.focus_style = styles.focus_style;
        self
    }

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    pub fn select_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.select_style = select_style.into();
        self
    }

    pub fn focus_style<S: Into<Style>>(mut self, focus_style: S) -> Self {
        self.focus_style = focus_style.into();
        self
    }

    // todo: select_symbol

    // todo: select_spacing
}

impl<'a, Selection> FTable<'a, Selection> {
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

    fn layout_columns(&self, columns: usize, mut area: Rect) -> (Rc<[Rect]>, Rc<[Rect]>) {
        let widths;
        let widths = if self.widths.is_empty() {
            widths = vec![Constraint::Fill(1); columns];
            widths.as_slice()
        } else {
            self.widths.as_slice()
        };

        area.x = 0;
        area.y = 0;
        area.height = 0;
        area.width = self.total_width(area.width);

        Layout::horizontal(widths)
            .flex(self.flex)
            .spacing(self.column_spacing)
            .split_with_spacers(area)
    }
}

impl<'a, Selection: ListSelection> StatefulWidget for FTable<'a, Selection> {
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
        state.area = self.block.inner_if_some(area);

        // vertical layout
        let l_rows = Layout::vertical([
            Constraint::Length(0),
            Constraint::Fill(1),
            Constraint::Length(0),
        ])
        .split(state.area);
        // todo: header, footer
        state.table_area = l_rows[1];

        // horizontal layout
        let (l_columns, l_spacers) = self.layout_columns(columns, state.table_area);
        assert_eq!(l_columns.len(), columns);

        // render visible
        state.row_areas.clear();
        state.row_page_len = 0;
        state.col_page_len = 0;

        let mut row = state.row_offset;
        let mut row_area = Rect::new(
            state.table_area.x,
            state.table_area.y,
            state.table_area.width,
            0,
        );
        loop {
            row_area.height = data.row_height(row);

            state.row_areas.push(row_area);
            state.row_page_len += 1;

            let style = if state.selection.is_selected(row) {
                if state.is_focused() {
                    self.focus_style
                } else {
                    self.select_style
                }
            } else {
                self.style
            };

            let mut col = state.col_offset;
            let x0 = l_columns[col].x;
            loop {
                let cell_area = Rect::new(
                    row_area.x + l_columns[col].x - x0,
                    row_area.y,
                    l_columns[col].width,
                    row_area.height,
                )
                .intersection(area);

                let space_area = Rect::new(
                    row_area.x + l_spacers[col + 1].x - x0,
                    row_area.y,
                    l_spacers[col + 1].width,
                    row_area.height,
                )
                .intersection(area);

                state.col_page_len += 1;

                buf.set_style(space_area, style.on_gray());
                buf.set_style(cell_area, style);
                data.render_cell(col, row, style, cell_area, buf);

                if cell_area.right() >= state.table_area.right() {
                    break;
                }
                if col + 1 >= columns {
                    break;
                }

                col += 1;
            }

            if row_area.bottom() >= state.table_area.bottom() {
                break;
            }
            if row + 1 >= rows {
                break;
            }

            row += 1;
            row_area.y += row_area.height;
        }

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
                if total_right - rect.right() > state.area.width {
                    state.max_col_offset = c;
                    break 'f;
                }
            }
            state.max_col_offset = 0;
        }
    }
}

impl<'a, State, Selection> ScrolledWidget<State> for FTable<'a, Selection> {
    fn need_scroll(&self, _area: Rect, _state: &mut State) -> ScrollParam {
        // todo: something better
        ScrollParam {
            has_hscroll: true,
            has_vscroll: true,
        }
    }
}

/// Combined style.
#[derive(Debug)]
pub struct FTableStyle {
    pub style: Style,
    pub select_style: Style,
    pub focus_style: Style,
    pub non_exhaustive: NonExhaustive,
}

impl Default for FTableStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FTableState<Selection> {
    pub area: Rect,
    pub table_area: Rect,
    pub row_areas: Vec<Rect>,

    pub rows: usize,
    pub columns: usize,

    pub row_offset: usize,
    pub col_offset: usize,

    pub row_page_len: usize,
    pub col_page_len: usize,

    pub max_row_offset: usize,
    pub max_col_offset: usize,

    pub focus: FocusFlag,
    pub selection: Selection,

    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl<Selection: Default> Default for FTableState<Selection> {
    fn default() -> Self {
        Self {
            area: Default::default(),
            table_area: Default::default(),
            row_areas: Default::default(),
            rows: 0,
            columns: 0,
            row_offset: 0,
            col_offset: 0,
            row_page_len: 0,
            col_page_len: 0,
            max_row_offset: 0,
            max_col_offset: 0,
            focus: Default::default(),
            selection: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Selection> HasFocusFlag for FTableState<Selection> {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl<Selection> HasScrolling for FTableState<Selection> {
    fn max_v_offset(&self) -> usize {
        self.max_row_offset
    }

    fn max_h_offset(&self) -> usize {
        self.max_col_offset
    }

    fn v_page_len(&self) -> usize {
        self.row_page_len
    }

    fn h_page_len(&self) -> usize {
        self.col_page_len
    }

    fn v_offset(&self) -> usize {
        self.row_offset
    }

    fn h_offset(&self) -> usize {
        self.col_offset
    }

    fn set_v_offset(&mut self, offset: usize) -> ScrollOutcome {
        if offset < self.rows {
            self.row_offset = offset;
            ScrollOutcome::Exact
        } else if self.row_offset == self.rows.saturating_sub(1) {
            ScrollOutcome::AtLimit
        } else {
            self.row_offset = self.rows.saturating_sub(1);
            ScrollOutcome::Limited
        }
    }

    fn set_h_offset(&mut self, offset: usize) -> ScrollOutcome {
        if offset < self.columns {
            self.col_offset = offset;
            ScrollOutcome::Exact
        } else if self.col_offset == self.columns.saturating_sub(1) {
            ScrollOutcome::AtLimit
        } else {
            self.col_offset = self.columns.saturating_sub(1);
            ScrollOutcome::Limited
        }
    }
}

impl<Selection> FTableState<Selection> {
    /// Row at given position.
    pub fn row_at_clicked(&self, pos: Position) -> Option<usize> {
        for (i, r) in self.row_areas.iter().enumerate() {
            if r.contains(pos) {
                return Some(self.v_offset() + i);
            }
        }
        None
    }

    /// Row when dragging. Can go outside the area.
    pub fn row_at_drag(&self, pos: Position) -> usize {
        let offset = self.v_offset();
        for (i, r) in self.row_areas.iter().enumerate() {
            if pos.y >= r.y && pos.y < r.y + r.height {
                return offset + i;
            }
        }

        let offset = self.v_offset() as isize;
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

impl<Selection: ListSelection> FTableState<Selection> {
    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    pub fn selection_mut(&mut self) -> &mut Selection {
        &mut self.selection
    }

    /// Scroll to selected.
    pub fn scroll_to_selected(&mut self) {
        if let Some(selected) = self.selection.lead_selection() {
            if self.v_offset() + self.row_areas.len() <= selected {
                self.set_v_offset(selected - self.row_areas.len() + 1);
            }
            if self.v_offset() > selected {
                self.set_v_offset(selected);
            }
        }
    }
}

impl FTableState<SingleSelection> {
    /// Returns the lead selection.
    pub fn selected(&self) -> Option<usize> {
        self.selection.lead_selection()
    }

    pub fn select(&mut self, n: Option<usize>) {
        self.selection.select(n)
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>> for FTableState<NoSelection> {
    fn handle(&mut self, _event: &Event, _keymap: DefaultKeys) -> ControlUI<A, E> {
        ControlUI::Continue
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for FTableState<NoSelection> {
    fn handle(&mut self, _event: &Event, _keymap: MouseOnly) -> ControlUI<A, E> {
        ControlUI::Continue
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>> for FTableState<SingleSelection> {
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        let res = if self.is_focused() {
            match event {
                ct_event!(keycode press Down) => {
                    self.selection.next(1, self.rows - 1);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press Up) => {
                    self.selection.prev(1);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press End) => {
                    self.selection.select(Some(self.rows - 1));
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press Home) => {
                    self.selection.select(Some(0));
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press PageUp) => {
                    self.selection.prev(self.table_area.height as usize / 2);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press PageDown) => {
                    self.selection
                        .next(self.table_area.height as usize / 2, self.rows - 1);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                _ => ControlUI::Continue,
            }
        } else {
            ControlUI::Continue
        };

        res.on_continue(|| {
            <Self as HandleCrossterm<ControlUI<A, E>, MouseOnly>>::handle(self, event, MouseOnly)
        })
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for FTableState<SingleSelection> {
    fn handle(&mut self, event: &Event, _: MouseOnly) -> ControlUI<A, E> {
        match event {
            ct_event!(scroll down for column,row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_down(self.table_area.height as usize / 10);
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(scroll up for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_up(self.table_area.height as usize / 10);
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(mouse down Left for column, row) => {
                let pos = Position::new(*column, *row);
                if self.area.contains(pos) {
                    if let Some(new_row) = self.row_at_clicked(pos) {
                        self.mouse.set_drag();
                        self.selection.select_clamped(new_row, self.rows - 1);
                        ControlUI::Change
                    } else {
                        ControlUI::NoChange
                    }
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(mouse drag Left for column, row) => {
                if self.mouse.do_drag() {
                    let pos = Position::new(*column, *row);
                    let new_row = self.row_at_drag(pos);
                    self.mouse.set_drag();
                    self.selection.select_clamped(new_row, self.rows - 1);
                    self.scroll_to_selected();
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(mouse moved) => {
                self.mouse.clear_drag();
                ControlUI::Continue
            }

            _ => ControlUI::Continue,
        }
    }
}

pub mod text {
    use crate::rere::ftable::TableData;
    #[allow(unused_imports)]
    use log::debug;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::Style;
    use ratatui::text::Text;
    use ratatui::widgets::WidgetRef;

    #[derive(Debug, Default, Clone)]
    pub struct TextTableData<'a> {
        pub columns: usize,
        pub rows: Vec<TextRow<'a>>,
    }

    #[derive(Debug, Default, Clone)]
    pub struct TextRow<'a> {
        pub cells: Vec<Text<'a>>,
    }

    impl<'a> TableData<'a> for TextTableData<'a> {
        fn size(&self) -> (usize, usize) {
            (self.columns, self.rows.len())
        }

        fn row_height(&self, r: usize) -> u16 {
            if let Some(row) = self.rows.get(r) {
                row.height()
            } else {
                0
            }
        }

        fn render_cell(&self, c: usize, r: usize, style: Style, area: Rect, buf: &mut Buffer) {
            buf.set_style(area, style);
            if let Some(row) = self.rows.get(r) {
                if let Some(text) = row.cell(c) {
                    let mut text = text.clone();

                    // debug!("render_cell {}|{} area {:?}  {:?}", c, r, area, text);

                    text.style = style.patch(text.style);
                    text.render_ref(area, buf)
                }
            }
        }
    }

    impl<'a> TextRow<'a> {
        pub fn new<T>(cells: T) -> Self
        where
            T: IntoIterator,
            T::Item: Into<Text<'a>>,
        {
            Self {
                cells: cells.into_iter().map(|v| v.into()).collect(),
            }
        }

        pub fn cells<T>(mut self, cells: T) -> Self
        where
            T: IntoIterator,
            T::Item: Into<Text<'a>>,
        {
            self.cells = cells.into_iter().map(Into::into).collect();
            self
        }

        // todo: top_margin, bottom_margin, style?

        pub fn height(&self) -> u16 {
            self.cells.iter().map(|v| v.height()).max().unwrap_or(1) as u16
        }

        pub fn cell<'b: 'a>(&'b self, c: usize) -> Option<&'a Text<'a>> {
            if let Some(t) = self.cells.get(c) {
                Some(t)
            } else {
                None
            }
        }
    }
}
