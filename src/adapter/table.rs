///
/// Extensions for [ratatui::widgets::Table]
///
use crate::_private::NonExhaustive;
use crate::events::Outcome;
use crate::{ScrollingOutcome, ScrollingState, ScrollingWidget};
use rat_event::{ct_event, FocusKeys, HandleEvent, MouseOnly};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Position, Rect};
use ratatui::prelude::*;
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, HighlightSpacing, Row, Table, TableState};
use std::cmp::{max, min};
use std::fmt::Debug;
use std::time::SystemTime;

/// Add some minor fixes to [ratatui::widgets::Table]
#[derive(Debug, Clone)]
pub struct TableS<'a> {
    ///
    table: Table<'a>,

    ///
    rows: Vec<Row<'a>>,
    header: Option<Row<'a>>,
    footer: Option<Row<'a>>,

    ///
    scroll_selection: bool,
    scroll_by: Option<usize>,
}

impl<'a> Default for TableS<'a> {
    fn default() -> Self {
        Self {
            rows: Default::default(),
            header: None,
            footer: None,
            table: Default::default(),
            scroll_selection: false,
            scroll_by: None,
        }
    }
}

impl<'a, State> ScrollingWidget<State> for TableS<'a> {
    fn need_scroll(&self, area: Rect, _uistate: &mut State) -> (bool, bool) {
        let v_scroll = 'f: {
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

            let mut height = 0;
            for _row in self.rows.iter() {
                // TODO: as long as height_with_margin() is not accessible we are limited
                //       to single row tables.
                // row_area.height = row.height_with_margin();
                let row_height = 1;

                height += row_height;
                if height >= layout[1].height {
                    break 'f true;
                }
            }
            false
        };

        (false, v_scroll)
    }
}

impl<'a> TableS<'a> {
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
            scroll_selection: false,
            scroll_by: None,
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

    pub fn scroll_by(mut self, n: usize) -> Self {
        self.scroll_by = Some(n);
        self
    }

    pub fn scroll_selection(mut self) -> Self {
        self.scroll_selection = true;
        self
    }

    pub fn scroll_offset(mut self) -> Self {
        self.scroll_selection = false;
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

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.table = self.table.style(style);
        self
    }

    pub fn highlight_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.table = self.table.highlight_style(style);
        self
    }

    pub fn highlight_symbol<T: Into<Text<'a>>>(mut self, select_symbol: T) -> Self {
        self.table = self.table.highlight_symbol(select_symbol);
        self
    }

    pub fn highlight_spacing(mut self, value: HighlightSpacing) -> Self {
        self.table = self.table.highlight_spacing(value);
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

impl<'a> StatefulWidget for TableS<'a> {
    type State = TableSState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // store to state
        state.area = area;
        state.scroll_selection = self.scroll_selection;
        state.v_len = self.rows.len();

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
        state.v_page = n;
        state.v_scroll_by = if let Some(scroll_by) = self.scroll_by {
            scroll_by
        } else {
            max(state.v_page / 10, 1)
        };
        state.v_max_offset = state.v_len - n;

        // prepare table widget
        let table = self.table.rows(self.rows);
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

        StatefulWidget::render(table, area, buf, &mut state.widget);
    }
}

impl<'a, Item> FromIterator<Item> for TableS<'a>
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
            scroll_selection: false,
            scroll_by: None,
        }
    }
}

/// Extended TableState, contains a [ratatui::widgets::TableState].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableSState {
    pub widget: TableState,

    pub scroll_selection: bool,
    pub v_scroll_by: usize,
    pub v_len: usize,
    pub v_page: usize,
    pub v_max_offset: usize,

    pub area: Rect,
    pub header_area: Rect,
    pub table_area: Rect,
    pub row_areas: Vec<Rect>,
    pub footer_area: Rect,

    pub mouse_drag: bool,
    pub mouse_click: Option<SystemTime>,

    pub non_exhaustive: NonExhaustive,
}

impl Default for TableSState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            scroll_selection: false,
            v_scroll_by: 0,
            v_len: 0,
            v_page: 0,
            v_max_offset: 0,
            area: Default::default(),
            header_area: Default::default(),
            table_area: Default::default(),
            row_areas: Default::default(),
            footer_area: Default::default(),
            mouse_drag: false,
            mouse_click: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl ScrollingState for TableSState {
    fn vertical_max_offset(&self) -> usize {
        if self.scroll_selection {
            self.v_len.saturating_sub(1)
        } else {
            self.v_max_offset
        }
    }

    fn vertical_offset(&self) -> usize {
        if self.scroll_selection {
            self.widget.selected().unwrap_or(0)
        } else {
            self.widget.offset()
        }
    }

    fn vertical_page(&self) -> usize {
        self.v_page
    }

    fn vertical_scroll(&self) -> usize {
        self.v_scroll_by
    }

    fn horizontal_max_offset(&self) -> usize {
        0
    }

    fn horizontal_offset(&self) -> usize {
        0
    }

    fn horizontal_page(&self) -> usize {
        0
    }

    fn horizontal_scroll(&self) -> usize {
        0
    }

    fn set_vertical_offset(&mut self, position: usize) -> ScrollingOutcome {
        if self.scroll_selection {
            let old_select = min(
                self.widget.selected().unwrap_or(0),
                self.v_len.saturating_sub(1),
            );
            let new_select = min(position, self.v_len.saturating_sub(1));

            *self.widget.selected_mut() = Some(new_select);

            if new_select > old_select {
                ScrollingOutcome::Scrolled
            } else {
                ScrollingOutcome::Denied
            }
        } else {
            let old_offset = min(self.vertical_offset(), self.v_len.saturating_sub(1));
            let new_offset = min(position, self.v_len.saturating_sub(1));

            *self.widget.offset_mut() = new_offset;

            // For scrolling purposes the selection of ratatui::Table is never None,
            // instead it defaults out to 0 which prohibits any scrolling attempt.
            // We do our own selection, so we don't really care.
            *self.widget.selected_mut() = Some(self.widget.offset());

            if new_offset > old_offset {
                ScrollingOutcome::Scrolled
            } else {
                ScrollingOutcome::Denied
            }
        }
    }

    fn set_horizontal_offset(&mut self, _offset: usize) -> ScrollingOutcome {
        ScrollingOutcome::Denied
    }
}

impl TableSState {
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.widget = self.widget.with_offset(offset);
        self
    }

    pub fn offset(&self) -> usize {
        self.widget.offset()
    }

    pub fn offset_mut(&mut self) -> &mut usize {
        self.widget.offset_mut()
    }

    pub fn selected(&self) -> Option<usize> {
        self.widget.selected()
    }

    pub fn selection_mut(&mut self) -> &mut Option<usize> {
        self.widget.selected_mut()
    }

    pub fn select(&mut self, position: Option<usize>) {
        self.widget.select(position)
    }

    pub fn select_next(&mut self, relative: usize) {
        let selected = self.widget.selected().unwrap_or(0);
        self.widget.select(Some(selected + relative));
    }

    pub fn select_prev(&mut self, relative: usize) {
        let selected = self.widget.selected().unwrap_or(0);
        self.widget.select(Some(selected.saturating_sub(relative)));
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
            // needs only to be in the same row. outside the widget is ok.
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
        if let Some(selected) = self.selected() {
            if self.vertical_offset() + self.row_areas.len() <= selected {
                self.set_vertical_offset(selected - self.row_areas.len() + 1);
            }
            if self.vertical_offset() > selected {
                self.set_vertical_offset(selected);
            }
        }
    }
}

impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for TableSState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
        let res = match event {
            ct_event!(keycode press Down) => {
                self.select_next(1);
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press Up) => {
                self.select_prev(1);
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press End) => {
                self.select(Some(self.v_len - 1));
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press Home) => {
                self.select(Some(0));
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press PageUp) => {
                self.select_prev(self.table_area.height as usize / 2);
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press PageDown) => {
                self.select_next(self.table_area.height as usize / 2);
                self.scroll_to_selected();
                Outcome::Changed
            }
            _ => Outcome::NotUsed,
        };

        if res == Outcome::NotUsed {
            HandleEvent::handle(self, event, MouseOnly)
        } else {
            res
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for TableSState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        match event {
            ct_event!(scroll down for column,row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_down(self.vertical_scroll());
                    Outcome::Changed
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(scroll up for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_up(self.vertical_scroll());
                    Outcome::Changed
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(mouse down Left for column, row) => {
                let pos = Position::new(*column, *row);
                if self.area.contains(pos) {
                    if let Some(new_row) = self.row_at_clicked(pos) {
                        self.mouse_drag = true;
                        self.select(Some(new_row));
                        Outcome::Changed
                    } else {
                        Outcome::NotUsed
                    }
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(mouse drag Left for column, row) => {
                if self.mouse_drag {
                    let pos = Position::new(*column, *row);
                    let new_row = self.row_at_drag(pos);
                    self.select(Some(new_row));
                    self.scroll_to_selected();
                    Outcome::Changed
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(mouse moved) => {
                self.mouse_drag = false;
                Outcome::NotUsed
            }
            _ => Outcome::NotUsed,
        }
    }
}

/// Extra mapping which does the double click a line in the table thing.
/// Returns `Option<usize>` if a double click is detected.
#[derive(Debug)]
pub struct DoubleClick;

impl HandleEvent<crossterm::event::Event, DoubleClick, Option<usize>> for TableSState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: DoubleClick) -> Option<usize> {
        match event {
            ct_event!(mouse up Left for column,row) => {
                let pos = Position::new(*column, *row);
                if self.area.contains(pos) {
                    let Some(select) = self.row_at_clicked(pos) else {
                        return None;
                    };
                    let Some(old_select) = self.selected() else {
                        return None;
                    };

                    if select == old_select {
                        let triggered = self
                            .mouse_click
                            .map(|v| {
                                let t = v.elapsed().unwrap_or_default();
                                t.as_millis() > 200
                            })
                            .unwrap_or_default();

                        if triggered {
                            Some(old_select)
                        } else {
                            None
                        }
                    } else {
                        self.mouse_click = Some(SystemTime::now());
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Extra mapping that reacts to the delete-key in a table.
///
/// Returns [ControlUI::Run(usize)] for which row should be deleted.
#[derive(Debug)]
pub struct DeleteRow;

impl HandleEvent<crossterm::event::Event, DeleteRow, Option<usize>> for TableSState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: DeleteRow) -> Option<usize> {
        match event {
            ct_event!(keycode press Delete) => {
                if let Some(sel) = self.selected() {
                    Some(sel)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
