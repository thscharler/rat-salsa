use crate::adapter::_private::NonExhaustive;
use rat_event::util::MouseFlags;
use rat_event::{ct_event, flow, HandleEvent, MouseOnly, Outcome, Regular};
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Style;
use ratatui::widgets::ListDirection::BottomToTop;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::{Block, HighlightSpacing, List, ListDirection, ListItem, ListState};
use std::cmp::{max, min};

///
/// Extensions for [ratatui::widgets::List]
///
#[derive(Debug, Clone)]
pub struct ListS<'a> {
    list: List<'a>,
    direction: ListDirection,
    block: Option<Block<'a>>,
    scroll: Option<Scroll<'a>>,
    scroll_selection: bool,
    items: Vec<ListItem<'a>>,
}

impl<'a> Default for ListS<'a> {
    fn default() -> Self {
        Self {
            list: Default::default(),
            direction: Default::default(),
            block: Default::default(),
            scroll: None,
            scroll_selection: false,
            items: Default::default(),
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
            direction: Default::default(),
            block: Default::default(),
            items,
            scroll_selection: false,
            scroll: None,
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

    pub fn scroll(mut self, scroll: Scroll<'a>) -> Self {
        self.scroll = Some(scroll.override_vertical());
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
        self.direction = direction;
        self.list = self.list.direction(direction);
        self
    }

    pub fn scroll_padding(mut self, padding: usize) -> Self {
        self.list = self.list.scroll_padding(padding);
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

    pub fn highlight_spacing(mut self, value: HighlightSpacing) -> Self {
        self.list = self.list.highlight_spacing(value);
        self
    }

    pub fn repeat_highlight_symbol(mut self, repeat: bool) -> Self {
        self.list = self.list.repeat_highlight_symbol(repeat);
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

impl<'a> StatefulWidget for ListS<'a> {
    type State = ListSState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.scroll_selection = self.scroll_selection;
        state.len = self.len();

        let scroll = ScrollArea::new()
            .block(self.block.as_ref())
            .v_scroll(self.scroll.as_ref());
        state.inner = scroll.inner(area, None, Some(&state.scroll));

        // area for each item
        state.item_areas.clear();
        if self.direction == BottomToTop {
            let mut item_area = Rect::new(
                state.inner.x,
                (state.inner.y + state.inner.height).saturating_sub(1),
                state.inner.width,
                1,
            );
            for item in self.items.iter().skip(state.offset()) {
                item_area.height = item.height() as u16;
                state.item_areas.push(item_area);

                item_area.y = item_area.y.saturating_sub(item_area.height);
                if item_area.y < state.inner.y {
                    break;
                }
            }
        } else {
            let mut item_area = Rect::new(state.inner.x, state.inner.y, state.inner.width, 1);
            for item in self.items.iter().skip(state.offset()) {
                item_area.height = item.height() as u16;
                state.item_areas.push(item_area);

                item_area.y += item_area.height;
                if item_area.y >= state.inner.y + state.inner.height {
                    break;
                }
            }
        }
        state.scroll.set_page_len(state.item_areas.len());

        // v_max_offset
        if self.scroll_selection {
            state.scroll.set_max_offset(state.len.saturating_sub(1));
        } else {
            let mut n = 0;
            let mut height = 0;
            for item in self.items.iter().rev() {
                height += item.height();
                if height > state.inner.height as usize {
                    break;
                }
                n += 1;
            }
            state.scroll.set_max_offset(state.len.saturating_sub(n));
        }

        scroll.render(
            area,
            buf,
            &mut ScrollAreaState::new().v_scroll(&mut state.scroll),
        );

        StatefulWidget::render(
            self.list.clone().items(self.items),
            state.inner,
            buf,
            &mut state.widget,
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListSState {
    pub widget: ListState,

    pub len: usize,
    pub scroll_selection: bool,
    pub scroll: ScrollState,

    pub area: Rect,
    pub inner: Rect,
    pub item_areas: Vec<Rect>,

    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl Default for ListSState {
    fn default() -> Self {
        Self {
            widget: Default::default(),
            scroll_selection: false,
            len: 0,
            scroll: Default::default(),
            area: Default::default(),
            inner: Default::default(),
            item_areas: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl ListSState {
    pub fn selected(&self) -> Option<usize> {
        self.widget.selected()
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.widget.select(index);
    }

    pub fn select_next(&mut self, n: usize) {
        let idx = self.selected().unwrap_or(0);
        self.select(Some(idx + n));
    }

    pub fn select_prev(&mut self, n: usize) {
        let idx = self.selected().unwrap_or(0);
        self.select(Some(idx.saturating_sub(n)));
    }

    /// Row at the given position.
    pub fn row_at_clicked(&self, pos: Position) -> Option<usize> {
        self.mouse
            .row_at(&self.item_areas, pos.y)
            .map(|v| self.offset() + v)
    }

    /// Row when dragging. Can go outside the area.
    pub fn row_at_drag(&self, pos: Position) -> usize {
        let offset = self.offset();
        match self.mouse.row_at_drag(self.inner, &self.item_areas, pos.y) {
            Ok(v) => offset + v,
            Err(v) if v <= 0 => offset.saturating_sub((-v) as usize),
            Err(v) => offset + self.item_areas.len() + v as usize,
        }
    }

    /// Scroll to selected.
    pub fn scroll_to_selected(&mut self) {
        if let Some(selected) = self.selected() {
            if selected > self.offset() + self.item_areas.len() {
                self.set_offset(selected - self.item_areas.len() + 1);
            }
            if selected < self.offset() {
                self.set_offset(selected);
            }
        }
    }
}

impl ListSState {
    #[inline]
    pub fn offset(&self) -> usize {
        self.scroll.offset()
    }

    pub fn set_offset(&mut self, position: usize) -> bool {
        self.scroll.set_offset(position)
    }

    pub fn scroll_to(&mut self, pos: usize) -> bool {
        if self.scroll_selection {
            let old_select = self.widget.selected();

            let new_select = pos;
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

            let new_select = min(
                max(self.widget.selected().unwrap_or(0) as isize + n, 0),
                self.len.saturating_sub(1) as isize,
            ) as usize;
            *self.widget.selected_mut() = Some(new_select);
            self.scroll_to_selected();

            self.widget.selected() != old_select
        } else {
            let old_offset = self.scroll.offset();

            let new_offset = self
                .scroll
                .limited_offset(self.scroll.offset().saturating_add_signed(n));
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

impl HandleEvent<crossterm::event::Event, Regular, Outcome> for ListSState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> Outcome {
        let r = match event {
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
                self.select(Some(self.len.saturating_sub(1)));
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press Home) => {
                self.select(Some(0));
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press PageUp) => {
                self.select_prev(self.scroll.page_len() / 2);
                self.scroll_to_selected();
                Outcome::Changed
            }
            ct_event!(keycode press PageDown) => {
                self.select_next(self.scroll.page_len() / 2);
                self.scroll_to_selected();
                Outcome::Changed
            }
            _ => Outcome::Continue,
        };

        match r {
            Outcome::Continue => HandleEvent::handle(self, event, MouseOnly),
            _ => Outcome::Continue,
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for ListSState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
        flow!(match event {
            ct_event!(mouse down Left for column, row) => {
                let pos = Position::new(*column, *row);
                if self.inner.contains(pos) {
                    if let Some(new_row) = self.row_at_clicked(pos) {
                        self.select(Some(new_row));
                        Outcome::Changed
                    } else {
                        Outcome::Unchanged
                    }
                } else {
                    Outcome::Continue
                }
            }
            _ => Outcome::Continue,
        });

        flow!(match self.scroll.handle(event, MouseOnly) {
            ScrollOutcome::VPos(v) => {
                self.scroll_to(v);
                Outcome::Changed
            }
            r => r.into(),
        });

        flow!({
            let mut sas = ScrollAreaState::new()
                .area(self.inner)
                .v_scroll(&mut self.scroll);
            match sas.handle(event, MouseOnly) {
                ScrollOutcome::Up(v) => Outcome::from(self.scroll(-(v as isize))),
                ScrollOutcome::Down(v) => Outcome::from(self.scroll(v as isize)),
                r => r.into(),
            }
        });

        Outcome::Continue
    }
}
