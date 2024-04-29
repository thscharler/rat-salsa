use crate::_private::NonExhaustive;
use crate::rere::ftable::text::{TextRow, TextTableData};
use crate::util::{DynBorrow, RCow};
use crate::widget::MouseFlags;
use crate::{
    ct_event, ControlUI, DefaultKeys, FocusFlag, HandleCrossterm, HasFocusFlag, HasScrolling,
    ListSelection, MouseOnly, NoSelection, ScrollParam, ScrolledWidget, SingleSelection,
};
use crossterm::event::Event;
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Position, Rect};
use ratatui::prelude::BlockExt;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, StatefulWidget};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

/// Trait for the table-data.
pub trait TableData<'a> {
    fn rows(&self) -> usize;

    fn columns(&self) -> usize;

    fn row_height(&self, row: usize) -> u16;

    fn col_width(&self, column: usize) -> u16;

    fn render_cell(&self, column: usize, row: usize, style: Style, area: Rect, buf: &mut Buffer);
}

/// Furious table.
pub struct FTable<'a, 'b: 'a, Selection> {
    data: RCow<'b, TextTableData<'a>, &'b dyn TableData<'a>>,

    widths: Vec<Constraint>,
    flex: Flex,
    column_spacing: u16,

    block: Option<Block<'a>>,

    /// Base style
    style: Style,
    /// Style for selected + not focused.
    select_style: Style,
    /// Style for selected + focused.
    focus_style: Style,

    _phantom: PhantomData<Selection>,
}

impl<'a, 'b: 'a, Selection: Debug> Debug for FTable<'a, 'b, Selection> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FTable")
            .field("rows", &self.data.dyn_borrow().rows())
            .field("columns", &self.data.dyn_borrow().columns())
            .field("style", &self.style)
            .field("select_style", &self.select_style)
            .field("focus_style", &self.focus_style)
            .finish()
    }
}

impl<'a, 'b: 'a, Selection: Default> Default for FTable<'a, 'b, Selection> {
    fn default() -> Self {
        Self {
            data: RCow::Owned(Default::default()),
            widths: Default::default(),
            flex: Default::default(),
            column_spacing: 0,
            block: None,
            style: Default::default(),
            select_style: Style::default().add_modifier(Modifier::REVERSED),
            focus_style: Style::default().add_modifier(Modifier::REVERSED),
            _phantom: Default::default(),
        }
    }
}

impl<'a, 'b: 'a, Selection> FTable<'a, 'b, Selection> {
    pub fn new<R, C>(rows: R, widths: C) -> Self
    where
        R: IntoIterator,
        R::Item: Into<TextRow<'a>>,
        C: IntoIterator,
        C::Item: Into<Constraint>,
        Selection: Default,
    {
        let data = TextTableData {
            rows: rows.into_iter().map(|v| v.into()).collect(),
        };

        Self {
            data: RCow::Owned(data),
            widths: widths.into_iter().map(|v| v.into()).collect(),
            ..Default::default()
        }
    }

    pub fn rows<T>(mut self, rows: T) -> Self
    where
        T: IntoIterator<Item = TextRow<'a>>,
    {
        let rows = rows.into_iter().collect();
        match &mut self.data {
            RCow::Borrowed(_) => {
                unimplemented!("doesn't work that way");
            }
            RCow::Owned(o) => {
                o.rows = rows;
            }
            RCow::Phantom(_) => {
                unreachable!()
            }
        }
        self
    }

    pub fn data(mut self, data: &'a dyn TableData<'a>) -> Self {
        self.data = RCow::Borrowed(data);
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

    // todo: header
    // todo: footer

    pub fn column_spacing(mut self, spacing: u16) -> Self {
        self.column_spacing = spacing;
        self
    }

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

    pub fn flex(mut self, flex: Flex) -> Self {
        self.flex = flex;
        self
    }
}

impl<'a, 'b: 'a, Selection: ListSelection> StatefulWidget for FTable<'a, 'b, Selection> {
    type State = FTableState<Selection>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let data = self.data.dyn_borrow();

        state.area = self.block.inner_if_some(area);

        state.table_area = area;
        state.row_areas.clear();

        state.row_len = data.rows();
        state.col_len = data.columns();
        state.row_page_len = 1; // todo
        state.col_page_len = 1; // todo
        state.max_row_offset = data.rows(); // todo
        state.max_col_offset = data.columns(); // todo

        let mut row = state.row_offset;
        let mut y = state.table_area.y;
        loop {
            let row_height = data.row_height(row);

            state.row_areas.push(Rect::new(
                state.table_area.x,
                state.table_area.y,
                state.table_area.width,
                row_height,
            ));

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
            let mut x = state.table_area.x;
            loop {
                let col_width = data.col_width(col);

                let cell_area = Rect::new(x, y, col_width, row_height).intersection(area);

                data.render_cell(col, row, style, cell_area, buf);

                if x + col_width >= state.table_area.right() {
                    break;
                }

                col += 1;
                x += col_width;
            }

            if y + row_height >= state.table_area.bottom() {
                break;
            }

            row += 1;
            y += row_height;
        }
    }
}

impl<'a, 'b: 'a, State, Selection> ScrolledWidget<State> for FTable<'a, 'b, Selection> {
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

    pub row_len: usize,
    pub col_len: usize,

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
            row_len: 0,
            col_len: 0,
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

    fn set_v_offset(&mut self, offset: usize) {
        self.row_offset = offset;
    }

    fn set_h_offset(&mut self, offset: usize) {
        self.col_offset = offset
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
                    self.selection.next(1, self.row_len - 1);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press Up) => {
                    self.selection.prev(1);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press End) => {
                    self.selection.select(Some(self.row_len - 1));
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
                        .next(self.table_area.height as usize / 2, self.row_len - 1);
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
                        self.selection.select_clamped(new_row, self.row_len - 1);
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
                    self.selection.select_clamped(new_row, self.row_len - 1);
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
    use crate::util::DynBorrow;
    use log::debug;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::Style;
    use ratatui::text::Text;
    use ratatui::widgets::{Cell, WidgetRef};

    #[derive(Debug, Default, Clone)]
    pub struct TextTableData<'a> {
        pub rows: Vec<TextRow<'a>>,
    }

    #[derive(Debug, Default, Clone)]
    pub struct TextRow<'a> {
        pub cells: Vec<Text<'a>>,
    }

    impl<'a, 'b> DynBorrow<'b, &'b dyn TableData<'a>> for TextTableData<'a> {
        fn dyn_borrow(&'b self) -> &'b dyn TableData<'a>
        where
            &'b dyn TableData<'a>: 'b,
        {
            &*self
        }
    }

    impl<'a> TableData<'a> for TextTableData<'a> {
        fn rows(&self) -> usize {
            self.rows.len()
        }

        fn columns(&self) -> usize {
            0
        }

        fn row_height(&self, r: usize) -> u16 {
            if let Some(row) = self.rows.get(r) {
                row.height()
            } else {
                0
            }
        }

        fn col_width(&self, c: usize) -> u16 {
            // if let Some(col) = self.col_width.get(c) {
            //     *col
            // } else {
            //     0
            // }
            0
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
