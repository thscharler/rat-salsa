//!
//! Extension for [ratatui::widgets::List]
//!

use crate::_private::NonExhaustive;
use crate::event::util::{row_at_clicked, row_at_drag, MouseFlags};
use crate::list::selection::{RowSelection, RowSetSelection};
use rat_focus::{FocusFlag, HasFocusFlag};
use rat_scrolled::{ScrollingState, ScrollingWidget};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::prelude::{BlockExt, StatefulWidget};
use ratatui::style::Style;
use ratatui::widgets::{Block, ListDirection, ListItem};
use std::cmp::min;
use std::collections::HashSet;
use std::marker::PhantomData;

/// Trait for list-selection.
pub trait ListSelection {
    /// Is selected.
    fn is_selected(&self, n: usize) -> bool;

    /// Selection lead.
    fn lead_selection(&self) -> Option<usize>;
}

#[derive(Debug, Default, Clone)]
pub struct List<'a, Selection> {
    block: Option<Block<'a>>,
    items: Vec<ListItem<'a>>,

    style: Style,
    select_style: Style,
    focus_style: Style,
    direction: ListDirection,

    _phantom: PhantomData<Selection>,
}

#[derive(Debug, Clone)]
pub struct ListStyle {
    /// Style
    pub style: Style,
    /// Style for selection
    pub select_style: Style,
    /// Style for selection when focused.
    pub focus_style: Style,

    pub non_exhaustive: NonExhaustive,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ListState<Selection> {
    /// Length in items.
    pub len: usize,

    /// Offset
    pub v_offset: usize,
    /// Max offset for full page.
    pub v_max_offset: usize,
    /// Page length
    pub v_page_len: usize,

    /// Total area
    pub area: Rect,
    /// Area inside the block.
    pub inner: Rect,
    /// Areas for the rendered items.
    pub item_areas: Vec<Rect>,

    /// Focus
    pub focus: FocusFlag,
    /// Selection model
    pub selection: Selection,

    /// Helper for mouse events.
    pub mouse: MouseFlags,
}

impl Default for ListStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a, Selection> List<'a, Selection> {
    pub fn new<T>(items: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<ListItem<'a>>,
    {
        let items = items.into_iter().map(|v| v.into()).collect();

        Self {
            block: Default::default(),
            items,
            style: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
            direction: Default::default(),
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

    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    #[inline]
    pub fn styles(mut self, styles: ListStyle) -> Self {
        self.style = styles.style;
        self.select_style = styles.select_style;
        self.focus_style = styles.focus_style;
        self
    }

    #[inline]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    #[inline]
    pub fn select_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.select_style = select_style.into();
        self
    }

    #[inline]
    pub fn focus_style<S: Into<Style>>(mut self, focus_style: S) -> Self {
        self.focus_style = focus_style.into();
        self
    }

    #[inline]
    pub fn direction(mut self, direction: ListDirection) -> Self {
        self.direction = direction;
        self
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<'a, Item, Selection> FromIterator<Item> for List<'a, Selection>
where
    Item: Into<ListItem<'a>>,
{
    fn from_iter<Iter: IntoIterator<Item = Item>>(iter: Iter) -> Self {
        Self::new(iter)
    }
}

impl<'a, State, Selection: ListSelection> ScrollingWidget<State> for List<'a, Selection> {
    fn need_scroll(&self, area: Rect, _state: &mut State) -> (bool, bool) {
        let vertical = 'f: {
            let mut height = 0;
            for item in self.items.iter() {
                height += item.height() as u16;
                if height >= area.height {
                    break 'f true;
                }
            }
            false
        };

        (false, vertical)
    }
}

impl<'a, Selection: ListSelection> StatefulWidget for List<'a, Selection> {
    type State = ListState<Selection>;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.len = self.len();

        state.inner = self.block.inner_if_some(area);

        // area for each item
        state.item_areas.clear();
        let mut item_area = Rect::new(state.inner.x, state.inner.y, state.inner.width, 1);
        for item in self.items.iter().skip(state.offset()) {
            item_area.height = item.height() as u16;

            state.item_areas.push(item_area);

            item_area.y += item_area.height;
            if item_area.y >= state.inner.y + state.inner.height {
                break;
            }
        }
        state.v_page_len = state.item_areas.len();

        // max_v_offset
        let mut n = 0;
        let mut height = 0;
        for item in self.items.iter().rev() {
            height += item.height();
            if height > state.inner.height as usize {
                break;
            }
            n += 1;
        }
        state.v_max_offset = state.len.saturating_sub(n);

        let (style, select_style) = if state.is_focused() {
            (self.focus_style, self.select_style.patch(self.focus_style))
        } else {
            (self.style, self.select_style)
        };

        // rendering
        self.items = self
            .items
            .into_iter()
            .enumerate()
            .map(|(i, v)| {
                if state.selection.is_selected(i) {
                    v.style(select_style)
                } else {
                    v.style(style)
                }
            })
            .collect();

        let mut list = ratatui::widgets::List::default()
            .items(self.items)
            .style(self.style)
            .direction(self.direction);
        if let Some(block) = self.block {
            list = list.block(block);
        }
        let mut list_state = ratatui::widgets::ListState::default().with_offset(state.v_offset);
        list.render(area, buf, &mut list_state);
    }
}

impl<Selection> HasFocusFlag for ListState<Selection> {
    #[inline]
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    #[inline]
    fn area(&self) -> Rect {
        self.area
    }
}

impl<Selection> ScrollingState for ListState<Selection> {
    #[inline]
    fn vertical_max_offset(&self) -> usize {
        self.v_max_offset
    }

    #[inline]
    fn vertical_offset(&self) -> usize {
        self.v_offset
    }

    #[inline]
    fn vertical_page(&self) -> usize {
        self.v_page_len
    }

    #[inline]
    fn horizontal_max_offset(&self) -> usize {
        0
    }

    #[inline]
    fn horizontal_offset(&self) -> usize {
        0
    }

    #[inline]
    fn horizontal_page(&self) -> usize {
        0
    }

    #[inline]
    fn set_vertical_offset(&mut self, offset: usize) -> bool {
        let old_offset = self.v_offset;
        self.v_offset = min(offset, self.len - 1);
        old_offset != self.v_offset
    }

    #[inline]
    fn set_horizontal_offset(&mut self, _offset: usize) -> bool {
        false
    }
}

impl<Selection: ListSelection> ListState<Selection> {
    #[inline]
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.v_offset = offset;
        self
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.v_offset
    }

    #[inline]
    pub fn offset_mut(&mut self) -> &mut usize {
        &mut self.v_offset
    }

    #[inline]
    pub fn row_at_clicked(&self, pos: Position) -> Option<usize> {
        row_at_clicked(&self.item_areas, pos.y).map(|v| self.v_offset + v)
    }

    /// Row when dragging. Can go outside the area.
    #[inline]
    pub fn row_at_drag(&self, pos: Position) -> usize {
        match row_at_drag(self.inner, &self.item_areas, pos.y) {
            Ok(v) => self.v_offset + v,
            Err(v) if v <= 0 => self.v_offset.saturating_sub((-v) as usize),
            Err(v) => self.v_offset + self.item_areas.len() + v as usize,
        }
    }

    /// Scroll to selected.
    #[inline]
    pub fn scroll_to_selected(&mut self) {
        if let Some(selected) = self.selection.lead_selection() {
            if self.vertical_offset() + self.item_areas.len() <= selected {
                self.set_vertical_offset(selected - self.item_areas.len() + 1);
            }
            if self.vertical_offset() > selected {
                self.set_vertical_offset(selected);
            }
        }
    }

    #[inline]
    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    #[inline]
    pub fn selection_mut(&mut self) -> &mut Selection {
        &mut self.selection
    }
}

impl ListState<RowSelection> {
    #[inline]
    pub fn with_selected(mut self, selected: Option<usize>) -> Self {
        self.selection.lead_row = selected;
        self
    }

    /// Returns the lead selection.
    #[inline]
    pub fn selected(&self) -> Option<usize> {
        self.selection.lead_selection()
    }

    #[inline]
    pub fn selected_mut(&mut self) -> &mut Option<usize> {
        &mut self.selection.lead_row
    }

    #[inline]
    pub fn select(&mut self, n: Option<usize>) -> bool {
        self.selection.select(n)
    }
}

impl ListState<RowSetSelection> {
    #[inline]
    pub fn selected(&self) -> HashSet<usize> {
        self.selection.selected()
    }

    #[inline]
    pub fn set_lead(&mut self, lead: Option<usize>, extend: bool) {
        self.selection.set_lead(lead, extend);
    }

    /// Set a new lead, at the same time limit the lead to max.
    #[inline]
    pub fn set_lead_clamped(&mut self, lead: usize, max: usize, extend: bool) {
        self.selection.set_lead_clamped(lead, max, extend);
    }

    /// Current lead.
    #[inline]
    pub fn lead(&self) -> Option<usize> {
        self.selection.lead()
    }

    /// Current anchor.
    #[inline]
    pub fn anchor(&self) -> Option<usize> {
        self.selection.anchor()
    }

    /// Clear the selection.
    #[inline]
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Add to selection.
    #[inline]
    pub fn add_selected(&mut self, idx: usize) {
        self.selection.add(idx);
    }

    /// Remove from selection. Only works for retired selections, not for the
    /// active anchor-lead range.
    #[inline]
    pub fn remove_selected(&mut self, idx: usize) {
        self.selection.remove(idx);
    }
}

pub mod selection {
    use crate::event::{ct_event, ConsumedEvent, FocusKeys, HandleEvent, MouseOnly, Outcome};
    use crate::list::{ListSelection, ListState};
    use crossterm::event::KeyModifiers;
    use rat_focus::HasFocusFlag;
    use rat_ftable::TableSelection;
    use rat_scrolled::ScrollingState;
    use ratatui::layout::Position;
    use std::mem;

    /// No selection
    pub type NoSelection = rat_ftable::selection::NoSelection;

    impl ListSelection for NoSelection {
        #[inline]
        fn is_selected(&self, _n: usize) -> bool {
            false
        }

        #[inline]
        fn lead_selection(&self) -> Option<usize> {
            None
        }
    }

    impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for ListState<NoSelection> {
        fn handle(&mut self, _event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
            Outcome::NotUsed
        }
    }

    impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for ListState<NoSelection> {
        fn handle(&mut self, _event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
            Outcome::NotUsed
        }
    }

    /// Single element selection.
    pub type RowSelection = rat_ftable::selection::RowSelection;

    impl ListSelection for RowSelection {
        #[inline]
        fn is_selected(&self, n: usize) -> bool {
            self.lead_row == Some(n)
        }

        #[inline]
        fn lead_selection(&self) -> Option<usize> {
            self.lead_row
        }
    }

    impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for ListState<RowSelection> {
        fn handle(&mut self, event: &crossterm::event::Event, _keymap: FocusKeys) -> Outcome {
            let res = if self.is_focused() {
                match event {
                    ct_event!(keycode press Down) => {
                        let r = self.selection.next(1, self.len.saturating_sub(1)).into();
                        self.scroll_to_selected();
                        r
                    }
                    ct_event!(keycode press Up) => {
                        let r = self.selection.prev(1).into();
                        self.scroll_to_selected();
                        r
                    }
                    ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press End) => {
                        let r = self
                            .selection
                            .select_clamped(self.len.saturating_sub(1), self.len.saturating_sub(1))
                            .into();
                        self.scroll_to_selected();
                        r
                    }
                    ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press Home) => {
                        let r = self
                            .selection
                            .select_clamped(0, self.len.saturating_sub(1))
                            .into();
                        self.scroll_to_selected();
                        r
                    }
                    ct_event!(keycode press PageUp) => {
                        let r = self.selection.prev(self.vertical_page() / 2).into();
                        self.scroll_to_selected();
                        r
                    }
                    ct_event!(keycode press PageDown) => {
                        let r = self
                            .selection
                            .next(self.vertical_page(), self.len.saturating_sub(1))
                            .into();
                        self.scroll_to_selected();
                        r
                    }
                    _ => Outcome::NotUsed,
                }
            } else {
                Outcome::NotUsed
            };

            if !res.is_consumed() {
                self.handle(event, MouseOnly)
            } else {
                res
            }
        }
    }

    impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for ListState<RowSelection> {
        fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
            match event {
                ct_event!(mouse any for m) if self.mouse.drag(self.inner, m) => {
                    let new_row = self.row_at_drag((m.column, m.row).into());
                    let r = self
                        .selection
                        .select_clamped(new_row, self.len.saturating_sub(1))
                        .into();
                    self.scroll_to_selected();
                    r
                }
                ct_event!(scroll down for column,row) => {
                    if self.area.contains(Position::new(*column, *row)) {
                        self.scroll_down(self.vertical_page() / 10).into()
                    } else {
                        Outcome::NotUsed
                    }
                }
                ct_event!(scroll up for column, row) => {
                    if self.area.contains(Position::new(*column, *row)) {
                        self.scroll_up(self.vertical_page() / 10).into()
                    } else {
                        Outcome::NotUsed
                    }
                }
                ct_event!(mouse down Left for column, row) => {
                    let pos = Position::new(*column, *row);
                    if self.area.contains(pos) {
                        if let Some(new_row) = self.row_at_clicked(pos) {
                            self.selection
                                .select_clamped(new_row, self.len.saturating_sub(1))
                                .into()
                        } else {
                            Outcome::NotUsed
                        }
                    } else {
                        Outcome::NotUsed
                    }
                }

                _ => Outcome::NotUsed,
            }
        }
    }

    pub type RowSetSelection = rat_ftable::selection::RowSetSelection;

    impl ListSelection for RowSetSelection {
        fn is_selected(&self, n: usize) -> bool {
            if let Some(mut anchor) = self.anchor {
                if let Some(mut lead) = self.lead {
                    if lead < anchor {
                        mem::swap(&mut lead, &mut anchor);
                    }

                    if n >= anchor && n <= lead {
                        return true;
                    }
                }
            } else {
                if let Some(lead) = self.lead {
                    if n == lead {
                        return true;
                    }
                }
            }

            self.selected.contains(&n)
        }

        fn lead_selection(&self) -> Option<usize> {
            self.lead
        }
    }

    impl HandleEvent<crossterm::event::Event, FocusKeys, Outcome> for ListState<RowSetSelection> {
        fn handle(&mut self, event: &crossterm::event::Event, _: FocusKeys) -> Outcome {
            let res = {
                match event {
                    ct_event!(keycode press Down) => {
                        let r = self.selection.next(1, self.len - 1, false).into();
                        self.scroll_to_selected();
                        r
                    }
                    ct_event!(keycode press SHIFT-Down) => {
                        let r = self.selection.next(1, self.len - 1, true).into();
                        self.scroll_to_selected();
                        r
                    }
                    ct_event!(keycode press Up) => {
                        let r = self.selection.prev(1, false).into();
                        self.scroll_to_selected();
                        r
                    }
                    ct_event!(keycode press SHIFT-Up) => {
                        let r = self.selection.prev(1, true).into();
                        self.scroll_to_selected();
                        r
                    }
                    ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press End) => {
                        let r = self.selection.set_lead(Some(self.len - 1), false).into();
                        self.scroll_to_selected();
                        r
                    }
                    ct_event!(keycode press SHIFT-End) => {
                        let r = self.selection.set_lead(Some(self.len - 1), true).into();
                        self.scroll_to_selected();
                        r
                    }
                    ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press Home) => {
                        let r = self.selection.set_lead(Some(0), false).into();
                        self.scroll_to_selected();
                        r
                    }
                    ct_event!(keycode press SHIFT-Home) => {
                        let r = self.selection.set_lead(Some(0), true).into();
                        self.scroll_to_selected();
                        r
                    }

                    ct_event!(keycode press PageUp) => {
                        let r = self.selection.prev(self.v_page_len, false).into();
                        self.scroll_to_selected();
                        r
                    }
                    ct_event!(keycode press SHIFT-PageUp) => {
                        let r = self.selection.prev(self.v_page_len, true).into();
                        self.scroll_to_selected();
                        r
                    }
                    ct_event!(keycode press PageDown) => {
                        let r = self
                            .selection
                            .next(self.v_page_len, self.len - 1, false)
                            .into();
                        self.scroll_to_selected();
                        r
                    }
                    ct_event!(keycode press SHIFT-PageDown) => {
                        let r = self
                            .selection
                            .next(self.v_page_len, self.len - 1, true)
                            .into();
                        self.scroll_to_selected();
                        r
                    }
                    _ => Outcome::NotUsed,
                }
            };

            if res == Outcome::NotUsed {
                self.handle(event, MouseOnly)
            } else {
                res
            }
        }
    }

    impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for ListState<RowSetSelection> {
        fn handle(&mut self, event: &crossterm::event::Event, _: MouseOnly) -> Outcome {
            match event {
                ct_event!(mouse any for m) | ct_event!(mouse any CONTROL for m)
                    if self.mouse.drag(self.inner, m)
                        || self.mouse.drag2(self.inner, m, KeyModifiers::CONTROL) =>
                {
                    let new_row = self.row_at_drag((m.column, m.row).into());
                    let r = self
                        .selection
                        .set_lead_clamped(new_row, self.len - 1, true)
                        .into();
                    self.scroll_to_selected();
                    r
                }
                ct_event!(scroll up for column, row) => {
                    if self.area.contains(Position::new(*column, *row)) {
                        self.scroll_up(self.vertical_scroll()).into()
                    } else {
                        Outcome::NotUsed
                    }
                }
                ct_event!(scroll down for column, row) => {
                    if self.area.contains(Position::new(*column, *row)) {
                        self.scroll_down(self.vertical_scroll()).into()
                    } else {
                        Outcome::NotUsed
                    }
                }
                ct_event!(mouse down Left for column, row) => {
                    let pos = Position::new(*column, *row);
                    if self.area.contains(pos) {
                        if let Some(new_row) = self.row_at_clicked(pos) {
                            self.selection
                                .set_lead_clamped(new_row, self.len - 1, false)
                                .into()
                        } else {
                            Outcome::Unchanged
                        }
                    } else {
                        Outcome::NotUsed
                    }
                }
                ct_event!(mouse down ALT-Left for column, row) => {
                    let pos = Position::new(*column, *row);
                    if self.area.contains(pos) {
                        if let Some(new_row) = self.row_at_clicked(pos) {
                            self.selection
                                .set_lead_clamped(new_row, self.len - 1, true)
                                .into()
                        } else {
                            Outcome::Unchanged
                        }
                    } else {
                        Outcome::NotUsed
                    }
                }
                ct_event!(mouse down CONTROL-Left for column, row) => {
                    if self.area.contains(Position::new(*column, *row)) {
                        let pos = Position::new(*column, *row);
                        if let Some(new_row) = self.row_at_clicked(pos) {
                            self.selection.transfer_lead_anchor();
                            if self.selection.is_selected_row(new_row) {
                                self.selection.remove(new_row);
                            } else {
                                self.selection.set_lead_clamped(new_row, self.len - 1, true);
                            }
                            Outcome::Changed
                        } else {
                            Outcome::Unchanged
                        }
                    } else {
                        Outcome::NotUsed
                    }
                }

                _ => Outcome::NotUsed,
            }
        }
    }
}
