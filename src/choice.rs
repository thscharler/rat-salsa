//!
//! Simple choice widget.
//!
//! Status: ALPHA/UNSTABLE
//!

use crate::_private::NonExhaustive;
use crate::util::{block_size, revert_style};
use log::debug;
use rat_event::{ct_event, HandleEvent, Outcome, Popup, Regular};
use rat_focus::{FocusFlag, HasFocus, ZRect};
use rat_popup::event::PopupOutcome;
use rat_popup::{Placement, PopupCore, PopupCoreState};
use rat_scrolled::{Scroll, ScrollState};
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
    popup_boundary: Option<Rect>,
    popup_block: Option<Block<'a>>,
    popup_scroll: Option<Scroll<'a>>,
    popup_style: Option<Style>,
    popup_len: Option<u16>,
}

/// Renders the main widget.
#[derive(Debug)]
pub struct RenderChoice<'a> {
    items: Rc<RefCell<Vec<Cow<'a, str>>>>,

    style: Style,
    button_style: Option<Style>,
    focus_style: Option<Style>,
    block: Option<Block<'a>>,
}

/// Renders the popup.
#[derive(Debug)]
pub struct RenderChoicePopup<'a> {
    items: Rc<RefCell<Vec<Cow<'a, str>>>>,

    style: Style,
    select_style: Option<Style>,

    placement: Placement,
    boundary: Option<Rect>,
    block: Option<Block<'a>>,
    scroll: Option<Scroll<'a>>,
    len: Option<u16>,
}

// todo: style

/// State.
#[derive(Debug, Clone)]
pub struct ChoiceState {
    /// Total area.
    pub area: Rect,
    /// All areas
    pub z_areas: [ZRect; 2],
    /// Selected item
    pub item_area: Rect,
    /// Button
    pub button_area: Rect,
    /// Visible items
    pub item_areas: Vec<Rect>,

    /// Select item.
    pub selected: Option<usize>,
    /// Vertical scroll.
    pub scroll: ScrollState,
    /// Popup state
    pub popup: PopupCoreState,

    /// Focus flag.
    pub focus: FocusFlag,

    pub non_exhaustive: NonExhaustive,
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
            popup_block: None,
            popup_placement: Placement::BelowOrAbove,
            popup_boundary: None,
            popup_style: None,
            popup_scroll: None,
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
        self.popup_boundary = Some(boundary);
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
        self.popup_style = Some(style);
        self
    }

    /// Block for the popup.
    pub fn popup_block(mut self, block: Block<'a>) -> Self {
        self.popup_block = Some(block);
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
            },
            RenderChoicePopup {
                items: self.items.clone(),
                placement: self.popup_placement,
                boundary: self.popup_boundary,
                block: self.popup_block,
                scroll: self.popup_scroll,
                style: self.popup_style.unwrap_or(self.style),
                len: self.popup_len,
                select_style: self.select_style,
            },
        )
    }
}

impl<'a> StatefulWidget for RenderChoice<'a> {
    type State = ChoiceState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;
        state.z_areas[0] = ZRect::from(area);

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
        let dy = state.button_area.height / 2;
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
                .len
                .unwrap_or_else(|| min(5, self.items.borrow().len()) as u16);

            let popup_len = len + block_size(&self.block).height;
            let pop_area = Rect::new(0, 0, area.width, popup_len);

            PopupCore::new()
                .constraint(self.placement.into_constraint(area))
                .boundary_opt(self.boundary)
                .style(self.style)
                .block_opt(self.block)
                .v_scroll_opt(self.scroll)
                .render(pop_area, buf, &mut state.popup);

            let inner = state.popup.widget_area;

            state.scroll.max_offset = self
                .items
                .borrow()
                .len()
                .saturating_sub(inner.height as usize);
            state.scroll.page_len = inner.height as usize;

            state.item_areas.clear();
            let mut row = inner.y;
            let mut idx = state.scroll.offset;
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
            item_area: Default::default(),
            button_area: Default::default(),
            item_areas: Default::default(),
            selected: None,
            scroll: Default::default(),
            popup: Default::default(),
            focus: Default::default(),
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

    // todo:
}

impl HandleEvent<crossterm::event::Event, Regular, Outcome> for ChoiceState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> Outcome {
        let r0 = if self.lost_focus() {
            self.popup.set_active(false);
            Outcome::Changed
        } else {
            Outcome::Continue
        };

        let r1 = match self.popup.handle(event, Popup) {
            PopupOutcome::Hide => {
                self.popup.set_active(false);
                Outcome::Changed
            }
            r => r.into(),
        };

        let r2 = if self.is_focused() {
            match event {
                // todo:
                ct_event!(key press ' ') => {
                    debug!("flip!");
                    self.popup.flip_active();
                    Outcome::Changed
                }
                ct_event!(keycode press Down) => {
                    if !self.popup.is_active() {
                        self.popup.set_active(true);
                    }
                    self.selected = Some(self.selected.map(|v| v + 1).unwrap_or(0));
                    Outcome::Changed
                }
                ct_event!(keycode press Up) => {
                    if !self.popup.is_active() {
                        self.popup.set_active(true);
                    }
                    self.selected = Some(self.selected.map(|v| v.saturating_sub(1)).unwrap_or(0));
                    Outcome::Changed
                }
                _ => Outcome::Continue,
            }
        } else {
            Outcome::Continue
        };

        max(r0, max(r1, r2))
    }
}
