use crate::_private::NonExhaustive;
use crate::event::{HandleEvent, MouseOnly};
use rat_focus::{FocusFlag, HasFocusFlag, ZRect};
use rat_input::event::Popup;
use rat_input::menuline::{MenuOutcome, MenuStyle};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, StatefulWidget, StatefulWidgetRef};

pub use rat_input::popup_menu::Placement;

/// Popup menu.
#[derive(Debug, Default, Clone)]
pub struct RPopupMenu<'a> {
    widget: rat_input::popup_menu::PopupMenu<'a>,
}

/// Popup menu state.
#[derive(Debug, Clone)]
pub struct RPopupMenuState {
    pub focus: FocusFlag,
    pub widget: rat_input::popup_menu::PopupMenuState,
    pub z_areas: [ZRect; 1],

    pub non_exhaustive: NonExhaustive,
}

impl<'a> RPopupMenu<'a> {
    /// New
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a formatted item.
    /// The navchar is optional, any markup for it is your problem.
    pub fn add(mut self, item: Line<'a>, navchar: Option<char>) -> Self {
        self.widget = self.widget.add(item, navchar);
        self
    }

    /// Add item.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn add_str(mut self, menu_item: &'a str) -> Self {
        self.widget = self.widget.add_str(menu_item);
        self
    }

    /// Fixed width for the menu.
    /// If not set it uses 1.5 times the length of the longest item.
    pub fn width(mut self, width: u16) -> Self {
        self.widget = self.widget.width(width);
        self
    }

    /// Placement relative to the render-area.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.widget = self.widget.placement(placement);
        self
    }

    /// Combined style.
    #[inline]
    pub fn styles(mut self, styles: MenuStyle) -> Self {
        self.widget = self.widget.styles(styles);
        self
    }

    /// Base style.
    #[inline]
    pub fn style(mut self, style: Style) -> Self {
        self.widget = self.widget.style(style);
        self
    }

    /// Selection
    #[inline]
    pub fn focus_style(mut self, style: Style) -> Self {
        self.widget = self.widget.focus_style(style);
        self
    }

    /// Block for borders.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.widget = self.widget.block(block);
        self
    }
}

impl<'a> StatefulWidgetRef for RPopupMenu<'a> {
    type State = RPopupMenuState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.active() {
            self.widget.render_ref(area, buf, &mut state.widget);
            state.z_areas[0] = ZRect::from((1, state.widget.area));
        } else {
            state.clear();
        }
    }
}

impl<'a> StatefulWidget for RPopupMenu<'a> {
    type State = RPopupMenuState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.active() {
            self.widget.render(area, buf, &mut state.widget);
            state.z_areas[0] = ZRect::from((1, state.widget.area));
        } else {
            state.clear();
        }
    }
}

impl Default for RPopupMenuState {
    fn default() -> Self {
        Self {
            focus: Default::default(),
            widget: Default::default(),
            z_areas: Default::default(),
            non_exhaustive: NonExhaustive,
        }
    }
}

impl RPopupMenuState {
    /// Reset the state to defaults.
    pub fn clear(&mut self) {
        *self = Default::default();
    }

    /// Show the popup.
    pub fn flip_active(&mut self) {
        self.focus.focus.set(!self.focus.get());
    }

    /// Show the popup.
    pub fn active(&self) -> bool {
        self.is_focused()
    }

    /// Show the popup.
    pub fn set_active(&self, active: bool) {
        self.focus.focus.set(active);
    }
}

impl RPopupMenuState {
    /// New
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of items.
    #[inline]
    pub fn len(&self) -> usize {
        self.widget.len()
    }

    /// Any items.
    pub fn is_empty(&self) -> bool {
        self.widget.is_empty()
    }

    /// Select
    #[inline]
    pub fn select(&mut self, select: Option<usize>) -> bool {
        self.widget.select(select)
    }

    /// Selected index
    #[inline]
    pub fn selected(&self) -> Option<usize> {
        self.widget.selected
    }

    /// Previous item.
    #[inline]
    pub fn prev(&mut self) -> bool {
        self.widget.prev()
    }

    /// Next item.
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> bool {
        self.widget.next()
    }

    /// Select by hotkey
    #[inline]
    pub fn navigate(&mut self, c: char) -> MenuOutcome {
        self.widget.navigate(c)
    }

    /// Select item at position
    #[inline]
    pub fn select_at(&mut self, pos: (u16, u16)) -> bool {
        self.widget.select_at(pos)
    }

    /// Item at position.
    #[inline]
    pub fn item_at(&self, pos: (u16, u16)) -> Option<usize> {
        self.widget.item_at(pos)
    }
}

impl HasFocusFlag for RPopupMenuState {
    /// Focus flag.
    fn focus(&self) -> &FocusFlag {
        &self.focus
    }

    /// Focus area.
    fn area(&self) -> Rect {
        self.widget.area
    }

    /// Widget area with z index.
    fn z_areas(&self) -> &[ZRect] {
        &self.z_areas
    }

    fn navigable(&self) -> bool {
        false
    }
}

impl HandleEvent<crossterm::event::Event, Popup, MenuOutcome> for RPopupMenuState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: Popup) -> MenuOutcome {
        if self.is_focused() {
            self.widget.handle(event, Popup)
        } else {
            self.widget.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, MenuOutcome> for RPopupMenuState {
    fn handle(&mut self, event: &crossterm::event::Event, _keymap: MouseOnly) -> MenuOutcome {
        self.widget.handle(event, MouseOnly)
    }
}
