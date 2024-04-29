use crate::_private::NonExhaustive;
///
/// Extensions for [ratatui::widgets::Table]
///
use crate::widget::MouseFlags;
use crate::{
    ct_event, ControlUI, DefaultKeys, FocusFlag, HandleCrossterm, HasFocusFlag, HasScrolling,
    ListSelection, MouseOnly, NoSelection, ScrollParam, ScrolledWidget, SetSelection,
    SingleSelection,
};
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Position, Rect};
use ratatui::prelude::*;
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, HighlightSpacing, Row, Table, TableState};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem;

/// Add some minor fixes to [ratatui::widgets::Table]
#[derive(Debug, Clone)]
pub struct TableExt<'a, Selection> {
    ///
    table: Table<'a>,

    ///
    rows: Vec<Row<'a>>,
    header: Option<Row<'a>>,
    footer: Option<Row<'a>>,

    /// Base style
    style: Style,
    /// Style for selected + not focused.
    select_style: Style,
    /// Style for selected + focused.
    focus_style: Style,

    _phantom: PhantomData<Selection>,
}

impl<'a, Selection> Default for TableExt<'a, Selection> {
    fn default() -> Self {
        Self {
            rows: Default::default(),
            header: None,
            footer: None,
            table: Default::default(),
            style: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<'a, State, Selection: ListSelection> ScrolledWidget<State> for TableExt<'a, Selection> {
    fn need_scroll(&self, _area: Rect, _uistate: &mut State) -> ScrollParam {
        ScrollParam {
            has_hscroll: false,
            has_vscroll: true,
        }
    }
}

/// Combined style.
#[derive(Debug)]
pub struct TableExtStyle {
    pub style: Style,
    pub select_style: Style,
    pub focus_style: Style,
    pub non_exhaustive: NonExhaustive,
}

impl Default for TableExtStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a, Selection> TableExt<'a, Selection> {
    pub fn new<R, C>(rows: R, widths: C) -> Self
    where
        R: IntoIterator,
        R::Item: Into<Row<'a>>,
        C: IntoIterator,
        C::Item: Into<Constraint>,
    {
        let rows = rows.into_iter().map(|v| v.into()).collect::<Vec<_>>();

        Self {
            rows,
            header: None,
            footer: None,
            table: Table::default().widths(widths),
            style: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
            _phantom: Default::default(),
        }
    }

    pub fn rows<T>(mut self, rows: T) -> Self
    where
        T: IntoIterator<Item = Row<'a>>,
    {
        let rows = rows.into_iter().collect::<Vec<_>>();
        self.rows = rows;
        self
    }

    pub fn header(mut self, header: Row<'a>) -> Self {
        self.header = Some(header);
        self
    }

    pub fn footer(mut self, footer: Row<'a>) -> Self {
        self.footer = Some(footer);
        self
    }

    pub fn widths<I>(mut self, widths: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        self.table = self.table.widths(widths);
        self
    }

    pub fn column_spacing(mut self, spacing: u16) -> Self {
        self.table = self.table.column_spacing(spacing);
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.table = self.table.block(block);
        self
    }

    pub fn styles(mut self, styles: TableExtStyle) -> Self {
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

    pub fn select_symbol<T: Into<Text<'a>>>(mut self, select_symbol: T) -> Self {
        self.table = self.table.highlight_symbol(select_symbol);
        self
    }

    pub fn select_spacing(mut self, value: HighlightSpacing) -> Self {
        self.table = self.table.highlight_spacing(value);
        self
    }

    pub fn flex(mut self, flex: Flex) -> Self {
        self.table = self.table.flex(flex);
        self
    }
}

impl<'a, Selection: ListSelection> StatefulWidget for TableExt<'a, Selection> {
    type State = TableExtState<Selection>;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // store to state
        state.area = area;
        state.len = self.rows.len();

        // row layout
        // TODO: as long as height_with_margin() is not accessible we are limited
        //       to single row tables.
        // let header_height = self.header.as_ref().map_or(0, |h| h.height_with_margin());
        // let footer_height = self.footer.as_ref().map_or(0, |f| f.height_with_margin());
        let header_height = 1;
        let footer_height = 1;
        let layout = Layout::vertical([
            Constraint::Length(header_height),
            Constraint::Min(0),
            Constraint::Length(footer_height),
        ])
        .split(area);

        state.header_area = layout[0];
        state.table_area = layout[1];
        state.footer_area = layout[2];
        state.row_areas.clear();
        let mut row_area = Rect::new(layout[1].x, layout[1].y, layout[1].width, 1);
        for _row in self.rows.iter().skip(state.offset()) {
            // TODO: as long as height_with_margin() is not accessible we are limited
            //       to single row tables.
            // row_area.height = row.height_with_margin();
            row_area.height = 1;

            state.row_areas.push(row_area);

            row_area.y += row_area.height;
            if row_area.y >= layout[1].y + layout[1].height {
                break;
            }
        }

        // max_v_offset
        let mut n = 0;
        let mut height = 0;
        for _row in self.rows.iter().rev() {
            // TODO: as long as height_with_margin() is not accessible we are limited
            //       to single row tables.
            // height += row.height_with_margin();
            height += 1;
            if height > layout[1].height {
                break;
            }
            n += 1;
        }
        state.v_page_len = n;
        state.max_v_offset = state.len - n;

        // style selection
        for (i, r) in self.rows.iter_mut().enumerate() {
            let style = if state.focus.get() {
                if state.selection.is_selected(i) {
                    self.focus_style
                } else {
                    self.style
                }
            } else {
                if state.selection.is_selected(i) {
                    self.select_style
                } else {
                    self.style
                }
            };

            *r = mem::take(r).style(style);
        }

        // prepare table widget
        let table = self.table.style(self.style).rows(self.rows);
        let table = if let Some(header) = self.header {
            table.header(header)
        } else {
            table
        };
        let table = if let Some(footer) = self.footer {
            table.footer(footer)
        } else {
            table
        };

        table.render(area, buf, &mut state.table_state);
    }
}

impl<'a, Selection, Item> FromIterator<Item> for TableExt<'a, Selection>
where
    Item: Into<Row<'a>>,
{
    fn from_iter<U: IntoIterator<Item = Item>>(iter: U) -> Self {
        let rows = iter.into_iter().map(|v| v.into()).collect::<Vec<_>>();

        Self {
            rows,
            header: None,
            footer: None,
            table: Table::default(),
            style: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
            _phantom: Default::default(),
        }
    }
}

/// Extended TableState, contains a [ratatui::widgets::TableState].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableExtState<Selection> {
    pub table_state: TableState,

    pub len: usize,
    pub v_page_len: usize,
    pub max_v_offset: usize,

    pub area: Rect,
    pub header_area: Rect,
    pub table_area: Rect,
    pub row_areas: Vec<Rect>,
    pub footer_area: Rect,

    pub focus: FocusFlag,
    pub selection: Selection,

    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl<Selection: Default> Default for TableExtState<Selection> {
    fn default() -> Self {
        Self {
            table_state: Default::default(),
            len: 0,
            v_page_len: 0,
            max_v_offset: 0,
            area: Default::default(),
            header_area: Default::default(),
            table_area: Default::default(),
            row_areas: Default::default(),
            footer_area: Default::default(),
            focus: Default::default(),
            selection: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Selection> HasFocusFlag for TableExtState<Selection> {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl<Selection> HasScrolling for TableExtState<Selection> {
    fn max_v_offset(&self) -> usize {
        self.max_v_offset
    }

    fn max_h_offset(&self) -> usize {
        0
    }

    fn v_page_len(&self) -> usize {
        self.v_page_len
    }

    fn h_page_len(&self) -> usize {
        0
    }

    fn v_offset(&self) -> usize {
        self.table_state.offset()
    }

    fn h_offset(&self) -> usize {
        0
    }

    fn set_v_offset(&mut self, offset: usize) {
        *self.table_state.offset_mut() = offset;
        // For scrolling purposes the selection of ratatui::Table is never None,
        // instead it defaults out to 0 which prohibits any scrolling attempt.

        // We do our own selection, so we don't really care.
        *self.table_state.selected_mut() = Some(offset);
    }

    fn set_h_offset(&mut self, _offset: usize) {
        // It's hard to escape somebody calling this.
        // Gracefully ignoring it seems best.

        // unimplemented!("no horizontal scrolling")
    }
}

impl<Selection: ListSelection> TableExtState<Selection> {
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.table_state = self.table_state.with_offset(offset);
        self
    }

    pub fn offset(&self) -> usize {
        self.table_state.offset()
    }

    pub fn offset_mut(&mut self) -> &mut usize {
        self.table_state.offset_mut()
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    pub fn selection_mut(&mut self) -> &mut Selection {
        &mut self.selection
    }

    /// Row at given position.
    pub fn row_at_clicked(&self, pos: Position) -> Option<usize> {
        for (i, r) in self.row_areas.iter().enumerate() {
            if r.contains(pos) {
                return Some(self.offset() + i);
            }
        }
        None
    }

    /// Row when dragging. Can go outside the area.
    pub fn row_at_drag(&self, pos: Position) -> usize {
        let offset = self.offset();
        for (i, r) in self.row_areas.iter().enumerate() {
            if pos.y >= r.y && pos.y < r.y + r.height {
                return offset + i;
            }
        }

        let offset = self.offset() as isize;
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

impl TableExtState<SingleSelection> {
    /// Returns the lead selection.
    pub fn selected(&self) -> Option<usize> {
        self.selection.lead_selection()
    }

    pub fn select(&mut self, n: Option<usize>) {
        self.selection.select(n)
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>> for TableExtState<NoSelection> {
    fn handle(&mut self, _event: &Event, _keymap: DefaultKeys) -> ControlUI<A, E> {
        ControlUI::Continue
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for TableExtState<NoSelection> {
    fn handle(&mut self, _event: &Event, _keymap: MouseOnly) -> ControlUI<A, E> {
        ControlUI::Continue
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>> for TableExtState<SingleSelection> {
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        let res = if self.is_focused() {
            match event {
                ct_event!(keycode press Down) => {
                    self.selection.next(1, self.len - 1);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press Up) => {
                    self.selection.prev(1);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press End) => {
                    self.selection.select(Some(self.len - 1));
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
                        .next(self.table_area.height as usize / 2, self.len - 1);
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

impl<A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for TableExtState<SingleSelection> {
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
                        self.selection.select_clamped(new_row, self.len - 1);
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
                    self.selection.select_clamped(new_row, self.len - 1);
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

/// Extra mapping which does the double click a line in the table thing.
/// Returns `ControlUI::Action(())` if a double click is detected.
///
/// ```rust ignore
///      uistate.page1.table1.handle(evt, DoubleClick).and_then(|_| {
///          if let Some(file) = data.files[uistate.page1.table1.selected().unwrap()] {
///              let file_path = data
///                  .config
///                  .base_path
///                  .join(format!("{}.ods", &file));
///              Control::Action(RunCalc(file_path))
///          } else {
///              Control::Err(anyhow::Error::msg("Nothing to do."))
///          }
///      })
/// ```
#[derive(Debug)]
pub struct DoubleClick;

impl<E, SEL: ListSelection> HandleCrossterm<ControlUI<usize, E>, DoubleClick>
    for TableExtState<SEL>
{
    fn handle(&mut self, event: &Event, _: DoubleClick) -> ControlUI<usize, E> {
        match event {
            ct_event!(keycode press Enter) => {
                if self.is_focused() {
                    if let Some(lead) = self.selection.lead_selection() {
                        ControlUI::Run(lead)
                    } else {
                        ControlUI::NoChange
                    }
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(mouse up Left for column,row) => {
                let pos = Position::new(*column, *row);
                if self.area.contains(pos) {
                    let Some(sel) = self.row_at_clicked(pos) else {
                        return ControlUI::NoChange;
                    };
                    let Some(lead) = self.selection.lead_selection() else {
                        return ControlUI::NoChange;
                    };

                    if sel == lead {
                        if self.mouse.pull_trigger(200) {
                            ControlUI::Run(lead)
                        } else {
                            ControlUI::NoChange
                        }
                    } else {
                        // can happen if Up and Down switch order.
                        self.mouse.arm_trigger();
                        ControlUI::NoChange
                    }
                } else {
                    ControlUI::Continue
                }
            }
            _ => ControlUI::Continue,
        }
    }
}

/// Extra mapping that reacts to the delete-key in a table.
///
/// Returns [ControlUI::Run(usize)] for which row should be deleted.
#[derive(Debug)]
pub struct DeleteRow;

impl<E, SEL: ListSelection> HandleCrossterm<ControlUI<usize, E>, DeleteRow> for TableExtState<SEL> {
    fn handle(&mut self, event: &Event, _: DeleteRow) -> ControlUI<usize, E> {
        match event {
            ct_event!(keycode press Delete) => {
                if self.focus.get() {
                    if let Some(sel) = self.selection.lead_selection() {
                        ControlUI::Run(sel)
                    } else {
                        ControlUI::NoChange
                    }
                } else {
                    ControlUI::Continue
                }
            }
            _ => ControlUI::Continue,
        }
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>> for TableExtState<SetSelection> {
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        let res = if self.is_focused() {
            match event {
                ct_event!(keycode press Down) => {
                    self.selection.next(1, self.len - 1, false);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press SHIFT-Down) => {
                    self.selection.next(1, self.len - 1, true);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press Up) => {
                    self.selection.prev(1, false);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press SHIFT-Up) => {
                    self.selection.prev(1, true);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press End) => {
                    self.selection.set_lead(Some(self.len - 1), false);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press SHIFT-End) => {
                    self.selection.set_lead(Some(self.len - 1), true);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press Home) => {
                    self.selection.set_lead(Some(0), false);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press SHIFT-Home) => {
                    self.selection.set_lead(Some(0), true);
                    self.scroll_to_selected();
                    ControlUI::Change
                }

                ct_event!(keycode press PageUp) => {
                    self.selection
                        .prev(self.table_area.height as usize / 2, false);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press SHIFT-PageUp) => {
                    self.selection
                        .prev(self.table_area.height as usize / 2, true);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press PageDown) => {
                    self.selection
                        .next(self.table_area.height as usize / 2, self.len - 1, false);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press SHIFT-PageDown) => {
                    self.selection
                        .next(self.table_area.height as usize / 2, self.len - 1, true);
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

impl<A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for TableExtState<SetSelection> {
    fn handle(&mut self, event: &Event, _: MouseOnly) -> ControlUI<A, E> {
        match event {
            ct_event!(scroll up for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_up(self.table_area.height as usize / 5);
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(scroll down for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_down(self.table_area.height as usize / 5);
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
                        self.selection
                            .set_lead_clamped(new_row, self.len - 1, false);
                        ControlUI::Change
                    } else {
                        ControlUI::NoChange
                    }
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(mouse down CONTROL-Left for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    let pos = Position::new(*column, *row);
                    if let Some(new_row) = self.row_at_clicked(pos) {
                        self.mouse.set_drag();
                        self.selection.transfer_lead_anchor();
                        if self.selection.is_selected(new_row) {
                            self.selection.remove(new_row);
                        } else {
                            self.selection.set_lead_clamped(new_row, self.len - 1, true);
                        }
                        ControlUI::Change
                    } else {
                        ControlUI::NoChange
                    }
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(mouse drag Left for column, row)
            | ct_event!(mouse drag CONTROL-Left for column, row) => {
                if self.mouse.do_drag() {
                    let pos = Position::new(*column, *row);
                    let new_row = self.row_at_drag(pos);
                    self.selection.set_lead_clamped(new_row, self.len - 1, true);
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
