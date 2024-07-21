//!
//! A popup-menu.
//!
//! It diverges from other widgets as this widget doesn't draw
//! *inside* the given area but aims to stay *outside* of it.
//!
//! You can give a [Placement] where the popup-menu should appear
//! relative to the given area.
//!
//! If you want it to appear at a mouse-click position, use a
//! `Rect::new(mouse_x, mouse_y, 0,0)` area.
//! If you want it to appear next to a given widget, use
//! the widgets drawing area.
//!
//! ## Navigation keys
//! If you give plain-text strings as items, the underscore
//! designates a navigation key. If you hit the key, the matching
//! item is selected. On the second hit, the matching item is
//! activated.
//!

use crate::_private::NonExhaustive;
use crate::event::Popup;
use crate::fill::Fill;
use crate::menuline::{MenuOutcome, MenuStyle};
use crate::util::{menu_str, next_opt, prev_opt, revert_style};
use rat_event::util::item_at_clicked;
use rat_event::{ct_event, ConsumedEvent, HandleEvent, MouseOnly};
use rat_focus::{FocusFlag, HasFocusFlag, ZRect};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::StatefulWidget;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, StatefulWidgetRef, Widget, WidgetRef};

/// Placement relative to the Rect given to render.
///
/// The popup-menu is always rendered outside the box,
/// and this gives the relative placement.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// On top of the given area. Placed slightly left, so that
    /// the menu text aligns with the left border.
    #[default]
    Top,
    /// Placed left-top of the given area.
    /// For a submenu opening to the left.
    Left,
    /// Placed right-top of the given area.
    /// For a submenu opening to the right.
    Right,
    /// Below the bottom of the given area. Placed slightly left,
    /// so that the menu text aligns with the left border.
    Bottom,
}

/// Popup menu.
#[derive(Debug, Default, Clone)]
pub struct PopupMenu<'a> {
    items: Vec<Line<'a>>,
    navchar: Vec<Option<char>>,

    width: Option<u16>,
    placement: Placement,
    boundary: Option<Rect>,

    style: Style,
    focus_style: Option<Style>,
    block: Option<Block<'a>>,
}

/// State & event handling.
#[derive(Debug, Clone)]
pub struct PopupMenuState {
    /// Focusflag is used to decide the visible/not-visible state.
    pub focus: FocusFlag,
    /// Total area
    pub area: Rect,
    /// Area with z-index for Focus.
    pub z_areas: [ZRect; 1],
    /// Areas for each item.
    pub item_areas: Vec<Rect>,
    /// Letter navigation
    pub navchar: Vec<Option<char>>,

    /// Selected item.
    pub selected: Option<usize>,

    pub non_exhaustive: NonExhaustive,
}

impl Default for PopupMenuState {
    fn default() -> Self {
        Self {
            focus: Default::default(),
            area: Default::default(),
            z_areas: [Default::default()],
            item_areas: vec![],
            navchar: vec![],
            selected: None,
            non_exhaustive: NonExhaustive,
        }
    }
}

impl<'a> PopupMenu<'a> {
    fn layout(&self, area: Rect, fit_in: Rect, state: &mut PopupMenuState) {
        let width = if let Some(width) = self.width {
            width
        } else {
            let text_width = self.items.iter().map(|v| v.width()).max();
            ((text_width.unwrap_or(10) as u16) * 3) / 2
        };

        let vertical_margin = if self.block.is_some() { 1 } else { 1 };
        let horizontal_margin = if self.block.is_some() { 2 } else { 1 };
        let len = self.items.len() as u16;

        let mut area = match self.placement {
            Placement::Top => Rect::new(
                area.x.saturating_sub(horizontal_margin),
                area.y.saturating_sub(len + vertical_margin * 2),
                width + horizontal_margin * 2,
                len + vertical_margin * 2,
            ),
            Placement::Left => Rect::new(
                area.x.saturating_sub(width + horizontal_margin * 2),
                area.y,
                width + horizontal_margin * 2,
                len + vertical_margin * 2,
            ),
            Placement::Right => Rect::new(
                area.x + area.width,
                area.y,
                width + horizontal_margin * 2,
                len + vertical_margin * 2,
            ),
            Placement::Bottom => Rect::new(
                area.x.saturating_sub(horizontal_margin),
                area.y + area.height,
                width + horizontal_margin * 2,
                len + vertical_margin * 2,
            ),
        };

        if area.right() >= fit_in.right() {
            area.x -= area.right() - fit_in.right();
        }
        if area.bottom() >= fit_in.bottom() {
            area.y -= area.bottom() - fit_in.bottom();
        }

        state.area = area;
        state.z_areas[0] = ZRect::from((1, area));

        state.item_areas.clear();
        let mut r = Rect::new(
            area.x + horizontal_margin,
            area.y + vertical_margin,
            width,
            1,
        );
        for _ in 0..len {
            state.item_areas.push(r);
            r.y += 1;
        }
    }
}

impl<'a> PopupMenu<'a> {
    /// New, empty.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a formatted item.
    /// The navchar is optional, any markup for it is your problem.
    pub fn add(mut self, item: Line<'a>, navchar: Option<char>) -> Self {
        self.items.push(item);
        self.navchar.push(navchar);
        self
    }

    /// Add a text-item.
    /// The first underscore is used to denote the navchar.
    pub fn add_str(mut self, txt: &'a str) -> Self {
        let (item, navchar) = menu_str(txt);
        self.items.push(item);
        self.navchar.push(navchar);
        self
    }

    /// Fixed width for the menu.
    /// If not set it uses 1.5 times the length of the longest item.
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Placement relative to the render-area.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// Set outer bounds for the popup-menu.
    /// If not used, the buffer-area is used as outer bounds.
    pub fn boundary(mut self, boundary: Rect) -> Self {
        self.boundary = Some(boundary);
        self
    }

    /// Take a style-set.
    pub fn styles(mut self, styles: MenuStyle) -> Self {
        self.style = styles.style;
        self.focus_style = styles.focus;
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Focus/Selection style.
    pub fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = Some(style);
        self
    }

    /// Block for borders.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> StatefulWidgetRef for PopupMenu<'a> {
    type State = PopupMenuState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(self, area, buf, state);
    }
}

impl<'a> StatefulWidget for PopupMenu<'a> {
    type State = PopupMenuState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_ref(&self, area, buf, state);
    }
}

fn render_ref(widget: &PopupMenu<'_>, area: Rect, buf: &mut Buffer, state: &mut PopupMenuState) {
    if !state.active() {
        state.clear();
        return;
    }

    state.navchar = widget.navchar.clone();

    let fit_in = if let Some(boundary) = widget.boundary {
        boundary
    } else {
        buf.area
    };

    widget.layout(area, fit_in, state);

    Fill::new().style(widget.style).render(state.area, buf);
    widget.block.render_ref(state.area, buf);

    for (n, txt) in widget.items.iter().enumerate() {
        let style = if state.selected == Some(n) {
            if let Some(focus) = widget.focus_style {
                focus
            } else {
                revert_style(widget.style)
            }
        } else {
            widget.style
        };

        buf.set_style(state.item_areas[n], style);
        txt.render(state.item_areas[n], buf);
    }
}

impl HasFocusFlag for PopupMenuState {
    /// Focus flag.
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    /// Focus area.
    fn area(&self) -> Rect {
        self.area
    }

    /// Widget area with z index.
    fn z_areas(&self) -> &[ZRect] {
        &self.z_areas
    }

    fn navigable(&self) -> bool {
        false
    }
}

impl PopupMenuState {
    /// New
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Reset the state to defaults.
    pub fn clear(&mut self) {
        *self = Default::default();
    }

    /// Show the popup.
    pub fn flip_active(&mut self) {
        self.focus.set(!self.focus.get());
    }

    /// Show the popup.
    pub fn active(&self) -> bool {
        self.is_focused()
    }

    /// Show the popup.
    pub fn set_active(&self, active: bool) {
        self.focus.set(active);
    }

    /// Number of items.
    #[inline]
    pub fn len(&self) -> usize {
        self.item_areas.len()
    }

    /// Any items.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.item_areas.is_empty()
    }

    /// Selected item.
    #[inline]
    pub fn select(&mut self, select: Option<usize>) -> bool {
        let old = self.selected;
        self.selected = select;
        old != self.selected
    }

    /// Selected item.
    #[inline]
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Select the previous item.
    #[inline]
    pub fn prev_item(&mut self) -> bool {
        let old = self.selected;
        self.selected = prev_opt(self.selected, 1, self.len());
        old != self.selected
    }

    /// Select the next item.
    #[inline]
    pub fn next_item(&mut self) -> bool {
        let old = self.selected;
        self.selected = next_opt(self.selected, 1, self.len());
        old != self.selected
    }

    /// Select by navigation key.
    #[inline]
    pub fn navigate(&mut self, c: char) -> MenuOutcome {
        let c = c.to_ascii_lowercase();
        for (i, cc) in self.navchar.iter().enumerate() {
            if *cc == Some(c) {
                if self.selected == Some(i) {
                    return MenuOutcome::Activated(i);
                } else {
                    self.selected = Some(i);
                    return MenuOutcome::Selected(i);
                }
            }
        }
        MenuOutcome::NotUsed
    }

    /// Select item at position.
    #[inline]
    pub fn select_at(&mut self, pos: (u16, u16)) -> bool {
        if let Some(idx) = item_at_clicked(&self.item_areas, pos.0, pos.1) {
            self.selected = Some(idx);
            true
        } else {
            false
        }
    }

    /// Item at position.
    #[inline]
    pub fn item_at(&self, pos: (u16, u16)) -> Option<usize> {
        item_at_clicked(&self.item_areas, pos.0, pos.1)
    }
}

impl HandleEvent<crossterm::event::Event, Popup, MenuOutcome> for PopupMenuState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Popup) -> MenuOutcome {
        let res = if self.active() {
            match event {
                ct_event!(key press ANY-c) => self.navigate(*c),
                ct_event!(keycode press Up) => {
                    if self.prev_item() {
                        MenuOutcome::Selected(self.selected.expect("selected"))
                    } else {
                        MenuOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Down) => {
                    if self.next_item() {
                        MenuOutcome::Selected(self.selected.expect("selected"))
                    } else {
                        MenuOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Home) => {
                    if self.select(Some(0)) {
                        MenuOutcome::Selected(self.selected.expect("selected"))
                    } else {
                        MenuOutcome::Unchanged
                    }
                }
                ct_event!(keycode press End) => {
                    if self.select(Some(self.len().saturating_sub(1))) {
                        MenuOutcome::Selected(self.selected.expect("selected"))
                    } else {
                        MenuOutcome::Unchanged
                    }
                }
                ct_event!(keycode press Esc) => {
                    self.set_active(false);
                    MenuOutcome::Changed
                }
                ct_event!(keycode press Enter) => {
                    if let Some(select) = self.selected {
                        self.set_active(false);
                        MenuOutcome::Activated(select)
                    } else {
                        MenuOutcome::NotUsed
                    }
                }

                ct_event!(key release _)
                | ct_event!(keycode release Up)
                | ct_event!(keycode release Down)
                | ct_event!(keycode release Home)
                | ct_event!(keycode release End)
                | ct_event!(keycode release Esc)
                | ct_event!(keycode release Enter) => MenuOutcome::Unchanged,

                _ => MenuOutcome::NotUsed,
            }
        } else {
            MenuOutcome::NotUsed
        };

        if !res.is_consumed() {
            self.handle(event, MouseOnly)
        } else {
            res
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, MenuOutcome> for PopupMenuState {
    fn handle(&mut self, event: &crossterm::event::Event, _: MouseOnly) -> MenuOutcome {
        if self.active() {
            match event {
                ct_event!(mouse moved for col, row) if self.area.contains((*col, *row).into()) => {
                    if self.select_at((*col, *row)) {
                        MenuOutcome::Selected(self.selected().expect("selection"))
                    } else {
                        MenuOutcome::Unchanged
                    }
                }
                ct_event!(mouse down Left for col, row)
                    if self.area.contains((*col, *row).into()) =>
                {
                    if self.select_at((*col, *row)) {
                        self.set_active(false);
                        MenuOutcome::Activated(self.selected().expect("selection"))
                    } else {
                        MenuOutcome::Unchanged
                    }
                }
                _ => MenuOutcome::NotUsed,
            }
        } else {
            MenuOutcome::NotUsed
        }
    }
}

/// Handle all events.
/// The assumption is, the popup-menu is focused or it is hidden.
/// This state must be handled outside of this widget.
pub fn handle_popup_events(
    state: &mut PopupMenuState,
    event: &crossterm::event::Event,
) -> MenuOutcome {
    state.handle(event, Popup)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut PopupMenuState,
    event: &crossterm::event::Event,
) -> MenuOutcome {
    state.handle(event, MouseOnly)
}
