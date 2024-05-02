use crate::_private::NonExhaustive;
use crate::events::{DefaultKeys, HandleEvent, MouseOnly, Outcome};
use crate::{ct_event, ScrollingOutcome, ScrollingState, ScrollingWidget};
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::prelude::{BlockExt, StatefulWidget, Style};
use ratatui::widgets::{Block, HighlightSpacing, List, ListDirection, ListItem, ListState, Widget};
use std::cmp::min;

///
/// Extensions for [ratatui::widgets::List]
///
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ListS<'a> {
    list: List<'a>,
    block: Option<Block<'a>>,
    items: Vec<ListItem<'a>>,
    scroll_selection: bool,

    // todo: pub scroll: ScrollPolicy
    /// Base style
    base_style: Style,
    /// Style for selected + not focused.
    select_style: Style,
    /// Style for selected + focused.
    focus_style: Style,
}

impl<'a> Default for ListS<'a> {
    fn default() -> Self {
        Self {
            list: Default::default(),
            block: Default::default(),
            items: Default::default(),
            scroll_selection: false,
            base_style: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
        }
    }
}

impl<'a> ListS<'a> {
    pub fn new<T>(items: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<ListItem<'a>>,
    {
        let items = items.into_iter().map(|v| v.into()).collect();

        Self {
            list: List::default(),
            block: Default::default(),
            items,
            scroll_selection: false,
            base_style: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
        }
    }

    pub fn items<T>(mut self, items: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<ListItem<'a>>,
    {
        let items = items.into_iter().map(|v| v.into()).collect();
        self.items = items;
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
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

    pub fn direction(mut self, direction: ListDirection) -> Self {
        self.list = self.list.direction(direction);
        self
    }

    pub fn scroll_padding(mut self, padding: usize) -> Self {
        self.list = self.list.scroll_padding(padding);
        self
    }

    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.base_style = style.into();
        self
    }

    pub fn highlight_symbol(mut self, highlight_symbol: &'a str) -> Self {
        self.list = self.list.highlight_symbol(highlight_symbol);
        self
    }

    pub fn highlight_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.list = self.list.highlight_style(style);
        self
    }

    pub fn repeat_highlight_symbol(mut self, repeat: bool) -> Self {
        self.list = self.list.repeat_highlight_symbol(repeat);
        self
    }

    pub fn highlight_spacing(mut self, value: HighlightSpacing) -> Self {
        self.list = self.list.highlight_spacing(value);
        self
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<'a, Item> FromIterator<Item> for ListS<'a>
where
    Item: Into<ListItem<'a>>,
{
    fn from_iter<Iter: IntoIterator<Item = Item>>(iter: Iter) -> Self {
        Self::new(iter)
    }
}

impl<'a, State> ScrollingWidget<State> for ListS<'a> {
    fn need_scroll(&self, _area: Rect, _uistate: &mut State) -> (bool, bool) {
        // todo: decide scrolling
        (false, true)
    }
}

impl<'a> Widget for ListS<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        StatefulWidget::render(self, area, buf, &mut ListSState::default());
    }
}

impl<'a> StatefulWidget for ListS<'a> {
    type State = ListSState;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.scroll_selection = self.scroll_selection;
        state.v_len = self.len();

        state.list_area = self.block.inner_if_some(area);

        // area for each item
        state.item_areas.clear();
        let mut item_area = Rect::new(
            state.list_area.x,
            state.list_area.y,
            state.list_area.width,
            1,
        );
        for item in self.items.iter().skip(state.offset()) {
            item_area.height = item.height() as u16;

            state.item_areas.push(item_area);

            item_area.y += item_area.height;
            if item_area.y >= state.list_area.y + state.list_area.height {
                break;
            }
        }
        state.v_page_len = state.item_areas.len();

        // v_max_offset

        if self.scroll_selection {
            state.v_max_offset = state.v_len.saturating_sub(1);
        } else {
            let mut n = 0;
            let mut height = 0;
            for item in self.items.iter().rev() {
                height += item.height();
                if height > state.list_area.height as usize {
                    break;
                }
                n += 1;
            }
            state.v_max_offset = state.v_len.saturating_sub(n);
        }

        if let Some(block) = self.block {
            self.list = self.list.block(block);
        }
        self.list = self.list.items(self.items);

        StatefulWidget::render(self.list, area, buf, &mut state.widget);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListSState {
    pub widget: ListState,

    pub scroll_selection: bool,
    pub v_len: usize,
    pub v_max_offset: usize,
    pub v_page_len: usize,

    pub area: Rect,
    pub list_area: Rect,
    pub item_areas: Vec<Rect>,

    pub mouse_drag: bool,

    pub non_exhaustive: NonExhaustive,
}

impl Default for ListSState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            scroll_selection: false,
            v_len: 0,
            v_max_offset: 0,
            v_page_len: 0,
            area: Default::default(),
            list_area: Default::default(),
            item_areas: Default::default(),
            mouse_drag: false,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl ListSState {
    pub fn offset(&self) -> usize {
        self.widget.offset()
    }

    pub fn offset_mut(&mut self) -> &mut usize {
        self.widget.offset_mut()
    }

    pub fn selected(&self) -> Option<usize> {
        self.widget.selected()
    }

    pub fn selected_mut(&mut self) -> &mut Option<usize> {
        self.widget.selected_mut()
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.widget.select(index);
    }

    pub fn select_next(&mut self, n: usize) {
        let idx = self.selected().unwrap_or(0);
        *self.selected_mut() = Some(idx + n);
    }

    pub fn select_prev(&mut self, n: usize) {
        let idx = self.selected().unwrap_or(0);
        *self.selected_mut() = Some(idx.saturating_sub(n));
    }

    /// Row at the given position.
    pub fn row_at_clicked(&self, pos: Position) -> Option<usize> {
        for (i, r) in self.item_areas.iter().enumerate() {
            if r.contains(pos) {
                return Some(self.offset() + i);
            }
        }
        None
    }

    /// Row when dragging. Can go outside the area.
    pub fn row_at_drag(&self, pos: Position) -> usize {
        let offset = self.offset();
        for (i, r) in self.item_areas.iter().enumerate() {
            if pos.y >= r.y && pos.y < r.y + r.height {
                return offset + i;
            }
        }

        let offset = self.offset() as isize;
        let rr = if pos.y < self.list_area.y {
            // assume row-height=1 for outside the box.
            let min_row = self.list_area.y as isize;
            offset + (pos.y as isize - min_row)
        } else if pos.y >= self.list_area.y + self.list_area.height {
            let max_row = self.list_area.y as isize + self.list_area.height as isize;
            let vis_rows = self.item_areas.len() as isize;
            offset + vis_rows + (pos.y as isize - max_row)
        } else {
            if let Some(last) = self.item_areas.last() {
                // count from last row.
                let min_row = last.y as isize + last.height as isize;
                let vis_rows = self.item_areas.len() as isize;
                offset + vis_rows + (pos.y as isize - min_row)
            } else {
                // empty list, count from header
                let min_row = self.list_area.y as isize;
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
            if self.vertical_offset() + self.item_areas.len() <= selected {
                self.set_vertical_offset(selected - self.item_areas.len() + 1);
            }
            if self.vertical_offset() > selected {
                self.set_vertical_offset(selected);
            }
        }
    }
}

impl ScrollingState for ListSState {
    fn vertical_max_offset(&self) -> usize {
        self.v_max_offset
    }

    fn vertical_offset(&self) -> usize {
        if self.scroll_selection {
            self.widget.selected().unwrap_or(0)
        } else {
            self.widget.offset()
        }
    }

    fn vertical_page(&self) -> usize {
        self.v_page_len
    }

    fn vertical_scroll(&self) -> usize {
        self.v_page_len / 10
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
            debug!("new select {:?}", self.widget.selected());

            if new_select > old_select {
                ScrollingOutcome::Scrolled
            } else {
                ScrollingOutcome::Denied
            }
        } else {
            let old_offset = min(self.widget.offset(), self.v_len.saturating_sub(1));
            let new_offset = min(position, self.v_len.saturating_sub(1));

            *self.widget.offset_mut() = new_offset;

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

impl HandleEvent<crossterm::event::Event, DefaultKeys, Outcome> for ListSState {
    fn handle(
        &mut self,
        event: &crossterm::event::Event,
        focus: bool,
        _keymap: DefaultKeys,
    ) -> Outcome {
        let r = if focus {
            match event {
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
                    *self.selected_mut() = Some(self.v_len.saturating_sub(1));
                    self.scroll_to_selected();
                    Outcome::Changed
                }
                ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press Home) => {
                    *self.selected_mut() = Some(0);
                    self.scroll_to_selected();
                    Outcome::Changed
                }
                ct_event!(keycode press PageUp) => {
                    self.select_prev(self.vertical_page() / 2);
                    self.scroll_to_selected();
                    Outcome::Changed
                }
                ct_event!(keycode press PageDown) => {
                    self.select_next(self.vertical_page() / 2);
                    self.scroll_to_selected();
                    Outcome::Changed
                }
                _ => Outcome::NotUsed,
            }
        } else {
            Outcome::NotUsed
        };

        match r {
            Outcome::NotUsed => HandleEvent::handle(self, event, focus, MouseOnly),
            _ => Outcome::NotUsed,
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for ListSState {
    fn handle(
        &mut self,
        event: &crossterm::event::Event,
        _focus: bool,
        _keymap: MouseOnly,
    ) -> Outcome {
        let r = match event {
            ct_event!(scroll down for column,row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_down(self.vertical_page() / 10);
                    Outcome::Changed
                } else {
                    Outcome::NotUsed
                }
            }
            ct_event!(scroll up for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_up(self.vertical_page() / 10);
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
                        Outcome::Unchanged
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
        };

        r
    }
}
