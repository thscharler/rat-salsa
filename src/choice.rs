//!
//! Simple choice widget.
//!
//! Status: ALPHA/UNSTABLE
//!

use crate::_private::NonExhaustive;
use crate::util::revert_style;
use rat_event::util::{item_at, MouseFlags};
use rat_event::{ct_event, ConsumedEvent, HandleEvent, MouseOnly, Outcome, Popup, Regular};
use rat_focus::{FocusFlag, HasFocus, ZRect};
use rat_popup::event::PopupOutcome;
use rat_popup::{Placement, PopupCore, PopupCoreState, PopupStyle};
use rat_scrolled::event::ScrollOutcome;
use rat_scrolled::{Scroll, ScrollAreaState};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::BlockExt;
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Block, StatefulWidget, Widget};
use std::borrow::Cow;
use std::cell::RefCell;
use std::cmp::{max, min};
use std::rc::Rc;

/// Choice widget.
///
/// Select one of a list. No editable mode for this one.
///
#[derive(Debug, Clone)]
pub struct Choice<'a> {
    items: Rc<RefCell<Vec<Cow<'a, str>>>>,

    style: Style,
    button_style: Option<Style>,
    select_style: Option<Style>,
    focus_style: Option<Style>,
    block: Option<Block<'a>>,

    popup_placement: Placement,
    popup_len: Option<u16>,
    popup: PopupCore<'a>,
}

/// Renders the main widget.
#[derive(Debug)]
pub struct RenderChoice<'a> {
    items: Rc<RefCell<Vec<Cow<'a, str>>>>,

    style: Style,
    button_style: Option<Style>,
    focus_style: Option<Style>,
    block: Option<Block<'a>>,
    len: Option<u16>,
}

/// Renders the popup.
#[derive(Debug)]
pub struct RenderChoicePopup<'a> {
    items: Rc<RefCell<Vec<Cow<'a, str>>>>,

    style: Style,
    select_style: Option<Style>,

    popup_placement: Placement,
    popup_len: Option<u16>,
    popup: PopupCore<'a>,
}

#[derive(Debug, Clone)]
pub struct ChoiceStyle {
    pub style: Style,
    pub button: Option<Style>,
    pub select: Option<Style>,
    pub focus: Option<Style>,
    pub block: Option<Block<'static>>,

    pub popup: PopupStyle,
    pub popup_len: Option<u16>,

    pub non_exhaustive: NonExhaustive,
}

/// State.
#[derive(Debug, Clone)]
pub struct ChoiceState {
    /// Total area.
    /// __read only__. renewed with each render.
    pub area: Rect,
    /// All areas
    /// __read only__. renewed with each render.
    pub z_areas: [ZRect; 2],
    /// First char of each item for navigation.
    /// __read only__. renewed with each render.
    pub nav_char: Vec<Vec<char>>,
    /// Item area in the main widget.
    /// __read only__. renewed with each render.
    pub item_area: Rect,
    /// Button area in the main widget.
    /// __read only__. renewed with each render.
    pub button_area: Rect,
    /// Visible items in the popup.
    /// __read only__. renewed with each render.
    pub item_areas: Vec<Rect>,
    /// Total number of items.
    /// __read only__. renewed with each render.
    pub len: usize,

    /// Select item.
    /// __read+write__
    pub selected: Option<usize>,
    /// Popup state.
    pub popup: PopupCoreState,

    /// Focus flag.
    /// __read+write__
    pub focus: FocusFlag,
    /// Mouse util.
    pub mouse: MouseFlags,

    pub non_exhaustive: NonExhaustive,
}

impl Default for ChoiceStyle {
    fn default() -> Self {
        Self {
            style: Default::default(),
            button: None,
            select: None,
            focus: None,
            block: None,
            popup: Default::default(),
            popup_len: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> Default for Choice<'a> {
    fn default() -> Self {
        Self {
            items: Default::default(),
            style: Default::default(),
            button_style: None,
            select_style: None,
            focus_style: None,
            block: None,
            popup_len: None,
            popup_placement: Placement::BelowOrAbove,
            popup: Default::default(),
        }
    }
}

impl<'a> Choice<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an item.
    pub fn item(self, item: impl Into<Cow<'a, str>>) -> Self {
        self.items.borrow_mut().push(item.into());
        self
    }

    /// Combined styles.
    pub fn styles(mut self, styles: ChoiceStyle) -> Self {
        self.style = styles.style;
        if styles.button.is_some() {
            self.button_style = styles.button;
        }
        if styles.select.is_some() {
            self.select_style = styles.select;
        }
        if styles.focus.is_some() {
            self.focus_style = styles.focus;
        }
        if styles.block.is_some() {
            self.block = styles.block;
        }
        if let Some(placement) = styles.popup.placement {
            self.popup_placement = placement;
        }
        if styles.popup_len.is_some() {
            self.popup_len = styles.popup_len;
        }
        self.popup = self.popup.styles(styles.popup);
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Style for the down button.
    pub fn button_style(mut self, style: Style) -> Self {
        self.button_style = Some(style);
        self
    }

    /// Selection in the list.
    pub fn select_style(mut self, style: Style) -> Self {
        self.select_style = Some(style);
        self
    }

    /// Focused style.
    pub fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = Some(style);
        self
    }

    /// Block for the main widget.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Placement of the popup.
    ///
    /// __Default__
    /// Default is BelowOrAbove.
    pub fn popup_placement(mut self, placement: Placement) -> Self {
        self.popup_placement = placement;
        self
    }

    /// Outer boundary for the popup.
    pub fn popup_boundary(mut self, boundary: Rect) -> Self {
        self.popup = self.popup.boundary(boundary);
        self
    }

    /// Override the popup length.
    ///
    /// __Default__
    /// Defaults to the number of items or 5.
    pub fn popup_len(mut self, len: u16) -> Self {
        self.popup_len = Some(len);
        self
    }

    /// Base style for the popup.
    pub fn popup_style(mut self, style: Style) -> Self {
        self.popup = self.popup.style(style);
        self
    }

    /// Block for the popup.
    pub fn popup_block(mut self, block: Block<'a>) -> Self {
        self.popup = self.popup.block(block);
        self
    }

    /// Scroll for the popup.
    pub fn popup_scroll(mut self, scroll: Scroll<'a>) -> Self {
        self.popup = self.popup.v_scroll(scroll);
        self
    }

    /// Choice itself doesn't render.
    ///
    /// This builds the widgets from the parameters set for Choice.
    pub fn into_widgets(self) -> (RenderChoice<'a>, RenderChoicePopup<'a>) {
        (
            RenderChoice {
                items: self.items.clone(),
                style: self.style,
                button_style: self.button_style,
                focus_style: self.focus_style,
                block: self.block,
                len: self.popup_len,
            },
            RenderChoicePopup {
                items: self.items.clone(),
                style: self.style,
                select_style: self.select_style,
                popup: self.popup,
                popup_placement: self.popup_placement,
                popup_len: self.popup_len,
            },
        )
    }
}

impl<'a> StatefulWidget for RenderChoice<'a> {
    type State = ChoiceState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.z_areas[0] = ZRect::from(area);
        state.len = self.items.borrow().len();

        if !state.popup.is_active() {
            let len = self
                .len
                .unwrap_or_else(|| min(5, self.items.borrow().len()) as u16);
            state.popup.v_scroll.max_offset =
                self.items.borrow().len().saturating_sub(len as usize);
            state.popup.v_scroll.page_len = len as usize;
            state
                .popup
                .v_scroll
                .scroll_to_pos(state.selected.unwrap_or_default());
        }

        state.nav_char.clear();
        state.nav_char.extend(self.items.borrow().iter().map(|v| {
            v.chars()
                .next()
                .map_or(Vec::default(), |c| c.to_lowercase().collect::<Vec<_>>())
        }));

        let inner = self.block.inner_if_some(area);

        state.item_area = Rect::new(
            inner.x,
            inner.y,
            inner.width.saturating_sub(3),
            inner.height,
        );
        state.button_area = Rect::new(inner.right().saturating_sub(3), inner.y, 3, inner.height);

        let button_style = self.button_style.unwrap_or(self.style);
        let focus_style = self.focus_style.unwrap_or(self.style);

        buf.set_style(area, self.style);
        self.block.render(area, buf);

        if state.is_focused() {
            buf.set_style(state.item_area, focus_style);
        } else {
            buf.set_style(state.item_area, self.style);
        }
        if let Some(selected) = state.selected {
            if let Some(item) = self.items.borrow().get(selected) {
                Span::from(item.as_ref()).render(state.item_area, buf);
            }
        }

        buf.set_style(state.button_area, button_style);
        let dy = if (state.button_area.height & 1) == 1 {
            state.button_area.height / 2
        } else {
            state.button_area.height.saturating_sub(1) / 2
        };
        Span::from(" ▼ ").render(
            Rect::new(state.button_area.x, state.button_area.y + dy, 3, 1),
            buf,
        );
    }
}

impl<'a> StatefulWidget for RenderChoicePopup<'a> {
    type State = ChoiceState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.popup.is_active() {
            let len = self
                .popup_len
                .unwrap_or_else(|| min(5, self.items.borrow().len()) as u16);

            let popup_len = len + self.popup.get_block_size().height;
            let pop_area = Rect::new(0, 0, area.width, popup_len);

            self.popup
                .constraint(self.popup_placement.into_constraint(area))
                .render(pop_area, buf, &mut state.popup);

            let inner = state.popup.widget_area;

            state.popup.v_scroll.max_offset = self
                .items
                .borrow()
                .len()
                .saturating_sub(inner.height as usize);
            state.popup.v_scroll.page_len = inner.height as usize;

            state.item_areas.clear();
            let mut row = inner.y;
            let mut idx = state.popup.v_scroll.offset;
            loop {
                if row >= inner.bottom() {
                    break;
                }

                let item_area = Rect::new(inner.x, row, inner.width, 1);
                state.item_areas.push(item_area);

                if let Some(item) = self.items.borrow().get(idx) {
                    let style = if state.selected == Some(idx) {
                        self.select_style.unwrap_or(revert_style(self.style))
                    } else {
                        self.style
                    };

                    buf.set_style(item_area, style);
                    Span::from(item.as_ref()).render(item_area, buf);
                } else {
                    // noop?
                }

                row += 1;
                idx += 1;
            }

            state.z_areas[1] = ZRect::from((1, state.popup.area));
            state.area = ZRect::union_all(&state.z_areas).expect("area").as_rect();
        } else {
            state.popup.clear_areas();
        }
    }
}

impl Default for ChoiceState {
    fn default() -> Self {
        Self {
            area: Default::default(),
            z_areas: [Default::default(); 2],
            nav_char: Default::default(),
            item_area: Default::default(),
            button_area: Default::default(),
            item_areas: Default::default(),
            len: 0,
            selected: None,
            popup: Default::default(),
            focus: Default::default(),
            mouse: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl HasFocus for ChoiceState {
    fn focus(&self) -> FocusFlag {
        self.focus.clone()
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn z_areas(&self) -> &[ZRect] {
        &self.z_areas
    }
}

impl ChoiceState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn named(name: &str) -> Self {
        Self {
            focus: FocusFlag::named(name),
            ..Default::default()
        }
    }

    pub fn is_popup_active(&self) -> bool {
        self.popup.is_active()
    }

    pub fn flip_popup_active(&mut self) {
        self.popup.flip_active();
    }

    pub fn set_popup_active(&mut self, active: bool) -> bool {
        let old_active = self.popup.is_active();
        self.popup.set_active(active);
        old_active != active
    }
}

impl ChoiceState {
    /// Select
    pub fn select(&mut self, select: Option<usize>) -> bool {
        let old_selected = self.selected;

        if let Some(select) = select {
            self.selected = Some(select.clamp(0, self.len.saturating_sub(1)));
        } else {
            self.selected = None;
        }

        old_selected != self.selected
    }

    pub fn clear_offset(&mut self) {
        self.popup.v_scroll.set_offset(0);
    }

    pub fn set_offset(&mut self, offset: usize) -> bool {
        self.popup.v_scroll.set_offset(offset)
    }

    pub fn offset(&self) -> usize {
        self.popup.v_scroll.offset()
    }

    pub fn max_offset(&self) -> usize {
        self.popup.v_scroll.max_offset()
    }

    pub fn page_len(&self) -> usize {
        self.popup.v_scroll.page_len()
    }

    pub fn scroll_by(&self) -> usize {
        self.popup.v_scroll.scroll_by()
    }

    pub fn scroll_to_selected(&mut self) -> bool {
        if let Some(selected) = self.selected {
            self.popup.v_scroll.scroll_to_pos(selected)
        } else {
            false
        }
    }
}

impl ChoiceState {
    /// Select by first character.
    pub fn select_by_char(&mut self, c: char) -> bool {
        if self.nav_char.is_empty() {
            return false;
        }

        let selected = self.selected.unwrap_or_default();

        let c = c.to_lowercase().collect::<Vec<_>>();
        let mut idx = selected + 1;
        loop {
            if idx >= self.nav_char.len() {
                idx = 0;
            }
            if idx == selected {
                break;
            }

            if self.nav_char[idx] == c {
                self.selected = Some(idx);
                return true;
            }

            idx += 1;
        }
        false
    }

    /// Select at position
    pub fn move_to(&mut self, n: usize) -> bool {
        let r1 = self.select(Some(n));
        let r2 = self.scroll_to_selected();
        r1 || r2
    }

    /// Select next entry.
    pub fn move_down(&mut self, n: usize) -> bool {
        let old_selected = self.selected;

        if let Some(selected) = self.selected {
            self.selected = Some((selected + n).clamp(0, self.len.saturating_sub(1)));
        } else {
            self.selected = Some(0);
        }

        let r2 = self.scroll_to_selected();

        old_selected != self.selected || r2
    }

    /// Select prev entry.
    pub fn move_up(&mut self, n: usize) -> bool {
        let old_selected = self.selected;

        if let Some(selected) = self.selected {
            self.selected = Some(
                selected
                    .saturating_sub(n)
                    .clamp(0, self.len.saturating_sub(1)),
            );
        } else {
            self.selected = Some(self.len.saturating_sub(1));
        }

        let r2 = self.scroll_to_selected();

        old_selected != self.selected || r2
    }
}

impl HandleEvent<crossterm::event::Event, Regular, Outcome> for ChoiceState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> Outcome {
        // todo: here???
        let r0 = if self.lost_focus() {
            self.set_popup_active(false);
            Outcome::Changed
        } else {
            Outcome::Continue
        };

        let r1 = if self.is_focused() {
            match event {
                ct_event!(key press ' ') => {
                    self.flip_popup_active();
                    Outcome::Changed
                }
                ct_event!(key press c) => {
                    if self.select_by_char(*c) {
                        self.scroll_to_selected();
                        Outcome::Changed
                    } else {
                        Outcome::Unchanged
                    }
                }
                ct_event!(keycode press Enter) | ct_event!(keycode press Esc) => {
                    self.set_popup_active(false).into()
                }
                ct_event!(keycode press Down) => {
                    let r0 = if !self.popup.is_active() {
                        self.popup.set_active(true);
                        Outcome::Changed
                    } else {
                        Outcome::Continue
                    };
                    let r1 = self.move_down(1).into();
                    max(r0, r1)
                }
                ct_event!(keycode press Up) => {
                    let r0 = if !self.popup.is_active() {
                        self.popup.set_active(true);
                        Outcome::Changed
                    } else {
                        Outcome::Continue
                    };
                    let r1 = self.move_up(1).into();
                    max(r0, r1)
                }
                _ => Outcome::Continue,
            }
        } else {
            Outcome::Continue
        };

        let r1 = if !r1.is_consumed() {
            self.handle(event, MouseOnly)
        } else {
            r1
        };

        max(r0, r1)
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, Outcome> for ChoiceState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> Outcome {
        let r = match event {
            ct_event!(mouse down Left for x,y)
                if self.item_area.contains((*x, *y).into())
                    || self.button_area.contains((*x, *y).into()) =>
            {
                if !self.is_popup_active() && !self.popup.active.lost() {
                    self.set_popup_active(true);
                    Outcome::Changed
                } else {
                    // hide is down by self.popup.handle() as this click
                    // is outside the popup area!!
                    Outcome::Continue
                }
            }
            _ => Outcome::Continue,
        };

        self.popup.active.set_lost(false);
        self.popup.active.set_gained(false);

        r
    }
}

impl HandleEvent<crossterm::event::Event, Popup, Outcome> for ChoiceState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Popup) -> Outcome {
        let r1 = match self.popup.handle(event, Popup) {
            PopupOutcome::Hide => {
                self.set_popup_active(false);
                PopupOutcome::Hide
            }
            r => r,
        };

        let mut sas = ScrollAreaState::new()
            .area(self.popup.area)
            .v_scroll(&mut self.popup.v_scroll);
        let r2 = match sas.handle(event, MouseOnly) {
            ScrollOutcome::Up(n) => self.move_up(n).into(),
            ScrollOutcome::Down(n) => self.move_down(n).into(),
            ScrollOutcome::VPos(n) => self.move_to(n).into(),
            _ => Outcome::Continue,
        };

        let r2 = r2.or_else(|| match event {
            ct_event!(mouse any for m) if self.mouse.doubleclick(self.popup.widget_area, m) => {
                if let Some(n) = item_at(&self.item_areas, m.column, m.row) {
                    let r = self.move_to(self.offset() + n).into();
                    let s = self.set_popup_active(false).into();
                    max(r, s)
                } else {
                    Outcome::Unchanged
                }
            }
            ct_event!(mouse down Left for x,y)
                if self.popup.widget_area.contains((*x, *y).into()) =>
            {
                if let Some(n) = item_at(&self.item_areas, *x, *y) {
                    self.move_to(self.offset() + n).into()
                } else {
                    Outcome::Unchanged
                }
            }
            ct_event!(mouse drag Left for x,y)
                if self.popup.widget_area.contains((*x, *y).into()) =>
            {
                if let Some(n) = item_at(&self.item_areas, *x, *y) {
                    self.move_to(self.offset() + n).into()
                } else {
                    Outcome::Unchanged
                }
            }
            _ => Outcome::Continue,
        });

        max(Outcome::from(r1), r2)
    }
}
