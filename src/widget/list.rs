use crate::_private::NonExhaustive;
use crate::widget::MouseFlags;
use crate::{
    ct_event, ControlUI, DefaultKeys, FocusFlag, HandleCrossterm, HasFocusFlag, HasScrolling,
    ListSelection, MouseOnly, NoSelection, ScrollParam, ScrolledWidget, SingleSelection,
};
use crossterm::event::Event;
#[allow(unused_imports)]
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::prelude::{BlockExt, StatefulWidget, Style};
use ratatui::widgets::{Block, HighlightSpacing, List, ListDirection, ListItem, ListState, Widget};
use std::marker::PhantomData;
use std::mem;

///
/// Extensions for [ratatui::widgets::List]
///
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ListExt<'a, Selection> {
    list: List<'a>,
    block: Option<Block<'a>>,
    items: Vec<ListItem<'a>>,

    // todo: pub scroll: ScrollPolicy
    /// Base style
    base_style: Style,
    /// Style for selected + not focused.
    select_style: Style,
    /// Style for selected + focused.
    focus_style: Style,

    _phantom: PhantomData<Selection>,
}

/// Combined style
#[derive(Debug)]
pub struct ListExtStyle {
    pub style: Style,
    pub select_style: Style,
    pub focus_style: Style,
    pub non_exhaustive: NonExhaustive,
}

impl Default for ListExtStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a, Selection> Default for ListExt<'a, Selection> {
    fn default() -> Self {
        Self {
            list: Default::default(),
            block: Default::default(),
            items: Default::default(),
            base_style: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
            _phantom: Default::default(),
        }
    }
}

impl<'a, Selection> ListExt<'a, Selection> {
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
            base_style: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
            _phantom: Default::default(),
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

    pub fn direction(mut self, direction: ListDirection) -> Self {
        self.list = self.list.direction(direction);
        self
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn styles(mut self, styles: ListExtStyle) -> Self {
        self.list = self.list.style(styles.style);
        self.select_style = styles.select_style;
        self.focus_style = styles.focus_style;
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
}

impl<'a, Item, Selection> FromIterator<Item> for ListExt<'a, Selection>
where
    Item: Into<ListItem<'a>>,
{
    fn from_iter<Iter: IntoIterator<Item = Item>>(iter: Iter) -> Self {
        Self::new(iter)
    }
}

impl<'a, State, Selection: ListSelection> ScrolledWidget<State> for ListExt<'a, Selection> {
    fn need_scroll(&self, _area: Rect, _uistate: &mut State) -> ScrollParam {
        ScrollParam {
            has_hscroll: false,
            has_vscroll: true,
        }
    }
}

impl<'a, Selection: Default + ListSelection> Widget for ListExt<'a, Selection> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        StatefulWidget::render(self, area, buf, &mut ListExtState::default());
    }
}

impl<'a, Selection: ListSelection> StatefulWidget for ListExt<'a, Selection> {
    type State = ListExtState<Selection>;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.len = self.len();

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

        // max_h_offset
        let mut n = 0;
        let mut height = 0;
        for item in self.items.iter().rev() {
            height += item.height();
            if height > state.list_area.height as usize {
                break;
            }
            n += 1;
        }
        state.max_v_offset = state.len.saturating_sub(n);

        // rendering
        let mut tmp_item = ListItem::from("".to_string());
        for (i, r) in self.items.iter_mut().enumerate() {
            let style = if state.focus.get() {
                if state.selection.is_selected(i) {
                    self.focus_style
                } else {
                    self.base_style
                }
            } else {
                if state.selection.is_selected(i) {
                    self.select_style
                } else {
                    self.base_style
                }
            };

            mem::swap(r, &mut tmp_item);
            tmp_item = tmp_item.style(style);
            mem::swap(r, &mut tmp_item);
        }

        if let Some(block) = self.block {
            self.list = self.list.block(block);
        }
        self.list = self.list.items(self.items);

        StatefulWidget::render(self.list, area, buf, &mut state.widget);
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ListExtState<Selection> {
    pub widget: ListState,

    pub len: usize,
    pub max_v_offset: usize,
    pub v_page_len: usize,

    pub area: Rect,
    pub list_area: Rect,
    pub item_areas: Vec<Rect>,

    pub focus: FocusFlag,
    pub selection: Selection,

    pub mouse: MouseFlags,
}

impl<Selection> HasFocusFlag for ListExtState<Selection> {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl<Selection> HasScrolling for ListExtState<Selection> {
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
        self.widget.offset()
    }

    fn h_offset(&self) -> usize {
        0
    }

    fn set_v_offset(&mut self, offset: usize) {
        *self.widget.offset_mut() = offset;
        // TODO: check selected prohibits scroll?
    }

    fn set_h_offset(&mut self, _offset: usize) {
        // It's hard to escape somebody calling this.
        // Gracefully ignoring it seems best.

        // unimplemented!("no horizontal scrolling")
    }
}

impl<Selection: ListSelection> ListExtState<Selection> {
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.widget = self.widget.with_offset(offset);
        self
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    pub fn selection_mut(&mut self) -> &mut Selection {
        &mut self.selection
    }

    pub fn offset(&self) -> usize {
        self.widget.offset()
    }

    pub fn offset_mut(&mut self) -> &mut usize {
        self.widget.offset_mut()
    }

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
        if let Some(selected) = self.selection.lead_selection() {
            if self.v_offset() + self.item_areas.len() <= selected {
                self.set_v_offset(selected - self.item_areas.len() + 1);
            }
            if self.v_offset() > selected {
                self.set_v_offset(selected);
            }
        }
    }
}

impl ListExtState<SingleSelection> {
    /// Returns the lead selection.
    pub fn selected(&self) -> Option<usize> {
        self.selection.lead_selection()
    }

    pub fn select(&mut self, n: Option<usize>) {
        self.selection.select(n)
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>> for ListExtState<NoSelection> {
    fn handle(&mut self, _event: &Event, _keymap: DefaultKeys) -> ControlUI<A, E> {
        ControlUI::Continue
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for ListExtState<NoSelection> {
    fn handle(&mut self, _event: &Event, _keymap: MouseOnly) -> ControlUI<A, E> {
        ControlUI::Continue
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>> for ListExtState<SingleSelection> {
    fn handle(&mut self, event: &Event, _: DefaultKeys) -> ControlUI<A, E> {
        let res = if self.is_focused() {
            match event {
                ct_event!(keycode press Down) => {
                    self.selection.next(1, self.len.saturating_sub(1));
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press Up) => {
                    self.selection.prev(1);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press End) => {
                    self.selection.select(Some(self.len.saturating_sub(1)));
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press Home) => {
                    self.selection.select(Some(0));
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press PageUp) => {
                    self.selection.prev(self.v_page_len() / 2);
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                ct_event!(keycode press PageDown) => {
                    self.selection
                        .next(self.v_page_len() / 2, self.len.saturating_sub(1));
                    self.scroll_to_selected();
                    ControlUI::Change
                }
                _ => ControlUI::Continue,
            }
        } else {
            ControlUI::Continue
        };

        res.or_else(|| {
            <Self as HandleCrossterm<ControlUI<A, E>, MouseOnly>>::handle(self, event, MouseOnly)
        })
    }
}

impl<A, E> HandleCrossterm<ControlUI<A, E>, MouseOnly> for ListExtState<SingleSelection> {
    fn handle(&mut self, event: &Event, _: MouseOnly) -> ControlUI<A, E> {
        match event {
            ct_event!(scroll down for column,row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_down(self.v_page_len() / 10);
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(scroll up for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_up(self.v_page_len() / 10);
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
                            .select_clamped(new_row, self.len.saturating_sub(1));
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
                    self.selection
                        .select_clamped(new_row, self.len.saturating_sub(1));
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
