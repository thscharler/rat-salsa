//!
//! A menubar widget with sub-menus.
//!
//! Combines [MenuLine] and [PopupMenu] and adds a [MenuStructure] trait
//! to bind all together.
//!
//! Rendering is split in two widgets [MenuBar] and [MenuPopup].
//! This should help with front/back rendering.
//!
//! Event-handling for the popup menu is split via the [Popup] qualifier.
//! All `Popup` event-handling should be called before the regular
//! `FocusKeys` handling.
//!
use crate::event::Popup;
use crate::menuline::{MenuLine, MenuLineState, MenuOutcome, MenuStyle};
use crate::popup_menu::{Placement, PopupMenu, PopupMenuState};
use crate::util::menu_str;
use rat_event::{flow, HandleEvent, MouseOnly, Regular};
use rat_focus::{FocusFlag, HasFocusFlag, ZRect};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, StatefulWidget, Style};
use ratatui::widgets::{Block, StatefulWidgetRef};
use std::fmt::{Debug, Formatter};

/// Trait for the structural data of the MenuBar.
pub trait MenuStructure<'a> {
    /// Main menu.
    fn menus(&'a self) -> Vec<(Line<'a>, Option<char>)>;
    /// Submenus.
    fn submenu(&'a self, n: usize) -> Vec<(Line<'a>, Option<char>)>;
}

/// Static menu structure.
#[derive(Debug)]
pub struct StaticMenu {
    pub menu: &'static [(&'static str, &'static [&'static str])],
}

impl MenuStructure<'static> for StaticMenu {
    fn menus(&'static self) -> Vec<(Line<'static>, Option<char>)> {
        self.menu.iter().map(|v| menu_str(v.0)).collect()
    }

    fn submenu(&'static self, n: usize) -> Vec<(Line<'static>, Option<char>)> {
        self.menu[n].1.iter().map(|v| menu_str(v)).collect()
    }
}

/// MenuBar widget.
///
/// This is only half of the widget. For popup rendering there is the separate
/// [MenuPopup].
#[derive(Debug, Default, Clone)]
pub struct MenuBar<'a> {
    menu: MenuLine<'a>,
}

/// Menubar widget.
///
/// Separate renderer for the popup part of the menubar.
#[derive(Default, Clone)]
pub struct MenuPopup<'a> {
    structure: Option<&'a dyn MenuStructure<'a>>,
    popup: PopupMenu<'a>,
}

/// State for the menubar.
#[derive(Debug, Default, Clone)]
pub struct MenuBarState {
    /// State for the menu.
    pub bar: MenuLineState,
    /// State for the last rendered popup menu.
    pub popup: PopupMenuState,

    pub area: Rect,
    pub z_areas: [ZRect; 2],
}

impl<'a> MenuBar<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Title text.
    #[inline]
    pub fn title(mut self, title: impl Into<Line<'a>>) -> Self {
        self.menu = self.menu.title(title);
        self
    }

    /// Menu-Structure
    pub fn menu(mut self, structure: &'a dyn MenuStructure<'a>) -> Self {
        for (m, n) in structure.menus() {
            self.menu = self.menu.add(m, n);
        }
        self
    }

    /// Combined style.
    #[inline]
    pub fn styles(mut self, styles: MenuStyle) -> Self {
        self.menu = self.menu.styles(styles.clone());
        self
    }

    /// Base style.
    #[inline]
    pub fn style(mut self, style: Style) -> Self {
        self.menu = self.menu.style(style);
        self
    }

    /// Menu-title style.
    #[inline]
    pub fn title_style(mut self, style: Style) -> Self {
        self.menu = self.menu.title_style(style);
        self
    }

    /// Selection
    #[inline]
    pub fn select_style(mut self, style: Style) -> Self {
        self.menu = self.menu.select_style(style);
        self
    }

    /// Selection + Focus
    #[inline]
    pub fn focus_style(mut self, style: Style) -> Self {
        self.menu = self.menu.focus_style(style);
        self
    }
}

impl<'a> Debug for MenuPopup<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuPopup")
            .field("popup", &self.popup)
            .finish()
    }
}

impl<'a> MenuPopup<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Menu.
    pub fn menu(mut self, structure: &'a dyn MenuStructure<'a>) -> Self {
        self.structure = Some(structure);
        self
    }

    /// Fixed width for the menu.
    /// If not set it uses 1.5 times the length of the longest item.
    pub fn width(mut self, width: u16) -> Self {
        self.popup = self.popup.width(width);
        self
    }

    /// Placement relative to the render-area.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.popup = self.popup.placement(placement);
        self
    }

    /// Combined style.
    #[inline]
    pub fn styles(mut self, styles: MenuStyle) -> Self {
        self.popup = self.popup.styles(styles.clone());
        self
    }

    /// Base style.
    pub fn style(mut self, style: Style) -> Self {
        self.popup = self.popup.style(style);
        self
    }

    /// Focus/Selection style.
    pub fn focus_style(mut self, style: Style) -> Self {
        self.popup = self.popup.focus_style(style);
        self
    }

    /// Block for borders.
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.popup = self.popup.block(block);
        self
    }
}

impl<'a> StatefulWidgetRef for MenuBar<'a> {
    type State = MenuBarState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_menubar(self, area, buf, state);
    }
}

impl<'a> StatefulWidget for MenuBar<'a> {
    type State = MenuBarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_menubar(&self, area, buf, state);
    }
}

fn render_menubar(widget: &MenuBar<'_>, area: Rect, buf: &mut Buffer, state: &mut MenuBarState) {
    widget.menu.render_ref(area, buf, &mut state.bar);

    // Combined area + each part with a z-index.
    state.area = state.bar.area;
    state.z_areas = [ZRect::from((0, state.bar.area)), ZRect::default()];
}

impl<'a> StatefulWidgetRef for MenuPopup<'a> {
    type State = MenuBarState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_menu_popup(self, area, buf, state);
    }
}

impl<'a> StatefulWidget for MenuPopup<'a> {
    type State = MenuBarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        render_menu_popup(&self, area, buf, state);
    }
}

fn render_menu_popup(
    widget: &MenuPopup<'_>,
    _area: Rect,
    buf: &mut Buffer,
    state: &mut MenuBarState,
) {
    // Combined area + each part with a z-index.
    state.area = state.bar.area;
    state.z_areas = [ZRect::from((0, state.bar.area)), ZRect::default()];

    let Some(selected) = state.bar.selected() else {
        return;
    };
    let Some(structure) = widget.structure else {
        return;
    };

    if state.popup.is_focused() {
        let mut len = 0;
        let mut popup = widget.popup.clone(); // TODO: ???
        for (item, navchar) in structure.submenu(selected) {
            popup = popup.add(item, navchar);
            len += 1;
        }

        if len > 0 {
            let area = state.bar.item_areas[selected];
            popup.render(area, buf, &mut state.popup);

            // Combined area + each part with a z-index.
            state.area = state.bar.area.union(state.popup.area);
            state.z_areas[1] = ZRect::from((1, state.popup.area));
        }
    } else {
        state.popup = Default::default();
    }
}

impl MenuBarState {
    /// State.
    /// For the specifics use the public fields `menu` and `popup`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Submenu visible/active.
    pub fn popup_active(&self) -> bool {
        self.popup.is_focused()
    }

    /// Submenu visible/active.
    pub fn set_popup_active(&mut self, active: bool) {
        self.popup.focus.set(active);
    }
}

impl HasFocusFlag for MenuBarState {
    fn focus(&self) -> &FocusFlag {
        &self.bar.focus
    }

    fn area(&self) -> Rect {
        self.area
    }

    fn z_areas(&self) -> &[ZRect] {
        &self.z_areas
    }
}

impl HandleEvent<crossterm::event::Event, Popup, MenuOutcome> for MenuBarState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Popup) -> MenuOutcome {
        if !self.is_focused() {
            self.set_popup_active(false);
        }

        if let Some(selected) = self.bar.selected() {
            if self.popup_active() {
                match self.popup.handle(event, Popup) {
                    MenuOutcome::Selected(n) => MenuOutcome::MenuSelected(selected, n),
                    MenuOutcome::Activated(n) => MenuOutcome::MenuActivated(selected, n),
                    r => r,
                }
            } else {
                MenuOutcome::NotUsed
            }
        } else {
            MenuOutcome::NotUsed
        }
    }
}

impl HandleEvent<crossterm::event::Event, Regular, MenuOutcome> for MenuBarState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: Regular) -> MenuOutcome {
        if !self.is_focused() {
            self.set_popup_active(false);
        }

        if self.bar.is_focused() {
            let old_selected = self.bar.selected();
            match self.bar.handle(event, Regular) {
                r @ MenuOutcome::Selected(_) => {
                    if self.bar.selected == old_selected {
                        self.popup.flip_active();
                    } else {
                        self.popup.set_active(true);
                    }
                    r
                }
                r => r,
            }
        } else {
            self.bar.handle(event, MouseOnly)
        }
    }
}

impl HandleEvent<crossterm::event::Event, MouseOnly, MenuOutcome> for MenuBarState {
    fn handle(&mut self, event: &crossterm::event::Event, _qualifier: MouseOnly) -> MenuOutcome {
        if !self.is_focused() {
            self.set_popup_active(false);
        }

        flow!(if let Some(selected) = self.bar.selected() {
            if self.popup_active() {
                match self.popup.handle(event, MouseOnly) {
                    MenuOutcome::Selected(n) => MenuOutcome::MenuSelected(selected, n),
                    MenuOutcome::Activated(n) => {
                        self.set_popup_active(false);
                        MenuOutcome::MenuActivated(selected, n)
                    }
                    r => r,
                }
            } else {
                MenuOutcome::NotUsed
            }
        } else {
            MenuOutcome::NotUsed
        });

        let old_selected = self.bar.selected();
        match self.bar.handle(event, MouseOnly) {
            r @ MenuOutcome::Selected(_) => {
                if self.bar.selected == old_selected {
                    self.popup.flip_active();
                } else {
                    self.popup.set_active(true);
                }
                r
            }
            r => r,
        }
    }
}

/// Handle menu events. Keyboard events are processed if focus is true.
///
/// Attention:
/// For the event-handling of the popup-menus you need to call handle_popup_events().
pub fn handle_events(
    state: &mut MenuBarState,
    focus: bool,
    event: &crossterm::event::Event,
) -> MenuOutcome {
    state.bar.focus.set(focus);
    state.handle(event, Regular)
}

/// Handle menu events for the popup-menu.
///
/// This one is separate, as it needs to be called before other event-handlers
/// to cope with overlapping regions.
///
/// focus - is the menubar focused?
pub fn handle_popup_events(
    state: &mut MenuBarState,
    focus: bool,
    event: &crossterm::event::Event,
) -> MenuOutcome {
    state.bar.focus.set(focus);
    state.handle(event, Popup)
}

/// Handle only mouse-events.
pub fn handle_mouse_events(
    state: &mut MenuLineState,
    event: &crossterm::event::Event,
) -> MenuOutcome {
    state.handle(event, MouseOnly)
}
