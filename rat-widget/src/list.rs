//!
//! Extensions for ratatui List.
//!

use crate::_private::NonExhaustive;
use crate::event::util::MouseFlags;
use crate::list::selection::{RowSelection, RowSetSelection};
use crate::text::HasScreenCursor;
use crate::util::{fallback_select_style, revert_style};
use rat_event::{HandleEvent, MouseOnly, Outcome, Regular};
use rat_focus::{FocusBuilder, FocusFlag, HasFocus};
use rat_reloc::{RelocatableState, relocate_areas};
use rat_scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState, ScrollStyle};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, HighlightSpacing, ListDirection, ListItem, StatefulWidget};
use std::cmp::min;
use std::collections::HashSet;
use std::marker::PhantomData;

pub mod edit;

/// Trait for list-selection.
pub trait ListSelection {
    /// Number of selected rows.
    fn count(&self) -> usize;

    /// Is selected.
    fn is_selected(&self, n: usize) -> bool;

    /// Selection lead.
    fn lead_selection(&self) -> Option<usize>;

    /// Scroll the selection instead of the offset.
    fn scroll_selected(&self) -> bool {
        false
    }
}

/// List widget.
///
/// Fully compatible with ratatui List.
/// Adds Scroll, selection models, and event-handling.
#[derive(Debug, Default, Clone)]
pub struct List<'a, Selection = RowSelection> {
    block: Option<Block<'a>>,
    scroll: Option<Scroll<'a>>,

    items: Vec<ListItem<'a>>,

    style: Style,
    select_style: Option<Style>,
    focus_style: Option<Style>,
    direction: ListDirection,
    highlight_spacing: HighlightSpacing,
    highlight_symbol: Option<&'static str>,
    repeat_highlight_symbol: bool,
    scroll_padding: usize,

    _phantom: PhantomData<Selection>,
}

/// Collected styles.
#[derive(Debug, Clone)]
pub struct ListStyle {
    /// Style
    pub style: Style,
    /// Style for selection
    pub select: Option<Style>,
    /// Style for selection when focused.
    pub focus: Option<Style>,

    pub block: Option<Block<'static>>,
    pub scroll: Option<ScrollStyle>,

    pub highlight_spacing: Option<HighlightSpacing>,
    pub highlight_symbol: Option<&'static str>,
    pub repeat_highlight_symbol: Option<bool>,
    pub scroll_padding: Option<usize>,

    pub non_exhaustive: NonExhaustive,
}

/// State & event handling.
#[derive(Debug, PartialEq, Eq)]
pub struct ListState<Selection = RowSelection> {
    /// Total area
    /// __readonly__. renewed for each render.
    pub area: Rect,
    /// Area inside the block.
    /// __readonly__. renewed for each render.
    pub inner: Rect,
    /// Areas for the rendered items.
    /// __readonly__. renewed for each render.
    pub row_areas: Vec<Rect>,

    /// Length in items.
    /// __mostly readonly__. renewed for each render.
    pub rows: usize,
    /// Offset etc.
    /// __read+write__
    pub scroll: ScrollState,

    /// Focus
    /// __read+write__
    pub focus: FocusFlag,
    /// Selection model
    /// __read+write__
    pub selection: Selection,

    /// Helper for mouse events.
    /// __used for mouse interaction__
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl Default for ListStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            select: None,
            focus: None,
            block: None,
            scroll: None,
            highlight_spacing: None,
            highlight_symbol: None,
            repeat_highlight_symbol: None,
            scroll_padding: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a, Selection> List<'a, Selection> {
    /// New list.
    pub fn new<T>(items: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<ListItem<'a>>,
    {
        let items = items.into_iter().map(|v| v.into()).collect();

        Self {
            block: None,
            scroll: None,
            items,
            style: Default::default(),
            select_style: Default::default(),
            focus_style: Default::default(),
            direction: Default::default(),
            highlight_spacing: Default::default(),
            highlight_symbol: Default::default(),
            repeat_highlight_symbol: false,
            scroll_padding: 0,
            _phantom: Default::default(),
        }
    }

    /// Set items.
    pub fn items<T>(mut self, items: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<ListItem<'a>>,
    {
        let items = items.into_iter().map(|v| v.into()).collect();
        self.items = items;
        self
    }

    /// Border support.
    #[inline]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Scroll support.
    #[inline]
    pub fn scroll(mut self, scroll: Scroll<'a>) -> Self {
        self.scroll = Some(scroll);
        self
    }

    /// Set all styles.
    #[inline]
    pub fn styles_opt(self, styles: Option<ListStyle>) -> Self {
        if let Some(styles) = styles {
            self.styles(styles)
        } else {
            self
        }
    }

    /// Set all styles.
    #[inline]
    pub fn styles(mut self, styles: ListStyle) -> Self {
        self.style = styles.style;
        if styles.select.is_some() {
            self.select_style = styles.select;
        }
        if styles.focus.is_some() {
            self.focus_style = styles.focus;
        }
        if let Some(styles) = styles.scroll {
            self.scroll = self.scroll.map(|v| v.styles(styles));
        }
        if let Some(block) = styles.block {
            self.block = Some(block);
        }
        if let Some(highlight_spacing) = styles.highlight_spacing {
            self.highlight_spacing = highlight_spacing;
        }
        if let Some(highlight_symbol) = styles.highlight_symbol {
            self.highlight_symbol = Some(highlight_symbol);
        }
        if let Some(repeat_highlight_symbol) = styles.repeat_highlight_symbol {
            self.repeat_highlight_symbol = repeat_highlight_symbol;
        }
        if let Some(scroll_padding) = styles.scroll_padding {
            self.scroll_padding = scroll_padding;
        }
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Base style
    #[inline]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self.block = self.block.map(|v| v.style(self.style));
        self
    }

    /// Select style.
    #[inline]
    pub fn select_style<S: Into<Style>>(mut self, select_style: S) -> Self {
        self.select_style = Some(select_style.into());
        self
    }

    /// Focused style.
    #[inline]
    pub fn focus_style<S: Into<Style>>(mut self, focus_style: S) -> Self {
        self.focus_style = Some(focus_style.into());
        self
    }

    /// List direction.
    #[inline]
    pub fn direction(mut self, direction: ListDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Number of items.
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Empty?
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

impl<Selection: ListSelection> StatefulWidget for List<'_, Selection> {
    type State = ListState<Selection>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_list(self, area, buf, state)
    }
}

fn render_list<Selection: ListSelection>(
    widget: List<'_, Selection>,
    area: Rect,
    buf: &mut Buffer,
    state: &mut ListState<Selection>,
) {
    state.area = area;
    state.rows = widget.items.len();

    let sa = ScrollArea::new()
        .block(widget.block.as_ref())
        .v_scroll(widget.scroll.as_ref());
    state.inner = sa.inner(area, None, Some(&state.scroll));

    // area for each item
    state.row_areas.clear();
    let mut item_area = Rect::new(state.inner.x, state.inner.y, state.inner.width, 1);
    let mut total_height = 0;
    for item in widget.items.iter().skip(state.offset()) {
        item_area.height = item.height() as u16;

        state.row_areas.push(item_area);

        item_area.y += item_area.height;
        total_height += item_area.height;
        if total_height >= state.inner.height {
            break;
        }
    }
    if total_height < state.inner.height {
        state.scroll.set_page_len(
            state.row_areas.len() + state.inner.height as usize - total_height as usize,
        );
    } else {
        state.scroll.set_page_len(state.row_areas.len());
    }

    let focus_style = widget.focus_style.unwrap_or(revert_style(widget.style));
    let select_style = widget
        .select_style
        .unwrap_or(fallback_select_style(widget.style));

    // max_v_offset
    let mut n = 0;
    let mut height = 0;
    for item in widget.items.iter().rev() {
        height += item.height();
        if height > state.inner.height as usize {
            break;
        }
        n += 1;
    }
    state.scroll.set_max_offset(state.rows.saturating_sub(n));

    let (style, select_style) = if state.is_focused() {
        (widget.style, focus_style)
    } else {
        (widget.style, select_style)
    };

    sa.render(
        area,
        buf,
        &mut ScrollAreaState::new().v_scroll(&mut state.scroll),
    );

    // rendering
    let items = widget
        .items
        .into_iter()
        .enumerate()
        .map(|(i, v)| {
            if state.selection.is_selected(i) {
                v.style(style.patch(select_style))
            } else {
                v.style(style)
            }
        })
        .collect::<Vec<_>>();

    let mut list_state = ratatui::widgets::ListState::default().with_offset(state.scroll.offset());

    let mut list = ratatui::widgets::List::default()
        .items(items)
        .style(widget.style)
        .direction(widget.direction)
        .highlight_spacing(widget.highlight_spacing)
        .repeat_highlight_symbol(widget.repeat_highlight_symbol)
        .scroll_padding(widget.scroll_padding);
    if let Some(highlight_symbol) = widget.highlight_symbol {
        list = list.highlight_symbol(highlight_symbol);
    }
    list.render(state.inner, buf, &mut list_state);
}

impl<Selection> HasFocus for ListState<Selection> {
    fn build(&self, builder: &mut FocusBuilder) {
        builder.leaf_widget(self);
    }

    #[inline]
    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    #[inline]
    fn area(&self) -> Rect {
        self.area
    }
}

impl<Selection> HasScreenCursor for ListState<Selection> {
    fn screen_cursor(&self) -> Option<(u16, u16)> {
        None
    }
}

impl<Selection: Default> Default for ListState<Selection> {
    fn default() -> Self {
        Self {
            area: Default::default(),
            inner: Default::default(),
            row_areas: Default::default(),
            rows: Default::default(),
            scroll: Default::default(),
            focus: Default::default(),
            selection: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Selection: Clone> Clone for ListState<Selection> {
    fn clone(&self) -> Self {
        Self {
            area: self.area,
            inner: self.inner,
            row_areas: self.row_areas.clone(),
            rows: self.rows,
            scroll: self.scroll.clone(),
            focus: self.focus.new_instance(),
            selection: self.selection.clone(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<Selection> RelocatableState for ListState<Selection> {
    fn relocate(&mut self, shift: (i16, i16), clip: Rect) {
        self.area.relocate(shift, clip);
        self.inner.relocate(shift, clip);
        relocate_areas(self.row_areas.as_mut_slice(), shift, clip);
        self.scroll.relocate(shift, clip);
    }
}

impl<Selection: ListSelection> ListState<Selection> {
    /// New initial state.
    pub fn new() -> Self
    where
        Selection: Default,
    {
        Default::default()
    }

    /// New state with a focus name
    pub fn named(name: &str) -> Self
    where
        Selection: Default,
    {
        let mut z = Self::default();
        z.focus = z.focus.with_name(name);
        z
    }

    #[inline]
    pub fn rows(&self) -> usize {
        self.rows
    }

    #[inline]
    pub fn clear_offset(&mut self) {
        self.scroll.set_offset(0);
    }

    #[inline]
    pub fn max_offset(&self) -> usize {
        self.scroll.max_offset()
    }

    #[inline]
    pub fn set_max_offset(&mut self, max: usize) {
        self.scroll.set_max_offset(max);
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.scroll.offset()
    }

    #[inline]
    pub fn set_offset(&mut self, offset: usize) -> bool {
        self.scroll.set_offset(offset)
    }

    #[inline]
    pub fn page_len(&self) -> usize {
        self.scroll.page_len()
    }

    pub fn scroll_by(&self) -> usize {
        self.scroll.scroll_by()
    }

    /// Scroll to selected.
    #[inline]
    pub fn scroll_to_selected(&mut self) -> bool {
        if let Some(selected) = self.selection.lead_selection() {
            self.scroll_to(selected)
        } else {
            false
        }
    }

    #[inline]
    pub fn scroll_to(&mut self, pos: usize) -> bool {
        if pos >= self.offset() + self.page_len() {
            self.set_offset(pos - self.page_len() + 1)
        } else if pos < self.offset() {
            self.set_offset(pos)
        } else {
            false
        }
    }

    #[inline]
    pub fn scroll_up(&mut self, n: usize) -> bool {
        self.scroll.scroll_up(n)
    }

    #[inline]
    pub fn scroll_down(&mut self, n: usize) -> bool {
        self.scroll.scroll_down(n)
    }
}

impl<Selection: ListSelection> ListState<Selection> {
    /// Returns the row-area for the given row, if it is visible.
    pub fn row_area(&self, row: usize) -> Option<Rect> {
        if row < self.scroll.offset() || row >= self.scroll.offset() + self.scroll.page_len() {
            return None;
        }
        let row = row - self.scroll.offset;
        if row >= self.row_areas.len() {
            return None;
        }
        Some(self.row_areas[row - self.scroll.offset])
    }

    #[inline]
    pub fn row_at_clicked(&self, pos: (u16, u16)) -> Option<usize> {
        self.mouse
            .row_at(&self.row_areas, pos.1)
            .map(|v| self.scroll.offset() + v)
    }

    /// Row when dragging. Can go outside the area.
    #[inline]
    pub fn row_at_drag(&self, pos: (u16, u16)) -> usize {
        match self.mouse.row_at_drag(self.inner, &self.row_areas, pos.1) {
            Ok(v) => self.scroll.offset() + v,
            Err(v) if v <= 0 => self.scroll.offset().saturating_sub((-v) as usize),
            Err(v) => self.scroll.offset() + self.row_areas.len() + v as usize,
        }
    }
}

impl ListState<RowSelection> {
    /// Update the state to match adding items.
    ///
    /// This corrects the number of rows, offset and selection.
    pub fn items_added(&mut self, pos: usize, n: usize) {
        self.scroll.items_added(pos, n);
        self.selection.items_added(pos, n);
        self.rows += n;
    }

    /// Update the state to match removing items.
    ///
    /// This corrects the number of rows, offset and selection.
    pub fn items_removed(&mut self, pos: usize, n: usize) {
        self.scroll.items_removed(pos, n);
        self.selection
            .items_removed(pos, n, self.rows.saturating_sub(1));
        self.rows -= n;
    }

    /// When scrolling the table, change the selection instead of the offset.
    #[inline]
    pub fn set_scroll_selection(&mut self, scroll: bool) {
        self.selection.set_scroll_selected(scroll);
    }

    /// Scroll delivers a value between 0 and max_offset as offset.
    /// This remaps the ratio to the selection with a range 0..row_len.
    ///
    pub(crate) fn remap_offset_selection(&self, offset: usize) -> usize {
        if self.scroll.max_offset() > 0 {
            (self.rows * offset) / self.scroll.max_offset()
        } else {
            0 // ???
        }
    }

    /// Clear the selection.
    #[inline]
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Anything selected?
    #[inline]
    pub fn has_selection(&mut self) -> bool {
        self.selection.has_selection()
    }

    /// Returns the lead selection.
    #[inline]
    pub fn selected(&self) -> Option<usize> {
        self.selection.lead_selection()
    }

    /// Returns the lead selection. Ensures the index is
    /// less than rows.
    #[inline]
    pub fn selected_checked(&self) -> Option<usize> {
        self.selection.lead_selection().filter(|v| *v < self.rows)
    }

    #[inline]
    pub fn select(&mut self, row: Option<usize>) -> bool {
        self.selection.select(row)
    }

    /// Move the selection to the given row. Limits the movement to the row-count.
    /// Ensures the row is visible afterwards.
    #[inline]
    pub fn move_to(&mut self, row: usize) -> bool {
        let r = self.selection.move_to(row, self.rows.saturating_sub(1));
        let s = self.scroll_to(self.selection.selected().expect("row"));
        r || s
    }

    /// Move the selection up n rows.
    /// Ensures the row is visible afterwards.
    #[inline]
    pub fn move_up(&mut self, n: usize) -> bool {
        let r = self.selection.move_up(n, self.rows.saturating_sub(1));
        let s = self.scroll_to(self.selection.selected().expect("row"));
        r || s
    }

    /// Move the selection down n rows.
    /// Ensures the row is visible afterwards.
    #[inline]
    pub fn move_down(&mut self, n: usize) -> bool {
        let r = self.selection.move_down(n, self.rows.saturating_sub(1));
        let s = self.scroll_to(self.selection.selected().expect("row"));
        r || s
    }
}

impl ListState<RowSetSelection> {
    /// Clear the selection.
    #[inline]
    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Anything selected?
    #[inline]
    pub fn has_selection(&mut self) -> bool {
        self.selection.has_selection()
    }

    #[inline]
    pub fn selected(&self) -> HashSet<usize> {
        self.selection.selected()
    }

    /// Change the lead-selection. Limits the value to the number of rows.
    /// If extend is false the current selection is cleared and both lead and
    /// anchor are set to the given value.
    /// If extend is true, the anchor is kept where it is and lead is changed.
    /// Everything in the range `anchor..lead` is selected. It doesn't matter
    /// if anchor < lead.
    #[inline]
    pub fn set_lead(&mut self, row: Option<usize>, extend: bool) -> bool {
        if let Some(row) = row {
            self.selection
                .set_lead(Some(min(row, self.rows.saturating_sub(1))), extend)
        } else {
            self.selection.set_lead(None, extend)
        }
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

    /// Set a new lead, at the same time limit the lead to max.
    #[inline]
    pub fn set_lead_clamped(&mut self, lead: usize, max: usize, extend: bool) {
        self.selection.move_to(lead, max, extend);
    }

    /// Retire the current anchor/lead selection to the set of selected rows.
    /// Resets lead and anchor and starts a new selection round.
    #[inline]
    pub fn retire_selection(&mut self) {
        self.selection.retire_selection();
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

    /// Move the selection to the given row.
    /// Ensures the row is visible afterwards.
    #[inline]
    pub fn move_to(&mut self, row: usize, extend: bool) -> bool {
        let r = self
            .selection
            .move_to(row, self.rows.saturating_sub(1), extend);
        let s = self.scroll_to(self.selection.lead().expect("row"));
        r || s
    }

    /// Move the selection up n rows.
    /// Ensures the row is visible afterwards.
    #[inline]
    pub fn move_up(&mut self, n: usize, extend: bool) -> bool {
        let r = self
            .selection
            .move_up(n, self.rows.saturating_sub(1), extend);
        let s = self.scroll_to(self.selection.lead().expect("row"));
        r || s
    }

    /// Move the selection down n rows.
    /// Ensures the row is visible afterwards.
    #[inline]
    pub fn move_down(&mut self, n: usize, extend: bool) -> bool {
        let r = self
            .selection
            .move_down(n, self.rows.saturating_sub(1), extend);
        let s = self.scroll_to(self.selection.lead().expect("row"));
        r || s
    }
}

pub mod selection {
    //!
    //! Different selection models.
    //!

    use crate::event::{HandleEvent, MouseOnly, Outcome, Regular, ct_event, flow};
    use crate::list::{ListSelection, ListState};
    use crossterm::event::KeyModifiers;
    use rat_focus::HasFocus;
    use rat_ftable::TableSelection;
    use rat_scrolled::ScrollAreaState;
    use rat_scrolled::event::ScrollOutcome;
    use std::mem;

    /// No selection
    pub type NoSelection = rat_ftable::selection::NoSelection;

    impl ListSelection for NoSelection {
        fn count(&self) -> usize {
            0
        }

        #[inline]
        fn is_selected(&self, _n: usize) -> bool {
            false
        }

        #[inline]
        fn lead_selection(&self) -> Option<usize> {
            None
        }
    }

    impl HandleEvent<crossterm::event::Event, Regular, Outcome> for ListState<NoSelection> {
        fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> Outcome {
            let res = if self.is_focused() {
                match event {
                    ct_event!(keycode press Down) => self.scroll_down(1).into(),
                    ct_event!(keycode press Up) => self.scroll_up(1).into(),
                    ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press End) => {
                        self.scroll_to(self.max_offset()).into()
                    }
                    ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press Home) => {
                        self.scroll_to(0).into()
                    }
                    ct_event!(keycode press PageUp) => {
                        self.scroll_up(self.page_len().saturating_sub(1)).into()
                    }
                    ct_event!(keycode press PageDown) => {
                        self.scroll_down(self.page_len().saturating_sub(1)).into()
                    }
                    _ => Outcome::Continue,
                }
            } else {
                Outcome::Continue
            };

            if res == Outcome::Continue {
                self.handle(event, MouseOnly)
            } else {
                res
            }
        }
    }

    impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for ListState<NoSelection> {
        fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
            let mut sas = ScrollAreaState::new()
                .area(self.inner)
                .v_scroll(&mut self.scroll);
            let r = match sas.handle(event, MouseOnly) {
                ScrollOutcome::Up(v) => self.scroll_up(v),
                ScrollOutcome::Down(v) => self.scroll_down(v),
                ScrollOutcome::VPos(v) => self.set_offset(v),
                ScrollOutcome::Left(_) => false,
                ScrollOutcome::Right(_) => false,
                ScrollOutcome::HPos(_) => false,

                ScrollOutcome::Continue => false,
                ScrollOutcome::Unchanged => false,
                ScrollOutcome::Changed => true,
            };
            if r {
                return Outcome::Changed;
            }

            Outcome::Unchanged
        }
    }

    /// Single element selection.
    pub type RowSelection = rat_ftable::selection::RowSelection;

    impl ListSelection for RowSelection {
        fn count(&self) -> usize {
            if self.lead_row.is_some() { 1 } else { 0 }
        }

        #[inline]
        fn is_selected(&self, n: usize) -> bool {
            self.lead_row == Some(n)
        }

        #[inline]
        fn lead_selection(&self) -> Option<usize> {
            self.lead_row
        }

        fn scroll_selected(&self) -> bool {
            self.scroll_selected
        }
    }

    impl HandleEvent<crossterm::event::Event, Regular, Outcome> for ListState<RowSelection> {
        fn handle(&mut self, event: &crossterm::event::Event, _keymap: Regular) -> Outcome {
            let res = if self.is_focused() {
                match event {
                    ct_event!(keycode press Down) => self.move_down(1).into(),
                    ct_event!(keycode press Up) => self.move_up(1).into(),
                    ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press End) => {
                        self.move_to(self.rows.saturating_sub(1)).into()
                    }
                    ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press Home) => {
                        self.move_to(0).into()
                    }
                    ct_event!(keycode press PageUp) => {
                        self.move_up(self.page_len().saturating_sub(1)).into()
                    }
                    ct_event!(keycode press PageDown) => {
                        self.move_down(self.page_len().saturating_sub(1)).into()
                    }
                    _ => Outcome::Continue,
                }
            } else {
                Outcome::Continue
            };

            if res == Outcome::Continue {
                self.handle(event, MouseOnly)
            } else {
                res
            }
        }
    }

    impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for ListState<RowSelection> {
        fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> Outcome {
            flow!(match event {
                ct_event!(mouse any for m) if self.mouse.drag(self.inner, m) => {
                    self.move_to(self.row_at_drag((m.column, m.row))).into()
                }
                ct_event!(mouse down Left for column, row) => {
                    if self.inner.contains((*column, *row).into()) {
                        if let Some(new_row) = self.row_at_clicked((*column, *row)) {
                            self.move_to(new_row).into()
                        } else {
                            Outcome::Continue
                        }
                    } else {
                        Outcome::Continue
                    }
                }

                _ => Outcome::Continue,
            });

            let mut sas = ScrollAreaState::new()
                .area(self.inner)
                .v_scroll(&mut self.scroll);
            let r = match sas.handle(event, MouseOnly) {
                ScrollOutcome::Up(v) => {
                    if ListSelection::scroll_selected(&self.selection) {
                        self.move_up(1)
                    } else {
                        self.scroll_up(v)
                    }
                }
                ScrollOutcome::Down(v) => {
                    if ListSelection::scroll_selected(&self.selection) {
                        self.move_down(1)
                    } else {
                        self.scroll_down(v)
                    }
                }
                ScrollOutcome::VPos(v) => {
                    if ListSelection::scroll_selected(&self.selection) {
                        self.move_to(self.remap_offset_selection(v))
                    } else {
                        self.set_offset(v)
                    }
                }
                ScrollOutcome::Left(_) => false,
                ScrollOutcome::Right(_) => false,
                ScrollOutcome::HPos(_) => false,

                ScrollOutcome::Continue => false,
                ScrollOutcome::Unchanged => false,
                ScrollOutcome::Changed => true,
            };
            if r {
                return Outcome::Changed;
            }

            Outcome::Continue
        }
    }

    pub type RowSetSelection = rat_ftable::selection::RowSetSelection;

    impl ListSelection for RowSetSelection {
        fn count(&self) -> usize {
            let n = if let Some(anchor) = self.anchor_row {
                if let Some(lead) = self.lead_row {
                    anchor.abs_diff(lead) + 1
                } else {
                    0
                }
            } else {
                0
            };

            n + self.selected.len()
        }

        fn is_selected(&self, n: usize) -> bool {
            if let Some(mut anchor) = self.anchor_row {
                if let Some(mut lead) = self.lead_row {
                    if lead < anchor {
                        mem::swap(&mut lead, &mut anchor);
                    }

                    if n >= anchor && n <= lead {
                        return true;
                    }
                }
            } else {
                if let Some(lead) = self.lead_row {
                    if n == lead {
                        return true;
                    }
                }
            }

            self.selected.contains(&n)
        }

        fn lead_selection(&self) -> Option<usize> {
            self.lead_row
        }
    }

    impl HandleEvent<crossterm::event::Event, Regular, Outcome> for ListState<RowSetSelection> {
        fn handle(&mut self, event: &crossterm::event::Event, _: Regular) -> Outcome {
            let res = if self.is_focused() {
                match event {
                    ct_event!(keycode press Down) => self.move_down(1, false).into(),
                    ct_event!(keycode press SHIFT-Down) => self.move_down(1, true).into(),
                    ct_event!(keycode press Up) => self.move_up(1, false).into(),
                    ct_event!(keycode press SHIFT-Up) => self.move_up(1, true).into(),
                    ct_event!(keycode press CONTROL-Down) | ct_event!(keycode press End) => {
                        self.move_to(self.rows.saturating_sub(1), false).into()
                    }
                    ct_event!(keycode press SHIFT-End) => {
                        self.move_to(self.rows.saturating_sub(1), true).into()
                    }
                    ct_event!(keycode press CONTROL-Up) | ct_event!(keycode press Home) => {
                        self.move_to(0, false).into()
                    }
                    ct_event!(keycode press SHIFT-Home) => self.move_to(0, true).into(),

                    ct_event!(keycode press PageUp) => self
                        .move_up(self.page_len().saturating_sub(1), false)
                        .into(),
                    ct_event!(keycode press SHIFT-PageUp) => {
                        self.move_up(self.page_len().saturating_sub(1), true).into()
                    }
                    ct_event!(keycode press PageDown) => self
                        .move_down(self.page_len().saturating_sub(1), false)
                        .into(),
                    ct_event!(keycode press SHIFT-PageDown) => self
                        .move_down(self.page_len().saturating_sub(1), true)
                        .into(),
                    _ => Outcome::Continue,
                }
            } else {
                Outcome::Continue
            };

            if res == Outcome::Continue {
                self.handle(event, MouseOnly)
            } else {
                res
            }
        }
    }

    impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for ListState<RowSetSelection> {
        fn handle(&mut self, event: &crossterm::event::Event, _: MouseOnly) -> Outcome {
            flow!(match event {
                ct_event!(mouse any for m) | ct_event!(mouse any CONTROL for m)
                    if self.mouse.drag(self.inner, m)
                        || self.mouse.drag2(self.inner, m, KeyModifiers::CONTROL) =>
                {
                    self.move_to(self.row_at_drag((m.column, m.row)), true)
                        .into()
                }
                ct_event!(mouse down Left for column, row) => {
                    let pos = (*column, *row);
                    if self.inner.contains(pos.into()) {
                        if let Some(new_row) = self.row_at_clicked(pos) {
                            self.move_to(new_row, false).into()
                        } else {
                            Outcome::Continue
                        }
                    } else {
                        Outcome::Continue
                    }
                }
                ct_event!(mouse down ALT-Left for column, row) => {
                    let pos = (*column, *row);
                    if self.area.contains(pos.into()) {
                        if let Some(new_row) = self.row_at_clicked(pos) {
                            self.move_to(new_row, true).into()
                        } else {
                            Outcome::Continue
                        }
                    } else {
                        Outcome::Continue
                    }
                }
                ct_event!(mouse down CONTROL-Left for column, row) => {
                    let pos = (*column, *row);
                    if self.area.contains(pos.into()) {
                        if let Some(new_row) = self.row_at_clicked(pos) {
                            self.retire_selection();
                            if self.selection.is_selected_row(new_row) {
                                self.selection.remove(new_row);
                            } else {
                                self.move_to(new_row, true);
                            }
                            Outcome::Changed
                        } else {
                            Outcome::Continue
                        }
                    } else {
                        Outcome::Continue
                    }
                }
                _ => Outcome::Continue,
            });

            let mut sas = ScrollAreaState::new()
                .area(self.inner)
                .v_scroll(&mut self.scroll);
            let r = match sas.handle(event, MouseOnly) {
                ScrollOutcome::Up(v) => self.scroll_up(v),
                ScrollOutcome::Down(v) => self.scroll_down(v),
                ScrollOutcome::VPos(v) => self.set_offset(v),
                ScrollOutcome::Left(_) => false,
                ScrollOutcome::Right(_) => false,
                ScrollOutcome::HPos(_) => false,

                ScrollOutcome::Continue => false,
                ScrollOutcome::Unchanged => false,
                ScrollOutcome::Changed => true,
            };
            if r {
                return Outcome::Changed;
            }

            Outcome::Unchanged
        }
    }
}

/// Handle events for the popup.
/// Call before other handlers to deal with intersections
/// with other widgets.
pub fn handle_events(
    state: &mut ListState,
    focus: bool,
    event: &crossterm::event::Event,
) -> Outcome {
    state.focus.set(focus);
    HandleEvent::handle(state, event, Regular)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(state: &mut ListState, event: &crossterm::event::Event) -> Outcome {
    HandleEvent::handle(state, event, MouseOnly)
}
