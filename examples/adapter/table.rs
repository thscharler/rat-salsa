///
/// Extensions for [ratatui::widgets::Table]
///
/// This is limited to Tables with row-heights == 1.
///
use crate::adapter::_private::NonExhaustive;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, flow, FocusKeys, HandleEvent, MouseOnly, Outcome};
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::{layout_scroll, Scroll, ScrollArea, ScrollState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Position, Rect};
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Block, HighlightSpacing, Row, StatefulWidget, Table, TableState, Widget};
use std::fmt::Debug;

/// Add some minor fixes to [ratatui::widgets::Table]
#[derive(Debug, Clone)]
pub struct TableS<'a> {
    block: Option<Block<'a>>,
    scroll: Option<Scroll<'a>>,
    scroll_selection: bool,

    table: Table<'a>,
    rows: Vec<Row<'a>>,
    header: Option<Row<'a>>,
    footer: Option<Row<'a>>,
}

impl<'a> Default for TableS<'a> {
    fn default() -> Self {
        Self {
            block: None,
            rows: Default::default(),
            header: None,
            footer: None,
            table: Default::default(),
            scroll_selection: false,
            scroll: None,
        }
    }
}

impl<'a> TableS<'a> {
    /// New Table.
    pub fn new<R, C>(rows: R, widths: C) -> Self
    where
        R: IntoIterator,
        R::Item: Into<Row<'a>>,
        C: IntoIterator,
        C::Item: Into<Constraint>,
    {
        let rows = rows.into_iter().map(|v| v.into()).collect::<Vec<_>>();

        Self {
            block: None,
            scroll: None,
            rows,
            header: None,
            table: Table::default().widths(widths),
            scroll_selection: false,
            footer: None,
        }
    }

    /// Set data rows
    pub fn rows<T>(mut self, rows: T) -> Self
    where
        T: IntoIterator<Item = Row<'a>>,
    {
        let rows = rows.into_iter().collect::<Vec<_>>();
        self.rows = rows;
        self
    }

    /// Scrollbars
    pub fn scroll(mut self, scroll: Scroll<'a>) -> Self {
        self.scroll = Some(scroll.override_vertical());
        self.block = None;
        self
    }

    /// Block
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self.scroll = None;
        self
    }

    /// Scroll the selection.
    pub fn scroll_selection(mut self) -> Self {
        self.scroll_selection = true;
        self
    }

    /// Scroll the offset.
    pub fn scroll_offset(mut self) -> Self {
        self.scroll_selection = false;
        self
    }

    /// Set the header.
    pub fn header(mut self, header: Row<'a>) -> Self {
        self.header = Some(header);
        self
    }

    /// Set the footer.
    pub fn footer(mut self, footer: Row<'a>) -> Self {
        self.footer = Some(footer);
        self
    }

    /// Column widths
    pub fn widths<I>(mut self, widths: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        self.table = self.table.widths(widths);
        self
    }

    /// Spacing
    pub fn column_spacing(mut self, spacing: u16) -> Self {
        self.table = self.table.column_spacing(spacing);
        self
    }

    /// Style
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.table = self.table.style(style);
        self
    }

    /// Style for selection
    pub fn highlight_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.table = self.table.highlight_style(style);
        self
    }

    /// Extra symbol for selection
    pub fn highlight_symbol<T: Into<Text<'a>>>(mut self, select_symbol: T) -> Self {
        self.table = self.table.highlight_symbol(select_symbol);
        self
    }

    /// Spacing
    pub fn highlight_spacing(mut self, value: HighlightSpacing) -> Self {
        self.table = self.table.highlight_spacing(value);
        self
    }

    /// Spacing
    pub fn select_spacing(mut self, value: HighlightSpacing) -> Self {
        self.table = self.table.highlight_spacing(value);
        self
    }

    /// Flex layout.
    pub fn flex(mut self, flex: Flex) -> Self {
        self.table = self.table.flex(flex);
        self
    }
}

impl<'a> StatefulWidget for TableS<'a> {
    type State = TableSState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.scroll_selection = self.scroll_selection;
        state.len = self.rows.len();

        let (_, scroll_area, inner) =
            layout_scroll(area, self.block.as_ref(), None, self.scroll.as_ref());
        state.area = inner;

        // row layout
        // TODO: as long as height_with_margin() is not accessible we are limited
        //       to single row tables.
        // let header_height = self.header.as_ref().map_or(0, |h| h.height_with_margin());
        // let footer_height = self.footer.as_ref().map_or(0, |f| f.height_with_margin());

        let header_height = 0;
        let footer_height = 0;
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
            // row_area.height = _row.height_with_margin();
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
        state.scroll.set_page_len(n);
        if state.scroll_selection {
            state.scroll.set_scroll_by(Some(1));
        }
        state.scroll.set_max_offset(state.len - n);

        self.block.render(area, buf);
        if let Some(scroll) = self.scroll {
            scroll.render(scroll_area, buf, &mut state.scroll);
        }

        *state.widget.offset_mut() = state.scroll.offset();
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
        StatefulWidget::render(table, inner, buf, &mut state.widget);
    }
}

impl<'a, Item> FromIterator<Item> for TableS<'a>
where
    Item: Into<Row<'a>>,
{
    fn from_iter<U: IntoIterator<Item = Item>>(iter: U) -> Self {
        let rows = iter.into_iter().map(|v| v.into()).collect::<Vec<_>>();

        Self {
            block: None,
            rows,
            header: None,
            footer: None,
            table: Table::default(),
            scroll_selection: false,
            scroll: None,
        }
    }
}

/// Extended TableState, contains a [ratatui::widgets::TableState].
#[derive(Debug, Clone)]
pub struct TableSState {
    /// inner widget
    pub widget: TableState,

    /// Total length.
    pub len: usize,
    /// Scroll the selection.
    pub scroll_selection: bool,
    /// Scroll state
    pub scroll: ScrollState,

    /// Complete area.
    pub area: Rect,
    /// Header
    pub header_area: Rect,
    /// Table content
    pub table_area: Rect,
    /// Visible rows.
    pub row_areas: Vec<Rect>,
    /// Footer
    pub footer_area: Rect,

    /// Mouse behaviour.
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl Default for TableSState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            len: 0,
            scroll_selection: false,
            scroll: Default::default(),
            area: Default::default(),
            header_area: Default::default(),
            table_area: Default::default(),
            row_areas: Default::default(),
            footer_area: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl TableSState {
    #[inline]
    pub fn max_offset(&self) -> usize {
        self.scroll.max_offset()
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.scroll.offset()
    }

    #[inline]
    pub fn page_len(&self) -> usize {
        self.scroll.page_len()
    }

    #[inline]
    pub fn scroll_by(&self) -> usize {
        self.scroll.scroll_by()
    }

    #[inline]
    pub fn set_offset(&mut self, position: usize) -> bool {
        self.scroll.set_offset(position)
    }

    pub fn scroll_to(&mut self, pos: usize) -> bool {
        if self.scroll_selection {
            let old_select = self.widget.selected();

            let new_select = (pos * self.len) / self.scroll.max_offset();
            *self.widget.selected_mut() = Some(new_select);
            self.scroll_to_selected();

            self.widget.selected() != old_select
        } else {
            let old_offset = self.scroll.offset();

            let new_offset = pos;
            self.scroll.set_offset(new_offset);
            *self.widget.offset_mut() = new_offset;

            // For scrolling purposes the selection of ratatui::Table is never None,
            // instead it defaults out to 0 which prohibits any scrolling attempt.
            // Losing the selection here is a bit inconvenient, but this is more of a demo
            // anyway.
            if let Some(selected) = self.widget.selected() {
                if selected < new_offset {
                    *self.widget.selected_mut() = Some(new_offset);
                } else if selected >= new_offset + self.scroll.page_len() {
                    *self.widget.selected_mut() = Some(new_offset + self.scroll.page_len());
                }
            }

            self.scroll.offset() != old_offset
        }
    }

    pub fn scroll(&mut self, n: isize) -> bool {
        if self.scroll_selection {
            let old_select = self.widget.selected();

            let sel = self.widget.selected().unwrap_or(0) as isize;
            let max = self.len.saturating_sub(1) as isize;
            let new_select = (sel + n).clamp(0, max) as usize;
            *self.widget.selected_mut() = Some(new_select);
            self.scroll_to_selected();

            self.widget.selected() != old_select
        } else {
            let old_offset = self.scroll.offset();

            let new_offset = self.scroll.clamp_offset(self.scroll.offset() as isize + n);
            self.scroll.set_offset(new_offset);
            *self.widget.offset_mut() = new_offset;
            // For scrolling purposes the selection of ratatui::Table is never None,
            // instead it defaults out to 0 which prohibits any scrolling attempt.
            // Losing the selection here is a bit inconvenient, but this is more of a demo
            // anyway.
            if let Some(selected) = self.widget.selected() {
                if selected < new_offset {
                    *self.widget.selected_mut() = Some(new_offset);
                } else if selected >= new_offset + self.scroll.page_len() {
                    *self.widget.selected_mut() = Some(new_offset + self.scroll.page_len());
                }
            }

            self.scroll.offset() != old_offset
        }
    }
}

impl TableSState {
    #[inline]
    pub fn selected(&self) -> Option<usize> {
        self.widget.selected()
    }

    #[inline]
    pub fn selection_mut(&mut self) -> &mut Option<usize> {
        self.widget.selected_mut()
    }

    #[inline]
    pub fn select(&mut self, position: Option<usize>) {
        self.widget.select(position)
    }

    #[inline]
    pub fn select_next(&mut self, relative: usize) {
        let selected = self.widget.selected().unwrap_or(0);
        self.widget.select(Some(selected + relative));
    }

    #[inline]
    pub fn select_prev(&mut self, relative: usize) {
        let selected = self.widget.selected().unwrap_or(0);
        self.widget.select(Some(selected.saturating_sub(relative)));
    }

    /// Row at given position.
    #[inline]
    pub fn row_at_clicked(&self, pos: Position) -> Option<usize> {
        rat_event::util::row_at_clicked(&self.row_areas, pos.y).map(|v| self.scroll.offset() + v)
    }

    /// Row when dragging. Can go outside the area.
    #[inline]
    pub fn row_at_drag(&self, pos: Position) -> usize {
        let offset = self.scroll.offset();
        match rat_event::util::row_at_drag(self.table_area, &self.row_areas, pos.y) {
            Ok(v) => offset + v,
            Err(v) if v <= 0 => offset.saturating_sub((-v) as usize),
            Err(v) => offset + self.row_areas.len() + v as usize,
        }
    }

    /// Scroll to selected.
    #[inline]
    pub fn scroll_to_selected(&mut self) {
        if let Some(selected) = self.selected() {
            if selected >= self.scroll.offset() + self.scroll.page_len() {
                self.set_offset(selected.saturating_sub(self.scroll.page_len() * 9 / 10));
            }
            if selected < self.scroll.offset() {
                self.set_offset(selected.saturating_sub(self.scroll.page_len() * 1 / 10));
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
                self.select(Some(self.len - 1));
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
        flow!(match event {
            ct_event!(mouse down Left for column, row) => {
                let pos = Position::new(*column, *row);
                if self.area.contains(pos) {
                    if let Some(new_row) = self.row_at_clicked(pos) {
                        self.select(Some(new_row));
                        Outcome::Changed
                    } else {
                        Outcome::NotUsed
                    }
                } else {
                    Outcome::NotUsed
                }
            }
            _ => Outcome::NotUsed,
        });

        flow!(match self.scroll.handle(event, MouseOnly) {
            ScrollOutcome::VPos(v) => {
                self.scroll_to(v).into()
            }
            r => Outcome::from(r),
        });

        flow!(
            match ScrollArea(self.table_area, None, Some(&mut self.scroll)).handle(event, MouseOnly)
            {
                ScrollOutcome::Up(v) => {
                    Outcome::from(self.scroll(-(v as isize)))
                }
                ScrollOutcome::Down(v) => {
                    Outcome::from(self.scroll(v as isize))
                }
                r => Outcome::from(r),
            }
        );

        Outcome::NotUsed
    }
}
