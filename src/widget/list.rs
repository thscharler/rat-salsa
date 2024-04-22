use crate::widget::MouseFlags;
use crate::{
    ct_event, ControlUI, DefaultKeys, FocusFlag, HandleCrossterm, HasFocusFlag, HasScrolling,
    ListSelection, MouseOnly, NoSelection, SingleSelection,
};
use crossterm::event::Event;
use log::debug;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::prelude::{BlockExt, StatefulWidget, Style};
use ratatui::widgets::{Block, HighlightSpacing, List, ListDirection, ListItem, ListState};
use std::marker::PhantomData;

///
/// Extensions for [ratatui::widgets::List]
///
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ListExt<'a, SEL> {
    pub list: List<'a>,
    pub block: Option<Block<'a>>,

    pub items: Vec<ListItem<'a>>,

    /// Style for selected + not focused.
    pub select_style: Style,
    /// Style for selected + focused.
    pub focus_style: Style,

    pub non_exhaustive: (),
    pub _phantom: PhantomData<SEL>,
}

/// Combined style
#[derive(Debug, Default)]
pub struct ListExtStyle {
    pub style: Style,
    pub select_style: Style,
    pub focus_style: Style,
    pub non_exhaustive: (),
}

impl<'a, SEL> Default for ListExt<'a, SEL> {
    fn default() -> Self {
        Self {
            list: Default::default(),
            block: Default::default(),
            items: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
            non_exhaustive: (),
            _phantom: Default::default(),
        }
    }
}

impl<'a, SEL> ListExt<'a, SEL> {
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
            select_style: Default::default(),
            focus_style: Default::default(),
            non_exhaustive: (),
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
        self.list = self.list.style(style);
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
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
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

impl<'a, Item, SEL> FromIterator<Item> for ListExt<'a, SEL>
where
    Item: Into<ListItem<'a>>,
{
    fn from_iter<Iter: IntoIterator<Item = Item>>(iter: Iter) -> Self {
        Self::new(iter)
    }
}

impl<'a, SEL: ListSelection> StatefulWidget for ListExt<'a, SEL> {
    type State = ListExtState<SEL>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.len = self.len();

        state.list_area = self.block.inner_if_some(area);

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

        let mut list = if state.focus.get() {
            self.list.highlight_style(self.focus_style)
        } else {
            self.list.highlight_style(self.select_style)
        };
        if let Some(block) = self.block {
            list = list.block(block);
        }
        list = list.items(self.items);

        StatefulWidget::render(list, area, buf, &mut state.widget);
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ListExtState<SEL> {
    pub widget: ListState,

    pub len: usize,

    pub area: Rect,
    pub list_area: Rect,
    pub item_areas: Vec<Rect>,

    pub focus: FocusFlag,
    pub selection: SEL,

    pub mouse: MouseFlags,
}

impl<SEL> HasFocusFlag for ListExtState<SEL> {
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    fn area(&self) -> Rect {
        self.area
    }
}

impl<SEL> HasScrolling for ListExtState<SEL> {
    fn has_vscroll(&self) -> bool {
        true
    }

    fn has_hscroll(&self) -> bool {
        false
    }

    fn vlen(&self) -> usize {
        self.len
    }

    fn hlen(&self) -> usize {
        0
    }

    fn vmax_offset(&self) -> usize {
        self.len
    }

    fn hmax_offset(&self) -> usize {
        0
    }

    fn voffset(&self) -> usize {
        self.widget.offset()
    }

    fn hoffset(&self) -> usize {
        0
    }

    fn set_voffset(&mut self, offset: usize) {
        *self.widget.offset_mut() = offset;
        // TODO: check selected fix
    }

    fn set_hoffset(&mut self, _offset: usize) {
        unimplemented!("no horizontal scrolling")
    }
}

impl<SEL: ListSelection> ListExtState<SEL> {
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.widget = self.widget.with_offset(offset);
        self
    }

    pub fn selection(&self) -> &SEL {
        &self.selection
    }

    pub fn selection_mut(&mut self) -> &mut SEL {
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
                debug!("row_at_drag found row {}", offset + i);
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
    pub fn adjust_view(&mut self) {
        if let Some(selected) = self.selection.lead_selection() {
            if self.voffset() + self.item_areas.len() <= selected {
                self.set_voffset(selected - self.item_areas.len() + 1);
            }
            if self.voffset() > selected {
                self.set_voffset(selected);
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
                    self.selection.next(1, self.len - 1);
                    self.adjust_view();
                    ControlUI::Change
                }
                ct_event!(keycode press Up) => {
                    self.selection.prev(1);
                    self.adjust_view();
                    ControlUI::Change
                }
                ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press End) => {
                    self.selection.select(Some(self.len - 1));
                    self.adjust_view();
                    ControlUI::Change
                }
                ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press Home) => {
                    self.selection.select(Some(0));
                    self.adjust_view();
                    ControlUI::Change
                }
                ct_event!(keycode press PageUp) => {
                    self.selection.prev(self.list_area.height as usize / 2);
                    self.adjust_view();
                    ControlUI::Change
                }
                ct_event!(keycode press PageDown) => {
                    self.selection
                        .next(self.list_area.height as usize / 2, self.len - 1);
                    self.adjust_view();
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
                    self.scroll_down(self.list_area.height as usize / 10);
                    ControlUI::Change
                } else {
                    ControlUI::Continue
                }
            }
            ct_event!(scroll up for column, row) => {
                if self.area.contains(Position::new(*column, *row)) {
                    self.scroll_up(self.list_area.height as usize / 10);
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
                    self.adjust_view();
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
